use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    initialize_db(&pool).await?;
    println!("Connected to database : {}", database_url);

    Ok(pool)
}

async fn initialize_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS feeds (
            id TEXT PRIMARY KEY,
            last_check INTEGER
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            feed_id TEXT,
            title TEXT,
            content TEXT,
            summary TEXT,
            pub_date INTEGER,
            last_update INTEGER,
            FOREIGN KEY(feed_id) REFERENCES feeds(id)
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

