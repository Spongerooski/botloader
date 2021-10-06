use configstore::Script;
use configstore::ScriptContext;
use futures::future::LocalBoxFuture;
use rusty_v8::IsolateHandle;
use sandbox::AnyError;
use sandbox::Sandbox;
use serde::Serialize;
use std::fmt::Display;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock as StdRwLock;
use std::task::Poll;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{error, info};
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::gateway;
use twilight_model::gateway::event::Event;
use twilight_model::id::GuildId;
use vmthread::CreateVmSuccess;
use vmthread::VmInterface;

use crate::commonmodels;
use crate::error_reporter::ErrorReporter;
use crate::prepend_script_source_header;
use crate::ContextScript;
use crate::ModuleNamer;

pub type ContextScriptId = (u64, ScriptContext);

#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    HandleEvent(Box<gateway::event::Event>),
    LoadScriptContext(ContextScript),

    // note that this also reloads the runtime, shutting it down and starting it again
    // we send a message when that has been accomplished
    UnloadScripts(Vec<ContextScriptId>),
    UnloadAllScript(u64),
    UpdateScript(Script),
    Terminate,
    Restart,
}

#[derive(Debug)]
pub enum RuntimeEvent {
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

pub type GuildRuntimeEvent = (GuildId, VmRole, RuntimeEvent);

pub struct Runtime {
    ctx: RuntimeContext,
    sandbox: Option<Sandbox>,

    rx: UnboundedReceiver<RuntimeCommand>,
    tx: UnboundedSender<GuildRuntimeEvent>,

    loaded_scripts: Vec<ContextScript>,

    timeout_handle: TimeoutHandle,
    error_reporter: Arc<dyn ErrorReporter>,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub guild_id: GuildId,
    pub bot_state: InMemoryCache,
    pub dapi: twilight_http::Client,
    pub role: VmRole,
}

impl Runtime {
    pub fn sandbox_ref(&mut self) -> &mut Sandbox {
        self.sandbox
            .as_mut()
            .expect("if you get this error someone messed up very badly")
    }

    async fn new(create_req: CreateRt, timeout_handle: TimeoutHandle) -> Self {
        let mut sandbox = Self::create_sandbox().await;
        sandbox.add_state_data(create_req.ctx.clone());

        let mut rt = Self {
            error_reporter: create_req.error_reporter,
            ctx: create_req.ctx,
            rx: create_req.rx,
            tx: create_req.tx,
            sandbox: Some(sandbox),
            loaded_scripts: vec![],
            timeout_handle,
        };

        rt.emit_isolate_handle();

        for script in create_req.load_scripts {
            rt.load_script(script).await
        }

        rt
    }

    async fn create_sandbox() -> Sandbox {
        let extension = crate::jsextensions::init();
        let mut sandbox = Sandbox::new(vec![extension]);
        crate::jsmodules::load_core_modules(&mut sandbox).await;
        sandbox
    }

    fn emit_isolate_handle(&mut self) {
        if let Some(sbox) = &mut self.sandbox {
            let isolate = sbox.runtime.v8_isolate();
            let handle = isolate.thread_safe_handle();

            let mut th = self.timeout_handle.inner.write().unwrap();
            th.isolate_handle = Some(handle);
        }
    }

    pub async fn run(&mut self) {
        self.emit_isolate_handle();

        info!("rt {} running runtime", self.ctx.guild_id);
        while !self.check_terminated() {
            let fut = TickFuture {
                rx: &mut self.rx,
                sandbox: &mut self.sandbox.as_mut().unwrap(),
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
                RuntimeEvent::Shutdown(if let Some(reason) = shutdown_reason {
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

    async fn handle_cmd(&mut self, cmd: RuntimeCommand) {
        match cmd {
            RuntimeCommand::Terminate => todo!(),
            RuntimeCommand::Restart => self.reset_sandbox().await,
            RuntimeCommand::HandleEvent(evt) => self.handle_discord_event(*evt),
            RuntimeCommand::LoadScriptContext(script) => self.load_script(script).await,
            RuntimeCommand::UnloadScripts(scripts) => {
                self.unload_scripts(scripts).await;
            }
            RuntimeCommand::UnloadAllScript(id) => {
                let to_unload = self
                    .loaded_scripts
                    .iter()
                    .filter(|e| e.0.id == id)
                    .map(|e| (e.0.id, e.1.clone()))
                    .collect::<Vec<_>>();

                self.unload_scripts(to_unload).await;
            }

            RuntimeCommand::UpdateScript(script) => {
                let mut need_reset = false;
                for (old, _) in &mut self.loaded_scripts {
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

    async fn load_script(&mut self, script: ContextScript) {
        info!("rt {} loading script: {}", self.ctx.guild_id, script.0.id);
        if self
            .loaded_scripts
            .iter()
            .any(|e| e.0.id == script.0.id && e.1 == script.1)
        {
            info!(
                "rt {} aborted loading script, duplicate.",
                self.ctx.guild_id
            );
            return;
        }

        if self
            .loaded_scripts
            .iter()
            .any(|(sc, ctx)| sc.id == script.0.id && *ctx == script.1)
        {
            info!(
                "rt {} loading script: {} was already loaded, skipping",
                self.ctx.guild_id, script.0.id
            );

            return;
        }

        self.sandbox_ref()
            .add_eval_module(
                format!(
                    "user/{}/{}/{}",
                    script.0.name,
                    script.0.id,
                    script.1.module_name()
                ),
                prepend_script_source_header(&script.0.compiled_js, Some(&script)),
            )
            .await
            .unwrap();

        self.loaded_scripts.push(script);
    }

    async fn unload_scripts(&mut self, scripts: Vec<ContextScriptId>) {
        info!(
            "rt {} unloading scripts: {}",
            self.ctx.guild_id,
            scripts.len()
        );

        let new_scripts = self
            .loaded_scripts
            .drain(..)
            .filter(|e| scripts.iter().find(|x| e.0.id == x.0 && e.1 == x.1) == None)
            .collect::<Vec<_>>();

        self.loaded_scripts = new_scripts;

        self.reset_sandbox().await;
    }

    fn handle_discord_event(&mut self, evt: Event) {
        match evt {
            Event::MessageCreate(m) => {
                self.dispatch_event("MESSAGE_CREATE", &commonmodels::message::Message::from(m.0))
            }
            Event::MessageUpdate(m) => self.dispatch_event(
                "MESSAGE_UPDATE",
                &commonmodels::messageupdate::MessageUpdate::from(*m),
            ),
            Event::MessageDelete(m) => self.dispatch_event(
                "MESSAGE_DELETE",
                &commonmodels::message::MessageDelete::from(m),
            ),
            _ => {
                todo!();
            }
        }
    }

    fn dispatch_event<P>(&mut self, name: &str, args: &P)
    where
        P: Serialize,
    {
        info!("rt {} dispatching event: {}", self.ctx.guild_id, name);

        match self.sandbox_ref().call(
            "$jackGlobal.disaptchEvent",
            &JackRTEvent {
                name: name.to_string(),
                data: args,
            },
        ) {
            Ok(()) => {}
            Err(e) => {
                error!("failed calling dispatch: {}", e)
            }
        }
    }

    // Simply recreates the vm and loads the scripts in self.loaded_scripts
    async fn reset_sandbox(&mut self) {
        info!("rt {} resetting sandbox", self.ctx.guild_id,);

        // complete the event loop
        // TODO: we could potentially have some long running futures
        // so maybe call a function that cancels all long running futures or something?
        self.sandbox_ref()
            .runtime
            .run_event_loop(false)
            .await
            .unwrap();
        {
            self.sandbox.take();
        }

        // create a new sandbox
        let mut sandbox = Self::create_sandbox().await;
        sandbox.add_state_data(self.ctx.clone());

        self.sandbox = Some(sandbox);
        self.emit_isolate_handle();

        let initial_scripts = self.loaded_scripts.clone();
        self.loaded_scripts = Vec::new();

        for script in initial_scripts {
            self.load_script(script).await;
        }
    }
}

struct TickFuture<'a> {
    rx: &'a mut UnboundedReceiver<RuntimeCommand>,
    sandbox: &'a mut Sandbox,
}

// Future which drives the js event loop while at the same time retrieving commands
impl<'a> core::future::Future for TickFuture<'a> {
    type Output = Result<Option<RuntimeCommand>, AnyError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Poll::Ready(Err(e)) = self.sandbox.runtime.poll_event_loop(cx, false) {
            // error!("Got a error in polling: {}", e)
            return Poll::Ready(Err(e));
        }

        match self.rx.poll_recv(cx) {
            Poll::Ready(opt) => Poll::Ready(Ok(opt)),
            _ => Poll::Pending,
        }
    }
}

#[derive(Serialize)]
struct JackRTEvent<T: Serialize> {
    name: String,
    data: T,
}

impl VmInterface for Runtime {
    type BuildDesc = CreateRt;

    type Future = LocalBoxFuture<'static, ()>;

    type VmId = RtId;

    fn create_vm(
        b: Self::BuildDesc,
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
            let mut rt = Runtime::new(b, thandle_clone).await;
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
        shutdown_handle
            .terminated
            .store(true, std::sync::atomic::Ordering::SeqCst);
        inner.isolate_handle.as_ref().unwrap().terminate_execution();
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
    pub rx: UnboundedReceiver<RuntimeCommand>,
    pub tx: UnboundedSender<GuildRuntimeEvent>,
    pub ctx: RuntimeContext,
    pub load_scripts: Vec<ContextScript>,
}

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
