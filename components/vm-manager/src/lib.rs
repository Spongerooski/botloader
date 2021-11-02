#![doc = include_str!("../README.md")]

use std::{collections::HashMap, sync::Arc};

use stores::config::{ConfigStore, Script};

use runtime::RuntimeContext;
// use swc_common::errors::Diagnostic;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    RwLock,
};
use tracing::info;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Event;
use twilight_model::id::GuildId;
use vm::{
    error_reporter::ErrorReporter,
    vm::{CreateRt, GuildVmEvent, ScriptLoad, Vm, VmCommand, VmContext, VmEvent, VmRole},
    AnyError,
};
use vmthread::{VmThreadCommand, VmThreadFuture, VmThreadHandle};

type GuildMap = HashMap<GuildId, GuildState>;
pub struct InnerManager<CT> {
    guilds: RwLock<GuildMap>,
    worker_thread: VmThreadHandle<Vm>,

    http: Arc<twilight_http::Client>,
    state: Arc<InMemoryCache>,
    config_store: CT,
    rt_evt_tx: UnboundedSender<GuildVmEvent>,
    error_reporter: Arc<dyn ErrorReporter + Send + Sync>,
}

#[derive(Clone)]
pub struct Manager<CT> {
    inner: Arc<InnerManager<CT>>,
}

/// The manager is responsible for managing all the js vm's
impl<CT> Manager<CT>
where
    CT: ConfigStore + Send + 'static + Sync,
{
    pub fn new(
        error_reporter: Arc<dyn ErrorReporter + Send + Sync>,
        twilight_http_client: Arc<twilight_http::Client>,
        state: Arc<InMemoryCache>,
        config_store: CT,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let manager = Manager {
            inner: Arc::new(InnerManager {
                guilds: RwLock::new(Default::default()),
                worker_thread: VmThreadFuture::create(),

                http: twilight_http_client,
                rt_evt_tx: tx,
                error_reporter,
                config_store,
                state,
            }),
        };

        let manager_cloned = manager.clone();
        tokio::spawn(async move {
            manager_cloned.vm_events_rcv(rx).await;
        });

        manager
    }

    pub async fn init_guild(&self, guild_id: GuildId) -> Result<(), String> {
        self.restart_guild_vm(guild_id).await
    }

    pub async fn restart_guild_vm(&self, guild_id: GuildId) -> Result<(), String> {
        let mut guilds = self.inner.guilds.write().await;

        match guilds.get(&guild_id) {
            // already running vm
            Some(&GuildState {
                main_vm: VmState::Running(ref rs),
                ..
            }) => {
                rs.tx.send(VmCommand::Restart).unwrap();
                Ok(())
            }

            // stopped vm, create a new one
            Some(&GuildState {
                main_vm: VmState::Stopped,
                ..
            }) => self.crate_new_guild_rt(&mut guilds, guild_id).await,

            // not tracking this guild yet, create a new state for it
            None => {
                guilds.insert(
                    guild_id,
                    GuildState {
                        id: guild_id,
                        main_vm: VmState::Stopped,
                        pack_vms: Vec::new(),
                        log_subscribers: Vec::new(),
                    },
                );
                self.crate_new_guild_rt(&mut guilds, guild_id).await
            }
        }
    }

    async fn crate_new_guild_rt(
        &self,
        guilds: &mut GuildMap,
        guild_id: GuildId,
    ) -> Result<(), String> {
        let scripts = self
            .inner
            .config_store
            .list_scripts(guild_id)
            .await
            .unwrap();

        // start all the runtimes!
        let to_load = self.filter_load_scripts(guild_id, scripts);

        let (tx, rx) = mpsc::unbounded_channel();

        let rt_ctx = RuntimeContext {
            bot_state: self.inner.state.clone(),
            dapi: self.inner.http.clone(),
            guild_id,
            role: VmRole::Main,
        };

        info!("spawning guild vm for {}", guild_id);
        self.inner
            .worker_thread
            .send_cmd
            .send(VmThreadCommand::StartVM(CreateRt {
                error_reporter: self.inner.clone(),
                rx,
                tx: self.inner.rt_evt_tx.clone(),
                ctx: VmContext {
                    // bot_state: self.inner.shared_state.bot_context.state.clone(),
                    // dapi: self.inner.shared_state.bot_context.http.clone(),
                    guild_id,
                    role: VmRole::Main,
                },
                load_scripts: to_load,
                extension_factory: Box::new(move || {
                    vec![runtime::create_extension(rt_ctx.clone())]
                }),
                extension_modules: runtime::jsmodules::create_module_map(),
            }))
            .map_err(|_| panic!("failed creating vm"))
            .unwrap();

        guilds.insert(
            guild_id,
            GuildState {
                id: guild_id,
                main_vm: VmState::Running(VmRunningState { tx }),
                pack_vms: Vec::new(),
                log_subscribers: Vec::new(),
            },
        );

        Ok(())
    }

    fn filter_load_scripts(&self, guild_id: GuildId, scripts: Vec<Script>) -> Vec<ScriptLoad> {
        scripts
            .into_iter()
            .filter(|e| e.enabled)
            .filter_map(
                |sc| match tscompiler::compile_typescript(&sc.original_source) {
                    Ok(compiled) => Some(ScriptLoad {
                        compiled_js: compiled,
                        inner: sc,
                    }),
                    Err(e) => {
                        self.handle_script_compilcation_errors(guild_id, e);
                        None
                    }
                },
            )
            .collect()
    }

    // This function just loads all the guild scripts as a pack vm and is largely just used for bench marking atm
    pub async fn create_guild_scripts_vm_as_pack(
        &self,
        guild_id: GuildId,
        pack_id: u64,
    ) -> Result<(), String> {
        let scripts = self
            .inner
            .config_store
            .list_scripts(guild_id)
            .await
            .unwrap();

        // start all the runtimes!
        let to_load = self.filter_load_scripts(guild_id, scripts);

        let mut guilds = self.inner.guilds.write().await;
        if let Some(g) = guilds.get_mut(&guild_id) {
            let (tx, rx) = mpsc::unbounded_channel();

            let rt_ctx = RuntimeContext {
                bot_state: self.inner.state.clone(),
                dapi: self.inner.http.clone(),
                guild_id,
                role: VmRole::Main,
            };

            info!("spawning guild vm for {}", guild_id);
            self.inner
                .worker_thread
                .send_cmd
                .send(VmThreadCommand::StartVM(CreateRt {
                    error_reporter: self.inner.clone(),
                    rx,
                    tx: self.inner.rt_evt_tx.clone(),
                    ctx: VmContext {
                        // bot_state: self.inner.shared_state.bot_context.state.clone(),
                        // dapi: self.inner.shared_state.bot_context.http.clone(),
                        guild_id,
                        role: VmRole::Pack(pack_id),
                    },
                    load_scripts: to_load,
                    extension_factory: Box::new(move || {
                        vec![runtime::create_extension(rt_ctx.clone())]
                    }),
                    extension_modules: runtime::jsmodules::create_module_map(),
                }))
                .map_err(|_| panic!("failed creating vm"))
                .unwrap();

            g.pack_vms
                .push((pack_id, VmState::Running(VmRunningState { tx })));
        }
        Ok(())
    }

    async fn vm_events_rcv(&self, mut rx: UnboundedReceiver<GuildVmEvent>) {
        loop {
            if let Some((guild_id, r, evt)) = rx.recv().await {
                self.handle_vm_evt(guild_id, r, evt).await;
            }
        }
    }

    async fn handle_vm_evt(&self, guild_id: GuildId, _vr: VmRole, evt: VmEvent) {
        match evt {
            VmEvent::Shutdown(reason) => {
                self.with_guild_mut(guild_id, |g| {
                    g.main_vm = VmState::Stopped;
                    Ok(())
                })
                .await
                .ok();

                // report the shutdown to the guild
                let err_reporter = self.inner.clone();
                tokio::spawn(async move {
                    err_reporter
                        .report_error(
                            guild_id,
                            format!(
                                "Runtime for your guild has shut down. (use the command `!jack \
                                 startvm` to start again)\nReason: {:?}",
                                reason
                            ),
                        )
                        .await
                        .ok();
                });
            }
        }
    }

    pub async fn detach_all_script(&self, guild_id: GuildId, script_id: u64) -> Result<(), String> {
        self.send_vm_command(
            guild_id,
            VmRole::Main,
            VmCommand::UnloadAllScript(script_id),
        )
        .await
    }

    pub async fn update_script(&self, guild_id: GuildId, script: Script) -> Result<(), String> {
        let compiled = match tscompiler::compile_typescript(&script.original_source) {
            Ok(compiled) => compiled,
            Err(diag) => return Err(self.handle_script_compilcation_errors(guild_id, diag)),
        };

        self.send_vm_command(
            guild_id,
            VmRole::Main,
            VmCommand::UpdateScript(ScriptLoad {
                inner: script,
                compiled_js: compiled,
            }),
        )
        .await
    }

    pub async fn unload_scripts(
        &self,
        guild_id: GuildId,
        scripts: Vec<Script>,
    ) -> Result<(), String> {
        self.send_vm_command(guild_id, VmRole::Main, VmCommand::UnloadScripts(scripts))
            .await
    }

    pub async fn load_script(&self, guild_id: GuildId, script: Script) -> Result<(), String> {
        let compiled = match tscompiler::compile_typescript(&script.original_source) {
            Ok(compiled) => compiled,
            Err(diag) => return Err(self.handle_script_compilcation_errors(guild_id, diag)),
        };

        self.send_vm_command(
            guild_id,
            VmRole::Main,
            VmCommand::LoadScript(ScriptLoad {
                inner: script,
                compiled_js: compiled,
            }),
        )
        .await
    }

    fn handle_script_compilcation_errors(&self, guild_id: GuildId, msg: String) -> String {
        let cloned = msg.clone();
        let error_reporter = self.inner.error_reporter.clone();
        tokio::spawn(async move {
            error_reporter
                .report_error(guild_id, format!("script compilation failed: {}", msg))
                .await
        });

        cloned
    }

    pub async fn handle_discord_event(&self, evt: Event) {
        let dispatch = runtime::dispatchevents::discord_event_to_dispatch(evt);
        if let Some(inner) = dispatch {
            self.broadcast_vm_command(
                inner.guild_id,
                VmCommand::DispatchEvent(inner.name, inner.data),
            )
            .await
            .ok();
        }
    }

    async fn send_vm_command(
        &self,
        guild_id: GuildId,
        vmt: VmRole,
        cmd: VmCommand,
    ) -> Result<(), String> {
        self.with_running_vm(guild_id, vmt, |rs| {
            rs.tx.send(cmd).unwrap();
            Ok(())
        })
        .await
    }

    async fn broadcast_vm_command(&self, guild_id: GuildId, cmd: VmCommand) -> Result<(), String> {
        self.with_guild(guild_id, |g| {
            for vm in g.iter() {
                if let VmState::Running(rs) = vm {
                    rs.tx.send(cmd.clone()).unwrap();
                }
            }
            Ok(())
        })
        .await
    }

    async fn with_guild_mut<F>(&self, guild_id: GuildId, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut GuildState) -> Result<(), String>,
    {
        let mut guilds = self.inner.guilds.write().await;
        match guilds.get_mut(&guild_id) {
            Some(gs) => f(gs),
            None => Err("Unknown guild".to_string()),
        }
    }

    async fn with_guild<F>(&self, guild_id: GuildId, f: F) -> Result<(), String>
    where
        F: FnOnce(&GuildState) -> Result<(), String>,
    {
        let guilds = self.inner.guilds.read().await;
        match guilds.get(&guild_id) {
            Some(gs) => f(gs),
            None => Err("unknown guild".to_string()),
        }
    }

    async fn with_running_vm<F>(&self, guild_id: GuildId, vmt: VmRole, f: F) -> Result<(), String>
    where
        F: FnOnce(&VmRunningState) -> Result<(), String>,
    {
        self.with_guild(guild_id, |g| {
            let vm = match vmt {
                VmRole::Main => &g.main_vm,
                VmRole::Pack(id) => match g.get_pack_vm(id) {
                    Some(vm) => &vm.1,
                    None => return Err("pack not found".to_string()),
                },
            };

            match vm {
                VmState::Running(ref rs) => f(rs),
                VmState::Stopped => Err("vm not running".to_string()),
            }
        })
        .await
    }

    async fn with_running_vm_mut<F>(
        &self,
        guild_id: GuildId,
        vmt: VmRole,
        f: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut VmRunningState) -> Result<(), String>,
    {
        self.with_guild_mut(guild_id, |g| {
            let vm = match vmt {
                VmRole::Main => &mut g.main_vm,
                VmRole::Pack(id) => match g.get_pack_vm_mut(id) {
                    Some(vm) => &mut vm.1,
                    None => return Err("pack not found".to_string()),
                },
            };

            match vm {
                VmState::Running(ref mut rs) => f(rs),
                VmState::Stopped => Err("vm not running".to_string()),
            }
        })
        .await
    }

    pub async fn subscribe_to_guild_logs(
        &self,
        guild_id: GuildId,
    ) -> Option<UnboundedReceiver<String>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut guilds = self.inner.guilds.write().await;
        if let Some(guild) = guilds.get_mut(&guild_id) {
            guild.log_subscribers.push(tx);
            Some(rx)
        } else {
            None
        }
    }
}

struct GuildState {
    id: GuildId,
    main_vm: VmState,
    pack_vms: Vec<(u64, VmState)>,
    log_subscribers: Vec<UnboundedSender<String>>,
}

impl GuildState {
    fn get_pack_vm(&self, id: u64) -> Option<&(u64, VmState)> {
        self.pack_vms.iter().find(|(vid, _)| *vid == id)
    }
    fn get_pack_vm_mut(&mut self, id: u64) -> Option<&mut (u64, VmState)> {
        self.pack_vms.iter_mut().find(|(vid, _)| *vid == id)
    }

    fn iter(&self) -> impl Iterator<Item = &VmState> {
        GuildStateIter(self, 0)
    }

    // fn broadcast_command()
}

struct GuildStateIter<'a>(&'a GuildState, usize);

impl<'a> Iterator for GuildStateIter<'a> {
    type Item = &'a VmState;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 == 0 {
            self.1 += 1;
            Some(&self.0.main_vm)
        } else if self.0.pack_vms.len() > self.1 - 1 {
            let index = self.1 - 1;
            self.1 += 1;
            Some(&self.0.pack_vms[index].1)
        } else {
            None
        }
    }
}

enum VmState {
    Stopped,
    Running(VmRunningState),
}

/// The state of a vm, the details are set by an event from the runtime so it's set after the fact
struct VmRunningState {
    tx: UnboundedSender<VmCommand>,
}

#[async_trait::async_trait]
impl<CT: Send + Sync> ErrorReporter for InnerManager<CT> {
    async fn report_error(&self, guild_id: GuildId, error: String) -> Result<(), AnyError> {
        {
            let mut guilds = self.guilds.write().await;
            if let Some(guild) = guilds.get_mut(&guild_id) {
                // if the send failed then the subscription is no longer active
                guild
                    .log_subscribers
                    .retain(|gs| gs.send(error.clone()).is_ok());
            }
        }

        self.error_reporter.report_error(guild_id, error).await
    }
}
