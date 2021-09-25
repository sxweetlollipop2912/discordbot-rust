/// Streaming music depending solely on Songbird (other words, no Lavalink).

pub mod legacy_join;
pub use legacy_join::*;

pub mod legacy_leave;
pub use legacy_leave::*;

pub mod legacy_play;
pub use legacy_play::*;

pub mod legacy_mute;
pub use legacy_mute::*;

pub mod legacy_unmute;
pub use legacy_unmute::*;

pub mod legacy_deafen;
pub use legacy_deafen::*;

pub mod legacy_undeafen;
pub use legacy_undeafen::*;