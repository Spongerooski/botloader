use axum::{extract::Extension, response::IntoResponse, Json};
use stores::web::SessionStore;

use crate::{errors::ApiErrorResponse, middlewares::LoggedInSession, ApiResult};

use tracing::error;

pub async fn get_current_user<ST: SessionStore + 'static>(
    Extension(session): Extension<LoggedInSession<ST>>,
) -> ApiResult<impl IntoResponse> {
    let user = session.api_client.current_user().await.map_err(|err| {
        error!(%err, "failed fetching user");
        ApiErrorResponse::InternalError
    })?;

    Ok(Json(user))
}
