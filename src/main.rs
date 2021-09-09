mod system_fn;
mod commands;

use std::{
    collections::HashSet,
    env,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents},
    framework::standard::{
        buckets::LimitedFor,
        StandardFramework,
    },
    http::Http,
    model::{
        gateway::Ready,
        event::ResumedEvent,
    },
};

use tracing::{debug, error, info, instrument, Level};

use crate::system_fn::unknown_command;
use crate::system_fn::dispatch_error;
use crate::system_fn::delay_action;
use crate::system_fn::before;
use crate::system_fn::after;
use crate::commands::help::GENERAL_HELP;
use crate::commands::general_group::GENERAL_GROUP;
use crate::commands::owner_group::OWNER_GROUP;
use crate::commands::emoji_group::EMOJI_GROUP;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /*async fn message(&self, ctx: Context, msg: Message) {
    }*/

    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        info!("{} is connected!", ready.user.name);
    }

    // For instrument to work, all parameters must implement Debug.
    //
    // Handler doesn't implement Debug here, so we specify to skip that argument.
    // Context doesn't implement Debug either, so it is also skipped.
    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        // Log at the DEBUG level.
        //
        // In this example, this will not show up in the logs because DEBUG is
        // below INFO, which is the set debug level.
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}


#[tokio::main]
#[instrument(name = "main")]
async fn main() {
    // Create a non-blocking rolling file-appender.
    let file_appender = tracing_appender::rolling::daily("./logs", "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // Call tracing_subscriber's initialize function, which configures `tracing` via environment variables.
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::INFO)
        .init();

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
            // Set a function to be called prior to each command execution. This
            // provides the context of the command, the message that was received,
            // and the full name of the command that will be called.
            //
            // Avoid using this to determine whether a specific command should be
            // executed. Instead, prefer using the `#[check]` macro which
            // gives you this functionality.
            //
            // **Note**: Async closures are unstable, you may use them in your
            // application if you are fine using nightly Rust.
            // If not, we need to provide the function identifiers to the
            // hook-functions (before, after, normal, ...).
            .before(before)
            // Similar to `before`, except will be called directly _after_
            // command execution.
            .after(after)
            // Set a function that's called whenever an attempted command-call's
            // command could not be found.
            .unrecognised_command(unknown_command)
            // Set a function that's called whenever a command's execution didn't complete for one
            // reason or another. For example, when a user has exceeded a rate-limit or a command
            // can only be performed by the bot owner.
            .on_dispatch_error(dispatch_error)
            // Can't be used more than once per 3 seconds:
            .bucket("emoji", |b| b.delay(3)
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
            .group(&GENERAL_GROUP)
            .group(&EMOJI_GROUP)
            .group(&OWNER_GROUP);

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
        error!("Client error: {:?}", why);
    }
}