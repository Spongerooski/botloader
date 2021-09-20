use super::{traits, Key, Value};
use async_trait::async_trait;
use sqlx::{Executor, Postgres};

pub struct TestDb<Db>(Db);

impl<Db> TestDb<Db> {
    pub fn from(db: Db) -> Self {
        Self(db)
    }
}

#[async_trait]
impl<'a, Db> traits::KVStore<'a, Key, Value> for TestDb<Db>
where
    Db: Executor<'a, Database = Postgres> + Sync + Send + Copy,
{
    type Error = sqlx::Error;

    async fn get(&self, key: Key) -> Result<Option<Value>, Self::Error>
    where
        'a: 'async_trait,
    {
        Ok(sqlx::query_scalar!(
            r#"
                select store.val from kvstore_values store 
                where store.guild_id = $1
                and store.pack_id = $2
                and store.key_text = $3"#,
            key.guild.0 as i64,
            Into::<i64>::into(key.namespace),
            key.key
        )
        .fetch_optional(self.0)
        .await?
        .and_then(|row| Some(Value(row))))
    }

    async fn set(&self, key: Key, value: Value) -> Result<(), Self::Error>
    where
        'a: 'async_trait,
    {
        panic!()
    }

    async fn remove(&self, key: Key) -> Result<(), Self::Error>
    where
        'a: 'async_trait,
    {
        panic!()
    }
}
