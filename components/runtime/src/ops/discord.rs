use std::{cell::RefCell, rc::Rc};

use deno_core::OpState;
use vm::{AnyError, JsValue};

use crate::{
    commonmodels::{
        guild::Guild,
        message::Message,
        ops_messages::{
            OpCreateChannelMessage, OpCreateFollowUpMessage, OpDeleteMessage, OpDeleteMessagesBulk,
            OpEditChannelMessage,
        },
    },
    RuntimeContext,
};

use super::check_guild_channel;

pub fn op_get_guild(state: &mut OpState, _args: JsValue, _: ()) -> Result<Guild, AnyError> {
    let rt_ctx = state.borrow::<RuntimeContext>();

    match rt_ctx.bot_state.guild(rt_ctx.guild_id) {
        Some(c) => Ok(c.value().into()),
        None => Err(anyhow::anyhow!("guild not in state")),
    }
}

pub async fn op_create_channel_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateChannelMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_guild_channel(&rt_ctx, args.channel_id)?;

    let maybe_embeds = args
        .fields
        .embeds
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    let mut mc = rt_ctx
        .dapi
        .create_message(args.channel_id)
        .content(&args.fields.content)?
        .embeds(&maybe_embeds)?;

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_edit_channel_message(
    state: Rc<RefCell<OpState>>,
    args: OpEditChannelMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_guild_channel(&rt_ctx, args.channel_id)?;

    let maybe_embeds = args
        .fields
        .embeds
        .map(|inner| inner.into_iter().map(Into::into).collect::<Vec<_>>());

    let mut mc = rt_ctx
        .dapi
        .update_message(args.channel_id, args.message_id)
        .content(args.fields.content.as_deref())?;

    if let Some(embeds) = &maybe_embeds {
        mc = mc.embeds(embeds)?;
    }

    if let Some(mentions) = args.fields.allowed_mentions {
        mc = mc.allowed_mentions(mentions.into());
    }

    Ok(mc.exec().await?.model().await?.into())
}

pub async fn op_create_followup_message(
    state: Rc<RefCell<OpState>>,
    args: OpCreateFollowUpMessage,
    _: (),
) -> Result<Message, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let re = rt_ctx
        .dapi
        .create_followup_message(&args.interaction_token)
        .unwrap()
        .content(&args.fields.content)
        .exec()
        .await?
        .model()
        .await?;

    Ok(re.into())
}

pub async fn op_delete_message(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessage,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_guild_channel(&rt_ctx, args.channel_id)?;
    rt_ctx
        .dapi
        .delete_message(args.channel_id, args.message_id)
        .exec()
        .await?;

    Ok(())
}

pub async fn op_delete_messages_bulk(
    state: Rc<RefCell<OpState>>,
    args: OpDeleteMessagesBulk,
    _: (),
) -> Result<(), AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_guild_channel(&rt_ctx, args.channel_id)?;
    rt_ctx
        .dapi
        .delete_messages(args.channel_id, &args.message_ids)
        .exec()
        .await?;

    Ok(())
}
