use chrono::{Local, NaiveDateTime};
use sqlx::{Pool, Sqlite, SqlitePool};

#[derive(Debug)]
pub struct Db {
    pub verbose: bool,
    pub pool: Pool<Sqlite>,
    pub endpoint: EndpointModel,
    pub incident: IncidentModel,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub id: String,
    pub url: String,
    pub status: Status,
    pub uptime_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub url: String,
    pub message: String,
    pub created_at: NaiveDateTime,
}

type Conn = Pool<Sqlite>;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
                    url: endpoint.url,
                    status: Status::Pending,
                    uptime_at: None,
                    created_at: endpoint.created_at,
                };

                Ok(endpoint)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl Db {
    pub async fn new() -> anyhow::Result<Self> {
        // create the db file if not exists
        create_db_if_not_exists()?;

        let verbose = false;
        // connect the db
        let pool = connect(verbose).await?;

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
    let url = "sqlite://db.sqlite";

    let pool = SqlitePool::connect(url).await?;

    if verbose {
        println!("Connected to the database {}", url);
    }

    Ok(pool)
}

fn create_db_if_not_exists() -> anyhow::Result<()> {
    let exists = std::path::Path::new("db.sqlite").exists();

    if !exists {
        // create a file db.sqlite
        std::fs::write("db.sqlite", "")?;
        sqlx::migrate!("./migrations/");
    }

    Ok(())
}
