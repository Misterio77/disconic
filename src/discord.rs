use anyhow::{anyhow, Context as ErrContext, Result};
use serenity::{
    all::{ChannelId, GuildId},
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult,
    },
    model::{channel::Message, gateway::Ready},
    prelude::TypeMapKey,
    utils::MessageBuilder,
};
use songbird::{
    input::HttpRequest,
    tracks::{Track, TrackHandle},
    Songbird,
};
use sunk::{
    search::{self, SearchPage},
    song::Song,
    Streamable,
};
use tokio::sync::Mutex;

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
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = get_guild(ctx, msg)?;
    let manager = get_manager(ctx).await?;
    manager.remove(guild).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = get_guild(ctx, msg)?;
    let channel = get_channel(ctx, msg)?;
    let manager = get_manager(ctx).await?;
    let _handler = manager.join(guild, channel).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
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
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("No song matching search found"))?;

    log::info!("Found song {song:?}");
    queue_song(ctx, msg, song, music_client).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
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
        queue_song(ctx, msg, song, music_client).await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[aliases(r, rand)]
/// Play a random song
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let result = Song::random(music_client, 1).await?;

    let song: Song = result
        .first()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("No song matching search found"))?;
    queue_song(ctx, msg, song, music_client).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
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
#[only_in(guilds)]
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
#[only_in(guilds)]
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
#[only_in(guilds)]
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
#[only_in(guilds)]
#[aliases(nowplaying, now, np, playing)]
/// Show currently playing song
async fn nowplaying(ctx: &Context, msg: &Message) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let queue = handler.queue();
    let current = queue
        .current()
        .ok_or_else(|| anyhow!("Not currently playing"))?;

    let song = get_song(&current).await?;
    let text = format!(
        "**Currently playing**: {} - {}",
        song.title,
        song.artist.as_deref().unwrap_or_default()
    );
    msg.reply(&ctx.http, text).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
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

    msg.reply(&ctx.http, text).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
/// Remove song from queue, given id
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let call = get_call(ctx, msg).await?;
    let handler = call.lock().await;

    let index = args.single()?;

    let queue = handler.queue();
    let track = queue
        .dequeue(index)
        .ok_or_else(|| anyhow!("Song not found"))?;
    let song = get_song(&track).await?;
    let text = format!("Removed track: {}", song.title);
    msg.reply(&ctx.http, text).await?;
    Ok(())
}

// ==========================
// ==========================
// ==========================

struct SongHandler;
impl TypeMapKey for SongHandler {
    type Value = Song;
}

async fn queue_song(ctx: &Context, msg: &Message, song: Song, client: &sunk::Client) -> Result<()> {
    let call = get_call(ctx, msg).await?;
    let mut handler = call.lock().await;

    let song_info = format!(
        "{} - {} ",
        song.artist.as_deref().unwrap_or_default(),
        song.title,
    );

    let track = load_song(&song, client).await?;
    let track_handle = handler.enqueue(track).await;
    {
        let mut type_map = track_handle.typemap().write().await;
        type_map.insert::<SongHandler>(song)
    }

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

fn get_guild(ctx: &Context, msg: &Message) -> Result<GuildId> {
    let guild = msg.guild(&ctx.cache).unwrap();
    Ok(guild.id)
}

fn get_channel(ctx: &Context, msg: &Message) -> Result<ChannelId> {
    let guild = msg.guild(&ctx.cache).unwrap();
    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))?;

    Ok(channel_id)
}

async fn get_call(ctx: &Context, msg: &Message) -> Result<Arc<Mutex<songbird::Call>>> {
    let manager = get_manager(ctx).await?;
    let guild = get_guild(ctx, msg)?;
    let call = manager
        .get(guild)
        .ok_or_else(|| anyhow!("Not in a voice channel"))?;
    Ok(call)
}

async fn get_song(track: &TrackHandle) -> Result<Song> {
    let song = track
        .typemap()
        .read()
        .await
        .get::<SongHandler>()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("Sound information not found"))?;
    Ok(song)
}

async fn get_manager(ctx: &Context) -> Result<Arc<Songbird>> {
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))?;
    Ok(manager)
}

async fn load_song(song: &Song, client: &sunk::Client) -> Result<Track> {
    log::info!("Loading song {song:?}");
    let url = song.stream_url(client)?;
    let track: Track = HttpRequest::new(client.reqclient.clone(), url).into();
    Ok(track)
}
