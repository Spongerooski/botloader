use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;
use twilight_model::id::GuildId;

#[derive(Debug, Error)]
pub enum TimerStoreError<T: std::fmt::Debug + Error + 'static> {
    #[error("inner error occured: {0}")]
    Other(#[from] T),
}

pub type StoreResult<T, U> = Result<T, TimerStoreError<U>>;

#[async_trait::async_trait]
pub trait TimerStore {
    type Error: std::error::Error + Send + Sync;

    async fn get_all_interval_timers(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<Vec<IntervalTimer>, Self::Error>;
    async fn update_interval_timer(
        &self,
        guild_id: GuildId,
        timer: IntervalTimer,
    ) -> StoreResult<IntervalTimer, Self::Error>;
    // async fn update_interval_timers(&self, guild_id: GuildId);
    async fn del_interval_timer(
        &self,
        guild_id: GuildId,
        script_id: u64,
        timer_name: String,
    ) -> StoreResult<bool, Self::Error>;
}

#[derive(Clone)]
pub struct IntervalTimer {
    pub name: String,
    pub script_id: u64,
    pub interval: IntervalType,
    pub last_run: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum IntervalType {
    Minutes(u64),
    Cron(String),
}
