use serde::{Deserialize, Serialize};
use twilight_model::id::UserId;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: u16,
    pub email: Option<String>,
    pub id: UserId,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub username: String,
    pub premium_type: Option<PremiumType>,
    pub public_flags: Option<u64>,
    pub system: Option<bool>,
    pub verified: Option<bool>,
}

impl From<User> for twilight_model::user::User {
    fn from(v: User) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            email: v.email,
            id: v.id,
            locale: v.locale,
            mfa_enabled: v.mfa_enabled,
            name: v.username,
            premium_type: v.premium_type.map(From::from),
            // TODO: remove the unwrap used here
            public_flags: v
                .public_flags
                .map(|e| twilight_model::user::UserFlags::from_bits(e).unwrap()),
            system: v.system,
            verified: v.verified,
            flags: None,
            accent_color: None,
            banner: None,
        }
    }
}
impl From<twilight_model::user::User> for User {
    fn from(v: twilight_model::user::User) -> Self {
        Self {
            avatar: v.avatar,
            bot: v.bot,
            discriminator: v.discriminator,
            email: v.email,
            id: v.id,
            locale: v.locale,
            mfa_enabled: v.mfa_enabled,
            username: v.name,
            premium_type: v.premium_type.map(From::from),
            public_flags: v.public_flags.map(|e| e.bits()),
            system: v.system,
            verified: v.verified,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PremiumType {
    None,
    NitroClassic,
    Nitro,
}

impl From<PremiumType> for twilight_model::user::PremiumType {
    fn from(v: PremiumType) -> Self {
        match v {
            PremiumType::Nitro => Self::Nitro,
            PremiumType::NitroClassic => Self::NitroClassic,
            PremiumType::None => Self::None,
        }
    }
}
impl From<twilight_model::user::PremiumType> for PremiumType {
    fn from(v: twilight_model::user::PremiumType) -> Self {
        match v {
            twilight_model::user::PremiumType::Nitro => Self::Nitro,
            twilight_model::user::PremiumType::NitroClassic => Self::NitroClassic,
            twilight_model::user::PremiumType::None => Self::None,
        }
    }
}
