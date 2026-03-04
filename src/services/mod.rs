pub mod flux_service;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub summary: String,
    pub content: String,
    pub pub_date: u64,
}
