use std::convert::TryFrom;

use crate::timers::{IntervalTimer, IntervalType, StoreResult, TimerStoreError};

use super::Postgres;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use twilight_model::id::GuildId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("minute and cron interval both not set")]
    NoMinutesOrCronInterval,

    #[error(transparent)]
    Sql(#[from] sqlx::Error),
}

#[async_trait]
impl crate::timers::TimerStore for Postgres {
    type Error = Error;

    async fn get_all_interval_timers(
        &self,
        guild_id: GuildId,
    ) -> StoreResult<Vec<IntervalTimer>, Self::Error> {
        let res = sqlx::query_as!(
            DbIntervalTimer,
            "SELECT guild_id, script_id, timer_name, interval_minutes, interval_cron, \
             last_run_at, created_at, updated_at
            FROM interval_timers WHERE guild_id=$1;",
            guild_id.get() as i64,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|v| TimerStoreError::Other(v.into()))?;

        Ok(res
            .into_iter()
            .filter_map(|v| IntervalTimer::try_from(v).ok())
            .collect())
    }

    async fn update_interval_timer(
        &self,
        guild_id: GuildId,
        timer: IntervalTimer,
    ) -> StoreResult<IntervalTimer, Self::Error> {
        let (interval_minutes, interval_cron) = match timer.interval {
            IntervalType::Minutes(m) => (Some(m as i32), None),
            IntervalType::Cron(c) => (None, Some(c)),
        };

        let res = sqlx::query_as!(
            DbIntervalTimer,
            "
            INSERT INTO interval_timers (guild_id, script_id, timer_name, interval_minutes, \
             interval_cron, last_run_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, now(), now())
            ON CONFLICT (guild_id, script_id, timer_name)
            DO UPDATE SET
            interval_minutes = $4,
            interval_cron = $5,
            last_run_at = $6,
            updated_at = now()
            RETURNING guild_id, script_id, timer_name, interval_minutes, interval_cron, \
             last_run_at, created_at, updated_at;
             ",
            guild_id.get() as i64,
            timer.script_id as i64,
            timer.name,
            interval_minutes,
            interval_cron,
            timer.last_run,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|v| TimerStoreError::Other(v.into()))?;

        Ok(IntervalTimer::try_from(res)?)
    }

    async fn del_interval_timer(
        &self,
        guild_id: GuildId,
        script_id: u64,
        timer_name: String,
    ) -> StoreResult<bool, Self::Error> {
        let res = sqlx::query!(
            "DELETE FROM interval_timers WHERE guild_id=$1 AND script_id=$2 AND timer_name=$3",
            guild_id.get() as i64,
            script_id as i64,
            timer_name
        )
        .execute(&self.pool)
        .await
        .map_err(|v| TimerStoreError::Other(v.into()))?;

        Ok(res.rows_affected() > 0)
    }
}

struct DbIntervalTimer {
    guild_id: i64,
    script_id: i64,
    timer_name: String,
    interval_minutes: Option<i32>,
    interval_cron: Option<String>,
    last_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DbIntervalTimer> for IntervalTimer {
    type Error = Error;

    fn try_from(value: DbIntervalTimer) -> Result<Self, Self::Error> {
        let interval_type = if let Some(mins) = value.interval_minutes {
            IntervalType::Minutes(mins as u64)
        } else if let Some(cron_text) = value.interval_cron {
            IntervalType::Cron(cron_text)
        } else {
            return Err(Error::NoMinutesOrCronInterval);
        };

        Ok(Self {
            name: value.timer_name,
            script_id: value.script_id as u64,
            last_run: value.last_run_at,
            interval: interval_type,
        })
    }
}
