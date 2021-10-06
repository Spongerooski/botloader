use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, RwLock},
    task::Poll,
    time::Duration,
};

use isolatecell::IsolateCell;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::Instant,
};
use tracing::info;

pub enum VmThreadCommand<T> {
    StartVM(T),
    Ping(oneshot::Sender<bool>),
}

type RunningVmTimeout<T, U> = Arc<RwLock<Option<VmHandle<T, U>>>>;

pub struct VmThreadFuture<T: VmInterface> {
    rcv_cmd: UnboundedReceiver<VmThreadCommand<T::BuildDesc>>,
    vms: Vec<VmContext<T::Future, T::TimeoutHandle, T::VmId>>,
    running_vm: RunningVmTimeout<T::VmId, T::TimeoutHandle>,
    isolate_cell: Rc<IsolateCell>,
}

impl<T> VmThreadFuture<T>
where
    T: VmInterface + 'static,
    T::BuildDesc: 'static,
    T::Future: Unpin,
{
    pub fn create() -> VmThreadHandle<T> {
        info!("spawning vm thread");
        let (snd, rcv) = mpsc::unbounded_channel();

        let running = Arc::new(RwLock::new(None));
        let running_clone = running.clone();

        let tokio_current = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            let t = VmThreadFuture::<T> {
                rcv_cmd: rcv,
                running_vm: running_clone,
                vms: Vec::new(),
                isolate_cell: Rc::new(Default::default()),
            };

            tokio_current.block_on(t);
            info!("vm thread shut down");
        });

        let handle = VmThreadHandle {
            running_vm: running,
            send_cmd: snd,
        };

        tokio::spawn(Self::runaway_checker(handle.clone()));

        handle
    }

    fn handle_cmd(&mut self, cmd: Option<VmThreadCommand<T::BuildDesc>>) {
        match cmd {
            None => {
                // panic as opposed to leaking threads silently
                // better to be made aware of the problem and forced to fix it
                // then having it silently break and eventually turn into a problem
                // that could lead to longer downtimes
                panic!("all senders dropped. we have now leaked a vm thread which is really bad.");
            }

            // respond to pings from runaway script detection
            Some(VmThreadCommand::Ping(resp)) => {
                resp.send(true).unwrap();
            }

            Some(VmThreadCommand::StartVM(desc)) => {
                info!("spawning a vm");

                let CreateVmSuccess {
                    id,
                    future,
                    timeout_handle,
                } = T::create_vm(desc, self.isolate_cell.clone()).unwrap();

                self.vms.push(VmContext {
                    run_future: future,
                    handle: VmHandle { id, timeout_handle },
                });
            }
        }
    }

    // runaway script detection ensures that no single vm can
    // block the thread for more than the allowed interval
    //
    // to do so we simply track the running vm and send pings to the thread
    // if we get no pong back within the allowed interval, then we have a problem
    async fn runaway_checker(handle: VmThreadHandle<T>) {
        let ping_interval = Duration::from_secs(10);
        loop {
            let (send, rcv) = oneshot::channel();
            match handle.send_cmd.send(VmThreadCommand::Ping(send)) {
                Ok(_) => {
                    let last_ping = Instant::now();
                    match tokio::time::timeout(ping_interval, rcv).await {
                        Ok(_) => {
                            // sleep until the next ping
                            let remaining = ping_interval - last_ping.elapsed();
                            tokio::time::sleep(remaining).await;
                        }
                        Err(_) => {
                            // we hit a timeout, meaning there's a runaway script
                            //
                            // note that this logic is currently very flawed,
                            // you could make a vm that takes a 1 less millisecond than the ping interval
                            // then the next vm takes move than 1 millisecond and this logic
                            // will think the cause is the latter even though it was only responsible for 1ms
                            //
                            // in the future we will have to actively track the cpu usage of vm's to shut down the proper vm
                            // if there's a bad actor
                            let maybe_handle = handle.running_vm.read().unwrap();
                            match &*maybe_handle {
                                Some(h) => T::shutdown_runaway(&h.timeout_handle),
                                None => {
                                    // This could be possible during very high load scenarios where the thread never gets
                                    // cpu time form something else on the system, so for now we just ignore this case
                                    // we will deal with this properly when we add proper cpu time tracking
                                    info!(
                                        "no pong received and no active isolate? there be bugs..."
                                    );
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // receiver was dropped meaning the thread has shut downw
                    return;
                }
            }
        }
    }
}

impl<T: VmInterface + 'static> Future for VmThreadFuture<T>
where
    T::Future: Unpin,
{
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // TODO:
        //
        // Intelligently keep track of what vm's called cx.waker() and only poll those
        //
        // There is a potential bottleneck here with a lot of vms, the above would help that
        while let Poll::Ready(cmd) = self.rcv_cmd.poll_recv(cx) {
            self.handle_cmd(cmd);
        }

        let running_handle = self.running_vm.clone();

        // Poll the vm futures while removing finished ones
        let to_remove = self
            .vms
            .iter_mut()
            .enumerate()
            .filter_map(|(index, vm)| {
                // update the running vm
                set_running_vm(&*running_handle, Some(vm.handle.clone()));

                // poll the vm future, continuing evaluation of javascript
                match Pin::new(&mut vm.run_future).poll(cx) {
                    Poll::Ready(_) => Some(index),
                    Poll::Pending => None,
                }
            })
            // reverse so that we remove the largest index first
            // because otherwise we would have to subtract 1 from the index
            // for eah remove, its simpler to just do it in reverse and not worry about it
            .rev()
            .collect::<Vec<_>>();

        set_running_vm(&*running_handle, None);

        for index in to_remove {
            self.vms.remove(index);
        }

        Poll::Pending
    }
}

fn set_running_vm<T>(slot: &RwLock<Option<T>>, new_val: Option<T>) {
    let mut handle = slot.write().unwrap();
    *handle = new_val;
}

/// A handle to the thread, this is `Send` and `Sync`
pub struct VmThreadHandle<T: VmInterface> {
    pub send_cmd: UnboundedSender<VmThreadCommand<T::BuildDesc>>,
    running_vm: RunningVmTimeout<T::VmId, T::TimeoutHandle>,
}

impl<T: VmInterface> Clone for VmThreadHandle<T> {
    fn clone(&self) -> Self {
        Self {
            send_cmd: self.send_cmd.clone(),
            running_vm: self.running_vm.clone(),
        }
    }
}

/// A running vm
pub struct VmContext<T, U, V>
where
    T: Future,
    V: Display,
{
    run_future: T,
    handle: VmHandle<V, U>,
}

/// A handle to a running vm
#[derive(Clone)]
struct VmHandle<T: Display, U> {
    id: T,
    timeout_handle: U,
}

/// This defines the actual implementation for running vms
pub trait VmInterface {
    /// gets passed to create_vm to create vms
    type BuildDesc: Send;

    /// the future for running vms
    /// should only complete when the vm has shut down
    type Future: Future;

    /// the type for the vm ID's
    type VmId: Display + Send + Sync + Clone + Unpin;

    /// this should create a vm and return a future that can be polled to drive it
    fn create_vm(
        b: Self::BuildDesc,
        cell: Rc<IsolateCell>,
    ) -> VmCreateResult<Self::VmId, Self::Future, Self::TimeoutHandle>;

    // this is a handle that can contain for example a v8 isolate handle to interrupt execution
    // this is to stop runaway scripts.
    type TimeoutHandle: Send + Sync + Unpin + Clone;

    // shut down a runaway vm using the provided timeout handle
    // note that this means the vm thread is  currently busy executing the future
    fn shutdown_runaway(shutdown_handle: &Self::TimeoutHandle);
}

pub trait VmFactory {
    type BuildDesc: Send;

    /// the future for running vms
    /// should only complete when the vm has shut down
    type Future: Future;

    /// the type for the vm ID's
    type VmId: Display + Send + Sync + Clone + Unpin;

    type TimeoutHandle: Send + Sync + Unpin + Clone;

    fn create_vm(
        b: Self::BuildDesc,
    ) -> VmCreateResult<Self::VmId, Self::Future, Self::TimeoutHandle>;
}

pub type VmCreateResult<T, U, V> = Result<CreateVmSuccess<T, U, V>, String>;

pub struct CreateVmSuccess<T, U, V> {
    pub id: T,
    pub future: U,
    pub timeout_handle: V,
}
