use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures_core::Stream;
use stores::config::{ConfigStore, JoinedGuild};
use stores::postgres::Postgres;
use stores::timers::TimerStore;
use tracing::{error, info};
use twilight_cache_inmemory::InMemoryCacheBuilder;
use twilight_gateway::{Cluster, Event, Intents};
use twilight_model::application::callback::{CallbackData, InteractionResponse};
use vm::init_v8_flags;

mod commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = common::common_init();

    let discord_config = common::fetch_discord_config(config.discord_token.clone())
        .await
        .expect("failed fetching discord config");

    // helps memory usage, altough the improvements were very minor they're still improvements
    // more testing needs to be done on larger scripts
    init_v8_flags(&[
        "--optimize_for_size".to_string(),
        "--lazy_feedback_allocation".to_string(),
    ]);

    let intents = Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_VOICE_STATES;
    let (cluster, events) = Cluster::new(config.discord_token.clone(), intents).await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    info!("Launching!");

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let state = Arc::new(InMemoryCacheBuilder::new().build());

    let config_store = Postgres::new_with_url(&config.database_url).await?;

    let guild_log_sub_backend =
        Arc::new(guild_logger::guild_subscriber_backend::GuildSubscriberBackend::default());
    let logger = guild_logger::GuildLoggerBuilder::new()
        .add_backend(Arc::new(guild_logger::discord_backend::DiscordLogger::new(
            discord_config.client.clone(),
            config_store.clone(),
        )))
        .add_backend(guild_log_sub_backend.clone())
        .run();

    let vm_manager = vm_manager::Manager::new(
        logger.clone(),
        discord_config.client.clone(),
        state.clone(),
        config_store.clone(),
    );

    let bot_rpc_server = botrpc::Server::new(
        guild_log_sub_backend,
        vm_manager.clone(),
        config.bot_rpc_listen_addr.clone(),
    );

    tokio::spawn(handle_events(
        commands::CommandContext {
            http: discord_config.client.clone(),
            cluster: cluster.clone(),
            state,
            config_store,
            vm_manager: vm_manager.clone(),
        },
        events,
    ));

    tokio::spawn(bot_rpc_server.run());

    common::shutdown::wait_shutdown_signal().await;

    info!("cluster going down...");
    cluster.down();

    info!("shutting down vm's");
    vm_manager.shutdown().await;

    loop {
        let remaining = vm_manager.guilds_running().await;
        if remaining == 0 {
            break;
        }

        info!("waiting on {} guilds to shut down...", remaining);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn handle_events<CT: Clone + ConfigStore + TimerStore + Send + Sync + 'static>(
    ctx: commands::CommandContext<CT>,
    mut stream: impl Stream<Item = (u64, Event)> + Unpin,
) {
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
                match ctx.config_store.is_guild_whitelisted(gc.id).await {
                    Ok(false) => {
                        info!("leaving non whitelisted guild: {}", gc.id);
                        ctx.http.leave_guild(gc.id).exec().await.ok();
                        continue;
                    }
                    Err(err) => {
                        error!(%err, "failed checking whitelist");
                    }
                    _ => {}
                };

                ctx.vm_manager.init_guild(gc.id).await.unwrap();
                ctx.config_store
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
            Event::GuildDelete(gd) => {
                ctx.config_store.remove_joined_guild(gd.id).await.ok();
                ctx.vm_manager.remove_guild(gd.id).await;
            }
            Event::MessageCreate(m) => {
                if let Some(cmd) = commands::check_for_command(&ctx, *(m).clone()) {
                    commands::handle_command(ctx.clone(), cmd).await
                }
            }
            Event::InteractionCreate(evt) => match &evt.0 {
                twilight_model::application::interaction::Interaction::Ping(_) => {}
                twilight_model::application::interaction::Interaction::MessageComponent(_) => {}
                twilight_model::application::interaction::Interaction::ApplicationCommand(cmd) => {
                    ctx.http
                        .interaction_callback(
                            cmd.id,
                            &cmd.token,
                            &InteractionResponse::DeferredChannelMessageWithSource(CallbackData {
                                allowed_mentions: None,
                                components: None,
                                content: None,
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await
                        .ok();
                }
                _ => {}
            },
            _ => {}
        }

        ctx.vm_manager.handle_discord_event(event).await;

        // println!("Event: {:?}", event);
    }
}
