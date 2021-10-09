#[derive(Default, Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<DashMap<String, Session>>,
}

use std::{convert::Infallible, sync::Arc};

use async_trait::async_trait;
use dashmap::{mapref::entry::Entry, DashMap};
use oauth2::{basic::BasicTokenType, CsrfToken, EmptyExtraTokenFields, StandardTokenResponse};
use twilight_model::user::CurrentUser;

use crate::web::{gen_token, CsrfStore, Session};

type OauthToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

#[async_trait]
impl crate::web::SessionStore for InMemorySessionStore {
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
