use chrono::NaiveDateTime;
use std::sync::Arc;

use super::{url::Url, Conn};

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
    pool: Arc<Conn>,
}

impl EndpointModel {
    pub fn new(pool: Arc<Conn>) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<Endpoint>> {
        let endpoints = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint")
            .fetch_all(&*self.pool)
            .await?;

        Ok(endpoints)
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<Endpoint> {
        let endpoint = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint WHERE url = ?", url)
            .fetch_optional(&*self.pool)
            .await?;

        match endpoint {
            Some(endpoint) => Ok(endpoint),
            None => {
                let endpoint = sqlx::query!(
                    "INSERT INTO endpoint (url, status, uptime_at) VALUES (?, 'PENDING', NULL) RETURNING *",
                    url
                )
                .fetch_one(&*self.pool)
                .await?;

                let endpoint = Endpoint {
                    id: endpoint.id,
                    url: endpoint.url.into(),
                    status: Status::Pending,
                    uptime_at: None,
                    max_latency: None,
                    created_at: endpoint.created_at,
                };

                Ok(endpoint)
            }
        }
    }

    pub async fn get_max_latency(&self, url: &str) -> anyhow::Result<Option<i64>> {
        let latency = sqlx::query!("SELECT max_latency FROM endpoint WHERE url = ?", url)
            .fetch_one(&*self.pool)
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
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn reset_max_latency(&self, url: &str) -> anyhow::Result<()> {
        sqlx::query!("UPDATE endpoint SET max_latency = NULL WHERE url = ?", url)
            .execute(&*self.pool)
            .await?;

        Ok(())
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
