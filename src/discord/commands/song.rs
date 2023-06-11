use anyhow::{anyhow, Context as ErrContext, Result};
use poise::command;
use sunk::search::{self, SearchPage};

use crate::discord::{common::queue_song, Context};

/// Search for a song, and queue it
#[command(slash_command, prefix_command)]
pub async fn song(
    ctx: Context<'_>,
    #[description = "What to search for"] query: String,
) -> Result<()> {
    let music_client = &ctx.data().subsonic_client;

    let search_size = SearchPage::new().with_size(1);
    let ignore = search::NONE;

    let result = music_client
        .search(&query, ignore, ignore, search_size)
        .await
        .with_context(|| anyhow!("Could not search for song"))?
        .songs;

    let song = result
        .first()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("No song matching search found"))?;

    log::info!("Found song {song:?}");
    queue_song(ctx, &song, music_client).await?;

    let message = format!(
        "Added **{} - {}** to the queue",
        song.title,
        song.artist.as_deref().unwrap_or_default(),
    );
    ctx.say(message).await?;

    Ok(())
}
