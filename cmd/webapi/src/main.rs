use std::{convert::Infallible, sync::Arc};

use axum::{
    handler::{get, post},
    http::StatusCode,
    response::IntoResponse,
    routing::BoxRoute,
    AddExtensionLayer, BoxError, Router,
};
use config::RunConfig;
use oauth2::basic::BasicClient;
use routes::auth::AuthHandlers;
use stores::{inmemory::web::InMemoryCsrfStore, postgres::Postgres};
use structopt::StructOpt;
use tower::ServiceBuilder;
use tracing::{error, info};

mod config;
mod errors;
mod middlewares;
mod routes;

use errors::ApiErrorResponse;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt::format::FmtSpan, util::SubscriberInitExt, EnvFilter};

use crate::middlewares::{CurrentGuildLayer, RequireCurrentGuildAuthLayer, SessionLayer};

#[derive(Clone)]
pub struct ConfigData {
    oauth_client: BasicClient,
    run_config: RunConfig,
}

type CurrentSessionStore = Postgres;
type CurrentConfigStore = Postgres;
type AuthHandlerData = AuthHandlers<InMemoryCsrfStore, CurrentSessionStore>;
type ApiResult<T> = Result<T, ApiErrorResponse>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    init_tracing();
    info!("starting...");

    let conf = RunConfig::from_args();
    let oatuh_client = conf.get_discord_oauth2_client();

    let postgres_store = Postgres::new_with_url(&conf.database_url).await.unwrap();
    let config_store: CurrentConfigStore = postgres_store.clone();
    let session_store: CurrentSessionStore = postgres_store.clone();

    let auth_handler: AuthHandlerData =
        routes::auth::AuthHandlers::new(session_store.clone(), InMemoryCsrfStore::default());

    let session_layer = SessionLayer {
        session_store: session_store.clone(),
        oauth_conf: oatuh_client.clone(),
    };
    let require_auth_layer = session_layer.require_auth_layer();

    let common_middleware_stack = ServiceBuilder::new() // Process at most 100 requests concurrently
        .layer(AddExtensionLayer::new(ConfigData {
            oauth_client: oatuh_client,
            run_config: conf,
        }))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(Arc::new(auth_handler)))
        .layer(AddExtensionLayer::new(config_store))
        .layer(session_layer)
        .layer(CurrentGuildLayer {
            session_store: session_store.clone(),
        })
        .into_inner();

    // TODO: See about the removal of the boxed method
    let script_routes: Router<BoxRoute> = Router::new()
        .route(
            "/:script_id/update",
            get(routes::scripts::update_guild_script),
        )
        .route(
            "/:script_id/delete",
            get(routes::scripts::delete_guild_script),
        )
        .route("/", get(routes::scripts::get_all_guild_scripts))
        .route("/new", post(routes::scripts::create_guild_script))
        .boxed();

    let authorized_api_guild_routes = Router::new()
        .nest("/scripts", script_routes)
        .boxed()
        .layer(RequireCurrentGuildAuthLayer)
        .handle_error(handle_mw_err_internal_err)
        .boxed();

    let authorized_api_routes = Router::new()
        .nest("/guilds/:guild/", authorized_api_guild_routes)
        .route(
            "/guilds",
            get(routes::guilds::list_user_guilds_route::<CurrentSessionStore, CurrentConfigStore>),
        )
        .boxed();

    let authorized_routes = Router::new()
        .route("/logout", get(AuthHandlerData::handle_logout))
        .nest("/api", authorized_api_routes)
        .boxed()
        .layer(require_auth_layer)
        .layer(common_middleware_stack.clone())
        .boxed();

    let public_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/error", get(routes::errortest::handle_errortest))
        .route("/login", get(AuthHandlerData::handle_login))
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

#[allow(dead_code)]
async fn todo_route() -> &'static str {
    "todo"
}

fn handle_mw_err_internal_err(err: BoxError) -> Result<impl IntoResponse, Infallible> {
    error!("internal error occured: {}", err);

    Ok((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unhandled internal error",
    ))
}
