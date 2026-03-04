use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::services::Item;
use anyhow::Result;
use feed_rs::parser;
use sqlx::{SqlitePool, Row};

#[derive(Debug, Clone)]
pub struct FluxService {
    pub db: SqlitePool,
    pub channels: Vec<String>,
}

impl FluxService {

    // A Initialize channel also
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            channels: vec![
                "https://ploum.net/atom_fr.xml".to_string(),
                "https://pluralistic.net/feed/".to_string(),
                "https://www.franceinfo.fr/monde.rss".to_string(),
            ],
        }
    }

    pub async fn should_refresh_feed(&self, feed_id: &str, ttl: u64) -> Result<bool> {
        let row = sqlx::query(
            "SELECT last_check FROM feeds WHERE id = ?",
        )
        .bind(feed_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = row {
            let updated_at: u64 = row.try_get("last_check")?;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            Ok(now - updated_at > ttl)
        } else {
            Ok(true)
        }
    }

    pub async fn refresh_feed(&self, feed_id: &str, url: &str) -> Result<()> {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;
        let feed = parser::parse(&bytes[..])?;

        sqlx::query(
            "INSERT OR REPLACE INTO feeds (id, last_check) VALUES (?, strftime('%s','now'))",
        )
        .bind(feed_id)
        // .bind(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
        .execute(&self.db)
        .await?;

        for entry in feed.entries {
            let title = entry.title.map_or("No title".to_string(), |t| t.content);
            let summary = entry.summary.map_or("No summary".to_string(), |c| c.content);
            let content = entry.content.map_or("No content".to_string(), |c| c.body.unwrap_or_default());
            let pub_date = entry.published.map_or(0, |p| p.timestamp());
            let updated = entry.updated.map_or(0, |u| u.timestamp());

            sqlx::query(
                "INSERT OR REPLACE INTO items (id, feed_id, title, summary, content, pub_date, last_update) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(entry.id)
            .bind(feed_id)
            .bind(title)
            .bind(summary)
            .bind(content)
            .bind(pub_date)
            .bind(updated)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    pub async fn get_feed_items(&self, feed_id: &str) -> Result<Vec<Item>> {
        const TTL: u64 = Duration::from_secs(60 * 10).as_secs();
        let url = self.channels.iter().find(|u| u.contains(feed_id)).ok_or(anyhow::anyhow!("Feed not found"))?;

        if self.should_refresh_feed(feed_id, TTL).await? {
            println!("TTL expired for {} : Refreshing feed...", url);
            self.refresh_feed(feed_id, url).await?;
        }

        let rows = sqlx::query(
            "SELECT title, summary, content, pub_date FROM items WHERE feed_id = ? ORDER BY last_update DESC",
        )
        .bind(feed_id)
        .fetch_all(&self.db)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(Item {
                title: row.try_get("title")?,
                content: row.try_get("content")?,
                summary: row.try_get("summary")?,
                pub_date: row.try_get("pub_date")?,
            });
        }

        Ok(items)
    }
}
