use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use structopt::StructOpt;

#[derive(Clone, StructOpt)]
pub struct RunConfig {
    #[structopt(short, long, env = "DISCORD_BOT_TOKEN")]
    pub discord_token: String,

    #[structopt(long, env = "DISCORD_CLIENT_ID")]
    pub client_id: String,

    #[structopt(long, env = "DISCORD_CLIENT_SECRET")]
    pub client_secret: String,

    /// points to the frontend's host base, this can be seperate from the api server(webapi)
    ///
    /// example: api may run on https://api.botlabs.io and the frontend could use https://botlabs.io
    /// in this case, the frontend host base is https://botlabs.io
    #[structopt(
        long,
        env = "FRONTEND_HOST_BASE",
        default_value = "http://localhost:3000"
    )]
    pub frontend_host_base: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[structopt(long, env = "WEBAPI_LISTEN_ADDR", default_value = "127.0.0.1:7447")]
    pub listen_addr: String,

    #[structopt(
        long,
        env = "BOT_RPC_CONNECT_ADDR",
        default_value = "http://127.0.0.1:7448"
    )]
    pub bot_rpc_connect_addr: String,
}

impl RunConfig {
    pub fn get_discord_oauth2_client(&self) -> BasicClient {
        BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/confirm_login", self.frontend_host_base)).unwrap(),
        )
    }
}
