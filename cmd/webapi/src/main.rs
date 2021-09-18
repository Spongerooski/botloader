use std::sync::Arc;

use axum::{
    extract,
    handler::get,
    http::{header::LOCATION, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    AddExtensionLayer, Router,
};
use config::RunConfig;
use structopt::StructOpt;

mod config;
mod errors;

use errors::ApiError;

#[tokio::main]
async fn main() {
    let conf = RunConfig::from_args();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/login", get(handle_login))
        .route("/confirm_login", get(confirm_login))
        .layer(AddExtensionLayer::new(conf));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

type ApiResult<T> = Result<T, ApiError>;

async fn handle_login(conf: extract::Extension<Arc<RunConfig>>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        HeaderValue::from_static("https://discord.com/api/oauth2/authorize"),
    );
    (StatusCode::SEE_OTHER, headers)
}
async fn confirm_login() -> impl IntoResponse {}
