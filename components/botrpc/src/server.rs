use std::{pin::Pin, task::Poll};

use futures::Stream;
use stores::config::ConfigStore;
use tonic::{Response, Status};
use twilight_model::id::GuildId;

use crate::proto;

pub struct Server<CT> {
    vm_manager: vm_manager::Manager<CT>,
}

type ResponseStream =
    Pin<Box<dyn Stream<Item = Result<proto::ScriptLogItem, Status>> + Send + Sync>>;

#[tonic::async_trait]
impl<CT: ConfigStore + Send + Sync + 'static> proto::bot_service_server::BotService for Server<CT> {
    async fn reload_vm(
        &self,
        request: tonic::Request<proto::GuildScriptSpecifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        let guild_id = GuildId(request.into_inner().guild_id);

        match self.vm_manager.restart_guild_vm(guild_id).await {
            Ok(()) => Ok(Response::new(proto::Empty {})),
            Err(err) => Err(Status::internal(err)),
        }
    }

    type StreamVmLogsStream = ResponseStream;

    async fn stream_vm_logs(
        &self,
        request: tonic::Request<proto::GuildScriptSpecifier>,
    ) -> Result<Response<Self::StreamVmLogsStream>, Status> {
        let guild_id = GuildId(request.into_inner().guild_id);

        if let Some(mut rx) = self.vm_manager.subscribe_to_guild_logs(guild_id).await {
            Ok(Response::new(Box::pin(futures::stream::poll_fn(
                move |ctx| match rx.poll_recv(ctx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(proto::ScriptLogItem {
                        filename: "todo".to_string(),
                        linenumber: 0,
                        column: 0,
                        kind: "".to_string(),
                        message: v,
                    }))),
                    Poll::Ready(None) => Poll::Ready(None),
                },
            ))))
        } else {
            Err(Status::not_found("unknown guild"))
        }
    }
}
