use axum::{Router, routing::get};
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("panic: {}", panic_info);
    }));

    let app = Router::new()
        .route("/", get(root))
        .route("/panic", get(panic_route));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;
}

async fn root() -> &'static str {
    tracing::info!("/ GET");
    "Hello world"
}

async fn panic_route() -> &'static str {
    tokio::spawn(async {
        panic!("intentional test panic");
    });
    "panic spawned"
}
