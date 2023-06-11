use anyhow::Result;
use clap::Parser;

use disconic::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config = Config::parse();
    let logger = config.logger();
    let subsonic = config.subsonic().await?;
    let mut discord = config.discord(subsonic).await?;

    logger.init()?;
    discord.start().await?;

    Ok(())
}
