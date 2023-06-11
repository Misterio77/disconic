use anyhow::{anyhow, Result};
use poise::command;

use crate::discord::{
    common::{get_call, get_song},
    Context,
};

/// Show what's currently playing
#[command(slash_command, prefix_command)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = queue
        .current()
        .ok_or_else(|| anyhow!("Not currently playing"))?;

    let song = get_song(&current).await?;
    let text = format!(
        "**Currently playing**: {} - {}",
        song.title,
        song.artist.as_deref().unwrap_or_default()
    );
    ctx.say(text).await?;

    Ok(())
}
