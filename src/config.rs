use crate::discord;
use anyhow::{Context as ErrContext, Result};
use clap::Parser;
use log::LevelFilter;
use serenity::client::Client as DiscordClient;
use simple_logger::SimpleLogger;
use sunk::Client as SubsonicClient;

#[derive(Parser, Clone)]
pub struct Config {
    #[clap(long, env = "DISCONIC_SUBSONIC_URL")]
    subsonic_url: String,
    #[clap(long, env = "DISCONIC_SUBSONIC_USER")]
    subsonic_user: String,
    #[clap(long, env = "DISCONIC_SUBSONIC_PASSWORD")]
    subsonic_password: String,
    #[clap(long, env = "DISCONIC_DISCORD_GUILD")]
    discord_guild: Option<u64>,
    #[clap(long, env = "DISCONIC_DISCORD_TOKEN")]
    discord_token: String,
    #[clap(long, env = "DISCONIC_LOG_LEVEL", default_value = "warn")]
    log_level: LevelFilter,
}

impl Config {
    pub async fn discord(&self, subsonic_client: SubsonicClient) -> Result<DiscordClient> {
        discord::create_client(&self.discord_token, self.discord_guild, subsonic_client).await
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
