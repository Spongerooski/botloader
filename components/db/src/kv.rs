use serde::{Deserialize, Serialize};
use twilight_model::id::*;

#[derive(Serialize, Deserialize)]
pub enum GuildNamespace {
    GuildScript,
    Pack(std::num::NonZeroU64),
}

impl From<GuildNamespace> for i64 {
    fn from(gn: GuildNamespace) -> Self {
        match gn {
            GuildNamespace::GuildScript => 0,
            GuildNamespace::Pack(val) => val.get() as i64,
        }
    }
}

impl From<GuildNamespace> for u64 {
    fn from(gn: GuildNamespace) -> Self {
        match gn {
            GuildNamespace::GuildScript => 0,
            GuildNamespace::Pack(val) => val.get(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Key {
    pub guild: GuildId,
    pub namespace: GuildNamespace,
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct Value(pub String);

pub struct KeyValuePair {
    pub key: Key,
    pub value: Value,
}
