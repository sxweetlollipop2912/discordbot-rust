use serenity::async_trait;

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::prelude::*;

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


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