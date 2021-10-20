use std::sync::Arc;

use futures::StreamExt;
use futures_core::Stream;
use runtime::error_reporter::DiscordErrorReporter;
use stores::config::ConfigStore;
use stores::postgres::Postgres;
use structopt::StructOpt;
use tracing::info;
use twilight_cache_inmemory::{InMemoryCache, InMemoryCacheBuilder};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_model::oauth::CurrentApplicationInfo;

mod commands;

#[derive(Clone, StructOpt)]
pub struct RunConfig {
    #[structopt(long, env = "DISCORD_TOKEN")]
    pub discord_token: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().expect("failed loading dotenv files");
    tracing_subscriber::fmt::init();
    // tracing_log::LogTracer::init().unwrap();

    let config = RunConfig::from_args();

    let token = config.discord_token;
    let database_url = config.database_url;

    let http = twilight_http::client::ClientBuilder::new()
        .token(token.clone())
        .build();

    let intents = Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_VOICE_STATES;
    let (cluster, events) = Cluster::new(token, intents).await?;

    let cluster_spawn = cluster.clone();

    info!("Launching!");

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let state = InMemoryCacheBuilder::new().build();

    let config_store = Postgres::new_with_url(&database_url).await?;

    let application_info = http
        .current_user_application()
        .exec()
        .await?
        .model()
        .await?;

    handle_events(
        BotContext {
            http,
            cluster,
            state,
            application_info,
            config_store,
        },
        events,
    )
    .await;

    Ok(())
}

#[derive(Clone)]
pub struct BotContext<CT> {
    http: twilight_http::Client,
    cluster: Cluster,
    state: InMemoryCache,
    application_info: CurrentApplicationInfo,
    config_store: CT,
}

async fn handle_events<CT: Clone + ConfigStore + Send + Sync + 'static>(
    ctx: BotContext<CT>,
    mut stream: impl Stream<Item = (u64, Event)> + Unpin,
) {
    let script_err_reporter = DiscordErrorReporter::new(ctx.config_store.clone(), ctx.http.clone());

    let vm_manager = vm_manager::Manager::new(
        Arc::new(script_err_reporter),
        ctx.http.clone(),
        ctx.state.clone(),
        ctx.config_store.clone(),
    );

    let cmd_context = commands::CommandContext {
        http: ctx.http.clone(),
        cluster: ctx.cluster.clone(),
        state: ctx.state.clone(),
        config_store: ctx.config_store.clone(),
        vm_manager: vm_manager.clone(),
    };

    while let Some((_, event)) = stream.next().await {
        ctx.state.update(&event);

        match &event {
            Event::Ready(r) => {
                let shard = r.shard.unwrap_or([0, 1]);
                info!(
                    shard_id = shard[0],
                    total_shard = shard[1],
                    session_id = r.session_id.as_str(),
                    "Got ready!"
                )
            }
            Event::GuildCreate(gc) => {
                vm_manager.init_guild(gc.id).await.unwrap();
            }
            Event::MessageCreate(m) => {
                if let Some(cmd) = commands::check_for_command(&ctx, *(m).clone()) {
                    commands::handle_command(cmd_context.clone(), cmd).await
                }
            }
            _ => {}
        }

        vm_manager.handle_discord_event(event).await;

        // println!("Event: {:?}", event);
    }
}
