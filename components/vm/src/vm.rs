use crate::error::create_error_fn;
use crate::moduleloader::{ModuleEntry, ModuleManager};
use crate::{prepend_script_source_header, AnyError, JsValue, ScriptLoad};
use anyhow::anyhow;
use deno_core::{op_async, Extension, OpState, RuntimeOptions, Snapshot};
use futures::{future::LocalBoxFuture, FutureExt};
use guild_logger::{GuildLogger, LogEntry};
use isolatecell::{IsolateCell, ManagedIsolate};
use serde::Serialize;
use std::pin::Pin;
use std::{
    cell::RefCell,
    fmt::Display,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, RwLock as StdRwLock},
    task::{Context, Poll, Wake, Waker},
};
use stores::config::Script;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{info, instrument};
use twilight_model::id::GuildId;
use url::Url;
use v8::{CreateParams, HeapStatistics, IsolateHandle};
use vmthread::{CreateVmSuccess, ShutdownReason, VmInterface};

#[derive(Debug, Clone)]
pub enum VmCommand {
    DispatchEvent(&'static str, serde_json::Value),
    LoadScript(Script),

    // note that this also reloads the runtime, shutting it down and starting it again
    // we send a message when that has been accomplished
    UnloadScripts(Vec<Script>),
    UpdateScript(Script),
    Terminate,
    Restart(Vec<Script>),
}

#[derive(Debug)]
pub enum VmEvent {
    Shutdown(ShutdownReason),
}

#[derive(Clone, Copy, Debug)]
pub enum VmRole {
    Main,
    Pack(u64),
}

pub type GuildVmEvent = (GuildId, VmRole, VmEvent);

#[derive(Serialize)]
struct ScriptDispatchData {
    name: String,
    data: serde_json::Value,
}

pub struct Vm {
    ctx: VmContext,
    runtime: ManagedIsolate,

    rx: UnboundedReceiver<VmCommand>,
    tx: UnboundedSender<GuildVmEvent>,

    loaded_scripts: Rc<RefCell<Vec<ScriptLoad>>>,
    failed_scripts: Vec<ScriptLoad>,

    timeout_handle: ShutdownHandle,
    guild_logger: GuildLogger,

    isolate_cell: Rc<IsolateCell>,

    extension_factory: ExtensionFactory,
    module_manager: Rc<ModuleManager>,

    script_dispatch_tx: UnboundedSender<ScriptDispatchData>,
    wakeup_rx: UnboundedReceiver<()>,
}

#[derive(Debug, Clone)]
pub struct VmContext {
    pub guild_id: GuildId,
    pub role: VmRole,
}

pub struct CoreCtxData {
    rcv_events: Option<UnboundedReceiver<ScriptDispatchData>>,
}

impl Vm {
    async fn new(
        create_req: CreateRt,
        timeout_handle: ShutdownHandle,
        isolate_cell: Rc<IsolateCell>,
        wakeup_rx: UnboundedReceiver<()>,
    ) -> Self {
        let module_manager = Rc::new(ModuleManager {
            module_map: create_req.extension_modules,
        });

        let (script_dispatch_tx, script_dispatch_rx) = mpsc::unbounded_channel();

        let loaded_scripts = Rc::new(RefCell::new(vec![]));
        let scripts_store = super::LoadedScriptsStore {
            loaded_scripts: loaded_scripts.clone(),
        };

        let sandbox = Self::create_isolate(
            &create_req.extension_factory,
            module_manager.clone(),
            script_dispatch_rx,
            create_error_fn(scripts_store.clone()),
            scripts_store,
        );

        let mut rt = Self {
            guild_logger: create_req.guild_logger,
            ctx: create_req.ctx,
            rx: create_req.rx,
            tx: create_req.tx,
            loaded_scripts,
            failed_scripts: vec![],

            script_dispatch_tx,
            timeout_handle,
            isolate_cell,
            runtime: sandbox,
            extension_factory: create_req.extension_factory,
            module_manager,
            wakeup_rx,
        };

        rt.emit_isolate_handle();

        for script in create_req.load_scripts {
            rt.load_script(script).await
        }

        rt
    }

    fn create_isolate(
        extension_factory: &ExtensionFactory,
        module_manager: Rc<ModuleManager>,
        evt_rx: UnboundedReceiver<ScriptDispatchData>,
        create_err_fn: Rc<deno_core::JsErrorCreateFn>,
        script_load_states: super::LoadedScriptsStore,
    ) -> ManagedIsolate {
        let mut extensions = extension_factory();
        extensions.insert(
            0,
            Extension::builder()
                .ops(vec![("op_botloader_rcv_event", op_async(op_rcv_event))])
                .state(move |op| {
                    op.put(script_load_states.clone());
                    Ok(())
                })
                .build(),
        );

        let options = RuntimeOptions {
            extensions,
            module_loader: Some(module_manager),
            // yeah i have no idea what these values needs to be aligned to, but this seems to work so whatever
            // if it breaks when you update deno or v8 try different values until it works, if only they'd document the alignment requirements somewhere...
            create_params: Some(CreateParams::default().heap_limits(512 * 1024, 20 * 512 * 1024)),
            startup_snapshot: Some(Snapshot::Static(crate::BOTLOADER_CORE_SNAPSHOT)),
            js_error_create_fn: Some(create_err_fn),
            ..Default::default()
        };

        ManagedIsolate::new_with_state(
            options,
            CoreCtxData {
                rcv_events: Some(evt_rx),
            },
        )
    }

    fn emit_isolate_handle(&mut self) {
        let handle = {
            let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);
            rt.v8_isolate().thread_safe_handle()
        };

        let mut th = self.timeout_handle.inner.write().unwrap();
        th.isolate_handle = Some(handle);
    }

    pub async fn run(&mut self) {
        self.emit_isolate_handle();

        info!("rt {} running runtime", self.ctx.guild_id);
        self.guild_logger.log(LogEntry::info(
            self.ctx.guild_id,
            "starting guild vm".to_string(),
        ));

        while !self.check_terminated() {
            let fut = TickFuture {
                rx: &mut self.rx,
                rt: &mut self.runtime,
                cell: &self.isolate_cell,
                wakeup: &mut self.wakeup_rx,
            };

            match fut.await {
                Ok(Some(cmd)) => self.handle_cmd(cmd).await,
                Ok(None) => {}
                Err(e) => {
                    self.guild_logger.log(LogEntry::error(
                        self.ctx.guild_id,
                        format!("Script error occured: {}", e),
                    ));
                }
            }
        }

        info!("terminating runtime for guild {}", self.ctx.guild_id);

        let shutdown_reason = {
            self.timeout_handle
                .inner
                .read()
                .unwrap()
                .shutdown_reason
                .clone()
        };

        if let Some(ShutdownReason::ThreadTermination) = shutdown_reason {
            info!("running vm until completion...");
            // cleanly finish the futures
            self.stop_vm().await;
            info!("done running vm until completion!");
        }

        self.tx
            .send((
                self.ctx.guild_id,
                self.ctx.role,
                VmEvent::Shutdown(if let Some(reason) = shutdown_reason {
                    reason
                } else {
                    ShutdownReason::Unknown
                }),
            ))
            .unwrap();
    }

    fn check_terminated(&mut self) -> bool {
        self.timeout_handle
            .terminated
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn handle_cmd(&mut self, cmd: VmCommand) {
        match cmd {
            VmCommand::Terminate => todo!(),
            VmCommand::Restart(new_scripts) => {
                self.restart(new_scripts).await;
            }
            VmCommand::DispatchEvent(name, evt) => self.dispatch_event(name, &evt),
            VmCommand::LoadScript(script) => self.load_script(script).await,

            VmCommand::UpdateScript(script) => {
                let mut cloned_scripts = self
                    .loaded_scripts
                    .borrow()
                    .clone()
                    .into_iter()
                    .map(|v| v.inner)
                    .collect::<Vec<_>>();

                let mut need_reset = false;
                for old in &mut cloned_scripts {
                    if old.id == script.id {
                        *old = script.clone();
                        need_reset = true;
                    }
                }

                if need_reset {
                    self.restart(cloned_scripts).await;
                }
            }
            VmCommand::UnloadScripts(scripts) => {
                let new_scripts = self
                    .loaded_scripts
                    .borrow()
                    .iter()
                    .filter_map(|sc| {
                        if !scripts.iter().any(|isc| isc.id == sc.inner.id) {
                            Some(sc.inner.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                self.restart(new_scripts).await;
            }
        }
    }

    #[instrument(skip(self, script))]
    fn compile_script(&self, script: Script) -> Option<ScriptLoad> {
        match tscompiler::compile_typescript(&script.original_source) {
            Ok(compiled) => Some(ScriptLoad {
                compiled,
                inner: script,
            }),
            Err(e) => {
                self.guild_logger.log(LogEntry::error(
                    self.ctx.guild_id,
                    format!("Script compilation failed for {}.ts: {}", script.name, e),
                ));
                None
            }
        }
    }

    #[instrument(skip(self, script))]
    async fn load_script(&mut self, script: Script) {
        if self
            .loaded_scripts
            .borrow()
            .iter()
            .any(|sc| sc.inner.id == script.id)
        {
            info!("script: {} was already loaded, skipping", script.id);
            return;
        }

        let compiled = if let Some(compiled) = self.compile_script(script) {
            compiled
        } else {
            return;
        };

        self.loaded_scripts.borrow_mut().push(compiled.clone());

        let eval_res = {
            let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);

            let parsed_uri =
                Url::parse(format!("file:///guild_scripts/{}.js", compiled.inner.name).as_str())
                    .unwrap();

            let fut = rt.load_side_module(
                &parsed_uri,
                Some(prepend_script_source_header(
                    &compiled.compiled.output,
                    Some(&compiled.inner),
                )),
            );

            // Yes this is very hacky, we should have a proper solution for this at some point.
            //
            // Why is this needed? because we can't hold the IsolateGuard across an await
            // this future should resolve instantly because our module loader has no awaits in it
            // and does no io.
            //
            // this might very well break in the future when we update to a newer version of deno
            // but hopefully it's caught before production.
            let res = {
                let mut pinned = Box::pin(fut);
                let waker: Waker = Arc::new(NoOpWaker).into();
                let mut cx = Context::from_waker(&waker);
                match pinned.poll_unpin(&mut cx) {
                    Poll::Pending => panic!("Future should resolve instantly!"),
                    Poll::Ready(v) => v,
                }
            };

            res.map(|id| rt.mod_evaluate(id))
        };

        match eval_res {
            Err(e) => {
                self.log_guild_err(e);
                self.failed_scripts.push(compiled)
            }
            Ok(rcv) => {
                self.complete_module_eval(rcv).await;
            }
        }
    }

    fn dispatch_event<P>(&mut self, name: &str, args: &P)
    where
        P: Serialize,
    {
        let loaded = self.loaded_scripts.borrow().len() - self.failed_scripts.len();
        if loaded < 1 {
            return;
        }

        // self._dump_heap_stats();
        info!("rt {} dispatching event: {}", self.ctx.guild_id, name);
        let serialized = serde_json::to_value(args).unwrap();
        self.script_dispatch_tx
            .send(ScriptDispatchData {
                name: name.to_string(),
                data: serialized,
            })
            .ok();
        // self._dump_heap_stats();
    }

    fn _dump_heap_stats(&mut self) {
        let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);
        let iso = rt.v8_isolate();
        let mut stats = HeapStatistics::default();
        iso.get_heap_statistics(&mut stats);
        dbg!(stats.total_heap_size());
        dbg!(stats.total_heap_size_executable());
        dbg!(stats.total_physical_size());
        dbg!(stats.total_available_size());
        dbg!(stats.total_global_handles_size());
        dbg!(stats.used_global_handles_size());
        dbg!(stats.used_heap_size());
        dbg!(stats.heap_size_limit());
        dbg!(stats.malloced_memory());
        dbg!(stats.external_memory());

        let policy = iso.get_microtasks_policy();
        dbg!(policy);
        // iso.low_memory_notification();
    }

    async fn stop_vm(&mut self) -> UnboundedReceiver<ScriptDispatchData> {
        info!("rt {} stopping sandbox", self.ctx.guild_id,);

        // take the rcv event channel, this way we keep the queued up events
        let evt_rx = {
            let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);
            let op_state_rc = rt.op_state();
            let mut op_state = op_state_rc.borrow_mut();
            let core = op_state.borrow_mut::<CoreCtxData>();
            core.rcv_events.take().unwrap()
        };

        // wake up the event loop, wihtout this script evt rx wont be polled and it will effectively hang
        self.script_dispatch_tx
            .send(ScriptDispatchData {
                name: "NOOP".to_string(),
                data: serde_json::Value::Null,
            })
            .ok();

        // complete the event loop and extract our core data (script event receiver)
        // TODO: we could potentially have some long running futures
        // so maybe call a function that cancels all long running futures or something?
        // or at the very least have a timeout?
        {
            self.run_until_completion().await;
        }

        evt_rx
    }

    async fn run_until_completion(&mut self) {
        let fut = RunUntilCompletion {
            cell: &self.isolate_cell,
            rt: &mut self.runtime,
        };

        if let Err(err) = fut.await {
            self.log_guild_err(err);
        }
    }

    async fn complete_module_eval(
        &mut self,
        rcv: futures::channel::oneshot::Receiver<Result<(), AnyError>>,
    ) {
        let fut = CompleteModuleEval {
            cell: &self.isolate_cell,
            rt: &mut self.runtime,
            rcv,
        };

        if let Err(err) = fut.await {
            self.log_guild_err(err);
        }
    }

    fn log_guild_err(&self, err: AnyError) {
        self.guild_logger.log(LogEntry::error(
            self.ctx.guild_id,
            format!("Script error occured: {}", err),
        ));
    }

    async fn restart(&mut self, new_scripts: Vec<Script>) {
        info!("rt {} restarting with new scripts", self.ctx.guild_id,);
        self.guild_logger.log(LogEntry::info(
            self.ctx.guild_id,
            "restarting guild vm with new scripts".to_string(),
        ));

        let core_data = self.stop_vm().await;

        // create a new sandbox
        let scripts_store = super::LoadedScriptsStore {
            loaded_scripts: self.loaded_scripts.clone(),
        };

        let new_rt = Self::create_isolate(
            &self.extension_factory,
            self.module_manager.clone(),
            core_data,
            create_error_fn(scripts_store.clone()),
            scripts_store,
        );

        self.runtime = new_rt;
        self.emit_isolate_handle();

        self.loaded_scripts.borrow_mut().clear();
        self.failed_scripts.clear();

        for script in new_scripts {
            self.load_script(script).await;
        }

        self.guild_logger.log(LogEntry::info(
            self.ctx.guild_id,
            "vm restarted".to_string(),
        ));
    }
}

async fn op_rcv_event(
    state: Rc<RefCell<OpState>>,
    _args: (),
    _: (),
) -> Result<ScriptDispatchData, AnyError> {
    let cloned_state = state.clone();
    return futures::future::poll_fn(move |ctx| {
        let mut op_state = cloned_state.borrow_mut();
        let core_data = op_state.borrow_mut::<CoreCtxData>();

        if let Some(rx) = &mut core_data.rcv_events {
            match rx.poll_recv(ctx) {
                Poll::Ready(Some(v)) => Poll::Ready(Ok(v)),
                Poll::Ready(None) => Poll::Ready(Err(anyhow!("no more events!"))),
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(Ok(ScriptDispatchData {
                name: "STOP".to_string(),
                data: JsValue::Null,
            }))
        }
    })
    .await;
}

struct TickFuture<'a> {
    rx: &'a mut UnboundedReceiver<VmCommand>,
    rt: &'a mut ManagedIsolate,
    cell: &'a IsolateCell,
    wakeup: &'a mut UnboundedReceiver<()>,
}

// Future which drives the js event loop while at the same time retrieving commands
impl<'a> core::future::Future for TickFuture<'a> {
    type Output = Result<Option<VmCommand>, AnyError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        {
            let mut rt = self.cell.enter_isolate(self.rt);

            if let Poll::Ready(Err(e)) = rt.poll_event_loop(cx, false) {
                return Poll::Ready(Err(e));
            }
        }

        match self.wakeup.poll_recv(cx) {
            Poll::Ready(_) => return Poll::Ready(Ok(None)),
            Poll::Pending => {}
        }

        match self.rx.poll_recv(cx) {
            Poll::Ready(opt) => Poll::Ready(Ok(opt)),
            _ => Poll::Pending,
        }
    }
}

// future that drives the vm to completion, acquiring the isolate guard when needed
struct RunUntilCompletion<'a> {
    rt: &'a mut ManagedIsolate,
    cell: &'a IsolateCell,
}

impl<'a> core::future::Future for RunUntilCompletion<'a> {
    type Output = Result<(), AnyError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut rt = self.cell.enter_isolate(self.rt);

        match rt.poll_event_loop(cx, false) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }
}

// future that drives the vm to completion, acquiring the isolate guard when needed
struct CompleteModuleEval<'a> {
    rt: &'a mut ManagedIsolate,
    cell: &'a IsolateCell,
    rcv: futures::channel::oneshot::Receiver<Result<(), AnyError>>,
}

impl<'a> core::future::Future for CompleteModuleEval<'a> {
    type Output = Result<(), AnyError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let pinned = Pin::new(&mut self.rcv);
        match pinned.poll(cx) {
            Poll::Ready(_) => return Poll::Ready(Ok(())),
            Poll::Pending => {}
        }

        let mut rt = self.cell.enter_isolate(self.rt);

        match rt.poll_event_loop(cx, false) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }
}

#[derive(Serialize)]
struct JackRTEvent<T: Serialize> {
    name: String,
    data: T,
}

impl VmInterface for Vm {
    type BuildDesc = CreateRt;

    type Future = LocalBoxFuture<'static, ()>;

    type VmId = RtId;

    fn create_vm(
        b: Self::BuildDesc,
        isolate_cell: Rc<IsolateCell>,
    ) -> vmthread::VmCreateResult<Self::VmId, Self::Future, Self::ShutdownHandle> {
        let (wakeup_tx, wakeup_rx) = mpsc::unbounded_channel();
        let shutdown_handle = ShutdownHandle {
            terminated: Arc::new(AtomicBool::new(false)),
            inner: Arc::new(StdRwLock::new(ShutdownHandleInner {
                isolate_handle: None,
                shutdown_reason: None,
            })),
            wakeup: wakeup_tx,
        };
        let id = RtId {
            guild_id: b.ctx.guild_id,
            role: b.ctx.role,
        };

        let thandle_clone = shutdown_handle.clone();
        let fut = Box::pin(async move {
            let mut rt = Vm::new(b, thandle_clone, isolate_cell, wakeup_rx).await;
            rt.run().await;
        });

        Ok(CreateVmSuccess {
            future: fut,
            id,
            shutdown_handle,
        })
    }

    type ShutdownHandle = ShutdownHandle;

    fn shutdown(shutdown_handle: &Self::ShutdownHandle, reason: ShutdownReason) {
        let mut inner = shutdown_handle.inner.write().unwrap();
        inner.shutdown_reason = Some(reason);
        if let Some(iso_handle) = &inner.isolate_handle {
            shutdown_handle
                .terminated
                .store(true, std::sync::atomic::Ordering::SeqCst);
            iso_handle.terminate_execution();
        } else {
            inner.shutdown_reason = None;
        }

        // trigger a shutdown check if we weren't in the js runtime
        shutdown_handle.wakeup.send(()).ok();
    }
}

#[derive(Clone)]
pub struct ShutdownHandle {
    terminated: Arc<AtomicBool>,
    inner: Arc<StdRwLock<ShutdownHandleInner>>,
    wakeup: mpsc::UnboundedSender<()>,
}

struct ShutdownHandleInner {
    shutdown_reason: Option<ShutdownReason>,
    isolate_handle: Option<IsolateHandle>,
}

pub struct CreateRt {
    pub guild_logger: GuildLogger,
    pub rx: UnboundedReceiver<VmCommand>,
    pub tx: UnboundedSender<GuildVmEvent>,
    pub ctx: VmContext,
    pub load_scripts: Vec<Script>,
    pub extension_factory: ExtensionFactory,
    pub extension_modules: Vec<ModuleEntry>,
}

type ExtensionFactory = Box<dyn Fn() -> Vec<Extension> + Send>;

#[derive(Clone)]
pub struct RtId {
    guild_id: GuildId,
    role: VmRole,
}

impl Display for RtId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Isolate (guild_id: {}, role: {:?})",
            self.guild_id, self.role
        ))
    }
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

struct NoOpWaker;

impl Wake for NoOpWaker {
    fn wake(self: Arc<Self>) {}
}
