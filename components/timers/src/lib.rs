use std::{collections::HashMap, ops::Add, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use runtime_models::util::NotBigU64;
use stores::timers::{IntervalTimer, IntervalType};
use tokio::sync::mpsc;
use twilight_model::id::GuildId;
use vm::vm::VmCommand;

#[derive(Debug)]
pub enum Error {
    StorageError(anyhow::Error),
    CronParseError(cron::error::Error),
    NoNextTime,
}

pub struct SyncGuildCommand {
    pub guild_id: GuildId,
    pub timers: Vec<ScriptTimer>,
    pub dispath_tx: mpsc::UnboundedSender<VmCommand>,
}

pub struct ScriptTimer {
    pub timer: stores::config::IntervalTimerContrib,
    pub script_id: u64,
}

pub enum Command {
    SyncGuild(SyncGuildCommand),
}

pub struct Scheduler<T> {
    storage: T,
    guilds: HashMap<GuildId, GuildState>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

struct GuildState {
    guild_id: GuildId,
    dispath_tx: mpsc::UnboundedSender<VmCommand>,
    loaded_intervals: Vec<WrappedIntervalTimer>,
}

// TODO: somehow clear intervals between resets
// TODO: recover intervals on start
impl<T: stores::timers::TimerStore + Send + Sync + 'static> Scheduler<T> {
    pub fn create(storage: T) -> mpsc::UnboundedSender<Command> {
        let (tx, rx) = mpsc::unbounded_channel();

        let scheduler = Self {
            storage,
            guilds: HashMap::new(),
            cmd_rx: rx,
        };

        tokio::spawn(async move { scheduler.run().await });

        tx
    }

    pub async fn run(mut self) {
        loop {
            if let Some(next) = self.next_event_time() {
                let to_sleep = next - chrono::Utc::now();
                if to_sleep > Duration::seconds(0) {
                    tokio::select! {
                        _ = tokio::time::sleep(to_sleep.to_std().unwrap()) => {
                            self.check_run_next_timer().await;
                        },
                        evt = self.cmd_rx.recv() => {
                            if let Some(evt) = evt{
                                self.handle_command(evt).await;
                            }else{
                                return;
                            }
                        },
                    }
                } else {
                    self.check_run_next_timer().await;
                }
            } else {
                let cmd = self.cmd_rx.recv().await;
                if let Some(cmd) = cmd {
                    self.handle_command(cmd).await;
                } else {
                    // commands sender end dropped, probably shutting down
                    return;
                }
            }
        }
    }

    async fn check_run_next_timer(&mut self) {
        let now = chrono::Utc::now();
        let mut triggered_timers = Vec::new();
        for (g, gs) in &self.guilds {
            let triggered_guild = gs.loaded_intervals.iter().filter(|v| v.is_triggered(now));
            for triggered in triggered_guild {
                triggered_timers.push((*g, triggered.clone()));
            }
        }

        for triggered in triggered_timers {
            self.trigger_timer(now, triggered.0, triggered.1).await;
        }
    }

    async fn trigger_timer(
        &mut self,
        t: DateTime<Utc>,
        guild_id: GuildId,
        timer: WrappedIntervalTimer,
    ) {
        let evt = runtime_models::events::timers::IntervalTimerEvent {
            name: timer.inner.name.clone(),
            script_id: NotBigU64(timer.inner.script_id),
        };

        let serialized = serde_json::to_value(&evt).unwrap();

        let delete = if let Some(g) = self.guilds.get(&guild_id) {
            match g.dispath_tx.send(VmCommand::DispatchEvent(
                "BOTLOADER_INTERVAL_TIMER_FIRED",
                serialized,
            )) {
                Ok(()) => {
                    self.update_next_run(
                        t,
                        guild_id,
                        timer.inner.script_id,
                        timer.inner.name.clone(),
                    )
                    .await;
                    false
                }
                // guild vm no longer active, stop tracking it
                Err(_) => true,
            }
        } else {
            false
        };

        if delete {
            self.guilds.remove(&guild_id);
        }
    }

    async fn update_next_run(
        &mut self,
        t: DateTime<Utc>,
        guild_id: GuildId,
        script_id: u64,
        name: String,
    ) {
        let gs = if let Some(gs) = self.guilds.get_mut(&guild_id) {
            gs
        } else {
            return;
        };

        let timer = if let Some(timer) = gs
            .loaded_intervals
            .iter_mut()
            .find(|v| v.inner.script_id == script_id && v.inner.name == name)
        {
            timer
        } else {
            return;
        };

        timer.inner.last_run = t;
        if let Some(next) = timer.parsed_type.next_run_time(t) {
            timer.next_run = next;
        } else {
            timer.next_run = t.add(Duration::hours(1000));
        }

        // update last run
        if let Err(err) = self
            .storage
            .update_interval_timer(guild_id, timer.inner.clone())
            .await
        {
            tracing::error!(%err, "failed updating timer")
        };
    }

    async fn handle_command(&mut self, cmd: Command) {
        let res = match cmd {
            Command::SyncGuild(g) => self.sync_guild(g).await,
        };

        if let Err(err) = res {
            tracing::error!(%err, "failed syncing guild");
        }
    }

    async fn sync_guild(&mut self, g: SyncGuildCommand) -> Result<(), anyhow::Error> {
        let all_guild_timers = self.storage.get_all_interval_timers(g.guild_id).await?;

        let to_del = all_guild_timers
            .iter()
            .filter(|iv| {
                !g.timers
                    .iter()
                    .any(|v| iv.script_id == v.script_id && iv.name == v.timer.name)
            })
            .collect::<Vec<_>>();

        // delete old timers
        for del in to_del {
            self.storage
                .del_interval_timer(g.guild_id, del.script_id, del.name.clone())
                .await?;
        }

        let mut new_timers = Vec::new();
        for updt in g.timers {
            tracing::info!("Timer: {}", updt.timer.name);
            let last_run = all_guild_timers
                .iter()
                .find(|v| v.script_id == updt.script_id && v.name == updt.timer.name)
                .map(|v| v.last_run)
                .unwrap_or_else(chrono::Utc::now);

            let timer = self
                .storage
                .update_interval_timer(
                    g.guild_id,
                    IntervalTimer {
                        last_run,
                        interval: updt.timer.interval,
                        name: updt.timer.name,
                        script_id: updt.script_id,
                    },
                )
                .await?;

            match wrap_timer(timer) {
                Ok(wrapped) => new_timers.push(wrapped),
                Err(err) => tracing::error!(?err, "failed wrapping timer"),
            };
        }

        self.guilds.insert(
            g.guild_id,
            GuildState {
                dispath_tx: g.dispath_tx,
                guild_id: g.guild_id,
                loaded_intervals: new_timers,
            },
        );

        Ok(())
    }

    pub fn next_event_time(&self) -> Option<DateTime<Utc>> {
        let lowest_interval = self
            .guilds
            .iter()
            .map(|(_, v)| &v.loaded_intervals)
            .flatten()
            .min_by(|a, b| a.next_run.cmp(&b.next_run));

        lowest_interval.map(|v| v.next_run)
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
pub struct WrappedIntervalTimer {
    inner: IntervalTimer,
    parsed_type: ParsedIntervalType,
    next_run: chrono::DateTime<chrono::Utc>,
}

fn wrap_timer(timer: IntervalTimer) -> Result<WrappedIntervalTimer, Error> {
    let interval_type = match &timer.interval {
        IntervalType::Minutes(mins) => ParsedIntervalType::Minutes(*mins),
        IntervalType::Cron(c) => {
            let parsed = cron::Schedule::from_str(format!("0 {}", c).as_str())
                .map_err(Error::CronParseError)?;
            ParsedIntervalType::Cron(c.clone(), Box::new(parsed))
        }
    };

    let next = if let Some(next) = interval_type.next_run_time(timer.last_run) {
        next
    } else {
        return Err(Error::NoNextTime);
    };

    Ok(WrappedIntervalTimer {
        inner: timer,
        next_run: next,
        parsed_type: interval_type,
    })
}

impl WrappedIntervalTimer {
    fn is_triggered(&self, t: DateTime<Utc>) -> bool {
        t > self.next_run
    }
}

#[derive(Clone, PartialEq)]
pub enum ParsedIntervalType {
    Minutes(u64),
    Cron(String, Box<cron::Schedule>),
}

impl ParsedIntervalType {
    fn next_run_time(&self, t: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Cron(_, c) => c.after(&t).next(),

            // TODO: proper stepping, calculate using old run time
            Self::Minutes(minutes) => Some(t.add(chrono::Duration::minutes(*minutes as i64))),
        }
    }
}
