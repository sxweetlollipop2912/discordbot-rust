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
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Hi, I am a multipurpose bot! : )").await);
    Ok(())
}