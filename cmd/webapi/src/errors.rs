use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("unknown error occured: {0}")]
    Other(#[from] anyhow::Error),
}

impl ApiError {
    pub fn public_desc(&self) -> (StatusCode, u32, String) {
        match &self {
            ApiError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, 0, format!("{}", e)),
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
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}
