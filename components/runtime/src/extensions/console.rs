use deno_core::{op_sync, Extension, OpState};
use guild_logger::LogEntry;
use runtime_models::ops::console::LogMessage;
use vm::AnyError;

use crate::RuntimeContext;

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![("op_botloader_log", op_sync(console_log))])
        .build()
}

pub fn console_log(state: &mut OpState, args: LogMessage, _: ()) -> Result<(), AnyError> {
    let ctx = state.borrow::<RuntimeContext>();

    ctx.guild_logger.log(LogEntry::script_console(
        ctx.guild_id,
        args.message,
        args.file_name.unwrap_or_default(),
        if let Some(line) = args.line_number {
            Some((line, args.col_number.unwrap_or_default()))
        } else {
            None
        },
    ));

    Ok(())
}
