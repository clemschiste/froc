use anyhow::Result;
use axum::{
    routing::get,
    Router,
    response::Json,
    extract::State,
};

mod db;
mod services;

use crate::services::flux_service::FluxService;
use crate::services::Item;


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

#[tokio::main]
async fn main() -> Result<()> {
    let db = db::connect("sqlite://flutendu.db").await?;
    let state = AppState { flux_service: FluxService::new(db) };

    let app = Router::new()
        .route("/rss", get(fetch_rss))
        .with_state(state);
    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on : {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

