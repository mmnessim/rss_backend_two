use axum::{Json, extract::State};

use crate::{AppState, data::crud::Article, watcher::SourceFeed};

pub async fn root() -> &'static str {
    "Hello world"
}

pub async fn panic_route() -> &'static str {
    tokio::spawn(async {
        panic!("intentional test panic");
    });
    "panic spawned"
}

pub async fn list_feeds(State(state): State<AppState>) -> Json<Vec<SourceFeed>> {
    let snapshot = {
        let r = state.feeds.read().await;
        r.clone()
    };
    Json(snapshot)
}

pub async fn all_articles(State(state): State<AppState>) -> Json<Vec<Article>> {
    let artiles = crate::data::crud::get_articles(&state.pool).await;
    Json(artiles)
}
