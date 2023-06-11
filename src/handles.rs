use serenity::prelude::TypeMapKey;
use sunk::{song::Song as SubsonicSong, Client as SubsonicClient};

pub struct SubsonicClientHandle;
impl TypeMapKey for SubsonicClientHandle {
    type Value = SubsonicClient;
}

pub struct SubsonicSongHandle;
impl TypeMapKey for SubsonicSongHandle {
    type Value = SubsonicSong;
}
