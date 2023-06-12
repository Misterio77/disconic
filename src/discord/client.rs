use crate::discord::{commands, Data};
use anyhow::{Error, Result};
use poise::{samples::create_application_commands, Framework};
use serenity::{
    all::{GuildId, Ready},
    builder::CreateCommand,
    client::Client as DiscordClient,
    prelude::{Context, GatewayIntents},
};
use songbird::SerenityInit;
use sunk::Client as SubsonicClient;

async fn on_error(error: poise::FrameworkError<'_, Data, anyhow::Error>) {
    let context = error.ctx();
    let message = match error {
        poise::FrameworkError::Command { error, ctx } => {
            format!(
                "Error while running `{}`: **{:?}**",
                ctx.command().name,
                error
            )
        }
        poise::FrameworkError::Listener { error, event, .. } => {
            format!(
                "Error on event `{:?}`: **{:?}**",
                event.snake_case_name(),
                error
            )
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                format!("Error: **{}**", e)
            } else {
                format!("Error: **unknown**")
            }
        }
    };
    log::error!("{message}");
    if let Some(ctx) = context {
        ctx.say(message).await.ok();
    }
}

async fn on_startup(
    guild: GuildId,
    data: Data,
    create_commands: Vec<CreateCommand>,
    ctx: &Context,
    ready: &Ready,
    framework: &Framework<Data, Error>,
) -> Result<Data> {
    let is_on_guild = (&ready.guilds).iter().any(|x| x.id == guild);

    let bot_id = framework.bot_id().await;
    let permissions = "311388293184";
    let scope = "bot%20applications.commands";

    if !is_on_guild {
        let invite_link = format!(
            "https://discord.com/oauth2/authorize?client_id={}&permissions={}&scope={}",
            bot_id, permissions, scope
        );
        log::error!("The bot is not your on your guild.");
        log::error!("Invite it with:\n {}", invite_link);
        framework.shard_manager().lock().await.shutdown_all().await;
        std::process::exit(1);
    }

    guild
        .set_commands(&ctx.http, create_commands)
        .await
        .expect("Failed to register command");
    log::info!("Registered commands");
    Ok(data)
}

pub async fn create_client(
    token: &str,
    guild_id: u64,
    subsonic_client: SubsonicClient,
) -> Result<DiscordClient> {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let commands = commands::commands();
    let data = Data { subsonic_client };
    let guild = GuildId::new(guild_id);

    let create_commands = create_application_commands(&commands);
    let options = poise::FrameworkOptions {
        commands,
        on_error: |e| Box::pin(on_error(e)),
        ..Default::default()
    };
    let framework = poise::Framework::new(options, move |ctx, ready, framework| {
        Box::pin(on_startup(
            guild,
            data,
            create_commands,
            ctx,
            ready,
            framework,
        ))
    });

    let client = DiscordClient::builder(token, intents)
        .framework(framework)
        .register_songbird()
        .await?;
    Ok(client)
}
