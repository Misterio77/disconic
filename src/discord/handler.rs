use serenity::{
    all::Ready,
    async_trait,
    prelude::{Context, EventHandler},
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
    }
}
