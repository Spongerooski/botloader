use std::{
    fmt::{Debug, Display},
    future::Future,
    sync::{Arc, RwLock},
};

use oauth2::reqwest::async_http_client;
use stores::web::{DiscordOauthToken, SessionStore};
use twilight_http::api_error::{ApiError, ErrorCode, GeneralApiError};
use twilight_model::{
    id::UserId,
    user::{CurrentUser, CurrentUserGuild},
};

struct ApiClientInner<T, TU, ST> {
    user_id: UserId,
    api_provider: T,
    token_refresher: TU,
    session_store: ST,
}

pub struct DiscordOauthApiClient<T, TU, ST> {
    inner: Arc<ApiClientInner<T, TU, ST>>,
}

impl<T, TU, ST> Clone for DiscordOauthApiClient<T, TU, ST> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TU, ST> DiscordOauthApiClient<TwilightApiProvider, TU, ST>
where
    TU: TokenRefresher,
    ST: SessionStore,
{
    pub fn new_twilight(
        user_id: UserId,
        bearer_token: String,
        token_refresher: TU,
        session_store: ST,
    ) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                api_provider: TwilightApiProvider {
                    client: RwLock::new(twilight_http::Client::new(format!(
                        "Bearer {}",
                        bearer_token
                    ))),
                },
                user_id,
                token_refresher,
                session_store,
            }),
        }
    }
}

impl<T, TU, ST> DiscordOauthApiClient<T, TU, ST>
where
    T: DiscordOauthApiProvider + 'static,
    TU: TokenRefresher + 'static,
    ST: SessionStore + 'static,
    T::OtherError: Debug + Display + 'static,
{
    pub fn new(user_id: UserId, api_provider: T, token_refresher: TU, session_store: ST) -> Self {
        Self {
            inner: Arc::new(ApiClientInner {
                user_id,
                api_provider,
                token_refresher,
                session_store,
            }),
        }
    }

    pub async fn current_user(&self) -> Result<CurrentUser, Box<dyn std::error::Error>> {
        self.run_api_check_err(|| self.inner.api_provider.get_current_user())
            .await
    }

    pub async fn current_user_guilds(
        &self,
    ) -> Result<Vec<CurrentUserGuild>, Box<dyn std::error::Error>> {
        self.run_api_check_err(|| self.inner.api_provider.get_user_guilds())
            .await
    }

    // runs the provided closure, refreshing the token if needed
    async fn run_api_check_err<F, FRT, Fut>(&self, f: F) -> Result<FRT, Box<dyn std::error::Error>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<FRT, ApiProviderError<T::OtherError>>>,
    {
        match f().await {
            Ok(v) => Ok(v),
            Err(ApiProviderError::InvalidToken) => {
                self.update_token().await?;
                Ok(f().await?)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn update_token(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current = self
            .inner
            .session_store
            .get_oauth_token(self.inner.user_id)
            .await?;

        let new_token = DiscordOauthToken::new(
            self.inner.user_id,
            self.inner.token_refresher.update_token(current).await?,
        );

        let access_token = new_token.access_token.clone();
        self.inner
            .session_store
            .set_user_oatuh_token(new_token)
            .await?;

        self.inner.api_provider.update_token(access_token).await;

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait TokenRefresher {
    async fn update_token(
        &self,
        token: DiscordOauthToken,
    ) -> Result<stores::web::OauthToken, Box<dyn std::error::Error>>;
}

#[derive(Debug)]
pub enum ApiProviderError<T> {
    InvalidToken,
    Other(T),
}

impl<T: std::fmt::Debug + Display> std::error::Error for ApiProviderError<T> {}

impl<T: std::fmt::Debug + Display> Display for ApiProviderError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("invalid token"),
            Self::Other(inner) => f.write_fmt(format_args!("{}", inner)),
        }
    }
}

#[async_trait::async_trait]
pub trait DiscordOauthApiProvider {
    type OtherError;

    async fn get_current_user(&self) -> Result<CurrentUser, ApiProviderError<Self::OtherError>>;
    async fn get_user_guilds(
        &self,
    ) -> Result<Vec<CurrentUserGuild>, ApiProviderError<Self::OtherError>>;
    async fn update_token(&self, access_token: String);
}

pub struct TwilightApiProvider {
    client: RwLock<twilight_http::Client>,
}

impl TwilightApiProvider {
    fn clone_client(&self) -> twilight_http::Client {
        let client = self.client.read().unwrap();
        client.clone()
    }
}

#[async_trait::async_trait]
impl DiscordOauthApiProvider for TwilightApiProvider {
    type OtherError = twilight_http::Error;

    async fn get_current_user(&self) -> Result<CurrentUser, ApiProviderError<Self::OtherError>> {
        let client = self.clone_client();
        Ok(client.current_user().exec().await?.model().await.unwrap())
    }
    async fn get_user_guilds(
        &self,
    ) -> Result<Vec<CurrentUserGuild>, ApiProviderError<Self::OtherError>> {
        let client = self.clone_client();
        Ok(client
            .current_user_guilds()
            .exec()
            .await?
            .model()
            .await
            .unwrap())
    }

    async fn update_token(&self, access_token: String) {
        let new_client = twilight_http::Client::new(format!("Bearer {}", access_token));
        let mut client = self.client.write().unwrap();
        *client = new_client;
    }
}

impl From<twilight_http::Error> for ApiProviderError<twilight_http::Error> {
    fn from(te: twilight_http::Error) -> Self {
        match te.kind() {
            twilight_http::error::ErrorType::Response {
                error:
                    ApiError::General(GeneralApiError {
                        code: ErrorCode::UnknownToken | ErrorCode::InvalidOAuthAccessToken,
                        ..
                    }),
                ..
            } => Self::InvalidToken,
            _ => Self::Other(te),
        }
    }
}

#[async_trait::async_trait]
impl TokenRefresher for oauth2::basic::BasicClient {
    async fn update_token(
        &self,
        token: DiscordOauthToken,
    ) -> Result<stores::web::OauthToken, Box<dyn std::error::Error>> {
        let token = oauth2::RefreshToken::new(token.refresh_token);

        Ok(self
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await?)
    }
}
