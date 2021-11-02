use tonic::Streaming;
use twilight_model::id::GuildId;

use crate::proto;

type ClientConn = proto::bot_service_client::BotServiceClient<tonic::transport::Channel>;

#[derive(Clone)]
pub struct Client {
    inner: ClientConn,
}

impl Client {
    pub async fn new(addr: String) -> Result<Client, tonic::transport::Error> {
        let client = proto::bot_service_client::BotServiceClient::connect(addr).await?;

        Ok(Client { inner: client })
    }

    pub fn get_conn(&self) -> ClientConn {
        self.inner.clone()
    }

    pub async fn restart_guild_vm(&self, guild_id: GuildId) -> Result<(), tonic::Status> {
        let mut conn = self.get_conn();

        conn.reload_vm(proto::GuildScriptSpecifier {
            guild_id: guild_id.get(),
            script: None,
        })
        .await?;

        Ok(())
    }

    pub async fn guild_log_stream(
        &self,
        guild_id: GuildId,
    ) -> Result<Streaming<proto::ScriptLogItem>, tonic::Status> {
        let mut conn = self.get_conn();

        let stream = conn
            .stream_vm_logs(proto::GuildScriptSpecifier {
                guild_id: guild_id.get(),
                script: None,
            })
            .await?
            .into_inner();

        Ok(stream)
    }
}
