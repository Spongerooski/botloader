use serde::{Deserialize, Serialize};
use ts_rs::TS;
use twilight_model::id::{ChannelId, GenericId, GuildId};

pub struct CategoryChannel {
    pub guild_id: GuildId,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChannelType {
    GuildText,
    Private,
    GuildVoice,
    Group,
    GuildCategory,
    GuildNews,
    GuildStore,
    GuildStageVoice,
    GuildNewsThread,
    GuildPublicThread,
    GuildPrivateThread,
}

impl From<twilight_model::channel::ChannelType> for ChannelType {
    fn from(v: twilight_model::channel::ChannelType) -> Self {
        match v {
            twilight_model::channel::ChannelType::GuildText => Self::GuildText,
            twilight_model::channel::ChannelType::Private => Self::Private,
            twilight_model::channel::ChannelType::GuildVoice => Self::GuildVoice,
            twilight_model::channel::ChannelType::Group => Self::Group,
            twilight_model::channel::ChannelType::GuildCategory => Self::GuildCategory,
            twilight_model::channel::ChannelType::GuildNews => Self::GuildNews,
            twilight_model::channel::ChannelType::GuildStore => Self::GuildStore,
            twilight_model::channel::ChannelType::GuildStageVoice => Self::GuildStageVoice,
            twilight_model::channel::ChannelType::GuildNewsThread => Self::GuildNewsThread,
            twilight_model::channel::ChannelType::GuildPublicThread => Self::GuildPublicThread,
            twilight_model::channel::ChannelType::GuildPrivateThread => Self::GuildPrivateThread,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PermissionOverwrite {
    pub allow: String,
    pub deny: String,
    pub kind: PermissionOverwriteType,
    pub id: GenericId,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PermissionOverwriteType {
    Member,
    Role,
}
