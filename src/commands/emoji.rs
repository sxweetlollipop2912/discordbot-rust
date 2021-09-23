use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::{command, group},
        Args,
        CommandResult,
    },
    model::{
        channel::Message,
    },
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[command]
async fn bird(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        ":bird: can find animals for you.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };
    check_msg(msg.channel_id.say(&ctx.http, say_content).await);
    Ok(())
}


#[command]
// Adds multiple aliases
#[aliases("kitty", "neko")]
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, ":cat:").await);
    // We can return one ticket to the bucket undoing the ratelimit.
    // Err(RevertBucket.into())
    Ok(())
}


#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, ":dog:").await);
    Ok(())
}