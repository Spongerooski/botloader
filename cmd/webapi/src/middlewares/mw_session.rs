use oauth2::TokenResponse;
use twilight_http::Client;

use axum::{
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::stores::{Session, SessionStore};

#[derive(Clone)]
pub struct LoggedInSession {
    pub raw: Session,
    pub discord_client: Client,
}

impl LoggedInSession {
    pub fn new(raw: Session) -> Self {
        let client = twilight_http::Client::new(format!(
            "Bearer {}",
            raw.discord_oauth2_token.access_token().secret()
        ));

        Self {
            discord_client: client,
            raw,
        }
    }
}

#[derive(Clone)]
pub struct SessionLayer<ST> {
    pub session_store: ST,
}

impl<ST: Clone, S> Layer<S> for SessionLayer<ST> {
    type Service = SessionMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionMiddleware {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct SessionMiddleware<S, ST> {
    pub inner: S,
    pub session_store: ST,
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

        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");

            match auth_header.map(|e| e.to_str()) {
                Some(Ok(t)) => {
                    if let Some(session) = store.get_session(t).await? {
                        let extensions = req.extensions_mut();
                        let logged_in_session = LoggedInSession::new(session);
                        extensions.insert(logged_in_session);
                    }
                }
                Some(Err(e)) => return Err(e.into()),
                None => {}
            };

            inner.call(req).await.map_err(|e| e.into())
        })
    }
}

#[derive(Clone)]
pub struct RequireAuthMiddleware<S> {
    pub inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequireAuthMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
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

        Box::pin(async move {
            let extensions = req.extensions();
            match extensions.get::<LoggedInSession>() {
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
        f.pad("no logged in session")
    }
}

impl std::error::Error for NoSession {}
