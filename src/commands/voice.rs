/// Music streaming utilizing Lavalink.
use serenity::{
    client::Context,
    framework::{
        standard::{
            macros::{
                command,
                group,
            },
            Args,
            CommandResult,
        },
    },
    model::channel::Message,
};

use lavalink_rs::{
    error::LavalinkResult,
    LavalinkClient,
};

use tracing::{debug, error, info};

use crate::lavalink::Lavalink;
use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;


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

    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            let data = ctx.data.read().await;
            let lava_client = data.get::<Lavalink>()
                .expect("Unable to retrieve Lavalink client from data.").clone();
            lava_client.create_session_with_songbird(&connection_info).await?;
        }
        Err(why) => {
            error!("Joining voice channel failed: {:?}", why);
            check_msg(msg.reply(&ctx.http, "Joining voice channel failed, please try again.").await);
        }
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


#[command]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot_or_bot_not_in_any_voice)]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let lava_client = {
        let data = ctx.data.read().await;
        data.get::<Lavalink>()
            .expect("Unable to retrieve Lavalink client from data.").clone()
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    // Attempt to join the user's current voice channel if not already in a channel.
    if manager.get(guild_id).is_none() {
        if let Err(why) = create_session_with_songbird_wrapper(ctx, msg, &lava_client).await {
            error!("Creating session failed: {:?}", why);
            check_msg(
                msg.reply(&ctx, "Joining voice channel failed, please try again.").await);
            return Ok(());
        }
    }

    if let Some(_handler) = manager.get(guild_id) {
        let query_information = lava_client.auto_search_tracks(&query).await?;

        if query_information.tracks.is_empty() {
            check_msg(
                msg.reply(&ctx, "Could not find any result of the search query.").await);

            return Ok(());
        }

        if let Err(why) = &lava_client
            .play(guild_id, query_information.tracks[0].clone())
            .queue()
            .await
        {
            error!("Queuing audio failed: {:?}", why);

            return Ok(());
        };
        info!("Added to queue: {}, guild: {}", query_information.tracks[0].info.as_ref().unwrap().title, guild_id.0);
        check_msg(msg.reply(&ctx.http, format!(
                        "Added to queue: {}",
                        query_information.tracks[0].info.as_ref().unwrap().title)).await);
    } else {
        check_msg(
            msg.reply(&ctx, "U are not in a voice channel!").await);
    }

    Ok(())
}

#[command]
#[aliases(np)]
#[only_in(guilds)]
#[checks(user_in_voice_with_bot)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let data = ctx.data.read().await;
    let lava_client = data.get::<Lavalink>()
                .expect("Unable to retrieve Lavalink client from data.").clone();

    if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {

        if let Some(track) = &node.now_playing {
            check_msg(msg.reply(&ctx.http,
                        format!("Now Playing: {}", track.track.info.as_ref().unwrap().title)).await);
        } else {
            check_msg(msg.reply(&ctx.http, "Nothing is playing at the moment.").await);
        }

    } else {
        check_msg(msg.reply(&ctx.http, "Nothing is playing at the moment.").await);
    }

    Ok(())
}


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
        if let Err(why) = handler.mute(true).await {
            check_msg(msg.reply(&ctx.http, "Muting failed, please try again.").await);
            error!("Muting voice failed: {:?}", why);
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
        if let Err(why) = handler.mute(false).await {
            check_msg(msg.reply(&ctx.http, "Unmuting failed, please try again.").await);
            error!("Unmuting failed: {:?}", why);
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
        if let Err(why) = handler.deafen(true).await {
            check_msg(msg.reply(&ctx.http, "Failed to deafen voice, please try again.").await);
            error!("Deafening voice failed: {:?}", why);
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
        if let Err(why) = handler.deafen(false).await {
            check_msg(msg.reply(&ctx.http, "Undeafening failed, please try again.").await);
            error!("Undeafening failed: {:?}", why);
        }
    }

    Ok(())
}


async fn create_session_with_songbird_wrapper(
    ctx: &Context,
    msg: &Message,
    lava_client: &LavalinkClient,
) -> LavalinkResult<()> {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;

    match handler {
        Ok(connection_info) => {
            lava_client.create_session_with_songbird(&connection_info).await?;
        }
        Err(why) => {
            error!("Joining gateway of voice channel failed: {:?}", why);
        }
    }

    Ok(())
}