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
#[aliases("kitty", "neko")]
#[bucket = "emoji"]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, ":cat:").await);
    // We can return one ticket to the bucket undoing the ratelimit.
    // Err(RevertBucket.into())
    Ok(())
}