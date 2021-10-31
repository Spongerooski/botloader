use super::Postgres;
use async_trait::async_trait;
use twilight_model::id::{ChannelId, GuildId, UserId};

use crate::config::{
    ConfigStoreError, CreateScript, GuildMetaConfig, JoinedGuild, Script, StoreResult,
};

// impl From<sqlx::

impl Postgres {
    async fn get_db_script_by_name(
        &self,
        guild_id: GuildId,
        script_name: &str,
    ) -> StoreResult<DbScript, sqlx::Error> {
        match sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled FROM guild_scripts WHERE \
             guild_id = $1 AND name = $2;",
            guild_id.0 as i64,
            script_name
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(s) => Ok(s),
            Err(sqlx::Error::RowNotFound) => Err(ConfigStoreError::ScriptNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_db_script_by_id(
        &self,
        guild_id: GuildId,
        id: i64,
    ) -> StoreResult<DbScript, sqlx::Error> {
        Ok(sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, name, original_source, enabled FROM guild_scripts WHERE \
             guild_id = $1 AND id = $2;",
            guild_id.0 as i64,
            id
        )
        .fetch_one(&self.pool)
        .await?)
    }
}

#[async_trait]
impl crate::config::ConfigStore for Postgres {
    type Error = sqlx::Error;

    async fn get_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<Script, Self::Error> {
        Ok(self
            .get_db_script_by_name(guild_id, &script_name)
            .await?
            .into())
    }

    async fn get_script_by_id(
        &self,
        guild_id: GuildId,
        script_id: u64,
    ) -> StoreResult<Script, Self::Error> {
        Ok(self
            .get_db_script_by_id(guild_id, script_id as i64)
            .await?
            .into())
    }

    async fn create_script(
        &self,
        guild_id: GuildId,
        script: CreateScript,
    ) -> StoreResult<Script, Self::Error> {
        let res = sqlx::query_as!(
            DbScript,
            "
                INSERT INTO guild_scripts (guild_id, name, original_source, enabled) 
                VALUES ($1, $2, $3, $4)
                RETURNING id, guild_id, name, original_source, enabled;
            ",
            guild_id.0 as i64,
            script.name,
            script.original_source,
            script.enabled,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn update_script(
        &self,
        guild_id: GuildId,
        script: Script,
    ) -> StoreResult<Script, Self::Error> {
        let res = sqlx::query_as!(
            DbScript,
            "
                UPDATE guild_scripts SET
                original_source = $3,
                enabled = $4
                WHERE guild_id = $1 AND id=$2
                RETURNING id, name, original_source, guild_id, enabled;
            ",
            guild_id.0 as i64,
            script.id as i64,
            script.original_source,
            script.enabled,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res.into())
    }

    async fn del_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<(), Self::Error> {
        let res = sqlx::query!(
            "DELETE FROM guild_scripts WHERE guild_id = $1 AND name = $2;",
            guild_id.0 as i64,
            script_name
        )
        .execute(&self.pool)
        .await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ConfigStoreError::ScriptNotFound)
        }
    }

    async fn list_scripts(&self, guild_id: GuildId) -> StoreResult<Vec<Script>, Self::Error> {
        let res = sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, name, enabled FROM guild_scripts WHERE \
             guild_id = $1",
            guild_id.0 as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(|e| e.into()).collect())
    }

    async fn get_guild_meta_config(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<Option<GuildMetaConfig>, Self::Error> {
        match sqlx::query_as!(
            DbGuildMetaConfig,
            "SELECT guild_id, error_channel_id FROM guild_meta_configs
        WHERE guild_id = $1;",
            guild_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(conf) => Ok(Some(conf.into())),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_guild_meta_config(
        &self,
        conf: &GuildMetaConfig,
    ) -> StoreResult<GuildMetaConfig, Self::Error> {
        let db_conf = sqlx::query_as!(
            DbGuildMetaConfig,
            "INSERT INTO guild_meta_configs (guild_id, error_channel_id) VALUES ($1, $2)
            ON CONFLICT (guild_id) DO UPDATE SET
            error_channel_id = $2
            RETURNING guild_id, error_channel_id;",
            conf.guild_id.0 as i64,
            conf.error_channel_id
                .map(|e| e.0 as i64)
                .unwrap_or_default(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_conf.into())
    }

    async fn add_update_joined_guild(
        &self,
        guild: JoinedGuild,
    ) -> StoreResult<JoinedGuild, Self::Error> {
        let db_guild = sqlx::query_as!(
            DbJoinedGuild,
            "INSERT INTO joined_guilds (id, name, icon, owner_id) VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET 
            name = $2, icon = $3, owner_id = $4
            RETURNING id, name, icon, owner_id;",
            guild.id.0 as i64,
            &guild.name,
            &guild.icon,
            guild.owner_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(db_guild.into())
    }

    async fn remove_joined_guild(&self, guild_id: GuildId) -> StoreResult<bool, Self::Error> {
        let res = sqlx::query!(
            "DELETE FROM joined_guilds WHERE id = $1;",
            guild_id.0 as i64,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }

    async fn get_joined_guilds(
        &self,
        ids: &[GuildId],
    ) -> StoreResult<Vec<JoinedGuild>, Self::Error> {
        let guilds = sqlx::query_as!(
            DbJoinedGuild,
            "SELECT id, name, icon, owner_id FROM joined_guilds WHERE id = ANY ($1)",
            &ids.into_iter().map(|e| e.0 as i64).collect::<Vec<_>>(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(guilds.into_iter().map(|e| e.into()).collect())
    }
}

#[allow(dead_code)]
struct DbScript {
    id: i64,
    guild_id: i64,
    original_source: String,
    name: String,
    enabled: bool,
}

impl From<DbScript> for Script {
    fn from(script: DbScript) -> Self {
        Self {
            id: script.id as u64,
            name: script.name,
            original_source: script.original_source,
            enabled: script.enabled,
        }
    }
}

struct DbGuildMetaConfig {
    pub guild_id: i64,
    pub error_channel_id: i64,
}

impl From<DbGuildMetaConfig> for GuildMetaConfig {
    fn from(mc: DbGuildMetaConfig) -> Self {
        Self {
            guild_id: GuildId(mc.guild_id as u64),
            error_channel_id: if mc.error_channel_id != 0 {
                Some(ChannelId(mc.error_channel_id as u64))
            } else {
                None
            },
        }
    }
}

pub struct DbJoinedGuild {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub owner_id: i64,
}

impl From<DbJoinedGuild> for JoinedGuild {
    fn from(g: DbJoinedGuild) -> Self {
        Self {
            id: GuildId(g.id as u64),
            name: g.name,
            icon: g.icon,
            owner_id: UserId(g.owner_id as u64),
        }
    }
}
