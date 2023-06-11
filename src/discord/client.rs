use crate::discord::{commands, Data};
use anyhow::Result;
use poise::samples::create_application_commands;
use serenity::{all::GuildId, client::Client as DiscordClient, prelude::GatewayIntents};
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

pub async fn create_client(
    token: &str,
    guild_id: Option<u64>,
    subsonic_client: SubsonicClient,
) -> Result<DiscordClient> {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let commands = commands::commands();
    let data = Data { subsonic_client };

    let create_commands = create_application_commands(&commands);
    let options = poise::FrameworkOptions {
        commands,
        on_error: |e| Box::pin(on_error(e)),
        ..Default::default()
    };
    let framework = poise::Framework::new(options, move |ctx, _ready, _framework| {
        Box::pin(async move {
            if let Some(id) = guild_id {
                let guild = GuildId::new(id);
                guild
                    .set_commands(&ctx.http, create_commands)
                    .await
                    .expect("Failed to register command");
            } else {
                log::warn!("Guild ID not configured. You'll have to run 'register' to register the slash commands.")
            }
            log::info!("Registered commands");
            Ok(data)
        })
    });

    let client = DiscordClient::builder(token, intents)
        .framework(framework)
        .register_songbird()
        .await?;
    Ok(client)
}
