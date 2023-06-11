use anyhow::Result;

use disconic::Client;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let client = Client::from_env().await?;
    let subsonic = client.subsonic().await?;
    let mut discord = client.discord(subsonic).await?;

    discord.start().await?;

    Ok(())
}
