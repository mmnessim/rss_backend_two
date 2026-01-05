use std::path::Path;

use notify::{Event, RecursiveMode, Watcher};
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("panic: {}", panic_info);
    }));

    tokio::spawn(async {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<notify::Result<Event>>(100);

        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.blocking_send(res);
        }) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Error creating watcher: {e}");
                return;
            }
        };

        watcher
            .watch(Path::new("."), RecursiveMode::NonRecursive)
            .unwrap();

        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => tracing::info!("event: {:?} {:?}", event.kind, event.paths),
                Err(e) => tracing::error!("watch error: {:?}", e),
            }
        }
    });

    let app = routes::router();

    let address = "127.0.0.1:3000";

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(lst) => lst,
        Err(err) => {
            tracing::error!("Failed to bind to address {} {}", address, err);
            std::process::exit(1);
        }
    };

    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;
}
