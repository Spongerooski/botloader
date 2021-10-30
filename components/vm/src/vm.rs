use crate::moduleloader::ModuleEntry;
use crate::moduleloader::ModuleManager;
use crate::AnyError;
use anyhow::anyhow;
use deno_core::op_async;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::RuntimeOptions;
use deno_core::Snapshot;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use isolatecell::IsolateCell;
use isolatecell::ManagedIsolate;
use rusty_v8::IsolateHandle;
use serde::Serialize;
use std::cell::RefCell;
use std::fmt::Display;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock as StdRwLock;
use std::task::Context;
use std::task::Poll;
use std::task::Wake;
use std::task::Waker;
use stores::config::Script;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{error, info};
use twilight_model::id::GuildId;
use url::Url;
use vmthread::CreateVmSuccess;
use vmthread::VmInterface;

use crate::error_reporter::ErrorReporter;
use crate::prepend_script_source_header;

#[derive(Debug, Clone)]
pub enum VmCommand {
    DispatchEvent(&'static str, serde_json::Value),
    LoadScript(Script),

    // note that this also reloads the runtime, shutting it down and starting it again
    // we send a message when that has been accomplished
    UnloadScripts(Vec<Script>),
    UnloadAllScript(u64),
    UpdateScript(Script),
    Terminate,
    Restart,
}

#[derive(Debug)]
pub enum VmEvent {
    Shutdown(ShutdownReason),
}

#[derive(Debug, Clone)]
pub enum ShutdownReason {
    Unknown,
    RunawayScript,
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

    loaded_scripts: Vec<Script>,

    timeout_handle: TimeoutHandle,
    error_reporter: Arc<dyn ErrorReporter>,

    isolate_cell: Rc<IsolateCell>,

    extension_factory: ExtensionFactory,
    module_manager: Rc<ModuleManager>,

    script_dispatch_tx: UnboundedSender<ScriptDispatchData>,
}

#[derive(Debug, Clone)]
pub struct VmContext {
    pub guild_id: GuildId,
    pub role: VmRole,
}

pub struct CoreCtxData {
    rcv_events: UnboundedReceiver<ScriptDispatchData>,
}

impl Vm {
    async fn new(
        create_req: CreateRt,
        timeout_handle: TimeoutHandle,
        isolate_cell: Rc<IsolateCell>,
    ) -> Self {
        let module_manager = Rc::new(ModuleManager {
            module_map: create_req.extension_modules,
        });

        let (script_dispatch_tx, script_dispatch_rx) = mpsc::unbounded_channel();

        let sandbox = Self::create_isolate(
            &create_req.extension_factory,
            module_manager.clone(),
            CoreCtxData {
                rcv_events: script_dispatch_rx,
            },
        )
        .await;
        // sandbox.add_state_data(create_req.ctx.clone());

        let mut rt = Self {
            error_reporter: create_req.error_reporter,
            ctx: create_req.ctx,
            rx: create_req.rx,
            tx: create_req.tx,
            loaded_scripts: vec![],

            script_dispatch_tx,
            timeout_handle,
            isolate_cell,
            runtime: sandbox,
            extension_factory: create_req.extension_factory,
            module_manager,
        };

        rt.emit_isolate_handle();

        for script in create_req.load_scripts {
            rt.load_script(script).await
        }

        rt
    }

    async fn create_isolate(
        extension_factory: &ExtensionFactory,
        module_manager: Rc<ModuleManager>,
        core_data: CoreCtxData,
    ) -> ManagedIsolate {
        let mut extensions = extension_factory();
        extensions.insert(
            0,
            Extension::builder()
                .ops(vec![("op_botloader_rcv_event", op_async(op_rcv_event))])
                .build(),
        );

        let options = RuntimeOptions {
            extensions,
            module_loader: Some(module_manager),
            // create_params: Some(CreateParams::default().heap_limits(64 * 8_000, 64 * 1_000_000)),
            startup_snapshot: Some(Snapshot::Static(crate::BOTLOADER_CORE_SNAPSHOT)),
            ..Default::default()
        };

        ManagedIsolate::new_with_state(options, core_data)
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
        while !self.check_terminated() {
            let fut = TickFuture {
                rx: &mut self.rx,
                rt: &mut self.runtime,
                cell: &self.isolate_cell,
            };

            match fut.await {
                Ok(Some(cmd)) => self.handle_cmd(cmd).await,
                Ok(None) => {}
                Err(e) => {
                    if let Err(e) = self
                        .error_reporter
                        .report_script_error(self.ctx.guild_id, e)
                        .await
                    {
                        error!(err = %e, "failed reporting script error");
                    }
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
            VmCommand::Restart => self.reset_sandbox().await,
            VmCommand::DispatchEvent(name, evt) => self.dispatch_event(name, &evt),
            VmCommand::LoadScript(script) => self.load_script(script).await,
            VmCommand::UnloadScripts(scripts) => {
                self.unload_scripts(scripts.into_iter().map(|e| e.id).collect())
                    .await;
            }
            VmCommand::UnloadAllScript(id) => {
                let to_unload = self
                    .loaded_scripts
                    .iter()
                    .filter(|e| e.id == id)
                    .map(|e| (e.id))
                    .collect::<Vec<_>>();

                self.unload_scripts(to_unload).await;
            }

            VmCommand::UpdateScript(script) => {
                let mut need_reset = false;
                for old in &mut self.loaded_scripts {
                    if old.id == script.id {
                        *old = script.clone();
                        need_reset = true;
                    }
                }

                if need_reset {
                    self.reset_sandbox().await;
                }
            }
        }
    }

    async fn load_script(&mut self, script: Script) {
        info!("rt {} loading script: {}", self.ctx.guild_id, script.id);

        if self.loaded_scripts.iter().any(|sc| sc.id == script.id) {
            info!(
                "rt {} loading script: {} was already loaded, skipping",
                self.ctx.guild_id, script.id
            );

            return;
        }

        {
            let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);

            let parsed_uri =
                Url::parse(format!("file://guild/{}.js", script.name).as_str()).unwrap();

            let fut = rt.load_module(
                &parsed_uri,
                Some(prepend_script_source_header(
                    &script.compiled_js,
                    Some(&script),
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
            let id = {
                let mut pinned = Box::pin(fut);
                let waker: Waker = Arc::new(NoOpWaker).into();
                let mut cx = Context::from_waker(&waker);
                match pinned.poll_unpin(&mut cx) {
                    Poll::Pending => panic!("Future should resolve instantly!"),
                    Poll::Ready(v) => v.unwrap(),
                }
            };

            // TODO: handle error on receiver result
            rt.mod_evaluate(id);
        }

        self.loaded_scripts.push(script);
    }

    async fn unload_scripts(&mut self, scripts: Vec<u64>) {
        info!(
            "rt {} unloading scripts: {}",
            self.ctx.guild_id,
            scripts.len()
        );

        let new_scripts = self
            .loaded_scripts
            .drain(..)
            .filter(|e| scripts.iter().any(|x| e.id == *x))
            .collect::<Vec<_>>();

        self.loaded_scripts = new_scripts;

        self.reset_sandbox().await;
    }

    fn dispatch_event<P>(&mut self, name: &str, args: &P)
    where
        P: Serialize,
    {
        info!("rt {} dispatching event: {}", self.ctx.guild_id, name);
        let serialized = serde_json::to_value(args).unwrap();
        self.script_dispatch_tx
            .send(ScriptDispatchData {
                name: name.to_string(),
                data: serialized,
            })
            .ok();
    }

    // Simply recreates the vm and loads the scripts in self.loaded_scripts
    async fn reset_sandbox(&mut self) {
        info!("rt {} resetting sandbox", self.ctx.guild_id,);

        // TODO: more robust solution for this.
        self.script_dispatch_tx
            .send(ScriptDispatchData {
                name: "STOP".to_string(),
                data: serde_json::Value::Null,
            })
            .ok();

        // complete the event loop and extract our core data (script event receiver)
        // TODO: we could potentially have some long running futures
        // so maybe call a function that cancels all long running futures or something?
        let core_data: CoreCtxData = {
            {
                let fut = RunUntilCompletion {
                    cell: &self.isolate_cell,
                    rt: &mut self.runtime,
                };
                fut.await.ok();
            }

            let mut rt = self.isolate_cell.enter_isolate(&mut self.runtime);
            let op_state = rt.op_state();
            let val = op_state.borrow_mut().take();
            val
        };

        // create a new sandbox
        let new_rt = Self::create_isolate(
            &self.extension_factory,
            self.module_manager.clone(),
            core_data,
        )
        .await;

        self.runtime = new_rt;
        self.emit_isolate_handle();

        let initial_scripts = self.loaded_scripts.clone();
        self.loaded_scripts = Vec::new();

        for script in initial_scripts {
            self.load_script(script).await;
        }
    }
}

async fn op_rcv_event(
    state: Rc<RefCell<OpState>>,
    _args: (),
    _: (),
) -> Result<ScriptDispatchData, AnyError> {
    let cloned_state = state.clone();
    return futures::future::poll_fn(move |ctx| {
        println!("Polling...");

        let mut op_state = cloned_state.borrow_mut();
        let core_data = op_state.borrow_mut::<CoreCtxData>();
        match core_data.rcv_events.poll_recv(ctx) {
            Poll::Ready(Some(v)) => Poll::Ready(Ok(v)),
            Poll::Ready(None) => Poll::Ready(Err(anyhow!("no more events!"))),
            Poll::Pending => Poll::Pending,
        }
    })
    .await;
}

struct TickFuture<'a> {
    rx: &'a mut UnboundedReceiver<VmCommand>,
    rt: &'a mut ManagedIsolate,
    cell: &'a IsolateCell,
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
                // error!("Got a error in polling: {}", e)
                return Poll::Ready(Err(e));
            }
        }

        match self.rx.poll_recv(cx) {
            Poll::Ready(opt) => Poll::Ready(Ok(opt)),
            _ => Poll::Pending,
        }
    }
}
struct RunUntilCompletion<'a> {
    rt: &'a mut ManagedIsolate,
    cell: &'a IsolateCell,
}

// Future which drives the js event loop while at the same time retrieving commands
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
    ) -> vmthread::VmCreateResult<Self::VmId, Self::Future, Self::TimeoutHandle> {
        let timeout_handle = TimeoutHandle {
            terminated: Arc::new(AtomicBool::new(false)),
            inner: Arc::new(StdRwLock::new(TimeoutHandleInner {
                isolate_handle: None,
                shutdown_reason: None,
            })),
        };
        let id = RtId {
            guild_id: b.ctx.guild_id,
            role: b.ctx.role,
        };

        let thandle_clone = timeout_handle.clone();
        let fut = Box::pin(async move {
            let mut rt = Vm::new(b, thandle_clone, isolate_cell).await;
            rt.run().await;
        });

        Ok(CreateVmSuccess {
            future: fut,
            id,
            timeout_handle,
        })
    }

    type TimeoutHandle = TimeoutHandle;

    fn shutdown_runaway(shutdown_handle: &Self::TimeoutHandle) {
        let mut inner = shutdown_handle.inner.write().unwrap();
        inner.shutdown_reason = Some(ShutdownReason::RunawayScript);
        if let Some(iso_handle) = &inner.isolate_handle {
            shutdown_handle
                .terminated
                .store(true, std::sync::atomic::Ordering::SeqCst);
            iso_handle.terminate_execution();
        } else {
            inner.shutdown_reason = None;
        }
    }
}

#[derive(Clone)]
pub struct TimeoutHandle {
    terminated: Arc<AtomicBool>,
    inner: Arc<StdRwLock<TimeoutHandleInner>>,
}

struct TimeoutHandleInner {
    shutdown_reason: Option<ShutdownReason>,
    isolate_handle: Option<IsolateHandle>,
}

pub struct CreateRt {
    pub error_reporter: Arc<dyn ErrorReporter + Send>,
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
