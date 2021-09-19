use std::sync::Arc;

use axum::{
    extract::{self, Query},
    handler::get,
    http::{header::LOCATION, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    AddExtensionLayer, Router,
};
use config::RunConfig;
use oauth2::{basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope};
use structopt::StructOpt;

mod config;
mod errors;

use errors::ApiError;

#[derive(Clone)]
struct ConfigData {
    oauth_client: BasicClient,
    run_config: RunConfig,
}

#[tokio::main]
async fn main() {
    let conf = RunConfig::from_args();
    let oatuh_client = conf.get_discord_oauth2_client();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/login", get(handle_login))
        .route("/confirm_login", get(confirm_login))
        .layer(AddExtensionLayer::new(ConfigData {
            oauth_client: oatuh_client,
            run_config: conf,
        }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

type ApiResult<T> = Result<T, ApiError>;

async fn handle_login(conf: extract::Extension<ConfigData>) -> impl IntoResponse {
    // Generate the full authorization URL.
    let (auth_url, _) = conf
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("guilds".to_string()))
        // Set the PKCE code challenge.
        // .set_pkce_challenge(pkce_challenge)
        // TODO: Do we need to use pkce challenges? wouldn't it be enough to verify the "state" parameter alone?
        .url();

    // TODO: store csrf token

    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        HeaderValue::from_str(&auth_url.to_string()).unwrap(),
    );
    (StatusCode::SEE_OTHER, headers)
}

#[derive(serde::Deserialize)]
struct ConfirmLoginQuery {
    code: String,
    state: String,
}

async fn confirm_login(
    conf: extract::Extension<ConfigData>,
    Query(data): Query<ConfirmLoginQuery>,
) -> ApiResult<Html<String>> {
    let token_result = conf
        .oauth_client
        .exchange_code(AuthorizationCode::new(data.code))
        // Set the PKCE code verifier.
        .request_async(async_http_client)
        .await
        .map_err(|err| ApiError::Other(err.into()))?;

    dbg!(token_result);

    Ok(Html(
        "
        <html>
        <body>Login successfull!</body>
        </html>"
            .to_string(),
    ))
}
