mod bot;
mod db;
mod request;
mod status;

use bot::create_bot;
use db::Db;
use futures::future;
use status::{check_url_status, server_update_cron};
use std::sync::Arc;
use tokio::sync::Mutex;

/// In ms
const DEFAULT_INTERVAL: u64 = 1000 * 60; // 1 minute
const UPDATE_INTERVAL: u64 = 1000 * 10;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let urls = std::env::var("URLS")
        .expect("URLS must be set")
        .split(',')
        .map(|item| item.to_string())
        .collect::<Vec<String>>();

    let interval = std::env::var("INTERVAL")
        .unwrap_or(DEFAULT_INTERVAL.to_string())
        .parse::<u64>()
        .expect("INTERVAL must be a number");

    let bot = Arc::new(create_bot());
    let db = Arc::new(Mutex::new(Db::new()));

    println!("Server monitor is running with the following settings:");
    println!("\n- Interval: {}ms", interval);

    server_update_cron(&db, UPDATE_INTERVAL, &bot);

    loop {
        let mut handles = Vec::new();

        for url in &urls {
            let url = url.clone();
            let bot = bot.clone();
            let db = db.clone();
            let handle = tokio::spawn(async move { check_url_status(&url, &bot, &db).await });
            handles.push(handle);
        }

        let results: Vec<Result<(), anyhow::Error>> = future::join_all(handles)
            .await
            .into_iter()
            .map(|e| e.unwrap())
            .collect();

        for result in results {
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    }
}
