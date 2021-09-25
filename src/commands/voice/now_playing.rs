use serenity::{
    client::Context,
    framework::{
        standard::{
            macros::command,
            CommandResult,
        },
    },
    model::channel::Message,
};

use tracing::{debug, error, info};

use crate::lavalink::Lavalink;
use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;


#[command]
#[aliases(np)]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>()
                .expect("Unable to retrieve Lavalink client from data.").clone();

    if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {

        if let Some(track) = &node.now_playing {
            check_msg(msg.reply(&ctx.http,
                        format!("Now Playing: {}", track.track.info.as_ref().unwrap().title)).await);
        } else {
            check_msg(msg.reply(&ctx.http, "Nothing is playing at the moment.").await);
        }

    } else {
        check_msg(msg.reply(&ctx.http, "Nothing is playing at the moment.").await);
    }

    Ok(())
}