use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::command,
        CommandResult,
    },
    model::{
        channel::Message,
    },
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, ":dog:").await);
    Ok(())
}