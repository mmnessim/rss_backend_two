pub async fn root() -> &'static str {
    tracing::info!("/ GET ");
    "Hello world"
}

pub async fn panic_route() -> &'static str {
    tracing::info!("/panic GET");
    tokio::spawn(async {
        panic!("intentional test panic");
    });
    "panic spawned"
}
