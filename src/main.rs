mod bot;
mod constants;
mod db;
mod status;

use bot::create_bot;
use constants::get_interval;
use db::{url::Url, Db};
use status::{check_url_status, create_server_update_cron};
use std::{sync::Arc, time::Duration};
use teloxide::Bot;

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

    let interval = get_interval();
    let bot = Arc::new(create_bot());
    let db = Arc::new(Db::new(&urls).await?);

    println!("\nServer monitor is running with the following settings:");
    println!("\n- Interval: {}ms", interval);
    println!("- URLs:");
    urls.iter().for_each(|url| {
        println!("  - {}", url);
    });

    create_server_update_cron(Arc::clone(&db), Arc::clone(&bot)).await?;

    let mut handles = Vec::new();
    urls.iter().for_each(|url| {
        let bot = Arc::clone(&bot);
        let db = Arc::clone(&db);
        let url = url.clone();

        let handle = tokio::spawn(async move {
            create_url_check_cron(&url, bot, db).await.unwrap();
        });

        handles.push(handle);
    });

    futures::future::join_all(handles).await;

    Ok(())
}

async fn create_url_check_cron(url: &Url, bot: Arc<Bot>, db: Arc<Db>) -> anyhow::Result<()> {
    let interval = get_interval();

    loop {
        let result = check_url_status(&url, &bot, &db).await;

        if let Err(e) = result {
            let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            eprintln!("{time} Error: {}", e);
        }

        tokio::time::sleep(Duration::from_millis(interval)).await;
    }
}
