mod bot;
mod db;
mod status;

use bot::create_bot;
use db::{url::Url, Db};
use status::{check_url_status, create_server_update_cron};
use std::sync::Arc;

const DEFAULT_INTERVAL: u64 = 1000 * 60; // 1 minute
const UPDATE_INTERVAL: u64 = 1000 * 60 * 60 * 24; // 24 hours

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let urls: Vec<Url> = std::env::var("URLS")
        .expect("URLS must be set")
        .split(',')
        .map(|item| Url::from(item.to_string()))
        .collect();

    let interval: u64 = std::env::var("INTERVAL")
        .unwrap_or_else(|_| DEFAULT_INTERVAL.to_string())
        .parse()
        .expect("INTERVAL must be a number");

    let bot = Arc::new(create_bot());
    let db = Arc::new(Db::new(&urls).await?);

    println!("\nServer monitor is running with the following settings:");
    println!("\n- Interval: {}ms", interval);
    println!("- URLs:");
    urls.iter().for_each(|url| {
        println!("  - {}", url);
    });

    create_server_update_cron(Arc::clone(&db), Arc::clone(&bot)).await?;

    loop {
        urls.iter().for_each(|url| {
            let url = url.clone(); // TODO: use Arc::clone instead
            let bot = Arc::clone(&bot);
            let db = Arc::clone(&db);

            tokio::spawn(async move {
                let result = check_url_status(&url, &bot, &db).await;

                if let Err(e) = result {
                    eprintln!("Error: {}", e);
                }
            });
        });

        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    }
}
