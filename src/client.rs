use anyhow::{Context as ErrContext, Result};
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
        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let framework = StandardFramework::new()
            .group(&GENERAL_GROUP)
            .after(after_hook);

        framework.configure(|c| c.prefix("~"));

        let client = DiscordClient::builder(&self.discord_token, intents)
            .event_handler(Handler)
            .framework(framework)
            .type_map_insert::<MusicClient>(ss)
            .register_songbird()
            .await?;

        Ok(client)
    }

    pub async fn subsonic(&self) -> Result<SubsonicClient> {
        let client = SubsonicClient::new(&self.ss_url, &self.ss_user, &self.ss_password)?;
        log::info!("Created subsonic client: {client:?}");
        // Check that connection works
        client
            .ping()
            .await
            .with_context(|| "Could not connect to subsonic server.")?;

        Ok(client)
    }
}

pub struct MusicClient;
impl TypeMapKey for MusicClient {
    type Value = SubsonicClient;
}
