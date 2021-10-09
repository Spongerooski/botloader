use async_trait::async_trait;
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
    type Error: std::error::Error + Send + Sync;

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

pub fn gen_token() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| thread_rng().gen::<u8>()).collect();
    base64::encode_config(&random_bytes, base64::URL_SAFE_NO_PAD)
}
