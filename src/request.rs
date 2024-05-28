use reqwest::StatusCode;
use std::{sync::Arc, time::Duration};

use crate::db::{url::Url, Db};

const DEFAULT_TIMEOUT: u64 = 10;

/// Returns `true` if the URL is up
/// Returns `false` if down
pub async fn url_lookup(url: &Url, db: &Arc<Db>) -> anyhow::Result<bool> {
    let interval = std::env::var("TIMEOUT")
        .unwrap_or(DEFAULT_TIMEOUT.to_string())
        .parse::<u64>()?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(interval))
        .build()?;

    let start = std::time::Instant::now();
    let res = client.get(url.as_str()).send().await;
    let latency = start.elapsed().as_millis() as i64;

    db.endpoint
        .relative_max_latency_update(url.as_str(), latency)
        .await?;

    match res {
        Err(_) => Ok(false),
        Ok(res) => {
            let status = res.status();

            if status.is_success() || status == StatusCode::TOO_MANY_REQUESTS {
                return Ok(true);
            }

            Ok(false)
        }
    }
}
