use std::{
    net::SocketAddr,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use application_shell_app::server::{router_for_package_root, serve_router_until_shutdown};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};

static TEMP_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn attribute_values<'a>(markup: &'a str, name: &str) -> Vec<&'a str> {
    markup
        .split(&format!(" {name}=\""))
        .skip(1)
        .filter_map(|suffix| suffix.split('"').next())
        .collect()
}

pub(super) async fn spawn_server(
    package_root: PathBuf,
) -> (SocketAddr, oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("read listener address");
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(async move {
        serve_router_until_shutdown(
            listener,
            router_for_package_root(package_root),
            async move {
                let _ = stopped.await;
            },
        )
        .await
        .expect("serve test router");
    });
    tokio::task::yield_now().await;
    (address, stop, server)
}

pub(super) async fn request(address: SocketAddr, request: &str) -> String {
    let mut stream = TcpStream::connect(address)
        .await
        .expect("connect to test listener");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write HTTP request");
    stream.flush().await.expect("flush HTTP request");

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .expect("read HTTP response");
    String::from_utf8(response).expect("HTTP response is UTF-8")
}

pub(super) fn temporary_package_root() -> PathBuf {
    let sequence = TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "application-shell-live-server-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create unique test package root");
    root
}

pub(super) fn assert_status(response: &str, expected: &str) {
    assert!(
        response.starts_with(&format!("HTTP/1.1 {expected}")),
        "expected HTTP {expected}, got: {response}"
    );
}
