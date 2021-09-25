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
#[name = "user_in_voice_with_bot_or_bot_not_in_any_voice"]
pub async fn user_in_voice_with_bot_or_bot_not_in_any_voice_check(
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

    if let Some(bot_channel_id) = handler.current_channel() {
        match guild
            .voice_states.get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id) {

            Some(author_channel_id) => {
                if bot_channel_id.0 != author_channel_id.0 {
                    return Err(Reason::User("Checking InVoiceWithBot failed, user not in the same voice channel with bot.".to_string()));
                }
            }
            None => {
                return Err(Reason::User("Checking InVoiceWithBot failed, user is not in a voice channel.".to_string()));
            }
        }
    }

    Ok(())
}