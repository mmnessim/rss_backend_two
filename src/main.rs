#![allow(unused_variables)]
#![allow(dead_code)]
use std::{
    fs,
    path::Path,
    str,
    sync::{Arc, atomic::AtomicU64},
    vec,
};

use notify::{Event, RecursiveMode, Watcher};
use serde::Deserialize;
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod data;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("panic: {}", panic_info);
    }));

    tokio::spawn(async { file_watcher().await });

    let pool = match data::initialize_database().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Error initializing DB: {:?}", e);
            std::process::exit(1);
        }
    };

    let transaction_count = Arc::new(AtomicU64::new(0));

    let feeds = match read_file().await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Error reading feeds: {:?}", e);
            vec![]
        }
    };

    for feed in feeds.iter() {
        let pool_clone = pool.clone();
        let feed_clone = feed.clone();
        tokio::spawn(async move {
            parse_feed(&feed_clone, &pool_clone).await;
        });
    }

    let app = routes::router();

    let address = "127.0.0.1:3000";

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(lst) => lst,
        Err(err) => {
            tracing::error!("Failed to bind to address {} {}", address, err);
            std::process::exit(1);
        }
    };

    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;
}

/// Watches the feeds.json file for changes and reloads feeds when it changes.
/// Eventaully this will simple update the in-memory feed list instead of reloading everything.
async fn file_watcher() {
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
                // tracing::info!("event: {:?} {:?}", event.kind, event.paths);
                if event.kind
                    == notify::EventKind::Modify(notify::event::ModifyKind::Data(
                        notify::event::DataChange::Any,
                    ))
                {
                    tracing::info!("Feeds.json changed: {:?}", event.kind);
                    // let _ = read_file(&pool).await;
                };
            }
            Err(e) => tracing::error!("watch error: {:?}", e),
        }
    }
}

/// Reads the feeds.json file and returns a vector of Feed structs.
async fn read_file() -> Result<Vec<SourceFeed>, Box<dyn std::error::Error>> {
    let bytes = match fs::read("./feeds.json") {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Error reading feeds.json {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    let feeds_opt = match serde_json::from_slice::<Vec<SourceFeed>>(&bytes) {
        Ok(feeds) => {
            tracing::info!("Loaded {} feeds", feeds.len());
            Some(feeds)
        }
        Err(e) => {
            tracing::error!("Error parsing feeds from feeds.json: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    // Temporary block to spawn tasks to parse each feed.
    if let Some(feeds) = feeds_opt {
        return Ok(feeds);
    };

    Ok(vec![])
}

/// Fetches and parses a feed from the given Feed struct.
/// Eventually this will either update the database directly or send the parsed feed to another service.
async fn parse_feed(feed: &SourceFeed, pool: &sqlx::SqlitePool) {
    let resp = match reqwest::get(&feed.url).await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Error fetching feed {} {:?}", feed.source, e);
            return;
        }
    };

    if let Ok(bytes) = resp.bytes().await {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        match feed_rs::parser::parse(cursor) {
            Ok(parsed) => {
                let inserted =
                    data::crud::insert_from_rss(parsed.clone(), &feed.source, pool).await;
            }
            Err(e) => tracing::error!("Failed to parse feed {}: {:?}", feed.source, e),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct SourceFeed {
    source: String,
    url: String,
}
