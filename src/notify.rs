use teloxide::{requests::Requester, Bot};

pub async fn notify<'a>(opts: &NotifyOpts<'a>) -> anyhow::Result<()> {
    let bot = opts.bot;
    let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set");
    let result = bot.send_message(telegram_chat_id, &opts.message).await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to send message to Telegram: {}", e);
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct NotifyOpts<'a> {
    pub message: String,
    pub bot: &'a Bot,
}
