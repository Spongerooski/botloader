use serde::Serialize;
use twilight_model::{
    guild::{Role as TwilightRole, RoleTags as TwilightRoleTags},
    id::{IntegrationId, RoleId, UserId},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub(crate) color: u32,
    pub(crate) hoist: bool,
    pub(crate) icon: Option<String>,
    pub(crate) id: RoleId,
    pub(crate) managed: bool,
    pub(crate) mentionable: bool,
    pub(crate) name: String,
    pub(crate) permissions: String,
    pub(crate) position: i64,
    pub(crate) tags: Option<RoleTags>,
    pub(crate) unicode_emoji: Option<String>,
}

impl From<&TwilightRole> for Role {
    fn from(v: &TwilightRole) -> Self {
        Self {
            color: v.color,
            hoist: v.hoist,
            icon: v.icon.clone(),
            id: v.id,
            managed: v.managed,
            mentionable: v.mentionable,
            name: v.name.clone(),
            permissions: v.permissions.bits().to_string(),
            position: v.position,
            tags: v.tags.clone().map(Into::into),
            unicode_emoji: v.unicode_emoji.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleTags {
    pub(crate) bot_id: Option<UserId>,
    pub(crate) integration_id: Option<IntegrationId>,
    pub(crate) premium_subscriber: bool,
}

impl From<TwilightRoleTags> for RoleTags {
    fn from(v: TwilightRoleTags) -> Self {
        Self {
            bot_id: v.bot_id,
            integration_id: v.integration_id,
            premium_subscriber: v.premium_subscriber,
        }
    }
}
