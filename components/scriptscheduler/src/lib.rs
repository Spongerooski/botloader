use std::{cmp::Ordering, ops::Add, str::FromStr};

use chrono::{DateTime, Utc};
use twilight_model::id::GuildId;

pub mod inmemstorage;

#[derive(Debug)]
pub enum Error {
    StorageError(anyhow::Error),
    CronParseError(cron::error::Error),
}

pub enum TimerFired {
    Task(ScheduledTask),
    Interval(IntervalTimer),
}

type SchedulerResult<T> = Result<T, Error>;

pub struct ScriptScheduler<T> {
    storage: T,
    guild_id: GuildId,

    loaded_intervals: Vec<IntervalTimer>,
    id_gen_counter: u64,
}

pub enum TaskId {
    Provided(String),
    Generate(DateTime<Utc>),
}

// TODO: somehow clear intervals between resets
// TODO: recover intervals on start
impl<T: Storage> ScriptScheduler<T> {
    pub fn new(guild_id: GuildId, storage: T) -> Self {
        Self {
            storage,
            guild_id,
            loaded_intervals: Vec::new(),
            id_gen_counter: 0,
        }
    }

    pub async fn set_interval(
        &mut self,
        name: String,
        persistent: bool,
        interval: RawIntervalType,
        t: DateTime<Utc>,
    ) -> SchedulerResult<()> {
        let parsed = Self::parse_interval(interval)?;

        if let Some(next) = parsed.next_run_time(t) {
            let timer = IntervalTimer {
                interval: parsed,
                name: name.clone(),
                next_run_time: next,
                persistent,
            };

            let mut updated = false;

            if let Some(existing) = self.loaded_intervals.iter_mut().find(|e| e.name == name) {
                if existing.interval != timer.interval {
                    // only update if the interval has changed
                    *existing = timer;
                    updated = true;
                }
            } else {
                self.loaded_intervals.push(timer);
                updated = true;
            }

            if persistent && updated {
                self.storage
                    .set_next_interval_exec(self.guild_id, name.clone(), next)
                    .await?;
            }
        }

        Ok(())
    }

    pub fn parse_interval(interval: RawIntervalType) -> SchedulerResult<IntervalType> {
        match interval {
            RawIntervalType::Seconds(n) => Ok(IntervalType::Seconds(n)),
            RawIntervalType::Cron(s) => Ok(IntervalType::Cron(
                s.clone(),
                Box::new(cron::Schedule::from_str(&s).map_err(Error::CronParseError)?),
            )),
        }
    }

    pub async fn del_interval(&mut self, name: String) -> SchedulerResult<()> {
        if let Some((index, _)) = self
            .loaded_intervals
            .iter()
            .enumerate()
            .find(|(_, v)| v.name == name)
        {
            self.loaded_intervals.remove(index);
        }

        self.storage.del_interval(self.guild_id, name).await?;

        Ok(())
    }

    pub async fn schedule_task(
        &mut self,
        name: String,
        id: TaskId,
        t: DateTime<Utc>,
        data: serde_json::Value,
    ) -> SchedulerResult<String> {
        let task = ScheduledTask {
            data,
            name,
            exec_at: t,
            guild_id: self.guild_id,
            id: match id {
                TaskId::Provided(id) => id,
                TaskId::Generate(now) => self.gen_id(now),
            },
        };

        self.storage.set_task(&task).await?;

        Ok(task.id.clone())
    }

    pub async fn del_task(&mut self, name: String, id: String) -> SchedulerResult<()> {
        self.storage.del_task(self.guild_id, name, id).await
    }

    pub async fn next_event_time(&mut self) -> SchedulerResult<Option<DateTime<Utc>>> {
        let lowest_interval = self
            .loaded_intervals
            .iter()
            .min_by(|a, b| a.next_run_time.cmp(&b.next_run_time));

        let mut next_run_time = None;

        if let Some(l) = lowest_interval {
            next_run_time = Some(l.next_run_time);
        }

        if let Some(next_task) = self.storage.get_next_task_run_time(self.guild_id).await? {
            if let Some(previous) = next_run_time {
                if let Ordering::Less = next_task.cmp(&previous) {
                    // task is sooner than interval
                    next_run_time = Some(next_task);
                }
            } else {
                // interval did not have a next run time
                next_run_time = Some(next_task);
            }
        }

        Ok(next_run_time)
    }

    pub async fn triggered_events(&mut self, t: DateTime<Utc>) -> SchedulerResult<Vec<TimerFired>> {
        let mut results = Vec::new();

        let triggered_tasks = self.storage.get_triggered_tasks(self.guild_id, t).await?;
        for t in triggered_tasks {
            self.storage
                .del_task(self.guild_id, t.name.clone(), t.id.clone())
                .await?;

            results.push(TimerFired::Task(t));
        }

        let triggered_intervals = self
            .loaded_intervals
            .iter_mut()
            .filter(|a| a.next_run_time.cmp(&t) == Ordering::Less)
            .collect::<Vec<_>>();

        for int in triggered_intervals {
            int.next_run_time = int.interval.next_run_time(t).unwrap_or(t);
            results.push(TimerFired::Interval(int.clone()));
        }

        Ok(results)
    }

    fn gen_id(&mut self, t: DateTime<Utc>) -> String {
        let ts = t.timestamp_nanos();
        let c = self.id_gen_counter;
        self.id_gen_counter += 1;
        format!("{}-{}", ts, c)
    }
}

#[derive(Clone)]
pub struct ScheduledTask {
    pub guild_id: GuildId,
    pub name: String,
    pub id: String,
    pub data: serde_json::Value,
    pub exec_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct IntervalTimer {
    pub name: String,
    pub persistent: bool,
    pub interval: IntervalType,
    pub next_run_time: DateTime<Utc>,
}

#[derive(Clone, PartialEq)]
pub enum IntervalType {
    Seconds(u64),
    Cron(String, Box<cron::Schedule>),
}

impl IntervalType {
    fn next_run_time(&self, t: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Cron(_, c) => c.after(&t).next(),

            // TODO: proper stepping, calculate using old run time
            Self::Seconds(secs) => Some(t.add(chrono::Duration::seconds(*secs as i64))),
        }
    }
}

pub enum RawIntervalType {
    Seconds(u64),
    Cron(String),
}

#[async_trait::async_trait]
pub trait Storage {
    async fn set_task(&self, t: &ScheduledTask) -> SchedulerResult<()>;
    async fn del_task(&self, guild_id: GuildId, name: String, id: String) -> SchedulerResult<()>;

    async fn get_next_task_run_time(
        &self,
        guild_id: GuildId,
    ) -> SchedulerResult<Option<DateTime<Utc>>>;

    async fn get_triggered_tasks(
        &self,
        guild_id: GuildId,
        t: DateTime<Utc>,
    ) -> SchedulerResult<Vec<ScheduledTask>>;

    async fn set_next_interval_exec(
        &self,
        guild_id: GuildId,
        name: String,
        t: DateTime<Utc>,
    ) -> SchedulerResult<()>;
    async fn get_all_intervals_next_exec(
        &self,
        guild_id: GuildId,
    ) -> SchedulerResult<Vec<(String, DateTime<Utc>)>>;

    async fn del_interval(&self, guild_id: GuildId, name: String) -> SchedulerResult<()>;
}
