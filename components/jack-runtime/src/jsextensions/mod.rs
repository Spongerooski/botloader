use deno_core::{op_async, op_sync, Extension, OpState};
use jack_sandbox::{AnyError, JsValue};

pub mod jackcore;
pub mod jsmodules;
pub mod sendmessage;

pub fn init() -> Extension {
    Extension::builder()
        .ops(vec![
            (
                "op_jack_sendmessage",
                op_async(sendmessage::op_send_message),
            ),
            ("op_jack_register_meta", op_sync(dummy_op)),
        ])
        .js(vec![(
            "jack_core.js",
            in_mem_source_load_fn(include_str!("jack_core.js")),
        )])
        .build()
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

pub fn dummy_op(_state: &mut OpState, _args: JsValue, _: ()) -> Result<(), AnyError> {
    Ok(())
}
