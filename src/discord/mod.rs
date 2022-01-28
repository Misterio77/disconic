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
use songbird::input::Input;
use std::sync::Arc;
use sunk::{
    search::{self, SearchPage},
    song::Song,
    Streamable,
};

use crate::MusicClient;

#[group]
#[commands(song, random)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[command]
#[only_in(guilds)]
#[aliases(s, p, play)]
#[example("~song Down Under")]
#[description("Play a named song")]
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
#[only_in(guilds)]
#[aliases(r, random, rand)]
#[example("~random")]
#[description("Play a random song")]
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
        "{} - {} ({}) ",
        song.artist.clone().unwrap_or_default(),
        song.title,
        song.album.clone().unwrap_or_default(),
    );

    msg.reply(
        &ctx.http,
        &MessageBuilder::new()
            .push("Added ")
            .push_bold_safe(song_info)
            .push("to the queue")
            .build(),
    )
    .await?;

    Ok(())
}

async fn join_channel<'a>(
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
