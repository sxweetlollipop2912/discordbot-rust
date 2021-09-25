use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::command,
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