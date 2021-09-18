use std::path::PathBuf;
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
}
