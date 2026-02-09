use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::data::models::{AppState, Article, SourceFeed};

/// Served at `/feeds`
/// Returns a list of all `SourceFeeds`
pub async fn list_feeds(State(state): State<AppState>) -> Json<Vec<SourceFeed>> {
    let snapshot = {
        let r = state.feeds.read().await;
        r.clone()
    };
    Json(snapshot)
}

/// Served at `/stats`
/// Returns a json object
/// `{num_articles: int, num_feeds: int}`
/// Used by frontend in About screen
pub async fn stats(State(state): State<AppState>) -> Json<Stats> {
    let num_feeds = {
        let r = state.feeds.read().await;
        r.len() as i64
    };

    let num_articles = match crate::data::crud::count_articles(&state.pool).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!("Error counting articles: {:?}", e);
            0
        }
    };

    Json(Stats {
        num_articles,
        num_sources: num_feeds,
    })
}

/// Served at `/unique`
/// Returns the list of each individual unique news source
/// Used by frontend to display in About and Options
pub async fn unique_sources(State(state): State<AppState>) -> Json<Vec<String>> {
    let feeds = {
        let r = state.feeds.read().await;
        r.clone()
    };

    let mut set: HashSet<String> = HashSet::new();
    for f in feeds {
        set.insert(f.source);
    }

    let mut sources: Vec<String> = set.into_iter().collect();
    sources.sort();
    Json(sources)
}

/// Served at `/legacy_search/{query}`
/// Plain searching with DB queries
pub async fn search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Article>> {
    let q = format!("%{}%", query);
    let articles = crate::data::crud::get_like(&q, &state.pool).await;
    tracing::info!("{} - {} results", query, articles.len());
    Json(articles)
}

/// Served at `/search/{query}`
/// Search results using Meilisearch limited to 100 sorted by most recent publication
/// TODO: Consider additional parameters that could be passed by the frontend
pub async fn meili_search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Article>> {
    let results = state
        .meili
        .search()
        .with_query(&query)
        .with_sort(&["pubDateMs:desc", "time_added:desc"])
        .with_limit(100)
        .execute::<Article>()
        .await
        .unwrap();

    let articles: Vec<Article> = results.hits.into_iter().map(|hit| hit.result).collect();

    Json(articles)
}

/// Served at `/health`
/// Provides healthcheck
pub async fn health() -> StatusCode {
    StatusCode::OK
}

/// Struct for returning stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    num_articles: i64,
    num_sources: i64,
}
