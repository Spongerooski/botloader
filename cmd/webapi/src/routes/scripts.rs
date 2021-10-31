use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use stores::config::{ConfigStore, CreateScript, Script};
use tracing::error;
use twilight_model::user::CurrentUserGuild;

use crate::{errors::ApiErrorResponse, ApiResult, CurrentConfigStore};

pub async fn get_all_guild_scripts(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
) -> ApiResult<impl IntoResponse> {
    let scripts = config_store
        .list_scripts(current_guild.id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild scripts");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(scripts))
}

#[derive(Deserialize)]
pub struct GuildScriptPathParams {
    script_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRequestData {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
}

pub async fn create_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Json(payload): Json<CreateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let script = config_store
        .create_script(
            current_guild.id,
            CreateScript {
                enabled: payload.enabled,
                original_source: payload.original_source,
                name: payload.name,
            },
        )
        .await
        .map_err(|err| {
            error!(%err, "failed creating guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRequestData {
    pub name: String,
    pub original_source: String,
    pub enabled: bool,
}

pub async fn update_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
    Json(payload): Json<UpdateRequestData>,
) -> ApiResult<impl IntoResponse> {
    let script = config_store
        .update_script(
            current_guild.id,
            Script {
                id: script_id,
                enabled: payload.enabled,
                original_source: payload.original_source,
                name: payload.name,
            },
        )
        .await
        .map_err(|err| {
            error!(%err, "failed updating guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}

pub async fn delete_guild_script(
    Extension(config_store): Extension<CurrentConfigStore>,
    Extension(current_guild): Extension<CurrentUserGuild>,
    Path(GuildScriptPathParams { script_id }): Path<GuildScriptPathParams>,
) -> ApiResult<impl IntoResponse> {
    let script = config_store
        .get_script_by_id(current_guild.id, script_id)
        .await
        .map_err(|err| {
            error!(%err, "failed fetching guild script");
            ApiErrorResponse::InternalError
        })?;

    let script = config_store
        .del_script(current_guild.id, script.name)
        .await
        .map_err(|err| {
            error!(%err, "failed deleting guild script");
            ApiErrorResponse::InternalError
        })?;

    Ok(Json(script))
}
