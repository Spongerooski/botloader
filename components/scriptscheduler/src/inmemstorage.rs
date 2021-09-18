use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use twilight_model::id::GuildId;

use crate::{ScheduledTask, SchedulerResult};

/// basic in memory stoage
pub struct InMemoryStorage {
    inner: Arc<RwLock<InMemoryStorageInner>>,
}

pub struct InMemoryStorageInner {
    intervals: Vec<(GuildId, String, DateTime<Utc>)>,
    tasks: Vec<ScheduledTask>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InMemoryStorageInner {
                intervals: Vec::new(),
                tasks: Vec::new(),
            })),
        }
    }
}

impl InMemoryStorageInner {
    fn task_index(&self, guild_id: GuildId, name: String, id: String) -> Option<usize> {
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, v)| v.guild_id == guild_id && v.name == name && v.id == id)
            .map(|(i, _)| i)
    }
}

#[async_trait::async_trait]
impl crate::Storage for InMemoryStorage {
    async fn set_task(&self, t: &ScheduledTask) -> SchedulerResult<()> {
        let mut inner = self.inner.write().unwrap();

        match inner
            .tasks
            .iter_mut()
            .find(|e| e.guild_id == t.guild_id && e.name == t.name && e.id == t.id)
        {
            Some(existing) => {
                *existing = t.clone();
            }
            None => {
                inner.tasks.push(t.clone());
            }
        };

        Ok(())
    }

    async fn del_task(&self, guild_id: GuildId, name: String, id: String) -> SchedulerResult<()> {
        let mut inner = self.inner.write().unwrap();

        if let Some(index) = inner.task_index(guild_id, name, id) {
            inner.tasks.remove(index);
        }

        Ok(())
    }

    async fn get_next_task_run_time(
        &self,
        guild_id: GuildId,
    ) -> SchedulerResult<Option<DateTime<Utc>>> {
        let inner = self.inner.read().unwrap();

        let mut lowest = None;

        for task in inner.tasks.iter().filter(|e| e.guild_id == guild_id) {
            match lowest {
                None => {
                    lowest = Some(task.exec_at);
                }
                Some(lt) => {
                    if lt > task.exec_at {
                        lowest = Some(task.exec_at);
                    }
                }
            }
        }

        Ok(lowest)
    }

    async fn get_triggered_tasks(
        &self,
        guild_id: GuildId,
        t: DateTime<Utc>,
    ) -> SchedulerResult<Vec<ScheduledTask>> {
        let inner = self.inner.read().unwrap();

        Ok(inner
            .tasks
            .iter()
            .filter(|e| e.guild_id == guild_id && t > e.exec_at)
            .cloned()
            .collect())
    }

    async fn set_next_interval_exec(
        &self,
        guild_id: GuildId,
        name: String,
        t: DateTime<Utc>,
    ) -> SchedulerResult<()> {
        let mut inner = self.inner.write().unwrap();

        match inner
            .intervals
            .iter_mut()
            .find(|(g, _name, _)| *g == guild_id && name == *_name)
        {
            None => {
                inner.intervals.push((guild_id, name, t));
            }
            Some((_, _, old_t)) => *old_t = t,
        };

        Ok(())
    }

    async fn get_all_intervals_next_exec(
        &self,
        guild_id: GuildId,
    ) -> SchedulerResult<Vec<(String, DateTime<Utc>)>> {
        let inner = self.inner.read().unwrap();

        Ok(inner
            .intervals
            .iter()
            .filter(|(g, _, _)| *g == guild_id)
            .map(|(_, name, t)| (name.clone(), *t))
            .collect())
    }

    async fn del_interval(&self, guild_id: GuildId, name: String) -> SchedulerResult<()> {
        let mut inner = self.inner.write().unwrap();

        if let Some((index, _)) = inner
            .intervals
            .iter()
            .enumerate()
            .find(|(_, v)| v.0 == guild_id && v.1 == name)
        {
            inner.intervals.remove(index);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryStorage;
    use crate::ScheduledTask;
    use crate::Storage;
    use chrono::prelude::*;
    use twilight_model::id::GuildId;

    const TEST_GUILD: GuildId = GuildId(1);

    #[tokio::test]
    async fn test_task_sngle() {
        // add task
        let storage = InMemoryStorage::new();

        let exec_at = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 0, 0));
        let exec_at_after =
            Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 1, 0));

        let task = ScheduledTask {
            data: serde_json::Value::Null,
            exec_at,
            guild_id: TEST_GUILD,
            id: "a".to_string(),
            name: "test".to_string(),
        };

        storage.set_task(&task).await.unwrap();

        // get next task
        let next_time = storage
            .get_next_task_run_time(TEST_GUILD)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(next_time, task.exec_at);

        // triggered tasks
        let triggered = storage
            .get_triggered_tasks(TEST_GUILD, exec_at_after)
            .await
            .unwrap();

        assert_eq!(triggered.len(), 1);

        // del the task
        storage
            .del_task(TEST_GUILD, "test".to_string(), "a".to_string())
            .await
            .unwrap();

        // ensure it was deleted
        let triggered_new = storage
            .get_triggered_tasks(TEST_GUILD, exec_at_after)
            .await
            .unwrap();

        assert_eq!(triggered_new.len(), 0);
    }

    #[tokio::test]
    async fn test_task_multi() {
        // add task
        let storage = InMemoryStorage::new();

        let exec_at_1 = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 0, 0));
        let exec_at_2 = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 10, 0));
        let exec_at_after_1 =
            Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 1, 0));
        let exec_at_after_2 =
            Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 11, 0));

        let task_1 = ScheduledTask {
            data: serde_json::Value::Null,
            exec_at: exec_at_1,
            guild_id: TEST_GUILD,
            id: "a".to_string(),
            name: "test".to_string(),
        };

        let task_2 = ScheduledTask {
            data: serde_json::Value::Null,
            exec_at: exec_at_2,
            guild_id: TEST_GUILD,
            id: "b".to_string(),
            name: "test".to_string(),
        };

        storage.set_task(&task_1).await.unwrap();
        storage.set_task(&task_2).await.unwrap();

        // get next task
        let next_time = storage
            .get_next_task_run_time(TEST_GUILD)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(next_time, task_1.exec_at);

        // triggered tasks
        {
            let triggered = storage
                .get_triggered_tasks(TEST_GUILD, exec_at_after_1)
                .await
                .unwrap();

            assert_eq!(triggered.len(), 1);
        }

        {
            let triggered = storage
                .get_triggered_tasks(TEST_GUILD, exec_at_after_2)
                .await
                .unwrap();

            assert_eq!(triggered.len(), 2);
        }

        // del the task
        storage
            .del_task(TEST_GUILD, "test".to_string(), "a".to_string())
            .await
            .unwrap();

        // ensure it was deleted
        {
            let triggered_new = storage
                .get_triggered_tasks(TEST_GUILD, exec_at_after_2)
                .await
                .unwrap();

            assert_eq!(triggered_new.len(), 1);

            // get next task
            let next_time_new = storage
                .get_next_task_run_time(TEST_GUILD)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(next_time_new, task_2.exec_at);
        }
    }

    #[tokio::test]
    async fn test_task_update() {
        // add task
        let storage = InMemoryStorage::new();

        let exec_at_1 = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(16, 0, 0));
        let exec_at_2 = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 10, 0));
        let exec_at_after_1 =
            Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(15, 11, 0));

        let mut task = ScheduledTask {
            data: serde_json::Value::Null,
            exec_at: exec_at_1,
            guild_id: TEST_GUILD,
            id: "a".to_string(),
            name: "test".to_string(),
        };

        storage.set_task(&task).await.unwrap();

        // get next task
        {
            let next_time = storage
                .get_next_task_run_time(TEST_GUILD)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(next_time, task.exec_at);
        }

        // triggered tasks
        {
            let triggered = storage
                .get_triggered_tasks(TEST_GUILD, exec_at_after_1)
                .await
                .unwrap();

            assert_eq!(triggered.len(), 0);
        }

        // update the task
        task.exec_at = exec_at_2;
        storage.set_task(&task).await.unwrap();

        // verify that it updated
        {
            let next_time = storage
                .get_next_task_run_time(TEST_GUILD)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(next_time, task.exec_at);
        }

        // triggered tasks
        {
            let triggered = storage
                .get_triggered_tasks(TEST_GUILD, exec_at_after_1)
                .await
                .unwrap();

            assert_eq!(triggered.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_intervals() {
        // add task
        let storage = InMemoryStorage::new();

        let exec_at_1 = Utc.from_utc_datetime(&NaiveDate::from_ymd(2020, 6, 1).and_hms(16, 0, 0));

        storage
            .set_next_interval_exec(TEST_GUILD, "test".to_string(), exec_at_1)
            .await
            .unwrap();

        {
            let inervals = storage
                .get_all_intervals_next_exec(TEST_GUILD)
                .await
                .unwrap();

            assert_eq!(inervals.len(), 1);
            assert_eq!(inervals[0], ("test".to_string(), exec_at_1));
        }

        // fake del them
        storage
            .del_interval(TEST_GUILD, "none".to_string())
            .await
            .unwrap();

        {
            let inervals = storage
                .get_all_intervals_next_exec(TEST_GUILD)
                .await
                .unwrap();

            assert_eq!(inervals.len(), 1);
            assert_eq!(inervals[0], ("test".to_string(), exec_at_1));
        }

        storage
            .del_interval(TEST_GUILD, "test".to_string())
            .await
            .unwrap();

        {
            let inervals = storage
                .get_all_intervals_next_exec(TEST_GUILD)
                .await
                .unwrap();

            assert_eq!(inervals.len(), 0);
        }
    }
}
