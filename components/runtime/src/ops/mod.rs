use twilight_model::id::ChannelId;
use vm::AnyError;

use crate::RuntimeContext;

pub mod discord;

/// Ensures that a guild channel resides in the rt_ctx guild
fn check_guild_channel(rt_ctx: &RuntimeContext, channel_id: ChannelId) -> Result<(), AnyError> {
    match rt_ctx.bot_state.guild_channel(channel_id) {
        Some(c) => {
            if c.value().guild_id() != rt_ctx.guild_id {
                Err(anyhow::anyhow!("Unknown channel"))
            } else {
                Ok(())
            }
        }
        None => Err(anyhow::anyhow!("Unknown channel")),
    }
}
