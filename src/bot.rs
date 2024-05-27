use teloxide::{prelude::*, RequestError};

pub fn create_bot() -> Bot {
    Bot::from_env()
}

pub async fn notify<'a>(opts: &NotifyOpts<'a>) -> anyhow::Result<Message, RequestError> {
    let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set");

    opts.bot.send_message(telegram_chat_id, &opts.message).await
}

#[derive(Debug)]
pub struct NotifyOpts<'a> {
    pub message: String,
    pub bot: &'a Bot,
}
