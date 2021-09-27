use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[command]
async fn some_long_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, &format!("Arguments: {:?}", args.rest())).await);
    Ok(())
}