use deno_core::{op_async, op_sync, Extension, OpState};
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::GuildId;
use vm::{vm::VmRole, AnyError, JsValue};

mod commonmodels;
pub mod dispatchevents;
pub mod error_reporter;
pub mod jsmodules;
mod sendmessage;

pub fn create_extension(ctx: RuntimeContext) -> Extension {
    Extension::builder()
        .ops(vec![
            (
                "op_jack_sendmessage",
                op_async(sendmessage::op_send_message),
            ),
            ("op_jack_register_meta", op_sync(dummy_op)),
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
    pub bot_state: InMemoryCache,
    pub dapi: twilight_http::Client,
    pub role: VmRole,
}
