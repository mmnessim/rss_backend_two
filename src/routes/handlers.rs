use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, data::crud::Article, watcher::SourceFeed};

pub async fn list_feeds(State(state): State<AppState>) -> Json<Vec<SourceFeed>> {
    let snapshot = {
        let r = state.feeds.read().await;
        r.clone()
    };
    Json(snapshot)
}

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

pub async fn all_articles(State(state): State<AppState>) -> Json<Vec<Article>> {
    let articles = crate::data::crud::get_articles(&state.pool).await;
    Json(articles)
}

pub async fn search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Article>> {
    let q = format!("%{}%", query);
    let articles = crate::data::crud::get_like(&q, &state.pool).await;
    tracing::info!("{} - {} results", query, articles.len());
    Json(articles)
}

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn search_by_source(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Article>> {
    let q = format!("%{}%", query);
    let articles = crate::data::crud::get_by_feed(&q, &state.pool).await;
    tracing::info!("{} - {} results", query, articles.len());
    Json(articles)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    num_articles: i64,
    num_sources: i64,
}
