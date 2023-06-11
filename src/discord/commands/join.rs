use anyhow::Result;
use poise::command;

use crate::discord::{
    common::{get_channel, get_guild, get_manager},
    Context,
};

/// Join the voice chat where the caller is
#[command(slash_command, prefix_command)]
pub async fn join(ctx: Context<'_>) -> Result<()> {
    let guild = get_guild(ctx);
    let channel = get_channel(ctx)?;
    let manager = get_manager(ctx).await?;
    log::info!("Got manager: {manager:?}");
    let call = manager.join(guild.id, channel).await?;
    log::info!("Got call: {call:?}");
    ctx.say("Hi! Try '/song' or '/album' to start").await?;
    Ok(())
}
