use twilight_model::{
    channel::GuildChannel,
    id::{ChannelId, GenericId},
};
use vm::AnyError;

use crate::RuntimeContext;

pub mod console;
pub mod discord;
pub mod storage;

// ensures the provided channel is in the guild, also checking the api as fallback
pub(crate) async fn get_guild_channel(
    rt_ctx: &RuntimeContext,
    channel_id_str: &str,
) -> Result<GuildChannel, AnyError> {
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
                Ok(c.value().resource().clone())
            }
        }
        None => {
            let channel = rt_ctx
                .dapi
                .channel(channel_id)
                .exec()
                .await?
                .model()
                .await?;

            let gc = match channel {
                twilight_model::channel::Channel::Guild(gc) => gc,
                _ => return Err(anyhow::anyhow!("Unknown channel")),
            };

            if matches!(gc.guild_id(), Some(guild_id) if guild_id == rt_ctx.guild_id) {
                Ok(gc)
            } else {
                Err(anyhow::anyhow!("Unknown channel"))
            }
        }
    }
}

pub(crate) fn parse_str_snowflake_id(id_str: &str) -> Result<GenericId, AnyError> {
    if let Some(id) = GenericId::new(id_str.parse()?) {
        Ok(id)
    } else {
        Err(anyhow::anyhow!("invalid channel id"))
    }
}
