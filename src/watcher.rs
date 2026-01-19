use std::{path::Path, str, sync::Arc};

use notify::{Event, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::data::DbPool;

/// Watches the feeds.json file for changes and updates in-memory store of feeds on changes
pub async fn file_watcher(feeds_store_clone: Arc<RwLock<Vec<SourceFeed>>>) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<notify::Result<Event>>(100);

    let mut watcher = match notify::recommended_watcher(move |res| {
        let _ = tx.blocking_send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Error creating watcher: {e}");
            return;
        }
    };

    watcher
        .watch(Path::new("./feeds.json"), RecursiveMode::NonRecursive)
        .unwrap();

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                tracing::info!("event: {:?} {:?}", event.kind, event.paths);
                if event.kind.is_modify() {
                    tracing::info!("Feeds.json changed: {:?}", event.kind);
                    if let Ok(new_feeds) = read_file().await {
                        let mut w = feeds_store_clone.write().await;
                        *w = new_feeds;
                        tracing::info!("Updated in-memory feeds ({} entries)", w.len());
                    }
                };
            }
            Err(e) => tracing::error!("watch error: {:?}", e),
        }
    }
}

/// Reads the feeds.json file and returns a vector of Feed structs.
pub async fn read_file()
-> Result<Vec<SourceFeed>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let bytes = match tokio::fs::read("./feeds.json").await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Error reading feeds.json {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>);
        }
    };

    match serde_json::from_slice::<Vec<SourceFeed>>(&bytes) {
        Ok(feeds) => {
            tracing::info!("Loaded {} feeds", feeds.len());
            Ok(feeds)
        }
        Err(e) => {
            tracing::error!("Error parsing feeds from feeds.json: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)
        }
    }
}

/// Fetches a feed from url with `reqwest` then parses with `feed_rs` and adds to database
pub async fn parse_feed(feed: &SourceFeed, pool: &DbPool, index: &meilisearch_sdk::indexes::Index) {
    let resp = match reqwest::get(&feed.url).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::debug!("Error fetching feed {} {:?}", feed.source, e);
            return;
        }
    };

    if let Ok(bytes) = resp.bytes().await {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        match feed_rs::parser::parse(cursor) {
            Ok(parsed) => {
                crate::data::crud::insert_from_rss(parsed.clone(), &feed.source, pool, &index)
                    .await;
            }
            Err(e) => tracing::debug!("Failed to parse feed {}: {:?}", feed.source, e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceFeed {
    pub source: String,
    pub url: String,
}
