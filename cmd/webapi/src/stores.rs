use std::convert::Infallible;

use async_trait::async_trait;
use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::{basic::BasicTokenType, CsrfToken, EmptyExtraTokenFields, StandardTokenResponse};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use twilight_model::user::CurrentUser;

type OauthToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub discord_oauth2_token: OauthToken,
    pub user: CurrentUser,
    pub token: String,
}

#[async_trait]
pub trait SessionStore {
    type Error: std::error::Error;

    async fn create_session(
        &self,
        user: CurrentUser,
        oauth2_token: OauthToken,
    ) -> Result<Session, Self::Error>;
    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error>;
    async fn del_session(&self, token: &str) -> Result<bool, Self::Error>;
}

#[async_trait]
pub trait CsrfStore {
    type Error: std::error::Error;

    async fn generate_csrf_token(&self) -> Result<CsrfToken, Self::Error>;
    async fn check_csrf_token(&self, token: &str) -> Result<bool, Self::Error>;
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
    type Error = Infallible;

    async fn create_session(
        &self,
        user: CurrentUser,
        oauth2_token: OauthToken,
    ) -> Result<Session, Self::Error> {
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

    async fn get_session(&self, token: &str) -> Result<Option<Session>, Self::Error> {
        Ok(self.sessions.get(token).map(|s| s.clone()))
    }

    async fn del_session(&self, token: &str) -> Result<bool, Self::Error> {
        Ok(self.sessions.remove(token).is_some())
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
