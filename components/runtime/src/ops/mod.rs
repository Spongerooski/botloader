use twilight_model::id::{ChannelId, GenericId};
use vm::AnyError;

use crate::RuntimeContext;

pub mod discord;

/// Ensures that a guild channel resides in the rt_ctx guild
fn check_guild_channel(
    rt_ctx: &RuntimeContext,
    channel_id_str: &str,
) -> Result<ChannelId, AnyError> {
    let channel_id = if let Some(channel_id) = ChannelId::new(channel_id_str.parse()?) {
        channel_id
    } else {
        return Err(anyhow::anyhow!("invalid channel id"));
    };

    match rt_ctx.bot_state.guild_channel(channel_id) {
        Some(c) => {
            if c.value().guild_id() != rt_ctx.guild_id {
                Err(anyhow::anyhow!("Unknown channel"))
            } else {
                Ok(channel_id)
            }
        }
        None => Err(anyhow::anyhow!("Unknown channel")),
    }
}

fn parse_str_snowflake_id(id_str: &str) -> Result<GenericId, AnyError> {
    if let Some(id) = GenericId::new(id_str.parse()?) {
        Ok(id)
    } else {
        Err(anyhow::anyhow!("invalid channel id"))
    }
}
