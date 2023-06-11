use anyhow::Result;
use poise::command;

use crate::discord::{common::get_call, Context};

/// Pause playback
#[command(slash_command, prefix_command)]
pub async fn pause(ctx: Context<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = queue.current();
    if let Some(track) = current {
        track.pause()?;
        ctx.say("Paused playback").await?;
    } else {
        ctx.say("Not currently playing").await?;
    }

    Ok(())
}
