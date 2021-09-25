use serenity::{
    client::Context,
    framework::standard::{
        CommandResult,
        macros::command,
    },
    model::channel::Message,
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn legacy_unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "I'm not in a voice channel!").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        if let Err(e) = handler.mute(false).await {
            check_msg(msg.reply(&ctx.http, "Unmuting failed, please try again.").await);
            error!("Unmuting failed: {:?}", e);
        }
    }

    Ok(())
}