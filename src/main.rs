use anyhow::Result;
use log::warn;

use disconic::Client;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let client = Client::from_env().await?;
    warn!("Initialized disconic client.");
    let subsonic = client.subsonic().await?;
    warn!("Initialized subsonic client.");
    let mut discord = client.discord(subsonic).await?;
    warn!("Initialized discord client.");

    discord.start().await?;

    Ok(())
}
