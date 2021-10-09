use async_trait::async_trait;
use twilight_model::id::GuildId;
use vm::{error_reporter::ErrorReporter, AnyError};

// A trait for handling errors in scripts mostly
// This might be moved into a more generalized "notification" trait later
//
// We need sync here because the provided method uses &self across an await

#[derive(Debug)]
pub struct DiscordErrorReporter<CT> {
    config_storage: CT,
    discord_client: twilight_http::Client,
}

impl<CT> DiscordErrorReporter<CT> {
    pub fn new(config_storage: CT, discord_client: twilight_http::Client) -> Self {
        Self {
            config_storage,
            discord_client,
        }
    }
}

#[async_trait]
impl<CT> ErrorReporter for DiscordErrorReporter<CT>
where
    CT: stores::config::ConfigStore + Sync + Send,
    CT::Error: 'static,
{
    async fn report_error(&self, guild_id: GuildId, error: String) -> Result<(), AnyError> {
        let conf = self
            .config_storage
            .get_guild_meta_config_or_default(guild_id)
            .await?;

        if let Some(channel_id) = conf.error_channel_id {
            self.discord_client
                .create_message(channel_id)
                .content(&error)?
                .exec()
                .await?;
        }

        Ok(())
    }
}
