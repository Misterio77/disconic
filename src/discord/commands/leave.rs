use anyhow::Result;
use poise::command;

use crate::discord::{
    common::{get_guild, get_manager},
    Context,
};

/// Leave the voice chat
#[command(slash_command, prefix_command)]
pub async fn leave(ctx: Context<'_>) -> Result<()> {
    let guild = get_guild(ctx);
    let manager = get_manager(ctx).await?;
    manager.remove(guild.id).await?;
    ctx.say("Bye!").await?;
    Ok(())
}
