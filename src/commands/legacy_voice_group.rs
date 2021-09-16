/// Streaming music depending solely on Songbird (other words, no Lavalink).
use serenity::{
    client::Context,
    framework::standard::{
        Args, CommandResult,
        macros::{command, group},
    },
    model::channel::Message,
};

use url::Url;

use tracing::{debug, error, info};

use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;


#[group]
#[prefixes("voice")]
#[commands(join, leave, play, deafen, undeafen, mute, unmute)]
struct LegacyVoice;


#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx.http, "U are not in a voice channel!").await);
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let (_, handler) = manager.join(guild_id, connect_to).await;

    match handler {
        Err(why) => {
            error!("Joining voice channel failed: {:?}", why);
            check_msg(msg.reply(&ctx.http, "Joining voice channel failed, please try again.").await);
        }
        _ => {}
    }

    Ok(())
}


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
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.reply(&ctx.http, "Leaving failed, please try again.").await);
            error!("Leaving voice channel failed: {:?}", e);
        }
    } else {
        check_msg(msg.reply(ctx, "I'm not in a voice channel!").await);
    }

    Ok(())
}


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot_or_bot_not_in_any_voice)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.reply(&ctx.http, "Must provide a URL to a video or audio").await);

            return Ok(());
        },
    };

    if let Err(_e) = Url::parse(&url) {
        check_msg(msg.reply(&ctx.http, "Are u sure this is a valid URL?").await);

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        // Attempt to join channel if not already in one.
        None => {
            let channel_id = guild
                .voice_states.get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);

            let connect_to = match channel_id {
                Some(channel) => channel,
                None => {
                    check_msg(msg.reply(&ctx.http, "U are not in a voice channel!").await);
                    return Ok(());
                }
            };

            let _handler = manager.join(guild_id, connect_to).await;
            manager.get(guild_id).unwrap()
        },
    };

    let mut handler = handler_lock.lock().await;

    let source = match songbird::ytdl(&url).await {
        Ok(source) => source,
        Err(why) => {
            error!("Sourcing FFmpeg failed: {:?}", why);
            check_msg(msg.reply(&ctx.http, "Attempt to play audio failed. Please try again.").await);

            return Ok(());
        },
    };

    handler.play_source(source);

    check_msg(msg.reply(&ctx.http, "Playing!").await);

    Ok(())
}


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
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

    if !handler.is_mute() {
        if let Err(e) = handler.mute(true).await {
            check_msg(msg.reply(&ctx.http, "Muting failed, please try again.").await);
            error!("Muting voice failed: {:?}", e);
        }
    }

    Ok(())
}


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
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

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if !handler.is_deaf() {
        if let Err(e) = handler.deafen(true).await {
            check_msg(msg.reply(&ctx.http, "Failed to deafen voice, please try again.").await);
            error!("Deafening voice failed: {:?}", e);
        }
    }

    Ok(())
}


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
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

    if handler.is_deaf() {
        if let Err(e) = handler.deafen(false).await {
            check_msg(msg.reply(&ctx.http, "Undeafening failed, please try again.").await);
            error!("Undeafening failed: {:?}", e);
        }
    }

    Ok(())
}