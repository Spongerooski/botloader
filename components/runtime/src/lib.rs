use std::sync::Arc;

use contrib_manager::LoadedScript;
use deno_core::{op_async, op_sync, Extension, OpState};
use tracing::info;
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::GuildId;
use vm::{vm::VmRole, AnyError, JsValue};

pub mod commonmodels;
pub mod contrib_manager;
pub mod dispatchevents;
pub mod jsmodules;
mod sendmessage;
pub mod validator;

pub use validator::validate_script;

use crate::commonmodels::script::ScriptMeta;

pub fn create_extension(ctx: RuntimeContext) -> Extension {
    Extension::builder()
        .ops(vec![
            (
                "op_jack_sendmessage",
                op_async(sendmessage::op_send_message),
            ),
            (
                "op_interaction_followup",
                op_async(sendmessage::op_send_interaction_response),
            ),
            ("op_botloader_script_start", op_sync(op_script_start)),
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
    Ok(())
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
        des.script_id,
        des.commands.len() + des.command_groups.len()
    );

    let ctx = state.borrow::<RuntimeContext>();
    ctx.contrib_manager_handle.send(LoadedScript {
        guild_id: ctx.guild_id,
        meta: des,
    });

    Ok(())
}
