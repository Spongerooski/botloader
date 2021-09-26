use std::sync::Arc;

use axum::{handler::get, AddExtensionLayer, Router};
use config::RunConfig;
use oauth2::basic::BasicClient;
use routes::auth::AuthHandlers;
use stores::{InMemoryCsrfStore, InMemorySessionStore};
use structopt::StructOpt;
use tracing::info;

mod config;
mod errors;
mod routes;
mod stores;

use errors::ApiError;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt::format::FmtSpan, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct ConfigData {
    oauth_client: BasicClient,
    run_config: RunConfig,
}

type AuthHandlerData = AuthHandlers<InMemoryCsrfStore, InMemorySessionStore>;
type ApiResult<T> = Result<T, ApiError>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    init_tracing();
    info!("starting...");

    let conf = RunConfig::from_args();
    let oatuh_client = conf.get_discord_oauth2_client();

    let auth_handler: AuthHandlerData = routes::auth::AuthHandlers::new(
        InMemorySessionStore::default(),
        InMemoryCsrfStore::default(),
    );

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/error", get(routes::errortest::handle_errortest))
        .route("/login", get(AuthHandlerData::handle_login))
        .route("/confirm_login", get(AuthHandlerData::handle_confirm_login))
        .layer(AddExtensionLayer::new(ConfigData {
            oauth_client: oatuh_client,
            run_config: conf,
        }))
        .layer(AddExtensionLayer::new(Arc::new(auth_handler)))
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
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
