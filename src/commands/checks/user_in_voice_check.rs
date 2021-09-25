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
#[name = "user_in_voice"]
pub async fn user_in_voice_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    match guild.voice_states.get(&msg.author.id) {
        None => {
            return Err(Reason::User("Checking InVoiceWithBot failed, user not in voice channel.".to_string()));
        }
        _ => {
            Ok(())
        }
    }
}