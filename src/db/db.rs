use chrono::Local;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use super::{
    endpoint::EndpointModel,
    helpers::{connect, create_db_if_not_exists, migrate},
    incident::IncidentModel,
    metadata::MetadataModel,
    url::Url,
};

pub type Conn = Pool<Sqlite>;

#[derive(Debug)]
pub struct Db {
    pub verbose: bool,
    pub pool: Arc<Conn>,
    pub endpoint: EndpointModel,
    pub incident: IncidentModel,
    pub metadata: MetadataModel,
}

impl Db {
    pub async fn new() -> anyhow::Result<Self> {
        let verbose = false;

        // create the db file if not exists
        create_db_if_not_exists()?;

        // connect the db
        let pool = Arc::new(connect(verbose).await?);

        // run the migrations
        migrate(&pool, verbose).await?;

        let incident = IncidentModel::new(Arc::clone(&pool));
        let endpoint = EndpointModel::new(Arc::clone(&pool));
        let metadata = MetadataModel::new(Arc::clone(&pool)).await?;

        let db = Self {
            verbose,
            pool,
            incident,
            endpoint,
            metadata,
        };

        Ok(db)
    }

    pub async fn set_status_up(&self, url: &str) -> anyhow::Result<()> {
        // Update the database
        let now = Local::now();
        sqlx::query!(
            "UPDATE endpoint SET status = 'UP', uptime_at = ? WHERE url = ?",
            now,
            url
        )
        .execute(&*self.pool)
        .await?;

        if self.verbose {
            println!("{} is up!", url);
        }

        Ok(())
    }

    pub async fn set_status_down(&self, url: &Url) -> anyhow::Result<()> {
        let url_str = url.as_str();
        sqlx::query!(
            "UPDATE endpoint SET status = 'DOWN', uptime_at = NULL WHERE url = ?",
            url_str
        )
        .execute(&*self.pool)
        .await?;

        let message = format!("{} was down!", url.strip_prefix());
        let created_at = Local::now();
        sqlx::query!(
            "INSERT INTO incident (url, message, created_at) VALUES (?, ?, ?)",
            url_str,
            message,
            created_at
        )
        .execute(&*self.pool)
        .await?;

        if self.verbose {
            println!("{}", message);
        }

        Ok(())
    }
}
