use teloxide::prelude::*;

pub fn create_bot() -> Bot {
    Bot::from_env()
}

pub async fn notify<'a>(opts: &NotifyOpts<'a>) -> anyhow::Result<()> {
    let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set");
    let result = opts.bot.send_message(telegram_chat_id, &opts.message).await;

    if let Err(e) = result {
        eprintln!("Failed to send message to Telegram: {}", e);
        return Err(e.into());
    }

    Ok(())
}

#[derive(Debug)]
pub struct NotifyOpts<'a> {
    pub message: String,
    pub bot: &'a Bot,
}
