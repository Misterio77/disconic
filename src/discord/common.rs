use anyhow::{anyhow, Context as ErrContext, Result};
use serenity::{
    all::{ChannelId, Guild},
    prelude::TypeMapKey,
};
use songbird::{
    input::HttpRequest,
    tracks::{Track, TrackHandle},
    Songbird,
};
use sunk::{song::Song, Streamable};
use tokio::sync::Mutex;

use std::sync::Arc;

pub struct SongHandle;
impl TypeMapKey for SongHandle {
    type Value = Song;
}

pub struct Data {
    pub subsonic_client: sunk::Client,
}
pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

pub async fn queue_song(ctx: Context<'_>, song: &Song, client: &sunk::Client) -> Result<()> {
    let call = get_call(ctx).await?;
    let mut handler = call.lock().await;

    let track = load_song(song, client).await?;
    let track_handle = handler.enqueue(track).await;
    let mut type_map = track_handle.typemap().write().await;
    type_map.insert::<SongHandle>(song.clone());

    Ok(())
}

pub fn get_guild(ctx: Context<'_>) -> Guild {
    ctx.guild()
        .expect("Invalid (or no) guild configured!")
        .to_owned()
}

pub fn get_channel(ctx: Context<'_>) -> Result<ChannelId> {
    let guild = get_guild(ctx);
    let voice_state = guild
        .voice_states
        .get(&ctx.author().id)
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))?;
    voice_state
        .channel_id
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))
}

pub async fn get_call(ctx: Context<'_>) -> Result<Arc<Mutex<songbird::Call>>> {
    let manager = get_manager(ctx).await?;
    let guild = get_guild(ctx);
    match manager.get(guild.id) {
        Some(c) => Ok(c),
        None => {
            let channel = get_channel(ctx)?;
            log::warn!("Not in a voice channel, trying to join {channel}");
            manager
                .join(guild.id, channel)
                .await
                .context("Couldn't join voice channel. Try running 'join' manually.")
        }
    }
}

pub async fn get_song(track: &TrackHandle) -> Result<Song> {
    track
        .typemap()
        .read()
        .await
        .get::<SongHandle>()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("Sound information not found"))
}

pub async fn get_manager(ctx: Context<'_>) -> Result<Arc<Songbird>> {
    songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))
}

pub async fn load_song(song: &Song, client: &sunk::Client) -> Result<Track> {
    let url = song.stream_url(client)?;
    let track = HttpRequest::new(client.reqclient.clone(), url).into();
    Ok(track)
}
