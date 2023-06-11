use anyhow::{anyhow, Result};
use serenity::all::{ChannelId, Guild};
use songbird::{
    input::HttpRequest,
    tracks::{Track, TrackHandle},
    Songbird,
};
use sunk::{song::Song, Streamable};
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::handles::SubsonicSongHandle;

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
    type_map.insert::<SubsonicSongHandle>(song.clone());

    Ok(())
}

pub fn get_guild(ctx: Context<'_>) -> Guild {
    let guild = ctx.guild().expect("No guild!").clone();
    log::info!("Got guild: {guild:?}");
    guild
}

pub fn get_channel(ctx: Context<'_>) -> Result<ChannelId> {
    let guild = get_guild(ctx);
    let voice_state = guild
        .voice_states
        .get(&ctx.author().id)
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))?;
    let channel = voice_state
        .channel_id
        .ok_or_else(|| anyhow!("You must be in a voice channel to use this command"))?;

    Ok(channel)
}

pub async fn get_call(ctx: Context<'_>) -> Result<Arc<Mutex<songbird::Call>>> {
    let manager = get_manager(ctx).await?;
    let guild = get_guild(ctx);
    let call = manager.get(guild.id);

    if let Some(c) = call {
        Ok(c)
    } else {
        let channel = get_channel(ctx)?;
        log::warn!("Not in a voice channel, trying to join {channel}");
        let _handler = manager.join(guild.id, channel).await;
        manager
            .get(guild.id)
            .ok_or_else(|| anyhow!("Not in a voice channel, try running 'join'"))
    }
}

pub async fn get_song(track: &TrackHandle) -> Result<Song> {
    let song = track
        .typemap()
        .read()
        .await
        .get::<SubsonicSongHandle>()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("Sound information not found"))?;
    Ok(song)
}

pub async fn get_manager(ctx: Context<'_>) -> Result<Arc<Songbird>> {
    let manager = songbird::get(ctx.discord())
        .await
        .ok_or_else(|| anyhow!("Couldn't start manager"))?;
    log::info!("Got manager: {manager:?}");
    Ok(manager)
}

pub async fn load_song(song: &Song, client: &sunk::Client) -> Result<Track> {
    log::info!("Loading song {song:?}");
    let url = song.stream_url(client)?;
    let track: Track = HttpRequest::new(client.reqclient.clone(), url).into();
    Ok(track)
}
