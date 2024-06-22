use sqlx::SqlitePool;

use super::Connection;

pub async fn connect(verbose: bool) -> anyhow::Result<Connection> {
    let db_url = "sqlite:db/db.sqlite";

    let pool = SqlitePool::connect(&db_url).await?;

    if verbose {
        println!("Connected to the database {}", &db_url);
    }

    Ok(pool)
}

pub fn create_db_if_not_exists() -> anyhow::Result<()> {
    let exists = std::path::Path::new("db/db.sqlite").exists();
    if !exists {
        println!("Creating the database...");

        std::fs::create_dir_all("db")?;
        std::fs::write("db/db.sqlite", "")?;
    }

    Ok(())
}

pub async fn migrate(pool: &Connection, verbose: bool) -> anyhow::Result<()> {
    if verbose {
        println!("Running the migrations...");
    }

    sqlx::migrate!("./migrations/").run(pool).await?;

    if verbose {
        println!("Migrations completed!");
    }

    Ok(())
}
