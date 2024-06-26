use chrono::NaiveDateTime;
use reqwest::StatusCode;
use std::time::Duration;

use super::{url::Url, Connection};

const DEFAULT_TIMEOUT: u64 = 10;

#[derive(Debug)]
#[allow(unused)]
pub struct Endpoint {
    pub id: String,
    pub url: Url,
    pub status: Status,
    pub uptime_at: Option<NaiveDateTime>,
    pub max_latency: Option<i64>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct EndpointModel {
    pool: Connection,
    client: reqwest::Client,
    tries: u8,
}

impl EndpointModel {
    pub async fn new(pool: Connection, urls: &Vec<Url>) -> anyhow::Result<Self> {
        let timeout = std::env::var("TIMEOUT")
            .unwrap_or(DEFAULT_TIMEOUT.to_string())
            .parse::<u64>()?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()?;

        for url in urls.iter() {
            let url = url.as_str();

            let row = sqlx::query!("SELECT COUNT(*) as count FROM endpoint WHERE url = ?", url)
                .fetch_one(&pool)
                .await?;

            let exists = row.count > 0;

            if exists {
                continue;
            }

            sqlx::query!(
                "INSERT INTO endpoint (url, status, uptime_at) VALUES (?, 'PENDING', NULL)",
                url
            )
            .execute(&pool)
            .await?;
        }

        let tries = std::env::var("TRIES")
            .unwrap_or("2".to_string())
            .parse::<u8>()?;

        if tries < 1 {
            panic!("TRIES must be greater than 0");
        }

        Ok(Self {
            pool,
            client,
            tries,
        })
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<Endpoint>> {
        let endpoints = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint")
            .fetch_all(&self.pool)
            .await?;

        Ok(endpoints)
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<Endpoint> {
        let endpoint = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint WHERE url = ?", url)
            .fetch_one(&self.pool)
            .await?;

        Ok(endpoint)
    }

    pub async fn get_max_latency(&self, url: &str) -> anyhow::Result<Option<i64>> {
        let latency = sqlx::query!("SELECT max_latency FROM endpoint WHERE url = ?", url)
            .fetch_one(&self.pool)
            .await?;

        Ok(latency.max_latency)
    }

    pub async fn relative_max_latency_update(&self, url: &str, latency: i64) -> anyhow::Result<()> {
        let max_latency = self.get_max_latency(url).await?.unwrap_or(0);

        if latency > max_latency {
            sqlx::query!(
                "UPDATE endpoint SET max_latency = ? WHERE url = ?",
                latency,
                url
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn reset_max_latency(&self, url: &str) -> anyhow::Result<()> {
        sqlx::query!("UPDATE endpoint SET max_latency = NULL WHERE url = ?", url)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Returns `true` if the URL is up
    /// Returns `false` if down
    pub async fn lookup(&self, url: &Url) -> anyhow::Result<bool> {
        for _ in 0..self.tries {
            let res = self.send_request(url).await?;

            if res {
                return Ok(true);
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(false)
    }

    async fn send_request(&self, url: &Url) -> anyhow::Result<bool> {
        let start = std::time::Instant::now();
        let res = self.client.get(url.as_str()).send().await;
        let latency = start.elapsed().as_millis() as i64;

        self.relative_max_latency_update(url.as_str(), latency)
            .await?;

        if let Ok(res) = res {
            let status = res.status();

            if status.is_success() || status == StatusCode::TOO_MANY_REQUESTS {
                return Ok(true);
            }
        };

        Ok(false)
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Up,
    Down,
    Pending,
}

impl From<String> for Status {
    fn from(s: String) -> Self {
        match s.as_str() {
            "UP" => Status::Up,
            "DOWN" => Status::Down,
            _ => Status::Pending,
        }
    }
}

impl From<Status> for String {
    fn from(s: Status) -> Self {
        match s {
            Status::Up => "UP".to_string(),
            Status::Down => "DOWN".to_string(),
            Status::Pending => "PENDING".to_string(),
        }
    }
}
