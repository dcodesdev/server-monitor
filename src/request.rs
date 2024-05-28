use reqwest::StatusCode;
use std::time::Duration;

const DEFAULT_TIMEOUT: u64 = 10;

/// Returns `true` if the URL is up
/// Returns `false` if down
pub async fn url_lookup(url: &str) -> anyhow::Result<bool> {
    let interval = std::env::var("TIMEOUT")
        .unwrap_or(DEFAULT_TIMEOUT.to_string())
        .parse::<u64>()?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(interval))
        .build()?;

    match client.get(url).send().await {
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
