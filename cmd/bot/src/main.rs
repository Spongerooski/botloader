use std::sync::Arc;

use futures::StreamExt;
use futures_core::Stream;
use stores::config::{ConfigStore, JoinedGuild};
use stores::postgres::Postgres;
use structopt::StructOpt;
use tracing::{error, info};
use twilight_cache_inmemory::{InMemoryCache, InMemoryCacheBuilder};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_model::oauth::CurrentApplicationInfo;
use vm::init_v8_flags;

mod commands;

#[derive(Clone, StructOpt)]
pub struct RunConfig {
    #[structopt(long, env = "DISCORD_BOT_TOKEN")]
    pub discord_token: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[structopt(long, env = "BOT_RPC_LISTEN_ADDR", default_value = "127.0.0.1:7448")]
    pub bot_rpc_listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().expect("failed loading dotenv files");
    tracing_subscriber::fmt::init();
    // tracing_log::LogTracer::init().unwrap();

    // helps memory usage, altough the improvements were very minor they're still improvements
    // more testing needs to be done on larger scripts
    init_v8_flags(&[
        "--optimize_for_size".to_string(),
        "--lazy_feedback_allocation".to_string(),
    ]);

    let config = RunConfig::from_args();

    let token = config.discord_token.clone();
    let database_url = config.database_url.clone();

    let http = Arc::new(
        twilight_http::client::ClientBuilder::new()
            .token(token.clone())
            .build(),
    );

    let intents = Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_VOICE_STATES;
    let (cluster, events) = Cluster::new(token, intents).await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    info!("Launching!");

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let state = Arc::new(InMemoryCacheBuilder::new().build());

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
            config,
        },
        events,
    )
    .await;

    Ok(())
}

#[derive(Clone)]
pub struct BotContext<CT> {
    config: RunConfig,
    http: Arc<twilight_http::Client>,
    cluster: Arc<Cluster>,
    state: Arc<InMemoryCache>,
    application_info: CurrentApplicationInfo,
    config_store: CT,
}

async fn handle_events<CT: Clone + ConfigStore + Send + Sync + 'static>(
    ctx: BotContext<CT>,
    mut stream: impl Stream<Item = (u64, Event)> + Unpin,
) {
    let guild_log_sub_backend =
        Arc::new(guild_logger::guild_subscriber_backend::GuildSubscriberBackend::default());
    let logger = guild_logger::GuildLoggerBuilder::new()
        .add_backend(Arc::new(guild_logger::discord_backend::DiscordLogger::new(
            ctx.http.clone(),
            ctx.config_store.clone(),
        )))
        .add_backend(guild_log_sub_backend.clone())
        .run();

    let vm_manager = vm_manager::Manager::new(
        logger.clone(),
        ctx.http.clone(),
        ctx.state.clone(),
        ctx.config_store.clone(),
    );

    let bot_rpc_server = botrpc::Server::new(
        guild_log_sub_backend,
        vm_manager.clone(),
        ctx.config.bot_rpc_listen_addr.clone(),
    );

    tokio::spawn(bot_rpc_server.run());

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
                cmd_context
                    .config_store
                    .add_update_joined_guild(JoinedGuild {
                        id: gc.id,
                        name: gc.name.clone(),
                        icon: gc.icon.clone().unwrap_or_default(),
                        owner_id: gc.owner_id,
                    })
                    .await
                    .map_err(|err| error!(%err, "failed updating joined guild"))
                    .ok();

                // Uncomment to spawn 1k vm's
                //
                // if gc.id.0 == 614909558585819162u64 {
                //     for i in 0..1000 {
                //         vm_manager
                //             .create_guild_scripts_vm_as_pack(gc.id, i as u64)
                //             .await
                //             .expect("failed creating vm");
                //     }
                // }
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
