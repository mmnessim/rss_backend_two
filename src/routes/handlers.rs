pub async fn root() -> &'static str {
    "Hello world"
}

pub async fn panic_route() -> &'static str {
    tokio::spawn(async {
        panic!("intentional test panic");
    });
    "panic spawned"
}
