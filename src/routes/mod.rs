use axum::{Router, middleware, routing::get};

use crate::AppState;

pub mod handlers;
pub mod mw;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/feeds", get(handlers::list_feeds))
        .route("/stats", get(handlers::stats))
        .route("/health", get(handlers::health))
        .route("/search/{query}", get(handlers::search))
        .route("/by-feed/{query}", get(handlers::search_by_source))
        // .route("/all", get(handlers::all_articles))
        .layer(middleware::from_fn(mw::log_request))
        .with_state(state)
}
