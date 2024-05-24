use chrono::Local;
use sqlx::{Pool, Sqlite, SqlitePool};

use super::{endpoint::EndpointModel, incident::IncidentModel};

pub type Conn = Pool<Sqlite>;

#[derive(Debug)]
pub struct Db {
    pub verbose: bool,
    pub pool: Conn,
    pub endpoint: EndpointModel,
    pub incident: IncidentModel,
}

impl Db {
    pub async fn new() -> anyhow::Result<Self> {
        let verbose = false;

        // create the db file if not exists
        create_db_if_not_exists()?;

        // connect the db
        let pool = connect(verbose).await?;

        // run the migrations
        migrate(&pool, verbose).await?;

        let incident = IncidentModel::new(pool.clone());
        let endpoint = EndpointModel::new(pool.clone());

        let db = Self {
            verbose,
            pool,
            incident,
            endpoint,
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
        .execute(&self.pool)
        .await?;

        sqlx::query!("DELETE FROM incident WHERE url = ?", url)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn set_status_down(&self, url: &str) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE endpoint SET status = 'DOWN', uptime_at = NULL WHERE url = ?",
            url
        )
        .execute(&self.pool)
        .await?;

        let message = format!("{} was down!", &url);
        let created_at = Local::now();
        sqlx::query!(
            "INSERT INTO incident (url, message, created_at) VALUES (?, ?, ?)",
            url,
            message,
            created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub async fn connect(verbose: bool) -> anyhow::Result<Pool<Sqlite>> {
    let db_url = "sqlite:db/db.sqlite";

    let pool = SqlitePool::connect(&db_url).await?;

    if verbose {
        println!("Connected to the database {}", &db_url);
    }

    Ok(pool)
}

fn create_db_if_not_exists() -> anyhow::Result<()> {
    let exists = std::path::Path::new("db/db.sqlite").exists();
    if !exists {
        println!("Creating the database...");

        std::fs::create_dir_all("db")?;
        std::fs::write("db/db.sqlite", "")?;
    }

    Ok(())
}

async fn migrate(pool: &Conn, verbose: bool) -> anyhow::Result<()> {
    if verbose {
        println!("Running the migrations...");
    }

    sqlx::migrate!("./migrations/").run(pool).await?;

    if verbose {
        println!("Migrations completed!");
    }

    Ok(())
}
