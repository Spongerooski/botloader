use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{ChannelId, GuildId, UserId};

#[derive(Debug, Error)]
pub enum ConfigStoreError<T: std::fmt::Debug + Error + 'static> {
    #[error("script not found")]
    ScriptNotFound,

    #[error("script link not found")]
    LinkNotFound,

    #[error("inner error occured: {0}")]
    Other(#[from] T),
}

pub type StoreResult<T, U> = Result<T, ConfigStoreError<U>>;

#[async_trait]
pub trait ConfigStore: Clone + Sync {
    type Error: std::error::Error + Send + Sync;

    async fn get_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<Script, Self::Error>;
    async fn get_script_by_id(
        &self,
        guild_id: GuildId,
        script_id: u64,
    ) -> StoreResult<Script, Self::Error>;
    async fn create_script(
        &self,
        guild_id: GuildId,
        script: CreateScript,
    ) -> StoreResult<Script, Self::Error>;
    async fn update_script(
        &self,
        guild_id: GuildId,
        script: UpdateScript,
    ) -> StoreResult<Script, Self::Error>;
    async fn update_script_contributes(
        &self,
        guild_id: GuildId,
        script_id: u64,
        contribs: ScriptContributes,
    ) -> StoreResult<Script, Self::Error>;
    async fn del_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<(), Self::Error>;
    async fn list_scripts(&self, guild_id: GuildId) -> StoreResult<Vec<Script>, Self::Error>;

    async fn get_guild_meta_config(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<Option<GuildMetaConfig>, Self::Error>;
    async fn update_guild_meta_config(
        &self,
        conf: &GuildMetaConfig,
    ) -> StoreResult<GuildMetaConfig, Self::Error>;

    async fn get_guild_meta_config_or_default(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<GuildMetaConfig, Self::Error> {
        match self.get_guild_meta_config(guild_id).await {
            Ok(Some(conf)) => Ok(conf),
            Ok(None) => Ok(GuildMetaConfig::guild_default(guild_id)),
            Err(e) => Err(e),
        }
    }

    async fn add_update_joined_guild(
        &self,
        guild: JoinedGuild,
    ) -> StoreResult<JoinedGuild, Self::Error>;

    async fn remove_joined_guild(&self, guild_id: GuildId) -> StoreResult<bool, Self::Error>;

    async fn get_joined_guilds(
        &self,
        ids: &[GuildId],
    ) -> StoreResult<Vec<JoinedGuild>, Self::Error>;
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub contributes: ScriptContributes,
}

/// Struct you get back from the store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScript {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
    pub contributes: Option<ScriptContributes>,
}

/// Struct used when creating a script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScript {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
}

/// Contribution points for a scripts, e.g triggers, commands etc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContributes {
    pub commands: Vec<twilight_model::application::command::Command>,
}

/// A guilds config, for storing core botloader settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuildMetaConfig {
    pub guild_id: GuildId,
    pub error_channel_id: Option<ChannelId>,
}

impl GuildMetaConfig {
    pub fn guild_default(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            error_channel_id: None,
        }
    }
}

/// A joined guild, we we store all guidls were connected to in the store
#[derive(Debug, Serialize, Deserialize)]
pub struct JoinedGuild {
    pub id: GuildId,
    pub name: String,
    pub icon: String,
    pub owner_id: UserId,
}
