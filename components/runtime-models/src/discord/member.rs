use crate::{discord::user::User, util::NotBigU64};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/discord/Member.ts")]
pub struct Member {
    pub deaf: bool,
    pub guild_id: String,
    pub joined_at: NotBigU64,
    pub mute: bool,
    pub nick: Option<String>,
    pub pending: bool,
    pub premium_since: Option<NotBigU64>,
    pub roles: Vec<String>,
    pub user: User,
}

impl From<twilight_model::guild::Member> for Member {
    fn from(v: twilight_model::guild::Member) -> Self {
        Self {
            deaf: v.deaf,
            guild_id: v.guild_id.to_string(),
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            mute: v.mute,
            nick: v.nick,
            pending: v.pending,
            premium_since: v
                .premium_since
                .map(|v| NotBigU64(v.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
            user: v.user.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/PartialMember.ts")]
pub struct PartialMember {
    pub deaf: bool,
    pub joined_at: NotBigU64,
    pub mute: bool,
    pub nick: Option<String>,
    pub premium_since: Option<NotBigU64>,
    pub roles: Vec<String>,
}

impl From<twilight_model::guild::PartialMember> for PartialMember {
    fn from(v: twilight_model::guild::PartialMember) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            mute: v.mute,
            nick: v.nick,
            premium_since: v
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "bindings/discord/InteractionMember.ts")]
pub struct InteractionMember {
    pub user: User,
    pub deaf: bool,
    pub joined_at: NotBigU64,
    pub mute: bool,
    pub nick: Option<String>,
    pub premium_since: Option<NotBigU64>,
    pub roles: Vec<String>,
    pub permissions: String,
}

impl From<twilight_model::guild::PartialMember> for InteractionMember {
    fn from(v: twilight_model::guild::PartialMember) -> Self {
        Self {
            deaf: v.deaf,
            joined_at: NotBigU64(v.joined_at.as_micros() as u64 / 1000),
            mute: v.mute,
            nick: v.nick,
            premium_since: v
                .premium_since
                .map(|ts| NotBigU64(ts.as_micros() as u64 / 1000)),
            roles: v.roles.iter().map(ToString::to_string).collect(),
            user: v.user.unwrap().into(),
            permissions: v.permissions.unwrap().bits().to_string(),
        }
    }
}
