use anyhow::Result;
use poise::command;

use crate::discord::{common::get_call, Context};

/// Resume playing current song
#[command(slash_command, prefix_command)]
pub async fn resume(ctx: Context<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.resume()?;

    ctx.say("Resumed playing").await?;

    Ok(())
}
