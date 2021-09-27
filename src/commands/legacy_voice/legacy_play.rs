use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
};

use url::Url;

use tracing::{debug, error, info};

use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot_or_bot_not_in_any_voice)]
async fn legacy_play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.reply(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    if let Err(_e) = Url::parse(&url) {
        check_msg(msg.reply(&ctx.http, "Are u sure this is a valid URL?").await);

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        // Attempt to join channel if not already in one.
        None => {
            let channel_id = guild
                .voice_states.get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);

            let connect_to = match channel_id {
                Some(channel) => channel,
                None => {
                    check_msg(msg.reply(&ctx.http, "U are not in a voice channel!").await);
                    return Ok(());
                }
            };

            let _handler = manager.join(guild_id, connect_to).await;
            manager.get(guild_id).unwrap()
        },
    };

    let mut handler = handler_lock.lock().await;

    let source = match songbird::ytdl(&url).await {
        Ok(source) => source,
        Err(why) => {
            error!("Sourcing FFmpeg failed: {:?}", why);
            check_msg(msg.reply(&ctx.http, "Attempt to play audio failed. Please try again.").await);

            return Ok(());
        },
    };

    handler.play_source(source);

    check_msg(msg.reply(&ctx.http, "Playing!").await);

    Ok(())
}