use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group, hook},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, id::GuildId, misc::Mentionable},
    Result as SerenityResult,
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;
use songbird::SerenityInit;

use tracing::{debug, error, info};

use crate::lavalink::Lavalink;
use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;



#[group]
#[commands(join, leave, play, now_playing, skip)]
struct VoiceLavalink;


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

    if let Some(track) = lava_client.skip(guild_id).await {
        check_msg(msg.reply(&ctx.http,
                            format!("Skipped: {}", track.track.info.as_ref().unwrap().title)).await);
    }
    /*else {
        if let Some(node) = lava_client.nodes().await.get(&guild_id.0) {
            if let Some(track) = &node.now_playing {
                check_msg(msg.reply(&ctx.http,
                                    format!("Skipped: {}", track.track.info.as_ref().unwrap().title)).await);
                /*if let Err(why) = lava_client.stop(guild_id).await {
                    error!("Stopping audio failed: {:?}", why);
                    check_msg(msg.reply(&ctx.http,
                                        "Skipping failed, please try again.").await);
                }
            }
        }
    }*/

    Ok(())
}
