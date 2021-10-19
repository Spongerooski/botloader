use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use structopt::StructOpt;

#[derive(Clone, StructOpt)]
pub struct RunConfig {
    #[structopt(short, long, env = "DISCORD_TOKEN")]
    pub discord_token: String,

    #[structopt(long, env = "CLIENT_ID")]
    pub client_id: String,

    #[structopt(long, env = "CLIENT_SECRET")]
    pub client_secret: String,

    #[structopt(long, env = "HOST_BASE", default_value = "localhost:3000")]
    pub host_base: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,
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
            RedirectUrl::new(format!("http://{}/confirm_login", self.host_base)).unwrap(),
        )
    }
}
