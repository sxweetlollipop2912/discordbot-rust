use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::check,
        Args,
        CommandOptions,
        Reason,
    },
    model::channel::Message,
};


#[check]
#[name = "bot_not_in_voice"]
pub async fn bot_not_in_voice_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Checking InVoiceWithBot failed, Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            return Ok(());
        },
    };

    let handler = handler_lock.lock().await;

    return match handler.current_channel() {
        None =>
            Ok(()),
        _ =>
            Err(Reason::User("Checking BotNotInVoice failed, bot is in a voice channel.".to_string())),
    }
}