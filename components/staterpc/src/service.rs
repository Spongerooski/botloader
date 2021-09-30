use futures::{
    future::{self, Ready},
    prelude::*,
};
use tarpc::{
    client, context,
    server::{self, incoming::Incoming},
};
use twilight_model::{guild::Member, id::UserId};

pub enum ErrorResponse {}

#[tarpc::service]
pub trait StateRpcService {
    async fn hello(name: String) -> String;

    async fn get_member(user_id: UserId) -> Option<Member>;
    async fn get_guild() -> Option<()>;
}
