use crate::{errors::ApiError, ApiResult};

use tracing::instrument;

#[instrument]
pub async fn handle_errortest() -> ApiResult<()> {
    Err(ApiError::Other(anyhow::anyhow!(
        "Yup this is an error alright"
    )))
}
