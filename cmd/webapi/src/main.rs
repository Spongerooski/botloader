use std::sync::Arc;

use axum::{handler::get, AddExtensionLayer, Router};
use config::RunConfig;
use oauth2::basic::BasicClient;
use routes::auth::AuthHandlers;
use stores::inmemory::web::{InMemoryCsrfStore, InMemorySessionStore};
use structopt::StructOpt;
use tower::ServiceBuilder;
use tracing::info;

mod config;
mod errors;
mod middlewares;
mod routes;

use errors::ApiErrorResponse;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt::format::FmtSpan, util::SubscriberInitExt, EnvFilter};

use crate::middlewares::{RequireAuthLayer, SessionLayer};

#[derive(Clone)]
pub struct ConfigData {
    oauth_client: BasicClient,
    run_config: RunConfig,
}

type AuthHandlerData = AuthHandlers<InMemoryCsrfStore, InMemorySessionStore>;
type ApiResult<T> = Result<T, ApiErrorResponse>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    init_tracing();
    info!("starting...");

    let conf = RunConfig::from_args();
    let oatuh_client = conf.get_discord_oauth2_client();

    let session_store = InMemorySessionStore::default();
    let auth_handler: AuthHandlerData =
        routes::auth::AuthHandlers::new(session_store.clone(), InMemoryCsrfStore::default());

    let common_middleware_stack = ServiceBuilder::new() // Process at most 100 requests concurrently
        .layer(AddExtensionLayer::new(ConfigData {
            oauth_client: oatuh_client,
            run_config: conf,
        }))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(Arc::new(auth_handler)))
        .layer(SessionLayer { session_store })
        .into_inner();

    // TODO: See about the removal of the boxed method

    let authorized_routes = Router::new()
        .boxed()
        .route("/logout", get(AuthHandlerData::handle_logout))
        .layer(RequireAuthLayer)
        .layer(common_middleware_stack.clone())
        .boxed();

    let public_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .boxed()
        .route("/error", get(routes::errortest::handle_errortest))
        .boxed()
        .route("/login", get(AuthHandlerData::handle_login))
        .boxed()
        .route("/confirm_login", get(AuthHandlerData::handle_confirm_login))
        .boxed()
        .layer(common_middleware_stack.clone())
        .boxed();

    let app = public_routes.or(authorized_routes);
    let make_service = app.into_make_service();

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(make_service)
        .await
        .unwrap();
}

fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
        .init();
}
