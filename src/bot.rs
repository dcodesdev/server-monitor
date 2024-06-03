use teloxide::{prelude::*, RequestError};

pub const TELEGRAM_MAX_MESSAGE_LENGTH: usize = 4096;

pub fn create_bot() -> Bot {
    Bot::from_env()
}

pub async fn notify<'a>(opts: &NotifyOpts<'a>) -> anyhow::Result<Vec<Message>, RequestError> {
    let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID must be set");

    // Telegram only allows messages up to 4096 characters
    let messages = split_into_chunks(&opts.message, TELEGRAM_MAX_MESSAGE_LENGTH);

    let mut results = Vec::new();
    for message in messages.into_iter() {
        let telegram_chat_id = telegram_chat_id.clone();
        let result = opts.bot.send_message(telegram_chat_id, &message).await?;
        results.push(result);
    }

    Ok(results)
}

#[derive(Debug)]
pub struct NotifyOpts<'a> {
    pub message: String,
    pub bot: &'a Bot,
}

fn split_into_chunks(message: &str, chunk_size: usize) -> Vec<String> {
    message
        .chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn generate_random_text(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 ";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    #[test]
    fn test_split_into_chunks() {
        assert_eq!(
            split_into_chunks("Hello, World!", 1),
            vec!["H", "e", "l", "l", "o", ",", " ", "W", "o", "r", "l", "d", "!"]
        );

        assert_eq!(split_into_chunks(&generate_random_text(1000), 500).len(), 2);

        assert_eq!(
            split_into_chunks(&generate_random_text(10000), 4096).len(),
            3
        );

        assert_eq!(
            split_into_chunks(&generate_random_text(100000), 4096).len(),
            25
        );
    }
}
