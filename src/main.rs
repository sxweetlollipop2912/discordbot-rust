mod help;
mod edge_cases;
mod wrapper;
mod general_group;

use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents, ShardId, ShardManager},
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        DispatchError,
        HelpOptions,
        Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions},
};

use crate::edge_cases::unknown_command;
use crate::edge_cases::dispatch_error;
use crate::edge_cases::delay_action;
use crate::help::GENERAL_HELP;
use crate::general_group::GENERAL_GROUP;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    let token = "ODg0OTk4MzM2NjkwNjU5MzQ4.YTgo7Q.CwYmpoPyoy7tyKoOXSc_o44-JgE";

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("^")
            // Sets the bot's owners. These will be used for commands that
            // are owners only.
            .owners(owners))
            // Set a function that's called whenever an attempted command-call's
            // command could not be found.
            .unrecognised_command(unknown_command)
            // Set a function that's called whenever a command's execution didn't complete for one
            // reason or another. For example, when a user has exceeded a rate-limit or a command
            // can only be performed by the bot owner.
            .on_dispatch_error(dispatch_error)
            // Can't be used more than once per 5 seconds:
            .bucket("emoji", |b| b.delay(5)
            // The target each bucket will apply to.
            .limit_for(LimitedFor::Channel)
            // The maximum amount of command invocations that can be delayed per target.
            // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
            .await_ratelimits(1)
            // A function to call when a rate limit leads to a delay.
            .delay_action(delay_action)).await
            // The `#[group]` macro generates `static` instances of the options set for the group.
            // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
            // #name is turned all uppercase
            .help(&GENERAL_HELP)
            .group(&GENERAL_GROUP);
            /*.group(&EMOJI_GROUP)
            .group(&MATH_GROUP)
            .group(&OWNER_GROUP);*/

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        // For this example to run properly, the "Presence Intent" and "Server Members Intent"
        // options need to be enabled.
        // These are needed so the `required_permissions` macro works on the commands that need to
        // use it.
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client.");

    /*{
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }*/

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}