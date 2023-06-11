use crate::handles::SubsonicClientHandle;
use anyhow::{Context as ErrContext, Result};
use clap::Parser;
use log::LevelFilter;
use serenity::{
    client::Client as DiscordClient, framework::standard::StandardFramework,
    prelude::GatewayIntents,
};
use simple_logger::SimpleLogger;
use songbird::SerenityInit;
use sunk::Client as SubsonicClient;

use crate::discord::{after_hook, Handler, GENERAL_GROUP};

#[derive(Parser, Clone)]
pub struct Config {
    #[clap(long, env = "DISCONIC_SUBSONIC_URL")]
    subsonic_url: String,
    #[clap(long, env = "DISCONIC_SUBSONIC_USER")]
    subsonic_user: String,
    #[clap(long, env = "DISCONIC_SUBSONIC_PASSWORD")]
    subsonic_password: String,
    #[clap(long, env = "DISCONIC_DISCORD_TOKEN")]
    discord_token: String,
    #[clap(long, env = "DISCONIC_LOG_LEVEL", default_value = "warn")]
    log_level: LevelFilter,
}

impl Config {
    pub async fn discord(&self, ss: SubsonicClient) -> Result<DiscordClient> {
        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let framework = StandardFramework::new()
            .group(&GENERAL_GROUP)
            .after(after_hook);

        framework.configure(|c| c.prefix("~"));

        let client = DiscordClient::builder(&self.discord_token, intents)
            .event_handler(Handler)
            .framework(framework)
            .type_map_insert::<SubsonicClientHandle>(ss)
            .register_songbird()
            .await?;

        Ok(client)
    }

    pub async fn subsonic(&self) -> Result<SubsonicClient> {
        let client = SubsonicClient::new(
            &self.subsonic_url,
            &self.subsonic_user,
            &self.subsonic_password,
        )?;
        log::info!("Created subsonic client: {client:?}");
        // Check that connection works
        client
            .ping()
            .await
            .with_context(|| "Could not connect to subsonic server.")?;

        Ok(client)
    }

    pub fn logger(&self) -> SimpleLogger {
        SimpleLogger::new().with_level(self.log_level)
    }
}
