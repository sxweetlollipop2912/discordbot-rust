use serenity::{
    async_trait,
    prelude::*,
};

use lavalink_rs::{gateway::*, model::*, LavalinkClient};

use tracing::{debug, error, info};

use crate::wrapper::check_msg;


pub struct Lavalink;

pub struct LavalinkHandler;

pub struct LoopingOption;

pub struct Track;


impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}

impl TypeMapKey for LoopingOption {
    type Value = bool;
}

impl TypeMapKey for Track {
    type Value = TrackQueue;
}


#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started in Guild: {}", event.guild_id);
    }


    async fn track_finish(&self, lava_client: LavalinkClient, event: TrackFinish) {
        info!("Track finished in Guild: {}", event.guild_id);

        let mut is_looping: bool = false;
        let mut looped_track: Option<TrackQueue> = None;

        // Get `is_looping` and `looped_track` info
        if let Some(node) = lava_client.nodes().await.get(&event.guild_id.0) {
            let node_data = node.data.read().await;

            if let Some(looping_option) = node_data.get::<LoopingOption>() {
                is_looping = *looping_option;
            }
            if let Some(track) = node_data.get::<Track>() {
                looped_track = Some(track.clone());
            }
        };

        let lava_client_lock = lava_client.inner.lock().await;

        match is_looping {
            true => {
                if let Some(mut node) = lava_client_lock.nodes.get_mut(&event.guild_id.0) {
                    if let Some(looped_track) = looped_track {
                        node.queue.insert(0, looped_track.clone());

                        info!("Track looped in Guild: {}", event.guild_id);
                    }
                    else {
                        error!("Looping track in Guild: {} failed!", event.guild_id);
                    }
                };
            }

            false => {
                if let Some(node) = lava_client_lock.nodes.get(&event.guild_id.0) {
                    let mut node_data = node.data.write().await;

                    if node_data.remove::<Track>().is_none() {
                        error!("Removing Track from node.data in Guild: {} failed!", event.guild_id);
                    }
                }
            }
        }
    }
}