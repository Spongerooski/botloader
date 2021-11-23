use serde::Serialize;
use ts_rs::TS;

use crate::{
    embed::Embed,
    message::{Attachment, Mention, MessageType},
    user::User,
};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/MessageUpdate.ts")]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdate {
    pub attachments: Option<Vec<Attachment>>,
    pub author: Option<User>,
    pub channel_id: String,
    pub content: Option<String>,
    pub edited_timestamp: Option<u64>,
    pub embeds: Option<Vec<Embed>>,
    pub guild_id: Option<String>,
    pub id: String,
    pub kind: Option<MessageType>,
    pub mention_everyone: Option<bool>,
    pub mention_roles: Option<Vec<String>>,
    pub mentions: Option<Vec<Mention>>,
    pub pinned: Option<bool>,
    pub timestamp: Option<u64>,
    pub tts: Option<bool>,
}

impl From<twilight_model::gateway::payload::incoming::MessageUpdate> for MessageUpdate {
    fn from(v: twilight_model::gateway::payload::incoming::MessageUpdate) -> Self {
        Self {
            attachments: v
                .attachments
                .map(|e| e.into_iter().map(From::from).collect()),
            author: v.author.map(From::from),
            channel_id: v.channel_id.to_string(),
            content: v.content,
            edited_timestamp: v.edited_timestamp.map(|ts| ts.as_secs()),
            embeds: v.embeds.map(|e| e.into_iter().map(From::from).collect()),
            guild_id: v.guild_id.as_ref().map(ToString::to_string),
            id: v.id.to_string(),
            kind: v.kind.map(From::from),
            mention_everyone: v.mention_everyone,
            mention_roles: v
                .mention_roles
                .map(|r| r.iter().map(ToString::to_string).collect()),
            mentions: v.mentions.map(|e| e.into_iter().map(From::from).collect()),
            pinned: v.pinned,
            timestamp: v.timestamp.map(|ts| ts.as_secs()),
            tts: v.tts,
        }
    }
}
