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
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions},
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Hi, I am a multipurpose bot! : )").await);
    Ok(())
}


#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                check_msg(msg.channel_id.say(&ctx.http, "Yes, you are.").await);
                return Ok(());
            }
        }
    }
    check_msg(msg.channel_id.say(&ctx.http, "No, you are not.").await);
    Ok(())
}


// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
#[command]
async fn repeat(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default().clean_channel(false).clean_role(false)
    };
    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;
    check_msg(msg.channel_id.say(&ctx.http, &content).await);
    Ok(())
}


#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong! :)").await);
    Ok(())
}


#[command]
async fn some_long_command(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, &format!("Arguments: {:?}", args.rest())).await);
    Ok(())
}


// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command("upper")]
#[sub_commands(sub)]
async fn upper_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    check_msg(msg.reply(&ctx.http, "This is the main function!").await);
    Ok(())
}


// This will only be called if preceded by the `upper`-command.
#[command]
#[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn sub(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    check_msg(msg.reply(&ctx.http, "This is a sub function!").await);
    Ok(())
}