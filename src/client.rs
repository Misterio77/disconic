use anyhow::Result;
use serenity::{
    client::Client as DiscordClient,
    framework::standard::StandardFramework,
    prelude::{GatewayIntents, TypeMapKey},
};
use songbird::SerenityInit;
use sunk::Client as SubsonicClient;

use std::{env, fs, io};

use crate::discord::{after_hook, Handler, GENERAL_GROUP};

pub struct Client {
    ss_url: String,
    ss_user: String,
    ss_password: String,
    discord_token: String,
}

impl Client {
    pub async fn from_env() -> Result<Self> {
        // Convert to std::io::Error, allowing usage of and_then
        let convert_err = |e| io::Error::new(io::ErrorKind::Other, e);

        let ss_url = env::var("SUBSONIC_URL")?;
        let ss_user = env::var("SUBSONIC_USER")?;
        let ss_password = env::var("SUBSONIC_PASSWORD").or_else(|_| {
            env::var("SUBSONIC_PASSWORD_FILE")
                .map_err(convert_err)
                .and_then(fs::read_to_string)
                .map(|s| s.trim().to_owned())
        })?;
        let discord_token = env::var("DISCORD_TOKEN").or_else(|_| {
            env::var("DISCORD_TOKEN_FILE")
                .map_err(convert_err)
                .and_then(fs::read_to_string)
                .map(|s| s.trim().to_owned())
        })?;

        Ok(Self {
            ss_url,
            ss_user,
            ss_password,
            discord_token,
        })
    }

    pub async fn discord(&self, ss: SubsonicClient) -> Result<DiscordClient> {
        Ok(
            DiscordClient::builder(&self.discord_token, GatewayIntents::default())
                .event_handler(Handler)
                .framework(
                    StandardFramework::new()
                        .configure(|c| c.prefix("~"))
                        .group(&GENERAL_GROUP)
                        .after(after_hook),
                )
                .type_map_insert::<MusicClient>(ss)
                .register_songbird()
                .await?,
        )
    }

    pub async fn subsonic(&self) -> Result<SubsonicClient> {
        Ok(SubsonicClient::new(
            &self.ss_url,
            &self.ss_user,
            &self.ss_password,
        )?)
    }
}

pub struct MusicClient;
impl TypeMapKey for MusicClient {
    type Value = SubsonicClient;
}
