use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args,
        CommandResult,
    },
    model::{
        channel::{Channel, Message},
    },
};

use tracing::{debug, error, info};


#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Set a description to appear if a user wants to display a single group
// e.g. via help using the group-name or one of its prefixes.
#[description = "A group of commands for server owners."]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
// This requires us to call commands in this group
// via `~owner` instead of just `~`.
#[prefixes("owner")]
#[commands(slow_mode)]
struct Owner;


#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) =
        msg.channel_id.edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds)).await
        {
            error!("Failed to set slow mode to `{}` seconds: {:?}", slow_mode_rate_seconds, why);
            format!("Failed to set slow mode to `{}` seconds", slow_mode_rate_seconds)
        } else {
            debug!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds);
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }

    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache).await
    {
        format!("Current slow mode rate is `{}` seconds.", channel.slow_mode_rate.unwrap_or(0))

    } else {
        error!("Failed to find channel in cache.");
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;
    Ok(())
}