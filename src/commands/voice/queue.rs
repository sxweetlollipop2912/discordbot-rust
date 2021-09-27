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
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>()
        .expect("Unable to retrieve Lavalink client from data.").clone();

    if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {
        check_msg(msg.reply(&ctx.http,
                            format!("There are **{}** songs currently in queue.", node.queue.len())).await);
    }
    Ok(())
}