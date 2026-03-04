use anyhow::Result;
use axum::{
    Json, Router, extract::State, routing::get
};
use axum::response::Html;
use html_escape::encode_text;
use serde_json::to_string_pretty;

mod db;
mod services;

use crate::services::{Item, flux_service::FluxService};


#[derive(Debug, Clone)]
pub struct AppState {
    pub flux_service: FluxService,
}


async fn fetch_rss(State(state): State<AppState>) -> Result<Json<Vec<Item>>, String> {
    let mut all_items = Vec::new();

    for feed_id in ["ploum", "pluralistic", "franceinfo"] {
        match state.flux_service.get_feed_items(feed_id).await {
            Ok(items) => all_items.extend(items),
            Err(e) => return Err(e.to_string()),
        }
    }

    all_items.sort_by(|a,b| b.pub_date.cmp(&a.pub_date) );

    Ok(Json(all_items))
}


async fn rss_html(State(state): State<AppState>) -> Result<Html<String>, String> {
    let mut all_items = Vec::new();
    for feed_id in ["ploum", "pluralistic", "franceinfo"] {
        match state.flux_service.get_feed_items(feed_id).await {
            Ok(items) => all_items.extend(items),
            Err(e) => return Err(e.to_string()),
        }
    }
    all_items.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

    // Pretty-print le JSON en Rust
    let pretty_json = to_string_pretty(&all_items).map_err(|e| e.to_string())?;

    // Échappe le HTML pour éviter les problèmes de balises
    let escaped_json = encode_text(&pretty_json);

    let html = format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>RSS Feed</title>
            <style>
                pre {{
                    white-space: pre-wrap;
                    word-wrap: break-word;
                    background: #f5f5f5;
                    padding: 10px;
                    border-radius: 5px;
                }}
            </style>
        </head>
        <body>
            <h1>RSS Feed</h1>
            <pre>{}</pre>
        </body>
        </html>
    "#, escaped_json);
    Ok(Html(html))
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = db::connect("sqlite://flutendu.db").await?;
    let state = AppState { flux_service: FluxService::new(db) };

    let app = Router::new()
        .route("/rss", get(fetch_rss))
        .route("/rss/html", get(rss_html))
        .with_state(state);
    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on : {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

