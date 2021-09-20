use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub enum DbError {
    KeyNotFound
}

pub trait Key : Serialize {

}

pub trait Value<'a> : Serialize + Deserialize<'a> {

}

impl<T> Key for T where T : Serialize {}
impl<'a, T> Value<'a> for T where T : Serialize + Deserialize<'a> {}

#[async_trait]
pub trait KVStore<'a, K: Key, V: Value<'a>> 
{
    type Error;

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> where 'a : 'async_trait;

    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> where 'a : 'async_trait;

    async fn remove(&self, key: K) -> Result<(), Self::Error> where 'a : 'async_trait;
}