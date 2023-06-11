use anyhow::Result;
use poise::command;

use crate::discord::{
    common::{get_call, get_song},
    Context,
};

/// Get play queue list
#[command(slash_command, prefix_command)]
pub async fn queue(ctx: Context<'_>) -> Result<()> {
    let call = get_call(ctx).await?;
    let handler = call.lock().await;

    let current_queue = handler.queue().current_queue();

    let text = if current_queue.is_empty() {
        "No songs queued".into()
    } else {
        let mut text = String::new();
        let mut queue = current_queue.iter().enumerate();

        let (_, track) = queue.next().unwrap();
        let song = get_song(track).await?;
        text.push_str(&format!(
            "**Currently playing**: {} - {}\n\n",
            song.title,
            song.artist.as_deref().unwrap_or_default()
        ));

        text.push_str("**Next songs in queue**:\n");
        for (i, track) in queue {
            let song = get_song(track).await?;
            let song_text = format!(
                "**{}.** {} - {}",
                i,
                song.title,
                song.artist.as_deref().unwrap_or_default()
            );
            text.push_str(&song_text);
            text.push('\n');
        }
        text
    };

    ctx.say(text).await?;
    Ok(())
}
