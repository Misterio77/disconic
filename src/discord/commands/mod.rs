pub mod album;
pub mod join;
pub mod leave;
pub mod nowplaying;
pub mod pause;
pub mod queue;
pub mod random;
pub mod remove;
pub mod resume;
pub mod skip;
pub mod song;
pub mod stop;

#[poise::command(prefix_command)]
pub async fn register(ctx: crate::discord::Context<'_>) -> Result<(), anyhow::Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub fn commands() -> Vec<poise::Command<super::Data, anyhow::Error>> {
    vec![
        register(),
        album::album(),
        join::join(),
        leave::leave(),
        nowplaying::nowplaying(),
        pause::pause(),
        queue::queue(),
        random::random(),
        remove::remove(),
        resume::resume(),
        skip::skip(),
        song::song(),
        stop::stop(),
    ]
}
