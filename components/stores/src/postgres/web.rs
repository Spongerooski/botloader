use crate::web::{gen_token, DiscordOauthToken, Session, SessionType};

use super::Postgres;
use async_trait::async_trait;
use twilight_model::{id::UserId, user::CurrentUser};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oauth token not found")]
    OauthTokenNotFound,

    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

#[async_trait]
impl crate::web::SessionStore for Postgres {
    type Error = Error;

    async fn set_user_oatuh_token(
        &self,
        oauth2_token: DiscordOauthToken,
    ) -> Result<DiscordOauthToken, Self::Error> {
        Ok(sqlx::query_as!(
            DbOauthToken,
            "INSERT INTO discord_oauth_tokens (user_id, discord_bearer_token, \
             discord_refresh_token, discord_token_expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET 
            discord_bearer_token = $2,
            discord_refresh_token = $3,
            discord_token_expires_at = $4
            RETURNING user_id, discord_bearer_token, discord_refresh_token, \
             discord_token_expires_at;",
            oauth2_token.user_id.0 as i64,
            oauth2_token.access_token,
            oauth2_token.refresh_token,
            oauth2_token.token_expires,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    async fn set_oauth_create_session(
        &self,
        oauth2_token: DiscordOauthToken,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        self.set_user_oatuh_token(oauth2_token).await?;
        Ok(self.create_session(user, kind).await?)
    }

    async fn create_session(
        &self,
        user: CurrentUser,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        let oauth_token = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user.id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await?;

        let token = gen_token();

        sqlx::query_as!(
            DbSession,
            "INSERT INTO web_sessions (token, kind, user_id, discriminator, username, avatar) \
             VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING token, kind, user_id, discriminator, username, avatar;",
            &token,
            i16::from(kind),
            user.id.0 as i64,
            user.discriminator.parse::<i16>().unwrap_or_default(),
            user.name,
            user.avatar,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Session {
            oauth_token: oauth_token.into(),
            token,
            kind,
            user,
        })
    }

    async fn get_oauth_token(&self, user_id: UserId) -> Result<DiscordOauthToken, Self::Error> {
        Ok(sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        let session = sqlx::query_as!(
            DbSession,
            "SELECT token, kind, user_id, discriminator, username, avatar FROM web_sessions WHERE \
             token = $1;",
            token
        )
        .fetch_one(&self.pool)
        .await?;

        let oauth_token = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            session.user_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(Session {
            token: token.to_string(),
            kind: SessionType::from(session.kind),
            oauth_token: oauth_token.into(),
            user: session.into(),
        }))
    }
    async fn get_all_sessions(&self, user_id: UserId) -> Result<Vec<Session>, Self::Error> {
        let oauth_token: DiscordOauthToken = sqlx::query_as!(
            DbOauthToken,
            "SELECT user_id, discord_bearer_token, discord_refresh_token, discord_token_expires_at
            FROM discord_oauth_tokens WHERE user_id = $1",
            user_id.0 as i64,
        )
        .fetch_one(&self.pool)
        .await?
        .into();

        let sessions = sqlx::query_as!(
            DbSession,
            "SELECT token, kind, user_id, discriminator, username, avatar FROM web_sessions WHERE \
             user_id = $1",
            user_id.0 as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions
            .into_iter()
            .map(|e| Session {
                token: e.token.clone(),
                kind: e.kind.into(),
                oauth_token: oauth_token.clone(),
                user: e.into(),
            })
            .collect())
    }

    async fn del_session(&self, token: &str) -> Result<bool, Self::Error> {
        let res = sqlx::query!("DELETE FROM web_sessions WHERE token= $1", token,)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() > 0)
    }

    async fn del_all_sessions(&self, user_id: UserId) -> Result<(), Self::Error> {
        sqlx::query!(
            "DELETE FROM discord_oauth_tokens WHERE user_id= $1",
            user_id.0 as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct DbOauthToken {
    user_id: i64,
    discord_bearer_token: String,
    discord_refresh_token: String,
    discord_token_expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<DbOauthToken> for DiscordOauthToken {
    fn from(db_t: DbOauthToken) -> Self {
        Self {
            access_token: db_t.discord_bearer_token,
            refresh_token: db_t.discord_refresh_token,
            token_expires: db_t.discord_token_expires_at,
            user_id: UserId(db_t.user_id as u64),
        }
    }
}

struct DbSession {
    token: String,
    kind: i16,
    user_id: i64,
    discriminator: i16,
    username: String,
    avatar: String,
}

impl From<DbSession> for CurrentUser {
    fn from(db_u: DbSession) -> Self {
        Self {
            avatar: if !db_u.avatar.is_empty() {
                Some(db_u.avatar)
            } else {
                None
            },
            bot: false,
            discriminator: db_u.discriminator.to_string(),
            email: None,
            flags: None,
            id: UserId(db_u.user_id as u64),
            locale: None,
            mfa_enabled: false,
            name: db_u.username,
            premium_type: None,
            public_flags: None,
            verified: None,
        }
    }
}

impl From<SessionType> for i16 {
    fn from(st: SessionType) -> Self {
        match st {
            SessionType::User => 1,
            SessionType::ApiKey => 2,
        }
    }
}

impl From<i16> for SessionType {
    fn from(st: i16) -> Self {
        match st {
            1 => SessionType::User,
            2 => SessionType::ApiKey,
            _ => panic!("unknown variant of sessiontype: {}", st),
        }
    }
}
