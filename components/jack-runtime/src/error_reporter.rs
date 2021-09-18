use async_trait::async_trait;
use twilight_model::id::GuildId;

// A trait for handling errors in scripts mostly
// This might be moved into a more generalized "notification" trait later
//
// We need sync here because the provided method uses &self across an await
// (wich would normaly just make the future !Send but it fucks with the object safety somehow)
#[async_trait]
pub trait ErrorReporter: Sync {
    async fn report_script_error(
        &self,
        guild_id: GuildId,
        error: jack_sandbox::AnyError,
    ) -> Result<(), jack_sandbox::AnyError> {
        self.report_error(
            guild_id,
            format!(
                "An error occured in one of your scripts:```\n{}\n```",
                error
            ),
        )
        .await
    }

    async fn report_error(
        &self,
        guild_id: GuildId,
        error: String,
    ) -> Result<(), jack_sandbox::AnyError>;
}

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
impl<CT: configstore::ConfigStore + Sync + Send> ErrorReporter for DiscordErrorReporter<CT> {
    async fn report_error(
        &self,
        guild_id: GuildId,
        error: String,
    ) -> Result<(), jack_sandbox::AnyError> {
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

pub struct NoOpErrorReporter;

#[async_trait]
impl ErrorReporter for NoOpErrorReporter {
    async fn report_error(&self, _: GuildId, _: String) -> Result<(), jack_sandbox::AnyError> {
        Ok(())
    }
}
