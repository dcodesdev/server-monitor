/// Returns `true` if the URL is up
/// Returns `false` if down
pub async fn url_lookup(url: &str) -> anyhow::Result<bool> {
    let response = reqwest::get(url).await?;
    let status = response.status();

    let status = if status.is_success() || status == 429 {
        true
    } else {
        false
    };

    Ok(status)
}
