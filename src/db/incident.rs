use chrono::NaiveDateTime;

use super::Conn;

#[derive(Debug)]
pub struct Incident {
    pub id: String,
    pub url: String,
    pub message: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct IncidentModel {
    pool: Conn,
}

impl IncidentModel {
    pub fn new(pool: Conn) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<Incident>> {
        let incidents = sqlx::query_as!(Incident, "SELECT * FROM incident")
            .fetch_all(&self.pool)
            .await?;

        Ok(incidents)
    }

    pub async fn is_empty(&self) -> anyhow::Result<bool> {
        let incidents = sqlx::query!("SELECT id FROM incident")
            .fetch_all(&self.pool)
            .await?;

        Ok(incidents.is_empty())
    }
}
