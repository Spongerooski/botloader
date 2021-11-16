use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::message::{
        allowed_mentions::ParseTypes as TwilightParseTypes,
        AllowedMentions as TwilightAllowedMentions,
    },
    id::{ChannelId, MessageId, RoleId, UserId},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessage {
    pub(crate) channel_id: ChannelId,
    pub(crate) message_id: MessageId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpDeleteMessagesBulk {
    pub(crate) channel_id: ChannelId,
    pub(crate) message_ids: Vec<MessageId>,
}

use crate::commonmodels::embed::Embed;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpCreateChannelMessage {
    pub(crate) channel_id: ChannelId,
    pub(crate) fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpEditChannelMessage {
    pub(crate) channel_id: ChannelId,
    pub(crate) message_id: MessageId,
    pub(crate) fields: OpEditMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpCreateFollowUpMessage {
    pub(crate) interaction_token: String,
    pub(crate) fields: OpCreateMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpCreateMessageFields {
    pub(crate) content: String,
    pub(crate) embeds: Option<Vec<Embed>>,
    pub(crate) allowed_mentions: Option<AllowedMentions>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpEditMessageFields {
    pub(crate) content: Option<String>,
    pub(crate) embeds: Option<Vec<Embed>>,
    pub(crate) allowed_mentions: Option<AllowedMentions>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedMentions {
    parse: Vec<ParseTypes>,
    users: Vec<UserId>,
    roles: Vec<RoleId>,
    replied_user: bool,
}

impl From<AllowedMentions> for TwilightAllowedMentions {
    fn from(v: AllowedMentions) -> Self {
        Self {
            parse: v.parse.into_iter().map(Into::into).collect(),
            users: v.users,
            roles: v.roles,
            replied_user: v.replied_user,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ParseTypes {
    Everyone,
    Roles,
    Users,
}

impl From<ParseTypes> for TwilightParseTypes {
    fn from(pt: ParseTypes) -> Self {
        match pt {
            ParseTypes::Everyone => Self::Everyone,
            ParseTypes::Roles => Self::Roles,
            ParseTypes::Users => Self::Users,
        }
    }
}
