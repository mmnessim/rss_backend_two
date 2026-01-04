use axum::{Router, middleware, routing::get};

pub mod handlers;
pub mod mw;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handlers::root))
        .route("/panic", get(handlers::panic_route))
        .layer(middleware::from_fn(mw::log_request))
}
