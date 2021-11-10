use std::sync::Arc;

use stores::config::{ConfigStore, CreateScript, UpdateScript};
use tracing::{error, info, instrument};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_model::{gateway::payload::incoming::MessageCreate, guild::Permissions, id::RoleId};
use twilight_util::permission_calculator::PermissionCalculator;
use validation::{validate, ValidationError};

use crate::BotContext;

#[derive(Clone)]
pub struct CommandContext<CT> {
    pub(crate) http: Arc<twilight_http::Client>,
    pub(crate) cluster: Arc<Cluster>,
    pub(crate) state: Arc<InMemoryCache>,
    pub(crate) config_store: CT,
    pub(crate) vm_manager: vm_manager::Manager<CT>,
}

#[derive(Debug)]
pub enum Command {
    AddScript(String, String),
    UpdateScript(String, String),
    GetScript(String),
    DeleteScript(String),
    ListScripts,

    EnabledScript(String),
    DisableScript(String),

    StartVM,
    SetErrorChannel(bool),
}

#[derive(Debug)]
pub struct ParsedCommand {
    m: MessageCreate,
    command: Command,
}

pub(crate) fn check_for_command<CT>(
    ctx: &BotContext<CT>,
    m: MessageCreate,
) -> Option<ParsedCommand> {
    if let Some(mem) = &m.member {
        let roles: Vec<_> = mem
            .roles
            .iter()
            .map(|id| {
                (
                    *id,
                    ctx.state
                        .role(*id)
                        .map(|p| p.permissions)
                        .unwrap_or_else(Permissions::empty),
                )
            })
            .collect();

        let everyone = ctx
            .state
            .role(RoleId(m.guild_id.unwrap().0))
            .map(|everyone_role| everyone_role.permissions)
            .unwrap_or_else(Permissions::empty);

        // if let Some(everyone_role) = ctx.state.role(RoleId(m.guild_id.unwrap().0)) {
        //     roles.push((everyone_role.id, everyone_role.permissions));
        // }

        let calculator = PermissionCalculator::new(m.guild_id?, m.author.id, everyone, &roles);
        if !(calculator.root().intersects(Permissions::MANAGE_GUILD)) {
            return None;
        }
    } else {
        return None;
    }

    let mut split = m.content.split(' ');
    if let Some(prefix) = split.next() {
        if prefix == "!jack" {
            let collected: Vec<String> = split.map(|e| e.to_string()).collect();
            match parse_command(&m, collected) {
                Ok(Some(cmd)) => {
                    info!("Parsed command: {:?}", cmd);
                    return Some(ParsedCommand { m, command: cmd });
                }
                Err(e) => {
                    info!("failed parsing command: {}", e);
                }
                _ => {}
            }
        }
    }

    None
}

fn parse_command(_m: &MessageCreate, split: Vec<String>) -> Result<Option<Command>, String> {
    let mut iter = split.into_iter();
    let prefix = if let Some(prefix) = iter.next() {
        prefix
    } else {
        return Ok(None);
    };

    match prefix.as_str() {
        "script" => {
            if let Some(sub) = iter.next() {
                match sub.as_str().to_lowercase().as_str() {
                    "add" => Ok(Some(Command::AddScript(
                        cmd_parse_string_word(&mut iter)?,
                        cmd_parse_string_rest(&mut iter)?,
                    ))),
                    "del" => Ok(Some(Command::DeleteScript(cmd_parse_string_word(
                        &mut iter,
                    )?))),
                    "get" => Ok(Some(Command::GetScript(cmd_parse_string_word(&mut iter)?))),
                    "update" => Ok(Some(Command::UpdateScript(
                        cmd_parse_string_word(&mut iter)?,
                        cmd_parse_string_rest(&mut iter)?,
                    ))),
                    "list" => Ok(Some(Command::ListScripts)),

                    "enable" => Ok(Some(Command::EnabledScript(cmd_parse_string_word(
                        &mut iter,
                    )?))),
                    "disable" => Ok(Some(Command::DisableScript(cmd_parse_string_word(
                        &mut iter,
                    )?))),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        "seterrorchannel" => Ok(Some(Command::SetErrorChannel(true))),
        "unseterrorchannel" => Ok(Some(Command::SetErrorChannel(false))),
        "startvm" => Ok(Some(Command::StartVM)),
        _ => Ok(None),
    }
}

#[allow(dead_code)]
fn cmd_parse_uint64<T: Iterator<Item = String>>(iter: &mut T) -> Result<u64, String> {
    match iter.next() {
        None => Err("no more args".to_string()),
        Some(s) => Ok(s.parse().map_err(|e| format!("failed parsing: {}", e))?),
    }
}

fn cmd_parse_string_rest<T: Iterator<Item = String>>(iter: &mut T) -> Result<String, String> {
    let mut rest = iter.collect::<Vec<_>>().join(" ");

    // strip codeblock
    if rest.starts_with("```") {
        // strip first line
        let mut lines = rest.split('\n');
        lines.next();

        rest = lines.collect::<Vec<_>>().join("\n");

        // strip end codeblock
        rest = rest.trim_end_matches("```").to_string();
    }

    Ok(rest)
}

fn cmd_parse_string_word<T: Iterator<Item = String>>(iter: &mut T) -> Result<String, String> {
    match iter.next() {
        None => Err("no more args".to_string()),
        Some(s) => Ok(s),
    }
}

#[instrument(skip(ctx, cmd))]
pub(crate) async fn handle_command<CT: ConfigStore + Send + Sync + 'static>(
    ctx: CommandContext<CT>,
    cmd: ParsedCommand,
) {
    match run_command(&ctx, &cmd).await {
        Ok(Some(s)) => {
            match ctx
                .http
                .create_message(cmd.m.channel_id)
                .content(&s)
                .unwrap()
                .exec()
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!(err = %e, "failed sending command response")
                }
            }
        }
        Ok(None) => {}
        Err(e) => {
            error!("Failed running command :(: {}", e);

            match ctx
                .http
                .create_message(cmd.m.channel_id)
                .content("Failed handling command :(")
                .unwrap()
                .exec()
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!(err = %e, "failed sending command response")
                }
            }
        }
    }
}

#[instrument(skip(ctx, cmd))]
async fn run_command<CT: ConfigStore + Send + Sync + 'static>(
    ctx: &CommandContext<CT>,
    cmd: &ParsedCommand,
) -> Result<Option<String>, String> {
    match &cmd.command {
        Command::AddScript(name, source) | Command::UpdateScript(name, source) => {
            let compiled = tscompiler::compile_typescript(source)
                .map_err(|e| format!("failed compiling: {:}", e))?;

            let _ = match runtime::validate_script(compiled.clone()).await {
                Ok(h) => h,
                Err(e) => {
                    return Ok(Some(format!("failed validating script: {}", e)));
                }
            };

            match ctx
                .config_store
                .get_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
            {
                Ok(existing) => {
                    let script = UpdateScript {
                        original_source: source.clone(),
                        contributes: None,
                        enabled: true,
                        id: existing.id,
                        name: existing.name,
                    };
                    if let Err(verr) = validate(&script) {
                        return Ok(Some(format!(
                            "failed validating script: {}",
                            format_validation_err(verr)
                        )));
                    }

                    let script = ctx
                        .config_store
                        .update_script(cmd.m.guild_id.unwrap(), script)
                        .await
                        .map_err(|e| format!("failed updating script :( {}", e))?;

                    ctx.vm_manager
                        .update_script(cmd.m.guild_id.unwrap(), script)
                        .await?;

                    Ok(Some(format!("Script {} has been updated!", name)))
                }

                Err(_) => {
                    let create = CreateScript {
                        name: name.clone(),
                        original_source: source.clone(),
                        enabled: true,
                    };

                    if let Err(verr) = validate(&create) {
                        return Ok(Some(format!(
                            "failed validating script: {}",
                            format_validation_err(verr)
                        )));
                    }

                    let script = ctx
                        .config_store
                        .create_script(cmd.m.guild_id.unwrap(), create)
                        .await
                        .map_err(|e| format!("failed creating script :( {}", e))?;

                    ctx.vm_manager
                        .load_script(cmd.m.guild_id.unwrap(), script)
                        .await?;

                    Ok(Some(format!(
                        "Script {} has been added! (note that it still needs to be linked to a \
                         context)",
                        name
                    )))
                }
            }
        }
        Command::DeleteScript(name) => {
            let script = ctx
                .config_store
                .get_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
                .map_err(|e| format!("unknown script: {}", e))?;

            ctx.vm_manager
                .detach_all_script(cmd.m.guild_id.unwrap(), script.id)
                .await
                .ok();

            ctx.config_store
                .del_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
                .map_err(|e| format!("failed deleting script: {}", e))?;

            Ok(Some(format!("Script {} has been deleted!", name)))
        }
        Command::GetScript(name) => {
            let script = ctx
                .config_store
                .get_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
                .map_err(|e| format!("unknown script: {}", e))?;

            Ok(Some(format!(
                "Script {}:\nSource: ```ts\n{}\n```",
                script.name, script.original_source
            )))
        }
        Command::ListScripts => {
            let scripts = ctx
                .config_store
                .list_scripts(cmd.m.guild_id.unwrap())
                .await
                .map_err(|e| format!("failed fetching scripts: {}", e))?;

            let summary = scripts
                .into_iter()
                .map(|script| format!("{}: enabled: {}\n", script.name, script.enabled))
                .collect::<String>();

            Ok(Some(format!(
                "Scripts on this guild: ```\n{}\n```",
                summary
            )))
        }
        Command::EnabledScript(name) => {
            let script = ctx
                .config_store
                .get_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
                .map_err(|e| format!("unknown script: {}", e))?;

            if script.enabled {
                Ok(Some("Script already enabled".to_string()))
            } else {
                let update = UpdateScript {
                    id: script.id,
                    name: script.name,
                    enabled: true,
                    original_source: script.original_source,
                    contributes: None,
                };
                let script = ctx
                    .config_store
                    .update_script(cmd.m.guild_id.unwrap(), update)
                    .await
                    .map_err(|e| format!("failed updating script :( {}", e))?;

                ctx.vm_manager
                    .load_script(cmd.m.guild_id.unwrap(), script)
                    .await
                    .ok();

                Ok(Some("Enabled script".to_string()))
            }
        }
        Command::DisableScript(name) => {
            let script = ctx
                .config_store
                .get_script(cmd.m.guild_id.unwrap(), name.clone())
                .await
                .map_err(|e| format!("unknown script: {}", e))?;

            if !script.enabled {
                Ok(Some("Script already disabled".to_string()))
            } else {
                let update = UpdateScript {
                    id: script.id,
                    name: script.name,
                    enabled: true,
                    original_source: script.original_source,
                    contributes: None,
                };
                let script = ctx
                    .config_store
                    .update_script(cmd.m.guild_id.unwrap(), update)
                    .await
                    .map_err(|e| format!("failed updating script :( {}", e))?;

                ctx.vm_manager
                    .unload_scripts(cmd.m.guild_id.unwrap(), vec![script])
                    .await
                    .ok();

                Ok(Some("Enabled script".to_string()))
            }
        }
        Command::StartVM => {
            ctx.vm_manager
                .restart_guild_vm(cmd.m.guild_id.unwrap())
                .await
                .map_err(|e| format!("failed restarting guild vm: {}", e))?;

            Ok(Some(
                "Restarting your guild's vm... (note that if it keeps stopping, there might be a \
                 runaway script that contains something like a infinite loop, you should find and \
                 remove the culprit)"
                    .to_string(),
            ))
        }
        Command::SetErrorChannel(set) => {
            let mut conf = ctx
                .config_store
                .get_guild_meta_config_or_default(cmd.m.guild_id.unwrap())
                .await
                .map_err(|e| format!("failed fetching your guild config: {}", e))?;

            if *set {
                conf.error_channel_id = Some(cmd.m.channel_id);
            } else {
                conf.error_channel_id = None;
            }

            ctx.config_store
                .update_guild_meta_config(&conf)
                .await
                .map_err(|e| format!("failed updating the config: {}", e))?;

            Ok(Some(if *set {
                "set the error channel to this channel".to_string()
            } else {
                "unset the error channel, you will no longer receive notifications anywhere about \
                 errors"
                    .to_string()
            }))
        }
    }
}

fn format_validation_err(errs: Vec<ValidationError>) -> String {
    errs.iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}
