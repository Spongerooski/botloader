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
