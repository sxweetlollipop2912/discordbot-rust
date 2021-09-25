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
use crate::lavalink::LoopingOption;
use crate::lavalink::Track;
use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn is_looping(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let lava_client = {
        let data = ctx.data.read().await;
        data.get::<Lavalink>()
            .expect("Unable to retrieve Lavalink client from data.").clone()
    };
    let lava_client_lock = lava_client.inner.lock().await;

    if let Some(node) = lava_client_lock.nodes.get(&guild_id.0) {
        let node_data = node.data.read().await;

        if let Some(looping_option) = node_data.get::<LoopingOption>() {

            match *looping_option {
                true => {
                    check_msg(msg.reply(&ctx.http, "Song is on loop!").await);
                }
                false => {
                    check_msg(msg.reply(&ctx.http, "Song is not on loop!").await);
                }
            }
        }
        else {
            check_msg(msg.reply(&ctx.http, "Song is not on loop!").await);
        }
    }
    else {
        check_msg(msg.reply(&ctx.http, "An error has occurred, please try again.").await);
    }

    Ok(())
}