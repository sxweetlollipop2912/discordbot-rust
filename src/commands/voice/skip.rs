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
use crate::commands::wrapper::create_session_with_songbird_wrapper;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>()
        .expect("Unable to retrieve Lavalink client from data.").clone();

    let mut track_title = String::new();
    let mut queue_size = 0;

    // We must seperate the use of `node` from `lava_client.skip()` and `lava_client.destroy()` functions
    // because both use lava_client, therefore doing otherwise would block.
    if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {
        queue_size = node.queue.len();
        if !node.queue.is_empty() {
            track_title = node.queue[0].track.info.as_ref().unwrap().title.clone();
        }
    }

    if queue_size == 0 {
        check_msg(msg.channel_id.say(&ctx.http, "Nothing to skip.").await);
    }

    // Only use `skip` if there are more than 1 song in queue.
    else if queue_size > 1 {
        if let Some(_track) = lava_client.skip(guild_id).await {
            check_msg(msg.reply(&ctx.http,
                                format!("Skipped: {}", track_title)).await);
        }
        else {
            error!("Skipping audio failed!");
            check_msg(msg.reply(&ctx.http,
                                "Skipping failed, please try again.").await);
        }
    }

    // If there is only 1 song in queue, use `destroy` instead, then `create_session` again.
    // We cannot use `stop`, as it would not remove the song from queue.
    // Perhaps we can use `stop` and manually remove the song, as running `create_session` is more expensive?
    else if queue_size == 1 {
        if let Err(why) = lava_client.destroy(guild_id).await {
            error!("Destroying audio failed: {:?}", why);
            check_msg(msg.reply(&ctx.http,
                                "Skipping failed, please try again.").await);
        }
        if let Err(why) = create_session_with_songbird_wrapper(ctx, msg, &lava_client).await {
            error!("Creating session with songbird failed: {:?}", why);
            check_msg(msg.reply(&ctx.http,
                                "An error has occurred, please have me rejoin the channel.").await);
        }

        check_msg(msg.reply(&ctx.http,
                            format!("Skipped: {}", track_title)).await);
    }

    Ok(())
}