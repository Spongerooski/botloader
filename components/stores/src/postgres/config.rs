use super::Postgres;
use async_trait::async_trait;
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

use crate::config::{
    ConfigStoreError, CreateScript, GuildMetaConfig, JoinedGuild, Script, ScriptContext,
    ScriptLink, StoreResult,
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
            "SELECT id, guild_id, original_source, compiled_js, name FROM guild_scripts WHERE \
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
            "SELECT id, guild_id, name, original_source, compiled_js FROM guild_scripts WHERE \
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

    async fn create_script(
        &self,
        guild_id: GuildId,
        script: CreateScript,
    ) -> StoreResult<Script, Self::Error> {
        let res = sqlx::query_as!(
            DbScript,
            "
                INSERT INTO guild_scripts (guild_id, name, original_source, compiled_js) 
                VALUES ($1, $2, $3, $4)
                RETURNING id, guild_id, name, original_source, compiled_js;
            ",
            guild_id.0 as i64,
            script.name,
            script.original_source,
            script.compiled_js,
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
                compiled_js = $4
                WHERE guild_id = $1 AND id=$2
                RETURNING id, name, original_source, compiled_js, guild_id;
            ",
            guild_id.0 as i64,
            script.id as i64,
            script.original_source,
            script.compiled_js,
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
            "SELECT id, guild_id, original_source, compiled_js, name FROM guild_scripts WHERE \
             guild_id = $1",
            guild_id.0 as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res.into_iter().map(|e| e.into()).collect())
    }

    async fn link_script(
        &self,
        guild_id: GuildId,
        script_name: String,
        ctx: ScriptContext,
    ) -> StoreResult<ScriptLink, Self::Error> {
        let script = self.get_db_script_by_name(guild_id, &script_name).await?;

        let (ctx_typ, ctx_id) = ContextPair::from(ctx);

        let res = sqlx::query_as!(
            DbLink,
            "INSERT INTO script_links (guild_id, script_id, context_type, context_id) 
            VALUES ($1, $2, $3, $4) RETURNING id, guild_id, script_id, context_type, context_id;",
            guild_id.0 as i64,
            script.id,
            ctx_typ,
            ctx_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ScriptLink {
            context: (res.context_type, res.context_id).into(),
            script_name,
        })
    }

    async fn unlink_script(
        &self,
        guild_id: GuildId,
        script_name: String,
        ctx: ScriptContext,
    ) -> StoreResult<(), Self::Error> {
        let script = self.get_db_script_by_name(guild_id, &script_name).await?;

        let (ctx_typ, ctx_id) = ContextPair::from(ctx);

        let res = sqlx::query!(
            "DELETE FROM script_links WHERE guild_id = $1 AND script_id = $2 AND context_type = \
             $3 AND context_id = $4;",
            guild_id.0 as i64,
            script.id,
            ctx_typ,
            ctx_id,
        )
        .execute(&self.pool)
        .await?;

        if res.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ConfigStoreError::LinkNotFound)
        }
    }

    async fn unlink_all_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<u64, Self::Error> {
        let script = self.get_db_script_by_name(guild_id, &script_name).await?;

        let res = sqlx::query!(
            "DELETE FROM script_links WHERE guild_id = $1 AND script_id = $2;",
            guild_id.0 as i64,
            script.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    async fn list_script_links(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> StoreResult<Vec<ScriptLink>, Self::Error> {
        let script = self.get_db_script_by_name(guild_id, &script_name).await?;

        let res = sqlx::query_as!(
            DbLink,
            "SELECT id, guild_id, script_id, context_type, context_id FROM script_links
        WHERE guild_id = $1 AND script_id = $2;",
            guild_id.0 as i64,
            script.id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res
            .into_iter()
            .map(|e| ScriptLink {
                script_name: script_name.clone(),
                context: (e.context_type, e.context_id).into(),
            })
            .collect())
    }

    async fn list_links(&self, guild_id: GuildId) -> StoreResult<Vec<ScriptLink>, Self::Error> {
        let res = sqlx::query_as!(
            DbLinkWithScript,
            "SELECT script_links.id, script_links.guild_id, script_links.script_id, \
             script_links.context_type, script_links.context_id, guild_scripts.name as script_name
             FROM script_links
            
            INNER JOIN guild_scripts ON script_id = guild_scripts.id

        WHERE script_links.guild_id = $1;",
            guild_id.0 as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res
            .into_iter()
            .map(|e| ScriptLink {
                script_name: e.script_name,
                context: (e.context_type, e.context_id).into(),
            })
            .collect())
    }

    async fn list_context_scripts(
        &self,
        guild_id: GuildId,
        ctx: ScriptContext,
    ) -> StoreResult<Vec<Script>, Self::Error> {
        let (ctx_type, ctx_id) = ContextPair::from(ctx);

        let links = sqlx::query_as!(
            DbLink,
            "SELECT id, guild_id, script_id, context_type, context_id FROM script_links
        WHERE guild_id = $1 AND context_type = $2 AND context_id = $3;",
            guild_id.0 as i64,
            ctx_type,
            ctx_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(links.len());

        for link in links {
            let script = self.get_db_script_by_id(guild_id, link.script_id).await?;
            result.push(script.into())
        }

        Ok(result)
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
    compiled_js: String,
    name: String,
}

impl From<DbScript> for Script {
    fn from(script: DbScript) -> Self {
        Self {
            id: script.id as u64,
            name: script.name,
            compiled_js: script.compiled_js,
            original_source: script.original_source,
        }
    }
}

#[allow(dead_code)]
struct DbLink {
    id: i64,
    guild_id: i64,
    script_id: i64,
    context_type: i16,
    context_id: i64,
}

#[allow(dead_code)]
struct DbLinkWithScript {
    id: i64,
    guild_id: i64,
    script_id: i64,
    context_type: i16,
    context_id: i64,
    script_name: String,
}

type ContextPair = (i16, i64);

impl From<ScriptContext> for ContextPair {
    fn from(sc: ScriptContext) -> Self {
        match sc {
            ScriptContext::Guild => (1, 0),
            ScriptContext::Channel(cid) => (2, cid.0 as i64),
            ScriptContext::Role(rid) => (3, rid.0 as i64),
        }
    }
}

impl From<ContextPair> for ScriptContext {
    fn from((typ, id): ContextPair) -> Self {
        match typ {
            1 => Self::Guild,
            2 => Self::Channel(ChannelId(id as u64)),
            3 => Self::Role(RoleId(id as u64)),
            _ => panic!("unknown script context type"),
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
