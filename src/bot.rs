use teloxide::prelude::*;

pub fn create_bot() -> Bot {
    let bot: Bot = Bot::from_env();

    bot
}
