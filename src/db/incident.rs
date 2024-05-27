use chrono::NaiveDateTime;
use sqlx::sqlite::SqliteQueryResult;

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

    pub async fn delete_many(&self, ids: Vec<&str>) -> anyhow::Result<SqliteQueryResult> {
        let query = format!(
            "DELETE FROM incident WHERE id IN ({})",
            ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
        );

        let mut q = sqlx::query(&query);

        for id in ids {
            q = q.bind(id);
        }

        let result = q.execute(&self.pool).await?;

        Ok(result)
    }
}
