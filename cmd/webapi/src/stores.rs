use async_trait::async_trait;
use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::{basic::BasicTokenType, CsrfToken, EmptyExtraTokenFields, StandardTokenResponse};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use twilight_model::user::CurrentUser;

use crate::errors::ApiError;

type OauthToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub discord_oauth2_token: OauthToken,
    pub user: CurrentUser,
    pub token: String,
}

#[async_trait]
pub trait SessionStore {
    async fn create_session(
        &self,
        user: CurrentUser,
        oauth2_token: OauthToken,
    ) -> Result<Session, ApiError>;
    async fn get_session(&self, token: &str) -> Result<Session, ApiError>;
    async fn del_session(&self, token: &str) -> Result<(), ApiError>;
}

#[async_trait]
pub trait CsrfStore {
    async fn generate_csrf_token(&self) -> Result<CsrfToken, ApiError>;
    async fn check_csrf_token(&self, token: &str) -> Result<bool, ApiError>;
}

fn gen_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    base64::encode_config(&random_bytes, base64::URL_SAFE_NO_PAD)
}

#[derive(Default)]
pub struct InMemorySessionStore {
    sessions: DashMap<String, Session>,
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create_session(
        &self,
        user: CurrentUser,
        oauth2_token: OauthToken,
    ) -> Result<Session, ApiError> {
        // altough very very low chance, handle the case where we generate 2 identical tokens
        loop {
            let token = gen_token();
            let session = Session {
                discord_oauth2_token: oauth2_token.clone(),
                token: token.clone(),
                user: user.clone(),
            };

            match self.sessions.entry(token.clone()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => {
                    e.insert(session.clone());
                    return Ok(session);
                }
            }
        }
    }

    async fn get_session(&self, token: &str) -> Result<Session, ApiError> {
        match self.sessions.get(token) {
            Some(session) => Ok((*session).clone()),
            None => Err(ApiError::SessionExpired),
        }
    }

    async fn del_session(&self, token: &str) -> Result<(), ApiError> {
        match self.sessions.remove(token) {
            Some(_) => Ok(()),
            None => Err(ApiError::SessionExpired),
        }
    }
}

#[derive(Default)]
pub struct InMemoryCsrfStore {
    tokens: DashMap<String, ()>,
}

#[async_trait]
impl CsrfStore for InMemoryCsrfStore {
    async fn generate_csrf_token(&self) -> Result<CsrfToken, ApiError> {
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

    async fn check_csrf_token(&self, token: &str) -> Result<bool, ApiError> {
        Ok(self.tokens.remove(token).is_some())
    }
}
