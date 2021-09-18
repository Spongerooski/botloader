use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twilight_model::id::{ChannelId, GuildId, RoleId};

pub mod postgres;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("script not found")]
    ScriptNotFound,

    #[error("script link not found")]
    LinkNotFound,

    #[error("sql error: {0}")]
    SqlError(#[from] sqlx::Error),

    #[error("no config for the provided guild")]
    GuildConfigNotFound,

    #[error("another error occured: {0}")]
    Other(#[from] anyhow::Error),
}

pub type StoreResult<T> = Result<T, StoreError>;

#[async_trait]
pub trait ConfigStore: Clone + Sync {
    async fn get_script(&self, guild_id: GuildId, script_name: String) -> StoreResult<Script>;
    async fn create_script(&self, guild_id: GuildId, script: CreateScript) -> StoreResult<Script>;
    async fn update_script(&self, guild_id: GuildId, script: Script) -> StoreResult<Script>;
    async fn del_script(&self, guild_id: GuildId, script_name: String) -> StoreResult<()>;
    async fn list_scripts(&self, guild_id: GuildId) -> StoreResult<Vec<Script>>;

    async fn link_script(
        &self,
        guild_id: GuildId,
        script_name: String,
        ctx: ScriptContext,
    ) -> StoreResult<ScriptLink>;

    async fn unlink_script(
        &self,
        guild_id: GuildId,
        script_name: String,
        ctx: ScriptContext,
    ) -> StoreResult<()>;

    async fn unlink_all_script(&self, guild_id: GuildId, script_name: String) -> StoreResult<u64>;

    async fn list_script_links(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<Vec<ScriptLink>>;

    async fn list_links(&self, guild_id: GuildId) -> StoreResult<Vec<ScriptLink>>;

    async fn list_context_scripts(
        &self,
        guild_id: GuildId,
        ctx: ScriptContext,
    ) -> StoreResult<Vec<Script>>;

    async fn get_guild_meta_config(&self, guild_id: GuildId) -> StoreResult<GuildMetaConfig>;
    async fn update_guild_meta_config(
        &self,
        conf: &GuildMetaConfig,
    ) -> StoreResult<GuildMetaConfig>;

    async fn get_guild_meta_config_or_default(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<GuildMetaConfig> {
        match self.get_guild_meta_config(guild_id).await {
            Ok(conf) => Ok(conf),
            Err(StoreError::GuildConfigNotFound) => Ok(GuildMetaConfig {
                guild_id,
                ..Default::default()
            }),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: u64,
    pub name: String,
    pub original_source: String,
    pub compiled_js: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScript {
    pub name: String,
    pub original_source: String,
    pub compiled_js: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptLink {
    pub script_name: String,
    pub context: ScriptContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptContext {
    Guild,
    Channel(ChannelId),
    Role(RoleId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GuildMetaConfig {
    pub guild_id: GuildId,
    pub error_channel_id: Option<ChannelId>,
}
