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



#[group]
#[commands(join, leave, play, now_playing, skip, queue)]
struct Voice;


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
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.reply(&ctx.http, "Leaving failed, please try again.").await);
            error!("Leaving voice channel failed: {:?}", e);

            {
                let data = ctx.data.read().await;
                let lava_client = data.get::<Lavalink>()
                    .expect("Unable to retrieve Lavalink client from data.").clone();
                lava_client.destroy(guild_id).await?;
            }
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
        check_msg(msg.reply(&ctx.http, format!(
                        "Added to queue: {}",
                        query_information.tracks[0].info.as_ref().unwrap().title)).await);
    } else {
        check_msg(msg.reply(&ctx.http,
                    "Use `^join` first.").await);
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
    // because both use lava_client, and doing otherwise would block.
    if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {
        queue_size = node.queue.len();
        if !node.queue.is_empty() {
            track_title = node.queue[0].track.info.as_ref().unwrap().title.clone();
        }
    }

    if queue_size == 0 {
        check_msg(msg.channel_id.say(&ctx.http, "Nothing to skip.").await);
        return Ok(());
    }

    // Only use `skip` if there are more than 1 song in queue.
    if queue_size > 1 {
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
    // We can't use `stop`, as it would not remove the song from queue.
    // Perhaps we can use `stop` and manually remove the song?
    else if queue_size == 1 {
        if let Err(why) = lava_client.destroy(guild_id).await {
            error!("Destroying audio failed: {:?}", why);
            check_msg(msg.reply(&ctx.http,
                                "Skipping failed, please try again.").await);
        }
        if let Err(why) = create_session_with_songbird_wrapper(ctx, msg, &lava_client).await {
            error!("Creating session with songbird failed: {:?}", why);
            check_msg(msg.reply(&ctx.http,
                                "Skipping failed, please try again.").await);
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