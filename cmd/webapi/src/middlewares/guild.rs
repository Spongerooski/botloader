use twilight_http::Client;

use axum::{
    extract::{FromRequest, Path, RequestParts},
    http::{Request, Response},
    BoxError,
};
use core::fmt;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::info;
use twilight_model::id::GuildId;

use stores::web::{Session, SessionStore};

use super::LoggedInSession;

#[derive(Clone)]
pub struct CurrentGuildLayer<ST> {
    pub session_store: ST,
}

impl<ST: Clone, S> Layer<S> for CurrentGuildLayer<ST> {
    type Service = CurrentGuildMiddleware<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        CurrentGuildMiddleware {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct CurrentGuildMiddleware<S, ST> {
    pub inner: S,
    pub session_store: ST,
}

#[derive(Clone, serde::Deserialize)]
struct GuildPath {
    guild: GuildId,
}

impl<S, ST, ReqBody, ResBody> Service<Request<ReqBody>> for CurrentGuildMiddleware<S, ST>
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

        Box::pin(async move {
            let mut req_parts = RequestParts::new(req);

            let guild_path = Path::<GuildPath>::from_request(&mut req_parts).await;
            let session: Option<&LoggedInSession> =
                req_parts.extensions().map(|e| e.get()).flatten();

            if let (Some(s), Ok(gp)) = (session, guild_path) {
                let g = fetch_guild(s, gp.guild).await;
            }

            inner
                .call(req_parts.try_into_request().unwrap())
                .await
                .map_err(|e| e.into())
        })
    }
}

async fn fetch_guild<ST>(session: &LoggedInSession<ST>, guild_id: GuildId) {}
