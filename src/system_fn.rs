use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::hook,
        CommandResult,
        DispatchError,
    },
    model::{
        channel::Message,
    },
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[hook]
// instrument will show additional information on all the logs that happen inside the function.
// This additional information includes the function name, along with all it's arguments
// formatted with the Debug impl.
// This additional information will also only be shown if the LOG level is set to `debug`
#[instrument]
pub async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    debug!("Got command '{}' by user '{}'", command_name, msg.author.name);
    true
}


#[hook]
#[instrument]
pub async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => info!("Processed command '{}'.", command_name),
        Err(why) => error!("Command '{}' returned error {:?}", command_name, why),
    }
}


#[hook]
pub async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
    check_msg(msg.reply(ctx, &format!("Could not find command named '{}'", unknown_command_name)).await);
}


#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            check_msg(msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs())).await);
        }
    }
}


#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}



