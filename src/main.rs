use anyhow::Result;
use axum::{
    routing::get,
    Router,
    response::Json,
    // extract::State,
};
use chrono::{DateTime, Utc};
use feed_rs::{parser};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
struct Item {
    pub title: String,
    pub last_modified: DateTime<Utc>,
    pub content: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: SqlitePool,
}

mod db;


async fn fetch_rss() -> Result<Json<Vec<Item>>, String> {
    let urls = vec!["https://ploum.net/atom_fr.xml", "https://pluralistic.net/feed/"];

    let mut entries: Vec<Item> = Vec::new();

    for url in urls {
        
        let response = reqwest::get(url)
            .await
            .map_err(|e| e.to_string())?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        let feed = parser::parse(&bytes[..])
            .map_err(|e| e.to_string())?;

        for entry in feed.entries {
            let item = Item {title: entry.title.expect("No title").content, content: entry.content.expect("No content").body.unwrap_or("No content".to_string()), last_modified: entry.updated.expect("Unknown")};
            entries.push(item);
        }
    }
    entries.sort_by(|a, b| b.last_modified.timestamp().cmp(&a.last_modified.timestamp()));


    Ok(Json(entries))
}


#[tokio::main]
async fn main() -> Result<()> {
    // Connect DB and initialize state
    let db = db::connect("sqlite://flutendu.db").await?;
    let state = AppState {db: db.clone()};

    // Server
    let app = Router::new()
        .route("/rss", get(fetch_rss))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}
