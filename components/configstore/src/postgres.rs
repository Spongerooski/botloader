use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};
use twilight_model::id::{ChannelId, GuildId, RoleId};

use crate::{Script, ScriptContext, ScriptLink, StoreError, StoreResult};

#[derive(Clone)]
pub struct Postgres {
    pool: PgPool,
}

impl Postgres {
    pub fn new_with_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

        Ok(Self { pool })
    }
}

impl Postgres {
    async fn get_db_script_by_name(
        &self,
        guild_id: GuildId,
        script_name: &str,
    ) -> StoreResult<DbScript> {
        Ok(sqlx::query_as!(
            DbScript,
            "SELECT id, guild_id, original_source, compiled_js, name FROM guild_scripts WHERE \
             guild_id = $1 AND name = $2;",
            guild_id.0 as i64,
            script_name
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn get_db_script_by_id(&self, guild_id: GuildId, id: i64) -> StoreResult<DbScript> {
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
impl crate::ConfigStore for Postgres {
    async fn get_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> crate::StoreResult<crate::Script> {
        Ok(self
            .get_db_script_by_name(guild_id, &script_name)
            .await?
            .into())
    }

    async fn create_script(
        &self,
        guild_id: GuildId,
        script: crate::CreateScript,
    ) -> StoreResult<Script> {
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

    async fn update_script(&self, guild_id: GuildId, script: Script) -> StoreResult<Script> {
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

    async fn del_script(&self, guild_id: GuildId, script_name: String) -> crate::StoreResult<()> {
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
            Err(StoreError::ScriptNotFound)
        }
    }

    async fn list_scripts(&self, guild_id: GuildId) -> crate::StoreResult<Vec<crate::Script>> {
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
        ctx: crate::ScriptContext,
    ) -> crate::StoreResult<crate::ScriptLink> {
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
        ctx: crate::ScriptContext,
    ) -> crate::StoreResult<()> {
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
            Err(StoreError::LinkNotFound)
        }
    }

    async fn unlink_all_script(
        &self,
        guild_id: GuildId,
        script_name: String,
    ) -> crate::StoreResult<u64> {
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
    ) -> crate::StoreResult<Vec<crate::ScriptLink>> {
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

    async fn list_links(&self, guild_id: GuildId) -> crate::StoreResult<Vec<crate::ScriptLink>> {
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
        ctx: crate::ScriptContext,
    ) -> crate::StoreResult<Vec<crate::Script>> {
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
    ) -> StoreResult<crate::GuildMetaConfig> {
        match sqlx::query_as!(
            DbGuildMetaConfig,
            "SELECT guild_id, error_channel_id FROM guild_meta_configs
        WHERE guild_id = $1;",
            guild_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(conf) => Ok(conf.into()),
            Err(sqlx::Error::RowNotFound) => Err(StoreError::GuildConfigNotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_guild_meta_config(
        &self,
        conf: &crate::GuildMetaConfig,
    ) -> StoreResult<crate::GuildMetaConfig> {
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

impl From<DbGuildMetaConfig> for crate::GuildMetaConfig {
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
