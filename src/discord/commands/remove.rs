use anyhow::{anyhow, Result};
use poise::command;

use crate::discord::{
    common::{get_call, get_song},
    Context,
};

/// Remove song from queue
#[command(slash_command, prefix_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Song position on the queue"] index: usize,
) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let track = queue
        .dequeue(index - 1)
        .ok_or_else(|| anyhow!("Song not found"))?;
    let song = get_song(&track).await?;
    let text = format!("Removed track: {}", song.title);
    ctx.say(text).await?;
    Ok(())
}
