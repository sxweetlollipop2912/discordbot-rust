use serenity::{
    model::channel::Message,
    Result as SerenityResult,
};

use tracing::{debug, error, info};


/// Checks that a message successfully sent; if not, then logs why.
pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}