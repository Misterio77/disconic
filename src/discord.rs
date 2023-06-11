use anyhow::{anyhow, Context as ErrContext, Result};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult,
    },
    model::{channel::Message, gateway::Ready},
    prelude::Mutex,
    utils::MessageBuilder,
};
use songbird::{
    input::{Input, Metadata},
    Call,
};
use sunk::{
    search::{self, SearchPage},
    song::Song,
    Streamable,
};

use std::sync::Arc;

use crate::MusicClient;

#[group]
#[commands(
    song, random, skip, stop, pause, resume, queue, nowplaying, remove, album, join, leave
)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[hook]
pub async fn after_hook(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    if let Err(why) = error {
        msg.reply(&ctx.http, format!("{:?}", why)).await.ok();
        eprintln!("{}: {:?}", cmd_name, why,);
    }
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))?;

    manager.remove(guild_id).await?;
    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))?;
    let _handler = manager.join(guild_id, channel).await;
    Ok(())
}

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
        .await
        .with_context(|| anyhow!("Could not search for song"))?
        .songs;

    let song = result
        .first()
        .ok_or_else(|| anyhow!("No song matching search found"))?;

    log::info!("Found song {song:?}");
    queue_song(ctx, msg, song, music_client).await?;

    Ok(())
}

#[command]
#[aliases(a, album)]
/// Play a named album
async fn album(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let search_size = SearchPage::new().with_size(1);
    let ignore = search::NONE;

    let result = music_client
        .search(args.rest(), ignore, search_size, ignore)
        .await?
        .albums;

    let album = result
        .first()
        .ok_or_else(|| anyhow!("No albums matching search found"))?;

    for song in album.songs(music_client).await? {
        queue_song(ctx, msg, &song, music_client).await?;
    }

    Ok(())
}

#[command]
#[aliases(r, rand)]
/// Play a random song
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let result = Song::random(music_client, 1).await?;

    let song = result
        .first()
        .ok_or_else(|| anyhow!("No song matching search found"))?;
    queue_song(ctx, msg, song, music_client).await?;

    Ok(())
}

#[command]
/// Skip current song
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.skip()?;

    msg.reply(&ctx.http, "Song skipped").await?;

    Ok(())
}

#[command]
/// Clear queue and stop playing
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.stop();

    msg.reply(&ctx.http, "Stopped playing").await?;

    Ok(())
}

#[command]
/// Pause playing current song
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = queue
        .current()
        .ok_or_else(|| anyhow!("Not currently playing"))?;
    current.pause()?;

    msg.reply(&ctx.http, "Paused playing").await?;

    Ok(())
}

#[command]
#[aliases(resume)]
/// Resume playing current song
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    queue.resume()?;

    msg.reply(&ctx.http, "Resumed playing").await?;

    Ok(())
}

#[command]
#[aliases(nowplaying, now, np, playing)]
/// Show currently playing song
async fn nowplaying(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = queue
        .current()
        .ok_or_else(|| anyhow!("Not currently playing"))?;

    msg.reply(&ctx.http, song_message(None, current.metadata()))
        .await?;

    Ok(())
}

#[command]
#[aliases(q)]
/// Show song queue
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let current_queue = handler.queue().current_queue();

    let text = if current_queue.is_empty() {
        "No songs queued".into()
    } else {
        let mut text = String::new();
        text.push_str("Next songs in queue:\n");
        for (i, track) in current_queue.iter().enumerate() {
            text.push_str(&song_message(Some(i), track.metadata()));
            text.push('\n');
        }
        text
    };

    msg.reply(&ctx.http, text).await?;
    Ok(())
}

#[command]
/// Remove song from queue, given id
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let index = args.single()?;

    let queue = handler.queue();
    let current_queue = queue.current_queue();

    let track = current_queue
        .get(index)
        .ok_or_else(|| anyhow!("No song with that index"))?;

    queue.dequeue(index);

    let metadata = track.metadata();
    let text = format!(
        "Removed track: {} - {} ",
        metadata.artist.to_owned().unwrap_or_default(),
        metadata.track.to_owned().unwrap_or_default(),
    );
    msg.reply(&ctx.http, text).await?;
    Ok(())
}

fn song_message(index: Option<usize>, metadata: &Metadata) -> String {
    let prefix = match index {
        Some(i) => format!("{}. ", i),
        None => "Current: ".into(),
    };

    let duration = metadata
        .duration
        .map(|d| format!("{}:{}", d.as_secs() / 60, d.as_secs() % 60));

    let song_info = format!(
        "{} - {} ({})",
        metadata.artist.to_owned().unwrap_or_default(),
        metadata.track.to_owned().unwrap_or_default(),
        duration.unwrap_or_default(),
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
    log::info!("Queueing song {song:?}");
    let call = get_call(ctx, msg).await?;
    log::info!("Will play in call {call:?}");
    let mut handler = call.lock().await;
    let input = load_song(song, client).await?;
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

async fn get_call(ctx: &Context, msg: &Message) -> Result<Arc<Mutex<Call>>> {
    let guild = msg
        .guild(&ctx.cache)
        .ok_or_else(|| anyhow!("Couldn't get guild id"))?;
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))?;
    Ok(manager
        .get(guild.id)
        .ok_or_else(|| anyhow!("Not currently in a channel"))?)
}

async fn load_song(song: &Song, client: &sunk::Client) -> Result<Input> {
    log::info!("Loading song {song:?}");
    let url = song.stream_url(client)?;
    log::info!("With url {url:?}");
    let mut input = songbird::ffmpeg(&url).await?;
    log::info!("Created songbird input");
    input.metadata.track = Some(song.title.clone());
    input.metadata.artist = song.artist.clone();
    input.metadata.source_url = Some(url);
    Ok(input)
}
