use discordoauthwrapper::{DiscordOauthApiClient, TwilightApiProvider};

use axum::{
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::{
    marker::PhantomData,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::{info, Instrument};

use stores::web::{Session, SessionStore};

#[derive(Clone)]
pub struct LoggedInSession<ST> {
    pub api_client: DiscordOauthApiClient<TwilightApiProvider, oauth2::basic::BasicClient, ST>,
    pub session: Session,
}

impl<T> LoggedInSession<T>
where
    T: SessionStore + 'static,
{
    pub fn new(oauth_client: oauth2::basic::BasicClient, session: Session, store: T) -> Self {
        Self {
            api_client: DiscordOauthApiClient::new_twilight(
                session.user.id,
                session.oauth_token.access_token.clone(),
                oauth_client,
                store,
            ),
            session,
        }
    }
}

#[derive(Clone)]
pub struct SessionLayer<ST> {
    pub session_store: ST,
    pub oauth_conf: oauth2::basic::BasicClient,
}

impl<ST> SessionLayer<ST> {
    pub fn require_auth_layer(&self) -> RequireAuthLayer<ST> {
        RequireAuthLayer {
            _phantom: PhantomData,
        }
    }
}

impl<ST: Clone, S> Layer<S> for SessionLayer<ST> {
    type Service = SessionMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            session_store: self.session_store.clone(),
            oauth_conf: self.oauth_conf.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S, ST> {
    pub inner: S,
    pub session_store: ST,
    pub oauth_conf: oauth2::basic::BasicClient,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for SessionMiddleware<S, ST>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    ST: 'static + SessionStore + Send + Sync + Clone,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let store = self.session_store.clone();
        let oauth_conf = self.oauth_conf.clone();

        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");

            let mut span = None;

            match auth_header.map(|e| e.to_str()) {
                Some(Ok(t)) => {
                    if let Some(session) = store.get_session(t).await? {
                        info!("we are logged in!");
                        let extensions = req.extensions_mut();

                        span = Some(tracing::debug_span!("session", user_id=%session.user.id));

                        let logged_in_session = LoggedInSession::new(oauth_conf, session, store);
                        extensions.insert(logged_in_session);
                    }
                }
                Some(Err(e)) => return Err(e.into()),
                None => {}
            };

            if let Some(s) = span {
                inner.call(req).instrument(s).await.map_err(|e| e.into())
            } else {
                inner.call(req).await.map_err(|e| e.into())
            }
        })
    }
}

#[derive(Clone)]
pub struct RequireAuthLayer<ST> {
    _phantom: PhantomData<ST>,
}

impl<S, ST> Layer<S> for RequireAuthLayer<ST> {
    type Service = RequireAuthMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthMiddleware {
            inner,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct RequireAuthMiddleware<S, ST> {
    inner: S,
    _phantom: PhantomData<ST>,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for RequireAuthMiddleware<S, ST>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    ST: Send + Sync + SessionStore + 'static,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // best practice is to clone the inner service like this
        // see https://github.com/tower-rs/tower/issues/547 for details
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        info!("Running");

        Box::pin(async move {
            let extensions = req.extensions();
            match extensions.get::<LoggedInSession<ST>>() {
                Some(_) => inner.call(req).await.map_err(|e| e.into()),
                None => Err(NoSession(()).into()),
            }
        })
    }
}

#[derive(Debug, Default)]
pub struct NoSession(pub ());

impl fmt::Display for NoSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("no session or session expired")
    }
}

impl std::error::Error for NoSession {}
