use deno_core::OpState;
use serde::Deserialize;
use std::{cell::RefCell, rc::Rc};
use twilight_model::{channel::Message, id::ChannelId};
use vm::AnyError;

use crate::RuntimeContext;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageArgs {
    content: String,
    channel_id: ChannelId,
}

pub async fn op_send_message(
    state: Rc<RefCell<OpState>>,
    args: SendMessageArgs,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let _ = match rt_ctx.bot_state.guild_channel(args.channel_id) {
        Some(c) => c,
        None => return Err(anyhow::anyhow!("Unknown channel")),
    };

    let re = rt_ctx
        .dapi
        .create_message(args.channel_id)
        .content(&args.content)?
        .exec()
        .await?
        .model()
        .await?;

    Ok(re)
}
