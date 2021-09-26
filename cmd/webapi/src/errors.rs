use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    DiscordDeserializeBodyError(#[from] twilight_http::response::DeserializeBodyError),

    #[error(transparent)]
    DiscordAPIError(#[from] twilight_http::Error),

    #[error("csrf token expired")]
    BadCsrfToken,

    #[error("Session expired")]
    SessionExpired,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ApiError {
    pub fn public_desc(&self) -> (StatusCode, u32, String) {
        match &self {
            Self::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                0,
                "an unknown error occured".to_string(),
            ),
            Self::SessionExpired => (StatusCode::BAD_REQUEST, 1, "session expired".to_string()),
            Self::BadCsrfToken => (StatusCode::BAD_REQUEST, 2, "csrf token expired".to_string()),
            Self::DiscordDeserializeBodyError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                3,
                "failed deserializing discord response".to_string(),
            ),
            Self::DiscordAPIError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                4,
                "failed interacting with the discord API".to_string(),
            ),
        }
    }
}

impl IntoResponse for ApiError {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        let (resp_code, err_code, msg) = self.public_desc();

        let body = json!({
            "code": err_code,
            "description": msg,
        })
        .to_string();

        Response::builder()
            .status(resp_code)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}
