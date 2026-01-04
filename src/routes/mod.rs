use axum::{Router, routing::get};

pub mod handlers;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/panic", get(handlers::panic_route))
}
