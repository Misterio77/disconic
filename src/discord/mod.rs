use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    utils::MessageBuilder,
};
use sunk::{
    search::{self, SearchPage},
    Streamable,
};

use crate::MusicClient;

#[group]
#[commands(play)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[command]
#[only_in(guilds)]
#[aliases(p, queue, song)]
#[example("~play Down Under")]
#[description("Tocar uma música")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    let music_client = data
        .get::<MusicClient>()
        .expect("Couldn't retrieve music client");

    let search_size = SearchPage::new().with_size(1);
    let ignore = search::NONE;

    let result = music_client
        .search(args.rest(), ignore, ignore, search_size)
        .await?
        .songs;

    match result.first() {
        Some(song) => {
            let guild = msg.guild(&ctx.cache).await.unwrap();
            let guild_id = guild.id;

            let caller_channel = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);

            let connect_to = match caller_channel {
                Some(channel) => channel,
                None => {
                    msg.reply(ctx, "Você não está em um canal de voz").await?;
                    return Ok(());
                }
            };

            let manager = songbird::get(ctx).await.unwrap();
            let handler = manager.join(guild_id, connect_to).await;
            let mut channel_handler = handler.0.lock().await;

            let url = song.stream_url(&music_client)?;
            let input = songbird::ffmpeg(url).await.unwrap();

            channel_handler.play_only_source(input);

            let message = MessageBuilder::new()
                .push("Agora tocando ")
                .push_bold_safe(format!(
                    "{} - {} ({})",
                    song.artist.clone().unwrap_or_default(),
                    song.title,
                    song.album.clone().unwrap_or_default(),
                ))
                .build();

            msg.channel_id.say(&ctx.http, &message).await?;

            Ok(())
        }
        None => {
            msg.channel_id
                .say(&ctx.http, "Nenhuma música encontrada")
                .await?;
            Ok(())
        }
    }
}
