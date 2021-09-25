use serenity::{
    client::Context,
    framework::{
        standard::{
            macros::command,
            Args,
            CommandResult,
        },
    },
    model::channel::Message,
};

use tracing::{debug, error, info};

use crate::lavalink::Lavalink;
use crate::wrapper::check_msg;
use crate::commands::wrapper::create_session_with_songbird_wrapper;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;


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