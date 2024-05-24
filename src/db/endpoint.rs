use chrono::NaiveDateTime;

use super::{url::Url, Conn};

#[derive(Debug)]
pub struct Endpoint {
    pub id: String,
    pub url: Url,
    pub status: Status,
    pub uptime_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct EndpointModel {
    pool: Conn,
}

impl EndpointModel {
    pub fn new(pool: Conn) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<Endpoint>> {
        let endpoints = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint")
            .fetch_all(&self.pool)
            .await?;

        Ok(endpoints)
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<Endpoint> {
        let endpoint = sqlx::query_as!(Endpoint, "SELECT * FROM endpoint WHERE url = ?", url)
            .fetch_optional(&self.pool)
            .await?;

        match endpoint {
            Some(endpoint) => Ok(endpoint),
            None => {
                let endpoint = sqlx::query!(
                    "INSERT INTO endpoint (url, status, uptime_at) VALUES (?, 'PENDING', NULL) RETURNING *",
                    url
                )
                .fetch_one(&self.pool)
                .await?;

                let endpoint = Endpoint {
                    id: endpoint.id,
                    url: endpoint.url.into(),
                    status: Status::Pending,
                    uptime_at: None,
                    created_at: endpoint.created_at,
                };

                Ok(endpoint)
            }
        }
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
