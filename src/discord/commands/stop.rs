use anyhow::Result;
use poise::command;

use crate::discord::{common::get_call, Context};

/// Stop playing and clear queue
#[command(slash_command, prefix_command)]
pub async fn stop(ctx: Context<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.stop();

    ctx.say("Stopped playing").await?;

    Ok(())
}
