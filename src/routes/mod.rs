use axum::{Router, middleware, routing::get};

use crate::AppState;

pub mod handlers;
pub mod mw;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/feeds", get(handlers::list_feeds))
        .route("/search/{query}", get(handlers::search))
        .route("/all", get(handlers::all_articles))
        .route("/panic", get(handlers::panic_route))
        .layer(middleware::from_fn(mw::log_request))
        .with_state(state)
}
