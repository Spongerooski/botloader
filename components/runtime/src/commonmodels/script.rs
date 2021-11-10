use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptMeta {
    pub description: String,
    pub script_id: u64,
    pub commands: Vec<Command>,
    pub command_groups: Vec<CommandGroup>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandGroup {
    pub name: String,
    pub description: String,
    pub sub_groups: Vec<CommandSubGroup>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSubGroup {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
    pub group: Option<String>,
    pub sub_group: Option<String>,
}

impl From<Command> for twilight_model::application::command::CommandOption {
    fn from(cmd: Command) -> Self {
        Self::SubCommand(
            twilight_model::application::command::OptionsCommandOptionData {
                name: cmd.name,
                description: cmd.description,
                options: cmd.options.into_iter().map(Into::into).collect(),
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CommandOptionType {
    // SubCommand,
    // SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
}

impl From<CommandOptionType> for twilight_model::application::command::CommandOptionType {
    fn from(v: CommandOptionType) -> Self {
        match v {
            CommandOptionType::String => Self::String,
            CommandOptionType::Integer => Self::Integer,
            CommandOptionType::Boolean => Self::Boolean,
            CommandOptionType::User => Self::User,
            CommandOptionType::Channel => Self::Channel,
            CommandOptionType::Role => Self::Role,
            CommandOptionType::Mentionable => Self::Mentionable,
            CommandOptionType::Number => Self::Number,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub kind: CommandOptionType,
    pub required: bool,
}

impl From<CommandOption> for twilight_model::application::command::CommandOption {
    fn from(v: CommandOption) -> Self {
        use twilight_model::application::command::BaseCommandOptionData;
        use twilight_model::application::command::ChannelCommandOptionData;
        use twilight_model::application::command::ChoiceCommandOptionData;

        match v.kind {
            CommandOptionType::String => Self::String(ChoiceCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
                ..Default::default()
            }),
            CommandOptionType::Integer => Self::Integer(ChoiceCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
                ..Default::default()
            }),
            CommandOptionType::Boolean => Self::Boolean(BaseCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
            }),
            CommandOptionType::User => Self::User(BaseCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
            }),
            CommandOptionType::Channel => Self::Channel(ChannelCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
                ..Default::default()
            }),
            CommandOptionType::Role => Self::Role(BaseCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
            }),
            CommandOptionType::Mentionable => Self::Mentionable(BaseCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
            }),
            CommandOptionType::Number => Self::Number(ChoiceCommandOptionData {
                name: v.name,
                description: v.description,
                required: v.required,
                ..Default::default()
            }),
        }
    }
}
