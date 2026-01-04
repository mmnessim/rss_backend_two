use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};

pub async fn log_request(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    tracing::info!(method = %method, uri = %uri);
    next.run(req).await
}
