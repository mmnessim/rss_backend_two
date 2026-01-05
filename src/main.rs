use std::{fs, path::Path, str};

use notify::{Event, RecursiveMode, Watcher};
use serde::Deserialize;
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("panic: {}", panic_info);
    }));

    tokio::spawn(async { file_watcher().await });

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
                read_file();
                tracing::info!("event: {:?} {:?}", event.kind, event.paths)
            }
            Err(e) => tracing::error!("watch error: {:?}", e),
        }
    }
}

fn read_file() {
    let bytes = match fs::read("./feeds.json") {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Error reading feeds.json {:?}", e);
            return;
        }
    };
    match str::from_utf8(&bytes) {
        Ok(s) => println!("{s}"),
        Err(e) => println!("Error: {:?}", e),
    }

    let feeds_opt = match serde_json::from_slice::<Vec<Feed>>(&bytes) {
        Ok(feeds) => {
            tracing::info!("Loaded {} feeds", feeds.len());
            Some(feeds)
        }
        Err(e) => {
            tracing::error!("Error parsing feeds from feeds.json: {:?}", e);
            None
        }
    };
    if let Some(feeds) = feeds_opt {
        println!("{} {}", feeds[0].source, feeds[0].url)
    }
}

#[derive(Deserialize, Debug)]
struct Feed {
    source: String,
    url: String,
}
