use std::{fs, path::Path, str, vec};

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

    let app = routes::router();

    let address = "127.0.0.1:3000";

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(lst) => lst,
        Err(err) => {
            tracing::error!("Failed to bind to address {} {}", address, err);
            std::process::exit(1);
        }
    };

    // let a = data::crud::Article {
    //     id: 0,
    //     title: String::from("Test"),
    //     description: String::from("Test article"),
    // };
    //let rows = data::crud::put_article(&pool, a).await;
    let pulled_rows = data::crud::get_articles(&pool).await;
    println!("rows pulled: {:?}", pulled_rows);

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
                    let _ = read_file().await;
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

    match str::from_utf8(&bytes) {
        Ok(s) => println!("{s}"),
        Err(e) => println!("Error: {:?}", e),
    }

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
        // println!("{} {}", feeds[0].source, feeds[0].url);
        let feeds_clone = feeds.clone();
        for feed in feeds_clone {
            tokio::spawn(async move {
                parse_feed(&feed).await;
            });
        }
        return Ok(feeds);
    };

    Ok(vec![])
}

/// Fetches and parses a feed from the given Feed struct.
/// Eventually this will either update the database directly or send the parsed feed to another service.
async fn parse_feed(feed: &SourceFeed) {
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
                let title = match parsed.title {
                    Some(t) => t.content,
                    None => String::new(),
                };

                let authors = if parsed.authors.is_empty() {
                    vec!["Staff".to_string()]
                } else {
                    parsed.authors.iter().map(|a| a.name.clone()).collect()
                };
                let description = match parsed.description {
                    Some(d) => d.content,
                    None => String::new(),
                };
                let categories = if parsed.categories.is_empty() {
                    vec!["No categories".to_string()]
                } else {
                    parsed.categories.iter().map(|c| c.term.clone()).collect()
                };
                tracing::info!("{} {:?} {} {}", title, authors, description, categories[0]);
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
