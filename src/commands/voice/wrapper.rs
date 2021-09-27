use serenity::{
    client::Context,
    model::channel::Message,
};

use lavalink_rs::{
    error::LavalinkResult,
    LavalinkClient,
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


pub async fn create_session_with_songbird_wrapper(
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