use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, RwLock as StdRwLock},
    time::Duration,
};

use configstore::{ConfigStore, Script, ScriptContext};
use jack_runtime::{
    error_reporter::ErrorReporter,
    runtime::{
        GuildRuntimeEvent, Runtime, RuntimeCommand, RuntimeContext, RuntimeEvent, ShutdownReason,
        VmRole,
    },
    ContextScript, ContextScriptId,
};
use rusty_v8::IsolateHandle;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    RwLock,
};
use tracing::info;
use twilight_gateway::Event;
use twilight_model::id::GuildId;

use crate::BotContext;

pub struct SharedState<CT> {
    bot_context: BotContext<CT>,
    rt_evt_tx: UnboundedSender<GuildRuntimeEvent>,
    error_reporter: Arc<dyn ErrorReporter + Send + Sync>,
}

type GuildMap = HashMap<GuildId, GuildState>;
pub struct InnerManager<CT> {
    shared_state: SharedState<CT>,
    guilds: RwLock<GuildMap>,
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
        bot_context: BotContext<CT>,
        error_reporter: Arc<dyn ErrorReporter + Send + Sync>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let shared = SharedState {
            bot_context,
            error_reporter,
            rt_evt_tx: tx,
        };

        let manager = Manager {
            inner: Arc::new(InnerManager {
                guilds: RwLock::new(Default::default()),
                shared_state: shared,
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
                rs.tx.send(RuntimeCommand::Restart).unwrap();
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
            .shared_state
            .bot_context
            .config_store
            .list_scripts(guild_id)
            .await
            .unwrap();

        let links = self
            .inner
            .shared_state
            .bot_context
            .config_store
            .list_links(guild_id)
            .await
            .unwrap();

        // start all the runtimes!
        // let to_load = links.into_iter().map(|sl| )
        let mut to_load = Vec::new();
        for link in links {
            let script = scripts.iter().find(|e| e.name == link.script_name);
            if let Some(script) = script {
                to_load.push((script.clone(), link.context));
                // self.launch_rt(script.name.clone(), script.contents.clone(), link.context);
            }
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let self_cloned = self.clone();
        std::thread::spawn(move || {
            // TODO: run multiple guilds per thread?
            // It might be very expensive a thread per guild but its the simplest method
            // we have to figure out a better way to share and measure cpu time then
            // probably implementing our own async abstraction layer instead of using the dyno one

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local = tokio::task::LocalSet::new();
            rt.block_on(local.run_until(self_cloned.run_vm(to_load, rx, guild_id)));
        });

        guilds.insert(
            guild_id,
            GuildState {
                id: guild_id,
                main_vm: VmState::Running(VmRunningState {
                    tx,
                    ping_rcvd: true,
                    control: None,
                }),
                pack_vms: vec![],
            },
        );

        let self_cloned = self.clone();
        tokio::spawn(async move {
            self_cloned.loop_ping_guild_vm(guild_id, VmRole::Main).await;
        });

        Ok(())
    }

    async fn vm_events_rcv(&self, mut rx: UnboundedReceiver<GuildRuntimeEvent>) {
        loop {
            if let Some((guild_id, r, evt)) = rx.recv().await {
                self.handle_vm_evt(guild_id, r, evt).await;
            }
        }
    }

    async fn handle_vm_evt(&self, guild_id: GuildId, vr: VmRole, evt: RuntimeEvent) {
        match evt {
            RuntimeEvent::Shutdown(reason) => {
                self.with_guild_mut(guild_id, |g| {
                    g.main_vm = VmState::Stopped;
                    Ok(())
                })
                .await
                .ok();

                // report the shutdown to the guild
                let err_reporter = self.inner.shared_state.error_reporter.clone();
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
            RuntimeEvent::SetIsolate(h, t, l) => {
                self.with_running_vm_mut(guild_id, vr, |rs| {
                    rs.control = Some(VmControl {
                        isolate_handle: h,
                        terminated: t,
                        shutdown_reason: l,
                    });

                    Ok(())
                })
                .await
                .ok();
            }
            RuntimeEvent::Pong => {
                self.with_running_vm_mut(guild_id, vr, |rs| {
                    rs.ping_rcvd = true;
                    Ok(())
                })
                .await
                .ok();
            }
        }
    }

    pub async fn detach_all_script(&self, guild_id: GuildId, script_id: u64) -> Result<(), String> {
        self.send_vm_command(
            guild_id,
            VmRole::Main,
            RuntimeCommand::UnloadAllScript(script_id),
        )
        .await
    }

    pub async fn update_script(&self, guild_id: GuildId, script: Script) -> Result<(), String> {
        self.send_vm_command(guild_id, VmRole::Main, RuntimeCommand::UpdateScript(script))
            .await
    }

    pub async fn detach_scripts(
        &self,
        guild_id: GuildId,
        scripts: Vec<ContextScriptId>,
    ) -> Result<(), String> {
        self.send_vm_command(
            guild_id,
            VmRole::Main,
            RuntimeCommand::UnloadScripts(scripts),
        )
        .await
    }

    pub async fn attach_script(
        &self,
        guild_id: GuildId,
        script: Script,
        script_context: ScriptContext,
    ) -> Result<(), String> {
        self.send_vm_command(
            guild_id,
            VmRole::Main,
            RuntimeCommand::LoadScriptContext((script, script_context)),
        )
        .await
    }

    pub async fn handle_discord_event(&self, evt: Event) {
        if let Some(guild_id) = get_evt_guild_id(&evt) {
            self.broadcast_vm_command(guild_id, RuntimeCommand::HandleEvent(Box::new(evt)))
                .await
                .ok();
        }
    }

    async fn send_vm_command(
        &self,
        guild_id: GuildId,
        vmt: VmRole,
        cmd: RuntimeCommand,
    ) -> Result<(), String> {
        self.with_running_vm(guild_id, vmt, |rs| {
            rs.tx.send(cmd).unwrap();
            Ok(())
        })
        .await
    }

    async fn broadcast_vm_command(
        &self,
        guild_id: GuildId,
        cmd: RuntimeCommand,
    ) -> Result<(), String> {
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

    /// Pings a guilds vm on a interval ensuring that there's no runaway scripts, shutting down the vm if there was no pong within a interval window
    async fn loop_ping_guild_vm(&self, guild_id: GuildId, vmt: VmRole) {
        let mut ticker = tokio::time::interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            if !self.ping_guild_vm(guild_id, vmt).await {
                return;
            }
        }
    }

    async fn ping_guild_vm(&self, guild_id: GuildId, vmt: VmRole) -> bool {
        matches!(
            self.with_running_vm_mut(guild_id, vmt, |rs| {
                if !rs.ping_rcvd {
                    // shut down this runtime...
                    if let Some(ctrl) = &rs.control {
                        {
                            let mut reason = ctrl.shutdown_reason.write().unwrap();
                            *reason = Some(ShutdownReason::RunawayScript);
                        }

                        if !ctrl
                            .terminated
                            .swap(true, std::sync::atomic::Ordering::SeqCst)
                        {
                            info!("shutting down the vm for {}, reason: timed out", guild_id);
                            ctrl.isolate_handle.terminate_execution();
                        }
                    }
                } else {
                    rs.ping_rcvd = false;
                    rs.tx.send(RuntimeCommand::Ping).unwrap();
                }

                Ok(())
            })
            .await,
            Ok(_)
        )
    }

    async fn run_vm(
        &self,
        loaded_scripts: Vec<ContextScript>,
        rx: UnboundedReceiver<RuntimeCommand>,
        guild_id: GuildId,
    ) {
        let mut rt = Runtime::new(
            self.inner.shared_state.error_reporter.clone(),
            rx,
            self.inner.shared_state.rt_evt_tx.clone(),
            RuntimeContext {
                bot_state: self.inner.shared_state.bot_context.state.clone(),
                dapi: self.inner.shared_state.bot_context.http.clone(),
                guild_id,
                role: VmRole::Main,
            },
            loaded_scripts,
        )
        .await;

        rt.run().await
    }

    // async fn with_guild<F>(&self, guild_id: GuildId, f: F) -> Result<(), String>
    // where
    //     F: FnOnce(&GuildState) -> Result<(), String>,
    // {
    //     let guilds = self.guilds.read().await;

    //     match guilds.get(&guild_id) {
    //         Some(gs) => f(gs),
    //         None => Err("Unknown guild".to_string()),
    //     }
    // }

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
}

struct GuildState {
    id: GuildId,
    main_vm: VmState,
    pack_vms: Vec<(u64, VmState)>,
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
            Some(&self.0.pack_vms[self.1 - 1].1)
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
    tx: UnboundedSender<RuntimeCommand>,
    ping_rcvd: bool,
    control: Option<VmControl>,
}

struct VmControl {
    /// The handle to the v8 isolate
    isolate_handle: IsolateHandle,

    // Set to true when the v8 js has been terminated for some reason
    terminated: Arc<AtomicBool>,

    // Reason for the shutdown
    shutdown_reason: Arc<StdRwLock<Option<ShutdownReason>>>,
}

fn get_evt_guild_id(evt: &Event) -> Option<GuildId> {
    match evt {
        Event::MessageCreate(m) => m.guild_id,
        Event::MessageUpdate(mu) => mu.guild_id,
        Event::MessageDelete(md) => md.guild_id,
        _ => None,
    }
}
