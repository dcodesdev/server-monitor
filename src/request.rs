use reqwest::StatusCode;
use std::time::Duration;

/// Returns `true` if the URL is up
/// Returns `false` if down
pub async fn url_lookup(url: &str) -> anyhow::Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    match client.get(url).send().await {
        Err(_) => return Ok(false),
        Ok(res) => {
            let status = res.status();

            if status.is_success() || status == StatusCode::TOO_MANY_REQUESTS {
                return Ok(true);
            }

            return Ok(false);
        }
    }
}
