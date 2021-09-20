#[cfg(test)]
mod db;

mod kv;
pub mod traits;
pub use kv::*;

#[cfg(test)]
mod tests {
    use super::{*, traits::KVStore};
    use sqlx::postgres::*;
    use twilight_model::id::GuildId;

    static DATABASE_ENV_KEY: &str = "DATABASE_URL";

    #[tokio::test]
    async fn kv_get_set_remove()
    {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var(DATABASE_ENV_KEY).unwrap()).await.unwrap();

        let store = db::TestDb::from(&pool);

        let key = Key { guild: GuildId(0), namespace: GuildNamespace::GuildScript, key: "key".into() };

        assert!(store.get(key).await.unwrap().is_some());
    }
}