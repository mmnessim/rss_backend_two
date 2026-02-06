use axum::{Router, middleware, routing::get};

use crate::data::models::AppState;

pub mod handlers;
pub mod mw;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/feeds", get(handlers::list_feeds))
        .route("/stats", get(handlers::stats))
        .route("/health", get(handlers::health))
        .route("/legacy_search/{query}", get(handlers::search))
        .route("/search/{query}", get(handlers::meili_search))
        .route("/unique", get(handlers::unique_sources))
        .layer(middleware::from_fn(mw::log_request))
        .with_state(state)
}
