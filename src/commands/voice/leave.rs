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
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>()
                .expect("Unable to retrieve Lavalink client from data.").clone();

            if let Err(why) = lava_client.destroy(guild_id).await {
                error!("Destroying audio failed: {:?}", why);
                check_msg(msg.reply(&ctx.http,
                                    "Leaving failed, please try again.").await);
                return Ok(());
            }
        }

        if let Err(why) = manager.remove(guild_id).await {
            check_msg(msg.reply(&ctx.http, "Leaving failed, please try again.").await);
            error!("Leaving voice channel failed: {:?}", why);
        }
    } else {
            check_msg(msg.reply(ctx, "I'm not in a voice channel!").await);
    }

    Ok(())
}