use serenity::{
    framework::standard::{
        macros::group,
    },
};


pub mod help;
pub use help::*;

pub mod checks;
pub use checks::*;


pub mod general;
pub use general::*;
#[group]
#[description = "A group of general commands."]
#[summary = "Trivial commands."]
//#[commands(about, am_i_admin, repeat, ping, some_long_command, upper_command)]
#[commands(about, repeat, ping)]
struct General;


pub mod voice;
pub use voice::*;
#[group]
#[description = "A group of audio streaming commands."]
#[summary = "🎵!"]
#[commands(join, leave, play, now_playing, skip, loop_song, unloop_song, is_looping, queue, mute, unmute, deafen, undeafen)]
struct Voice;


pub mod legacy_voice;
pub use legacy_voice::*;
#[group]
#[commands(legacy_join, legacy_leave, legacy_play, legacy_deafen, legacy_undeafen, legacy_mute, legacy_unmute)]
struct LegacyVoice;


pub mod emoji;
pub use emoji::*;
#[group]
#[prefixes("emoji", "em")]
#[description = "A group of commands providing an emoji as response."]
#[summary = "Do emoji fun!"]
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;


pub mod server_mod;
pub use server_mod::*;
#[group]
#[only_in(guilds)]
#[description = "A group of commands for server mods."]
#[summary = "Commands for server mods"]
#[prefixes("mod")]
#[commands(slow_mode)]
struct ServerMod;