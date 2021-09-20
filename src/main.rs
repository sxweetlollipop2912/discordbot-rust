mod system_fn;
mod commands;
mod conf_constants;
mod wrapper;
mod lavalink;

use std::{
    env,
    collections::HashSet,
    sync::Arc,
    process::Command,
    thread,
    time::Duration,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::{
        bridge::gateway::{
            GatewayIntents,
            ShardManager,
        },
        Client,
        EventHandler,
    },
    framework::standard::{
        buckets::LimitedFor,
        StandardFramework,
    },
    http::Http,
    model::{
        gateway::Ready,
        event::ResumedEvent,
        id::GuildId,
    },
};

// This trait adds the `register_songbird` and `register_songbird_with` methods
// to the client builder below, making it easy to install this voice client.
// The voice client can be retrieved in any command using `songbird::get(ctx).await`.
use songbird::SerenityInit;

use lavalink_rs::LavalinkClient;

use tracing::{debug, error, info, instrument, Level};

use crate::system_fn::unknown_command;
use crate::system_fn::dispatch_error;
use crate::system_fn::delay_action;
use crate::system_fn::before;
use crate::system_fn::after;
use crate::lavalink::Lavalink;
use crate::lavalink::LavalinkHandler;
use crate::conf_constants::BOT_TOKEN;
use crate::conf_constants::LAVALINK_PASS;
use crate::conf_constants::LAVALINK_PORT;
use crate::commands::help::GENERAL_HELP;
use crate::commands::general_group::GENERAL_GROUP;
use crate::commands::server_mod_group::SERVERMOD_GROUP;
use crate::commands::emoji_group::EMOJI_GROUP;
use crate::commands::legacy_voice_group::LEGACYVOICE_GROUP;
use crate::commands::voice_group::VOICE_GROUP;


pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /*async fn message(&self, ctx: Context, msg: Message) {
    }*/

    async fn cache_ready(&self, _: Context, guilds: Vec<GuildId>) {
        info!("cache is ready!\n{:#?}", guilds);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        // Log at the INFO level. This is a macro from the `tracing` crate.
        println!("{} is connected!", ready.user.name);
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

    // Setting up Lavalink server
    env::set_var("PASS", LAVALINK_PASS);

    // Running Lavalink server
    let mut lava_server_handle = Command::new("node")
                .current_dir("./lavalink_server/")
                .arg("bootstrap.js")
                .spawn()
                .expect("Running Lavalink server failed.");
    thread::sleep(Duration::from_secs(90));

    // Token is stored in ./src/conf_constants.rs, under `BOT_TOKEN` constant string
    let token = BOT_TOKEN;
    let http = Http::new_with_token(&token);

    let (bot_owners, bot_id) = match http.get_current_application_info().await {
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
            .owners(bot_owners))
            // Set a function to be called prior to each command execution. This
            // provides the context of the command, the message that was received,
            // and the full name of the command that will be called.
            //
            // Avoid using this to determine whether a specific command should be
            // executed. Instead, prefer using the `#[check]` macro which
            // gives you this functionality.
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
            //.group(&EMOJI_GROUP)
            //.group(&SERVERMOD_GROUP)
            //.group(&LEGACYVOICE_GROUP)
            .group(&VOICE_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        // For this to run properly, the "Presence Intent" and "Server Members Intent"
        // options need to be enabled.
        // These are needed so the `required_permissions` macro works on the commands that need to
        // use it.
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client.");

    let lava_client = LavalinkClient::builder(bot_id)
        .set_port(env::var("PORT").unwrap_or_else(|_| LAVALINK_PORT.to_string()).parse::<u16>().unwrap())
        .set_password(env::var("PASS").unwrap_or_else(|_| LAVALINK_PASS.to_string()))
        .build(LavalinkHandler)
        .await
        .expect("Error creating Lavalink client.");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<Lavalink>(lava_client);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        info!("Received Ctrl-C, shutting down.");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    // Shut down Lavalink server
    lava_server_handle.kill().expect("Shutting down Lavalink server failed.");
}