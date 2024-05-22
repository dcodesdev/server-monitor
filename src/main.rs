mod bot;
mod db;
mod request;
mod status;

use bot::create_bot;
use db::Db;
use futures::future;
use status::{check_url_status, server_update_cron};
use std::sync::Arc;

const DEFAULT_INTERVAL: u64 = 1000 * 60; // 1 minute
const UPDATE_INTERVAL: u64 = 1000 * 60 * 60 * 24; // 24 hours

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let urls: Vec<String> = std::env::var("URLS")
        .expect("URLS must be set")
        .split(',')
        .map(|item| item.to_string())
        .collect();

    let interval: u64 = std::env::var("INTERVAL")
        .unwrap_or_else(|_| DEFAULT_INTERVAL.to_string())
        .parse()
        .expect("INTERVAL must be a number");

    let bot = Arc::new(create_bot());
    let db = Arc::new(Db::new().await?);

    println!("\nServer monitor is running with the following settings:");
    println!("\n- Interval: {}ms", interval);
    println!("- URLs:");
    urls.iter().for_each(|url| {
        println!("  - {}", url);
    });

    server_update_cron(Arc::clone(&db), Arc::clone(&bot));

    loop {
        let handles: Vec<_> = urls
            .iter()
            .map(|url| {
                let url = url.clone();
                let bot = Arc::clone(&bot);
                let db = Arc::clone(&db);
                tokio::spawn(async move { check_url_status(&url, &bot, &db).await })
            })
            .collect();

        let results: Vec<_> = future::join_all(handles)
            .await
            .into_iter()
            .map(|res| res.unwrap())
            .collect();

        for result in results {
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    }
}
