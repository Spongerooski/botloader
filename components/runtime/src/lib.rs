use std::sync::Arc;

use contrib_manager::LoadedScript;
use deno_core::{op_async, op_sync, Extension, OpState};
use runtime_models::script::ScriptMeta;
use tracing::info;
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::GuildId;
use vm::{vm::VmRole, AnyError, JsValue};

pub mod contrib_manager;
pub mod dispatchevents;
pub mod jsmodules;
mod ops;
pub mod validator;

pub use validator::validate_script;

pub fn create_extension(ctx: RuntimeContext) -> Extension {
    Extension::builder()
        .ops(vec![
            // botloader stuff
            ("op_botloader_script_start", op_sync(op_script_start)),
            // discord stuff
            ("discord_get_guild", op_sync(ops::discord::op_get_guild)),
            ("discord_edit_guild", op_sync(dummy_op)),
            ("discord_get_message", op_sync(dummy_op)),
            ("discord_get_messages", op_sync(dummy_op)),
            (
                "discord_create_message",
                op_async(ops::discord::op_create_channel_message),
            ),
            (
                "discord_create_followup_message",
                op_async(ops::discord::op_create_followup_message),
            ),
            (
                "discord_edit_message",
                op_async(ops::discord::op_edit_channel_message),
            ),
            (
                "discord_delete_message",
                op_async(ops::discord::op_delete_message),
            ),
            (
                "discord_bulk_delete_messages",
                op_async(ops::discord::op_delete_messages_bulk),
            ),
            ("discord_get_role", op_sync(ops::discord::op_get_role)),
            ("discord_get_roles", op_sync(ops::discord::op_get_roles)),
            ("discord_create_role", op_sync(dummy_op)),
            ("discord_edit_role", op_sync(dummy_op)),
            ("discord_delete_role", op_sync(dummy_op)),
            (
                "discord_get_channel",
                op_async(ops::discord::op_get_channel),
            ),
            (
                "discord_get_channels",
                op_sync(ops::discord::op_get_channels),
            ),
            ("discord_create_channel", op_sync(dummy_op)),
            ("discord_edit_channel", op_sync(dummy_op)),
            ("discord_delete_channel", op_sync(dummy_op)),
            ("discord_get_invite", op_sync(dummy_op)),
            ("discord_get_invites", op_sync(dummy_op)),
            ("discord_create_invite", op_sync(dummy_op)),
            ("discord_delete_invite", op_sync(dummy_op)),
        ])
        .state(move |state| {
            state.put(ctx.clone());
            Ok(())
        })
        .build()
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

pub fn dummy_op(_state: &mut OpState, _args: JsValue, _: ()) -> Result<(), AnyError> {
    Err(anyhow::anyhow!(
        "unimplemented, this op is not implemented yet"
    ))
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub guild_id: GuildId,
    pub bot_state: Arc<InMemoryCache>,
    pub dapi: Arc<twilight_http::Client>,
    pub role: VmRole,
    pub contrib_manager_handle: contrib_manager::ContribManagerHandle,
}

pub fn op_script_start(state: &mut OpState, args: JsValue, _: ()) -> Result<(), AnyError> {
    let des: ScriptMeta = serde_json::from_value(args)?;

    info!(
        "running script! {}, commands: {}",
        des.script_id.0,
        des.commands.len() + des.command_groups.len()
    );

    let ctx = state.borrow::<RuntimeContext>();
    ctx.contrib_manager_handle.send(LoadedScript {
        guild_id: ctx.guild_id,
        meta: des,
    });

    Ok(())
}
