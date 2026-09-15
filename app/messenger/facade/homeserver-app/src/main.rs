#![forbid(unsafe_code)]

use messenger_homeserver_app::router;

#[tokio::main]
async fn main() {
    let listen = env_or("OYATIE_MESSENGER_HOMESERVER_LISTEN_ADDR", "127.0.0.1:8008");
    let server = env_or("OYATIE_MESSENGER_SERVER_NAME", "messenger.test");
    let listener = match tokio::net::TcpListener::bind(&listen).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = axum::serve(listener, router(server)).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}
