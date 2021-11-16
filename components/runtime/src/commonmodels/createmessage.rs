use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::message::{
        allowed_mentions::ParseTypes as TwilightParseTypes,
        AllowedMentions as TwilightAllowedMentions,
    },
    id::{ChannelId, MessageId, RoleId, UserId},
};

use crate::commonmodels::embed::Embed;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelMessage {
    pub(crate) channel_id: ChannelId,
    pub(crate) fields: CreateMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditChannelMessage {
    pub(crate) channel_id: ChannelId,
    pub(crate) message_id: MessageId,
    pub(crate) fields: EditMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFollowUpMessage {
    pub(crate) interaction_token: String,
    pub(crate) fields: CreateMessageFields,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageFields {
    pub(crate) content: String,
    pub(crate) embeds: Option<Vec<Embed>>,
    pub(crate) allowed_mentions: Option<AllowedMentions>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditMessageFields {
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
