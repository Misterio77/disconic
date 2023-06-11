use anyhow::{anyhow, Result};
use poise::command;
use serenity::utils::MessageBuilder;
use sunk::song::Song;

use crate::discord::{common::queue_song, Context};

/// Play a random song
#[command(slash_command, prefix_command)]
pub async fn random(ctx: Context<'_>) -> Result<()> {
    let music_client = &ctx.data().subsonic_client;

    let result = Song::random(music_client, 1).await?;
    let song: Song = result
        .first()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("No song matching search found"))?;
    queue_song(ctx, &song, music_client).await?;

    let message = format!(
        "Added **{} - {}** to the queue",
        song.title,
        song.artist.as_deref().unwrap_or_default(),
    );
    ctx.say(&MessageBuilder::new().push(&message).build())
        .await?;

    Ok(())
}
