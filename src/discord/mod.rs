use anyhow::{anyhow, Result};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::MessageBuilder,
};
use songbird::input::{Input, Metadata};
use std::sync::Arc;
use sunk::{
    search::{self, SearchPage},
    song::Song,
    Streamable,
};

use crate::MusicClient;

#[group]
#[commands(song, random, skip, stop, pause, resume, queue, now_playing, remove)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[command]
#[aliases(s, p, play)]
/// Play a named song
async fn song(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let search_size = SearchPage::new().with_size(1);
    let ignore = search::NONE;

    let result = music_client
        .search(args.rest(), ignore, ignore, search_size)
        .await?
        .songs;

    match result.first() {
        Some(song) => {
            queue_song(ctx, msg, &song, &music_client).await?;
            Ok(())
        }
        None => {
            let text = "No song matching search found";
            msg.reply(&ctx.http, text).await?;
            Err(anyhow!(text).into())
        }
    }
}

#[command]
#[aliases(r, rand)]
/// Play a random song
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let result = Song::random(&music_client, 1).await?;

    match result.first() {
        Some(song) => {
            queue_song(ctx, msg, &song, &music_client).await?;
            Ok(())
        }
        None => {
            let text = "No song found";
            msg.reply(&ctx.http, text).await?;
            Err(anyhow!(text).into())
        }
    }
}

#[command]
#[aliases(pl)]
/// Play a playlist
async fn playlist(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let result = Song::random(&music_client, 1).await?;

    match result.first() {
        Some(song) => {
            queue_song(ctx, msg, &song, &music_client).await?;
            Ok(())
        }
        None => {
            let text = "No song found";
            msg.reply(&ctx.http, text).await?;
            Err(anyhow!(text).into())
        }
    }
}

#[command]
/// Skip current song
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.skip()?;

    msg.reply(&ctx.http, "Song skipped").await?;

    Ok(())
}

#[command]
/// Clear queue and stop playing
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.stop();

    msg.reply(&ctx.http, "Stopped playing").await?;

    Ok(())
}

#[command]
/// Pause playing current song
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = match queue.current() {
        Some(channel) => channel,
        None => {
            let text = "Not currently playing";
            msg.reply(ctx, text).await?;
            return Err(anyhow!(text).into());
        }
    };
    current.pause()?;

    msg.reply(&ctx.http, "Paused playing").await?;

    Ok(())
}

#[command]
#[aliases(resume)]
/// Resume playing current song
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.resume()?;

    msg.reply(&ctx.http, "Resumed playing").await?;

    Ok(())
}

#[command]
#[aliases(now_playing, now, np, playing)]
/// Show currently playing song
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = match queue.current() {
        Some(channel) => channel,
        None => {
            let text = "Not currently playing";
            msg.reply(ctx, text).await?;
            return Err(anyhow!(text).into());
        }
    };
    let text = song_message(None, current.metadata());
    msg.reply(&ctx.http, text).await?;

    Ok(())
}

#[command]
#[aliases(q)]
/// Show song queue
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let current_queue = handler.queue().current_queue();

    let text = if current_queue.is_empty() {
        "No songs queued".into()
    } else {
        let mut text = String::new();
        for (i, track) in current_queue.iter().enumerate() {
            text.push_str(&song_message(Some(i), track.metadata()));
            text.push_str("\n");
        }
        text
    };

    msg.reply(&ctx.http, text).await?;
    Ok(())
}

#[command]
/// Remove song from queue, given id
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let call = join_channel(&ctx, &msg).await?;
    let handler = call.lock().await;

    let index = args.single()?;

    let queue = handler.queue();
    let text = if let Some(track) = queue.current_queue().get(index) {
        queue.dequeue(index);
        let metadata = track.metadata();
        format!(
            "Removed track: {} - {} ",
            metadata.artist.to_owned().unwrap_or_default(),
            metadata.track.to_owned().unwrap_or_default(),
        )
    } else {
        "No song with that index".into()
    };

    msg.reply(&ctx.http, text).await?;
    Ok(())
}

fn song_message(index: Option<usize>, metadata: &Metadata) -> String {
    let prefix = match index {
        Some(i) => format!("- {}: ", i),
        None => "Current: ".into(),
    };

    let duration = metadata
        .duration
        .map(|d| format!("{}:{}", d.as_secs() / 60, d.as_secs() % 60));

    let song_info = format!(
        "{} - {} ({})",
        metadata.artist.to_owned().unwrap_or_default(),
        metadata.track.to_owned().unwrap_or_default(),
        duration.to_owned().unwrap_or_default(),
    );

    MessageBuilder::new()
        .push_bold(prefix)
        .push(song_info)
        .build()
}

async fn queue_song(
    ctx: &Context,
    msg: &Message,
    song: &Song,
    client: &sunk::Client,
) -> Result<()> {
    let input = load_song(song, client).await?;
    let call = join_channel(&ctx, &msg).await?;
    let mut handler = call.lock().await;
    handler.enqueue_source(input);

    let song_info = format!(
        "{} - {} ",
        song.artist.to_owned().unwrap_or_default(),
        song.title,
    );

    msg.reply(
        &ctx.http,
        &MessageBuilder::new()
            .push("Added ")
            .push_bold_safe(song_info)
            .push("to the queue")
            // .push("\n")
            // .push(song.cover_art_url(client, 256)?)
            .build(),
    )
    .await?;

    Ok(())
}

async fn join_channel(
    ctx: &Context,
    msg: &Message,
) -> Result<Arc<serenity::prelude::Mutex<songbird::Call>>> {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let caller_channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match caller_channel {
        Some(channel) => channel,
        None => {
            let text = "You must be in a voice channel to use this command";
            msg.reply(ctx, text).await?;
            return Err(anyhow!(text));
        }
    };

    let manager = songbird::get(ctx).await.unwrap();
    let handler = manager.join(guild_id, connect_to).await.0;
    Ok(handler)
}

async fn load_song(song: &Song, client: &sunk::Client) -> Result<Input> {
    let url = song.stream_url(&client)?;
    Ok(songbird::ffmpeg(url).await?)
}
