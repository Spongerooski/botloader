use std::sync::Arc;

use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::{fmt::format::FmtSpan, util::SubscriberInitExt, EnvFilter};
use twilight_model::{
    oauth::CurrentApplicationInfo,
    user::{CurrentUser, User},
};

pub mod config;

use crate::config::RunConfig;

pub fn common_init() -> RunConfig {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {} // ignore io errors
        Err(e) => panic!("failed loading dotenv file: {}", e),
    }
    init_tracing();

    RunConfig::from_args()
}

fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
        .init();
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub bot_user: CurrentUser,
    pub application: CurrentApplicationInfo,
    pub owners: Vec<User>,
    pub client: Arc<twilight_http::Client>,
}

pub async fn fetch_discord_config(token: String) -> Result<DiscordConfig, twilight_http::Error> {
    let client = twilight_http::Client::new(token);

    // println!("fetching bot and application details from discord...");
    let bot_user = client.current_user().exec().await?.model().await.unwrap();
    info!("discord logged in as: {:?}", bot_user);

    let application = client
        .current_user_application()
        .exec()
        .await?
        .model()
        .await
        .unwrap();
    info!("discord application: {:?}", application.name);

    let owners = match &application.team {
        Some(t) => t.members.iter().map(|e| e.user.clone()).collect(),
        None => vec![application.owner.clone()],
    };
    info!(
        "discord application owners: {:?}",
        owners.iter().map(|o| o.id).collect::<Vec<_>>()
    );

    client.set_application_id(application.id);

    Ok(DiscordConfig {
        application,
        bot_user,
        owners,
        client: Arc::new(client),
    })
}
