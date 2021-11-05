use std::sync::Arc;

use crate::{LogEntry, LogLevel};
use stores::config::ConfigStore;
use tracing::error;

pub struct DiscordLogger<CT> {
    discord_client: Arc<twilight_http::Client>,
    config_store: CT,
}

impl<CT> DiscordLogger<CT> {
    pub fn new(discord_client: Arc<twilight_http::Client>, config_store: CT) -> Self {
        Self {
            config_store,
            discord_client,
        }
    }
}

#[async_trait::async_trait]
impl<CT> crate::GuildLoggerBackend for DiscordLogger<CT>
where
    CT: ConfigStore + Send,
    CT::Error: 'static,
{
    async fn handle_entry(&self, entry: LogEntry) {
        let guild_id = entry.guild_id;
        let message = match format_entry(entry) {
            Some(msg) => msg,
            None => return,
        };

        let conf = match self
            .config_store
            .get_guild_meta_config_or_default(guild_id)
            .await
        {
            Ok(v) => v,
            Err(err) => {
                error!(%err, "failed fetching config for guild logging");
                return;
            }
        };

        if let Some(channel_id) = conf.error_channel_id {
            if let Ok(next) = self
                .discord_client
                .create_message(channel_id)
                .content(&message)
            {
                next.exec().await.ok();
            }
        }
    }
}

fn format_entry(entry: LogEntry) -> Option<String> {
    if matches!(entry.level, LogLevel::Error | LogLevel::Critical) {
        let prefix = if let Some(script_ctx) = entry.script_context {
            format!("[{} {}]", entry.level, script_ctx)
        } else {
            format!("[{}]", entry.level)
        };
        Some(format!("{}: {}", prefix, entry.message))
    } else {
        None
    }
}
