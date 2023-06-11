use anyhow::{anyhow, Result};
use poise::command;
use sunk::search;

use crate::discord::{common::queue_song, Context};

/// Search for an album, and queue all its songs
#[command(slash_command, prefix_command)]
pub async fn album(
    ctx: Context<'_>,
    #[description = "What to search for"] query: String,
) -> Result<()> {
    let music_client = &ctx.data().subsonic_client;

    let search_size = search::SearchPage::new().with_size(1);
    let ignore = search::NONE;

    let result = music_client
        .search(&query, ignore, search_size, ignore)
        .await?
        .albums;

    let album = result
        .first()
        .ok_or_else(|| anyhow!("No albums matching search found"))?;

    let songs = album.songs(&music_client).await?;
    for song in songs.iter() {
        queue_song(ctx, &song, &music_client).await?;
    }

    let message = format!(
        "Added album **{} - {}** ({} songs) to the queue",
        album.name,
        album.artist.as_deref().unwrap_or_default(),
        songs.len(),
    );
    ctx.say(message).await?;

    Ok(())
}
