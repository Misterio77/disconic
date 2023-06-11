use anyhow::{anyhow, Result};
use poise::command;

use crate::discord::{common::get_call, Context};

/// Skip song(s)
#[command(slash_command, prefix_command)]
pub async fn skip(
    ctx: Context<'_>,
    #[description = "Number of songs to skip"] n: Option<usize>,
) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;
    let n = n.unwrap_or(1);

    let queue = handler.queue();
    for _ in 0..(n - 1) {
        queue.dequeue(1).ok_or_else(|| anyhow!("Song not found"))?;
    }
    queue.skip()?;

    ctx.say(&format!("{n} song(s) skipped")).await?;

    Ok(())
}
