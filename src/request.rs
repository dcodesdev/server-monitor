/// Returns `true` if the URL is up
/// Returns `false` if down
pub async fn url_lookup(url: &str) -> anyhow::Result<bool> {
    let response = match reqwest::get(url).await {
        Err(e) => {
            let status = e.status();

            if let Some(status) = status {
                if status == 429 {
                    return Ok(true);
                }

                return Ok(false);
            } else {
                return Ok(false);
            };
        }
        Ok(res) => res,
    };

    let status = response.status();

    let status = if status.is_success() || status == 429 {
        true
    } else {
        false
    };

    Ok(status)
}
