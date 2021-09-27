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
        permissions::Permissions,
    },
};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


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