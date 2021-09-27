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

use crate::commands::SUB_COMMAND;
use crate::wrapper::check_msg;


// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command("upper")]
#[sub_commands(sub)]
async fn upper_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    check_msg(msg.reply(&ctx.http, "This is the main function!").await);
    Ok(())
}