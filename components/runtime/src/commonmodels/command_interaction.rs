use serde::{Deserialize, Serialize};
use twilight_model::{
    application::interaction::{
        application_command::{CommandDataOption, CommandOptionValue},
        ApplicationCommand,
    },
    id::{ChannelId, GenericId, InteractionId, RoleId, UserId},
};

use crate::commonmodels::message::PartialMember;

// we perform some normalization to make things simpler on the script side
// and also simpler overall
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInteraction {
    pub channel_id: ChannelId,

    pub id: InteractionId,
    pub member: PartialMember,
    pub token: String,

    pub name: String,
    pub parent_name: Option<String>,
    pub parent_parent_name: Option<String>,

    pub options: Vec<CommandInteractionOption>,
}

impl From<ApplicationCommand> for CommandInteraction {
    fn from(cmd: ApplicationCommand) -> Self {
        let mut name = cmd.data.name;
        let mut parent_name: Option<String> = None;
        let mut parent_parent_name: Option<String> = None;
        let mut opts: Vec<CommandInteractionOption> = Vec::new();

        for opt in cmd.data.options {
            match opt.value {
                CommandOptionValue::SubCommand(sub_cmd) => {
                    // fix names, original name was the parent group name
                    let old = std::mem::replace(&mut name, opt.name.clone());
                    parent_name = Some(old);

                    for sub_opt in sub_cmd {
                        opts.push(sub_opt.into());
                    }
                }
                CommandOptionValue::SubCommandGroup(sub_cmd_group) => {
                    // fix names, original name was the parent of the parent group name
                    parent_name = Some(opt.name.clone());
                    parent_parent_name = Some(name.clone());

                    // there can only be 1, maybe add a check for that at some point
                    // never know when discord might break stuff
                    let cmd = sub_cmd_group[0].clone();
                    name = cmd.name;
                    if let CommandOptionValue::SubCommand(sub_sub_opts) = cmd.value {
                        for sub_sub_opt in sub_sub_opts {
                            opts.push(sub_sub_opt.into());
                        }
                    }
                }
                _ => {
                    opts.push(opt.into());
                }
            }
        }

        // TODO: remove these unwraps
        Self {
            name,
            parent_name,
            parent_parent_name,
            options: opts,
            channel_id: cmd.channel_id,
            id: cmd.id,
            member: cmd.member.unwrap().into(),
            token: cmd.token,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInteractionOption {
    pub name: String,
    pub value: CommandInteractionOptionValue,
}

impl From<CommandDataOption> for CommandInteractionOption {
    fn from(v: CommandDataOption) -> Self {
        Self {
            name: v.name,
            value: v.value.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CommandInteractionOptionValue {
    String { value: String },
    Integer { value: i64 },
    Boolean { value: bool },
    User { value: UserId },
    Channel { value: ChannelId },
    Role { value: RoleId },
    Mentionable { value: GenericId },
    Number { value: f64 },
}

impl From<CommandOptionValue> for CommandInteractionOptionValue {
    fn from(v: CommandOptionValue) -> Self {
        match v {
            CommandOptionValue::String(ov) => Self::String { value: ov },
            CommandOptionValue::Integer(ov) => Self::Integer { value: ov },
            CommandOptionValue::Boolean(ov) => Self::Boolean { value: ov },
            CommandOptionValue::User(ov) => Self::User { value: ov },
            CommandOptionValue::Channel(ov) => Self::Channel { value: ov },
            CommandOptionValue::Role(ov) => Self::Role { value: ov },
            CommandOptionValue::Mentionable(ov) => Self::Mentionable { value: ov },
            CommandOptionValue::Number(ov) => Self::Number { value: ov.0 },
            // the below are unreachable because of previous checkcs
            // altough it might be a bad idea since this is more or less a public function
            // aaa i should change it yeah.... later
            CommandOptionValue::SubCommand(_) => unreachable!(),
            CommandOptionValue::SubCommandGroup(_) => unreachable!(),
        }
    }
}
