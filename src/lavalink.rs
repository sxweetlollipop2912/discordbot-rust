use std::env;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group, hook},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, id::GuildId, misc::Mentionable},
    Result as SerenityResult,
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;
use songbird::SerenityInit;

use tracing::{debug, error, info};

use crate::wrapper::check_msg;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_CHECK;
use crate::commands::checks::USER_IN_VOICE_WITH_BOT_OR_BOT_NOT_IN_ANY_VOICE_CHECK;


pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}


pub struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }

    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finished!\nGuild: {}", event.guild_id);
    }
}