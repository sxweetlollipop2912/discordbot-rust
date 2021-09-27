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


// This will only be called if preceded by the `upper`-command.
#[command]
#[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn sub(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    check_msg(msg.reply(&ctx.http, "This is a sub function!").await);
    Ok(())
}