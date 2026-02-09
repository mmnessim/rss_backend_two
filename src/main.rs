// #![allow(unused_variables)]
// #![allow(dead_code)]
mod feature_check;

use std::{sync::Arc, vec};

use meilisearch_sdk::client::Client;
use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::data::crud;
use crate::util::watcher::{file_watcher, parse_feed, read_file};

mod data;
mod routes;
mod util;
mod watcher;

#[tokio::main]
async fn main() {
    // Meilisearch
    let client = Client::new("http://meilisearch:7700", None::<String>).unwrap();
    let meili_articles = client.index("articles");
    meili_articles
        .set_sortable_attributes(&["pubDateMs", "time_added"])
        .await
        .unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("panic: {}", panic_info);
    }));

    let pool = match data::initialize_database().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Error initializing DB: {:?}", e);
            std::process::exit(1);
        }
    };

    let feeds = match read_file().await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Error reading feeds: {:?}", e);
            vec![]
        }
    };

    let feeds_store = Arc::new(RwLock::new(feeds));
    let feeds_store_update = feeds_store.clone();

    let feeds_clone_fetch = feeds_store.clone();
    let pool_clone_fetch = pool.clone();
    let meili_clone = meili_articles.clone();

    // Watch feeds.json
    tokio::spawn(async move { file_watcher(feeds_store_update).await });

    // Poll RSS feeds every 15 minutes
    tokio::spawn(async move {
        loop {
            let snapshot = {
                let r = feeds_clone_fetch.read().await;
                r.clone()
            };

            for feed in snapshot {
                let pool2 = pool_clone_fetch.clone();
                let index2 = meili_clone.clone();
                tokio::spawn(async move {
                    parse_feed(&feed, &pool2, &index2).await;
                });
            }

            tokio::time::sleep(std::time::Duration::from_mins(15)).await;
        }
    });

    // Delete old articles once a day
    tokio::spawn({
        let pool_clone = pool.clone();
        async move {
            loop {
                match crate::data::crud::delete_old_articles(&pool_clone).await {
                    Ok(n) => tracing::info!("Pruned {} old articles", n),
                    Err(e) => tracing::error!("Error pruning old articles: {:?}", e),
                }
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 24)).await;
            }
        }
    });

    // Backfill articles
    let articles = crud::get_all_articles(&pool).await;
    let _ = meili_articles.add_documents(&articles, Some("id")).await;

    let app_state = data::models::AppState {
        pool: pool.clone(),
        feeds: feeds_store.clone(),
        meili: meili_articles,
    };

    let app = routes::router(app_state);

    let address = "0.0.0.0:3000";

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
