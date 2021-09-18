use serde::{Deserialize, Serialize};
use twilight_model::id::{ChannelId, GuildId, MessageId, RoleId};

use super::{
    embed::Embed,
    message::{Attachment, Mention, MessageType},
    user::User,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdate {
    pub attachments: Option<Vec<Attachment>>,
    pub author: Option<User>,
    pub channel_id: ChannelId,
    pub content: Option<String>,
    pub edited_timestamp: Option<String>,
    pub embeds: Option<Vec<Embed>>,
    pub guild_id: Option<GuildId>,
    pub id: MessageId,
    pub kind: Option<MessageType>,
    pub mention_everyone: Option<bool>,
    pub mention_roles: Option<Vec<RoleId>>,
    pub mentions: Option<Vec<Mention>>,
    pub pinned: Option<bool>,
    pub timestamp: Option<String>,
    pub tts: Option<bool>,
}

impl From<twilight_model::gateway::payload::MessageUpdate> for MessageUpdate {
    fn from(v: twilight_model::gateway::payload::MessageUpdate) -> Self {
        Self {
            attachments: v
                .attachments
                .map(|e| e.into_iter().map(From::from).collect()),
            author: v.author.map(From::from),
            channel_id: v.channel_id,
            content: v.content,
            edited_timestamp: v.edited_timestamp,
            embeds: v.embeds.map(|e| e.into_iter().map(From::from).collect()),
            guild_id: v.guild_id,
            id: v.id,
            kind: v.kind.map(From::from),
            mention_everyone: v.mention_everyone,
            mention_roles: v.mention_roles,
            mentions: v.mentions.map(|e| e.into_iter().map(From::from).collect()),
            pinned: v.pinned,
            timestamp: v.timestamp,
            tts: v.tts,
        }
    }
}
