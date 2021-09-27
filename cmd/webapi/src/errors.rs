use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiErrorResponse {
    #[error("csrf token expired")]
    BadCsrfToken,

    #[error("Session expired")]
    SessionExpired,

    #[error("Internal server error")]
    InternalError,
}

impl ApiErrorResponse {
    pub fn public_desc(&self) -> (StatusCode, u32, String) {
        let (resp_code, err_code) = match &self {
            Self::SessionExpired => (StatusCode::BAD_REQUEST, 1),
            Self::BadCsrfToken => (StatusCode::BAD_REQUEST, 2),
            Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, 3),
        };

        (resp_code, err_code, format!("{}", self))
    }
}

impl IntoResponse for ApiErrorResponse {
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
