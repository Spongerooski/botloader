use std::sync::Arc;
use std::time::{Duration, Instant};

use stores::config::{ConfigStore, Script, ScriptContributes};
use tokio::sync::mpsc;
use tracing::{error, info};
use twilight_model::application::command::{
    Command as TwilightCommand, CommandOption as TwilightCommandOption,
    CommandType as TwilightCommandType, OptionsCommandOptionData,
};
use twilight_model::id::GuildId;

use runtime_models::script::{Command, CommandGroup, ScriptMeta};

#[derive(Clone, Debug)]
pub struct ContribManagerHandle {
    send_loaded_script: mpsc::UnboundedSender<LoadedScript>,
}

impl ContribManagerHandle {
    pub fn send(&self, script: LoadedScript) {
        self.send_loaded_script.send(script).ok();
    }
}

pub struct ContribManager<CT> {
    config_store: CT,
    discord_client: Arc<twilight_http::Client>,
    rcv_loaded_script: mpsc::UnboundedReceiver<LoadedScript>,
    pending_checks: Vec<PendingCheckGroup>,
}

pub fn create_manager_pair<CT: ConfigStore>(
    config_store: CT,
    discord_client: Arc<twilight_http::Client>,
) -> (ContribManager<CT>, ContribManagerHandle) {
    let (send, rcv) = mpsc::unbounded_channel();

    (
        ContribManager {
            config_store,
            discord_client,
            rcv_loaded_script: rcv,
            pending_checks: Vec::new(),
        },
        ContribManagerHandle {
            send_loaded_script: send,
        },
    )
}

impl<CT: ConfigStore> ContribManager<CT>
where
    CT::Error: 'static,
{
    pub async fn run(&mut self) {
        let mut ticker = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.handle_tick().await;
                },
                evt = self.rcv_loaded_script.recv() => {
                    if let Some(evt) = evt{
                        self.handle_evt(evt).await;
                    }else{
                        info!("all receivers dead, shutting down contrib manager");
                        return;
                    }
                },
            }
        }
    }

    async fn handle_evt(&mut self, evt: LoadedScript) {
        self.update_db_contribs(&evt).await;

        if let Some(item) = self
            .pending_checks
            .iter_mut()
            .find(|e| e.guild_id == evt.guild_id)
        {
            // guild queue already exists

            // check if this script is already in the queue, and if so overwrite it
            if let Some(qi) = item
                .items
                .iter_mut()
                .find(|v| v.meta.script_id.0 == evt.meta.script_id.0)
            {
                *qi = evt
            } else {
                item.items.push(evt);
            }
        } else {
            // creata a new guild queue
            self.pending_checks.push(PendingCheckGroup {
                guild_id: evt.guild_id,
                items: vec![evt],
                started: Instant::now(),
            })
        }
    }

    async fn update_db_contribs(&mut self, evt: &LoadedScript) {
        let twilight_commands =
            to_twilight_commands(evt.guild_id, &evt.meta.commands, &evt.meta.command_groups);

        // TODO: handle errors here, maybe retry?
        if let Err(err) = self
            .config_store
            .update_script_contributes(
                evt.guild_id,
                evt.meta.script_id.0,
                ScriptContributes {
                    commands: twilight_commands,
                },
            )
            .await
        {
            error!(%err, "failed updating db contribs");
        }
    }

    async fn handle_tick(&mut self) {
        let old_list = std::mem::take(&mut self.pending_checks);

        for item in old_list {
            if item.started.elapsed() > Duration::from_secs(10) {
                if self.process_item(&item).await.is_err() {
                    // add back to queue if processing failed
                    self.pending_checks.push(item);
                }
            } else {
                self.pending_checks.push(item);
            }
        }
    }

    async fn process_item(&mut self, item: &PendingCheckGroup) -> Result<(), ()> {
        // TODO: only do this when they have actually changed
        // this will be more important when we need to scale it
        // but needs to be done before before we go beyond the 100 server mark
        self.update_guild_commands(item.guild_id).await?;
        Ok(())
    }

    async fn update_guild_commands(&mut self, guild_id: GuildId) -> Result<(), ()> {
        let all_guild_scripts = self
            .config_store
            .list_scripts(guild_id)
            .await
            .map_err(|err| {
                error!(%err, "failed retrieving guild scripts");
            })?;

        let merged = merge_script_commands(all_guild_scripts);
        info!(
            "updating guild commands for {}, n commands: {}",
            guild_id,
            merged.len()
        );

        if let Err(err) = self
            .discord_client
            .set_guild_commands(guild_id, &merged)
            .unwrap()
            .exec()
            .await
        {
            error!(%err, "failed updating guild commands")
            // TODO: for now this returns an ok, in the future once we have
            // more validation we could reutrn an err here and have it retry
            // (but not for client errors)
        }

        Ok(())
    }
}

static GROUP_DESC_PLACEHOLDER: &str = "no description";

struct PendingCheckGroup {
    guild_id: GuildId,
    started: Instant,
    items: Vec<LoadedScript>,
}

pub fn to_twilight_commands(
    guild_id: GuildId,
    commands: &[Command],
    groups: &[CommandGroup],
) -> Vec<TwilightCommand> {
    // handle top level commands
    let mut result = commands
        .iter()
        .filter(|c| c.group.is_none())
        .map(|cmd| TwilightCommand {
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            application_id: None,
            options: cmd.options.iter().map(|opt| opt.clone().into()).collect(),
            guild_id: Some(guild_id),
            default_permission: None,
            id: None,
            kind: TwilightCommandType::ChatInput,
        })
        .collect::<Vec<_>>();

    let mut groups = groups
        .iter()
        .map(|cg| group_to_twilight_command(guild_id, cg))
        .collect::<Vec<_>>();

    // add the commands to the groups and sub groups
    for cmd in commands.iter() {
        if let Some(cmd_group) = &cmd.group {
            // find (or create) the group
            let group = match groups.iter_mut().find(|g| *cmd_group == g.name) {
                Some(g) => g,
                None => {
                    // group not found, make a new one
                    groups.push(make_unknown_group(guild_id, cmd_group.clone()));

                    // return mut reference to the new group
                    let len = groups.len();
                    &mut groups[len - 1]
                }
            };

            // check if this belongs to a subgroup
            if let Some(cmd_sub_group) = &cmd.sub_group {
                match group.options.iter_mut().find(|sg| match sg {
                    TwilightCommandOption::SubCommandGroup(OptionsCommandOptionData {
                        name,
                        ..
                    }) if name == cmd_sub_group => todo!(),
                    _ => false,
                }) {
                    Some(g) => {
                        // add the cmd to the existing sub group
                        if let TwilightCommandOption::SubCommandGroup(sub_group) = g {
                            sub_group.options.push(cmd.clone().into())
                        }
                    }
                    None => {
                        // sub group not found, make a new one and add the cmd to it
                        group.options.push(TwilightCommandOption::SubCommandGroup(
                            OptionsCommandOptionData {
                                name: cmd_sub_group.clone(),
                                description: GROUP_DESC_PLACEHOLDER.to_string(),
                                options: vec![cmd.clone().into()],
                            },
                        ));
                    }
                };
            } else {
                // belongs to normal group (not sub group)
                group.options.push(cmd.clone().into())
            }
        }
    }

    result.append(&mut groups);
    result
}

fn make_unknown_group(guild_id: GuildId, name: String) -> TwilightCommand {
    TwilightCommand {
        application_id: None,
        default_permission: None,
        description: GROUP_DESC_PLACEHOLDER.to_string(),
        guild_id: Some(guild_id),
        id: None,
        kind: TwilightCommandType::ChatInput,
        options: Vec::new(),
        name,
    }
}

pub fn group_to_twilight_command(guild_id: GuildId, group: &CommandGroup) -> TwilightCommand {
    // handle sub groups
    let opts = group
        .sub_groups
        .iter()
        .map(|sg| {
            TwilightCommandOption::SubCommandGroup(OptionsCommandOptionData {
                name: sg.name.clone(),
                description: sg.description.clone(),
                options: Vec::new(),
            })
        })
        .collect::<Vec<_>>();

    TwilightCommand {
        application_id: None,
        guild_id: Some(guild_id),
        default_permission: None,
        description: group.description.clone(),
        id: None,
        kind: TwilightCommandType::ChatInput,
        name: group.name.clone(),
        options: opts,
    }
}

fn merge_script_commands(scripts: Vec<Script>) -> Vec<TwilightCommand> {
    let mut result = Vec::new();

    for script in scripts {
        for cmd in script.contributes.commands {
            if let Some(existing) = result
                .iter_mut()
                .find(|v: &&mut TwilightCommand| v.name == cmd.name)
            {
                merge_command(existing, cmd);
            } else {
                result.push(cmd);
            }
        }
    }

    result
}

// merges src into dst
fn merge_command(dst: &mut TwilightCommand, src: TwilightCommand) {
    if dst.description == GROUP_DESC_PLACEHOLDER && src.description != GROUP_DESC_PLACEHOLDER {
        dst.description = src.description;
    }

    for opt in &dst.options {
        if !matches!(
            opt,
            TwilightCommandOption::SubCommand(_) | TwilightCommandOption::SubCommandGroup(_)
        ) {
            // We can only merge sub commands
            return;
        }
    }

    for opt in src.options {
        if !matches!(
            opt,
            TwilightCommandOption::SubCommand(_) | TwilightCommandOption::SubCommandGroup(_)
        ) {
            // We can only merge sub commands
            return;
        }

        let src_opt_name = command_option_name(&opt);
        if let Some(dst_opt) = dst
            .options
            .iter_mut()
            .find(|v| command_option_name(v) == src_opt_name)
        {
            // we need to merge these options
            match (dst_opt, opt) {
                (
                    TwilightCommandOption::SubCommandGroup(dst_sg),
                    TwilightCommandOption::SubCommandGroup(src_sg),
                ) => merge_subgroups(dst_sg, src_sg),
                _ => {
                    // we can only merge subgroups, how would we merge subcommands?
                    continue;
                }
            }
        } else {
            // no conflict
            dst.options.push(opt);
        }
    }
}

fn merge_subgroups(dst: &mut OptionsCommandOptionData, src: OptionsCommandOptionData) {
    if dst.description == GROUP_DESC_PLACEHOLDER && src.description != GROUP_DESC_PLACEHOLDER {
        dst.description = src.description;
    }

    for opt in src.options {
        if dst
            .options
            .iter()
            .any(|v| command_option_name(v) == command_option_name(&opt))
        {
            // we command merge sub commands themselves
            continue;
        }

        // but we can add them to the group if there is no conflict!
        dst.options.push(opt);
    }
}

fn command_option_name(opt: &TwilightCommandOption) -> String {
    match opt {
        TwilightCommandOption::SubCommand(v) => v.name.clone(),
        TwilightCommandOption::SubCommandGroup(v) => v.name.clone(),
        TwilightCommandOption::String(v) => v.name.clone(),
        TwilightCommandOption::Integer(v) => v.name.clone(),
        TwilightCommandOption::Boolean(v) => v.name.clone(),
        TwilightCommandOption::User(v) => v.name.clone(),
        TwilightCommandOption::Channel(v) => v.name.clone(),
        TwilightCommandOption::Role(v) => v.name.clone(),
        TwilightCommandOption::Mentionable(v) => v.name.clone(),
        TwilightCommandOption::Number(v) => v.name.clone(),
    }
}

pub struct LoadedScript {
    pub guild_id: GuildId,
    pub meta: ScriptMeta,
}
