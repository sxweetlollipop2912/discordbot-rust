use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::channel::Message,
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[command]
#[only_in(guilds)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong! :)").await);
    Ok(())
}