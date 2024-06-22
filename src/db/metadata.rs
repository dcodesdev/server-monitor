use crate::{db::Connection, UPDATE_INTERVAL};
use chrono::{Local, NaiveDateTime};

#[derive(Debug)]
pub struct MetadataModel {
    pool: Connection,
}

#[derive(Debug)]
#[allow(unused)]
pub struct Metadata {
    pub id: i64,
    pub last_update_sent_at: Option<NaiveDateTime>,
}

impl MetadataModel {
    pub async fn new(pool: Connection) -> anyhow::Result<Self> {
        let metadata = sqlx::query_as!(Metadata, "SELECT * FROM metadata;")
            .fetch_optional(&pool)
            .await?;

        if metadata.is_none() {
            sqlx::query!("INSERT INTO metadata (last_update_sent_at) VALUES (NULL);")
                .execute(&pool)
                .await?;
        }

        Ok(Self { pool })
    }

    pub async fn get(&self) -> anyhow::Result<Metadata> {
        let metadata = sqlx::query_as!(Metadata, "SELECT * FROM metadata;")
            .fetch_one(&self.pool)
            .await?;

        Ok(metadata)
    }

    pub async fn update_last_sent_at(&self) -> anyhow::Result<()> {
        let now = Local::now().naive_local();

        sqlx::query!("UPDATE metadata SET last_update_sent_at = ?;", now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn interval(&self) -> anyhow::Result<u64> {
        let metadata = self.get().await?;
        let now = Local::now().naive_local();

        let interval = if let Some(last_sent) = metadata.last_update_sent_at {
            let elapsed = now.signed_duration_since(last_sent).num_milliseconds();
            if elapsed >= UPDATE_INTERVAL as i64 {
                0
            } else {
                (UPDATE_INTERVAL as i64 - elapsed) as u64
            }
        } else {
            UPDATE_INTERVAL
        };

        Ok(interval)
    }
}
