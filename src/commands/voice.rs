/// Music streaming utilizing Lavalink.

pub mod join;
pub use join::*;

pub mod leave;
pub use leave::*;

pub mod play;
pub use play::*;

pub mod skip;
pub use skip::*;

#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "I'm not in a voice channel!").await);

pub mod now_playing;
pub use now_playing::*;

pub mod queue;
pub use queue::*;

pub mod mute;
pub use mute::*;

pub mod unmute;
pub use unmute::*;

pub mod deafen;
pub use deafen::*;

pub mod undeafen;
pub use undeafen::*;

pub mod wrapper;
pub use wrapper::*;