use std::{convert::Infallible, sync::Arc};

use async_trait::async_trait;
use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::CsrfToken;
use twilight_model::id::UserId;

use crate::web::{gen_token, CsrfStore, DiscordOauthToken, Session, SessionType};

#[derive(Default, Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<DashMap<String, BareSession>>,
    tokens: Arc<DashMap<UserId, DiscordOauthToken>>,
}
pub struct BareSession {
    pub token: String,
    pub user_id: UserId,
    pub kind: SessionType,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oauth token not found")]
    OauthTokenNotFound,
}

#[async_trait]
impl crate::web::SessionStore for InMemorySessionStore {
    type Error = Error;

    async fn set_user_oatuh_token(
        &self,
        oauth_token: DiscordOauthToken,
    ) -> Result<DiscordOauthToken, Self::Error> {
        let user_id = oauth_token.user.id;
        self.tokens.insert(user_id, oauth_token.clone());
        Ok(oauth_token)
    }

    async fn set_oauth_create_session(
        &self,
        oauth_token: DiscordOauthToken,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        let user_id = oauth_token.user.id;
        self.set_user_oatuh_token(oauth_token).await?;
        self.create_session(user_id, kind).await
    }

    async fn create_session(
        &self,
        user_id: UserId,
        kind: SessionType,
    ) -> Result<Session, Self::Error> {
        let oauth_token = match self.tokens.get(&user_id) {
            Some(t) => t,
            None => return Err(Error::OauthTokenNotFound),
        };

        loop {
            let token = gen_token();
            let session = Session {
                oauth_token: oauth_token.clone(),
                token: token.clone(),
                kind,
            };
            let bare_session = BareSession {
                token: token.clone(),
                user_id,
                kind,
            };

            match self.sessions.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(bare_session);
                    return Ok(session);
                }
            }
        }
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        let bare_session = match self.sessions.get(token) {
            Some(s) => s,
            None => return Ok(None),
        };

        let token = match self.tokens.get(&bare_session.user_id) {
            Some(s) => s,
            None => return Err(Error::OauthTokenNotFound),
        };

        Ok(Some(Session {
            oauth_token: token.clone(),
            token: bare_session.token.clone(),
            kind: bare_session.kind,
        }))
    }

    async fn get_all_sessions(&self, user_id: UserId) -> Result<Vec<Session>, Self::Error> {
        let token = match self.tokens.get(&user_id) {
            Some(s) => s,
            None => return Ok(vec![]),
        };

        Ok(self
            .sessions
            .iter()
            .filter(|e| e.user_id == user_id)
            .map(|e| Session {
                oauth_token: token.clone(),
                token: e.token.clone(),
                kind: e.kind,
            })
            .collect())
    }

    async fn del_session(&self, token: &str) -> Result<bool, Self::Error> {
        Ok(self.sessions.remove(token).is_some())
    }

    async fn del_all_sessions(&self, user_id: UserId) -> Result<(), Self::Error> {
        self.sessions.retain(|_, v| v.user_id != user_id);
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryCsrfStore {
    tokens: DashMap<String, ()>,
}

#[async_trait]
impl CsrfStore for InMemoryCsrfStore {
    type Error = Infallible;

    async fn generate_csrf_token(&self) -> Result<CsrfToken, Self::Error> {
        // altough very very low chance, handle the case where we generate 2 identical tokens
        loop {
            let token = gen_token();
            match self.tokens.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(());
                    return Ok(CsrfToken::new(token));
                }
            }
        }
    }

    async fn check_csrf_token(&self, token: &str) -> Result<bool, Self::Error> {
        Ok(self.tokens.remove(token).is_some())
    }
}
