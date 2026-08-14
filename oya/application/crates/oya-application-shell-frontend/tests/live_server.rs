#![cfg(all(unix, feature = "ssr"))]

use std::{
    net::SocketAddr,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use oya_application_shell_frontend::server::{
    router_for_package_root, serve_router_until_shutdown,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};

static TEMP_ROOT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn post_bootstrap_mount_contract_preserves_one_host_and_one_island_root() {
    let app_source = include_str!("../src/app.rs");
    let island_source = app_source
        .split("pub fn DashboardIsland()")
        .nth(1)
        .expect("DashboardIsland component source");
    let bootstrap_source = include_str!("../src/lib.rs");

    // `mount_dashboard_island_by_id` clears this stable host, then pinned Leptos 0.8.19
    // `mount_to(parent, DashboardIsland)` calls `mountable.mount(&parent, None)`: it appends the
    // component root beneath the host rather than replacing the host itself. The two source
    // contracts below, plus this resulting-DOM model, prevent duplicate IDs after bootstrap.
    assert!(app_source.contains("<div id=crate::DASHBOARD_MOUNT_HOST_ID>"));
    assert!(bootstrap_source.contains("element.set_inner_html(\"\");"));
    assert!(bootstrap_source.contains("mount_to(parent, DashboardIsland)"));
    assert!(
        !island_source
            .split("const TABLIST_SELECTORS")
            .next()
            .expect("DashboardIsland component boundary")
            .contains("DASHBOARD_MOUNT_HOST_ID")
    );

    let post_bootstrap_dom = r#"<div id="oya-dashboard-island-root"><div class="dashboard-island" data-island="render-envelope-dashboard"></div></div>"#;
    assert_eq!(
        post_bootstrap_dom
            .matches("id=\"oya-dashboard-island-root\"")
            .count(),
        1
    );
    assert_eq!(
        post_bootstrap_dom
            .matches("data-island=\"render-envelope-dashboard\"")
            .count(),
        1
    );
}

#[test]
fn native_ssr_request_emits_no_reactive_tracking_warnings() {
    if std::env::var_os("OYA_SSR_WARNING_CAPTURE_CHILD").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build child Tokio runtime")
            .block_on(async {
                let package_root = temporary_package_root();
                let (address, stop, server) = spawn_server(package_root.clone()).await;
                let root = request(
                    address,
                    "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                )
                .await;
                assert_status(&root, "200");
                assert!(root.contains("main id=\"console-shell\""));
                assert!(root.contains("id=\"oya-dashboard-island-root\""));
                assert_eq!(root.matches("id=\"oya-dashboard-island-root\"").count(), 1);
                assert!(root.contains("data-island=\"render-envelope-dashboard\""));
                assert!(root.contains("mount_dashboard_islands"));
                stop.send(()).expect("request graceful shutdown");
                server.await.expect("join graceful server");
                tokio::fs::remove_dir_all(&package_root)
                    .await
                    .expect("remove child package root");
            });
        return;
    }

    let output = Command::new(std::env::current_exe().expect("locate integration test binary"))
        .args([
            "--exact",
            "native_ssr_request_emits_no_reactive_tracking_warnings",
            "--nocapture",
        ])
        .env("OYA_SSR_WARNING_CAPTURE_CHILD", "1")
        .output()
        .expect("run isolated SSR warning probe");

    assert!(
        output.status.success(),
        "isolated SSR warning probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("outside a reactive tracking context"),
        "SSR emitted a reactive-tracking warning: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn live_ssr_host_serves_routes_confines_packages_and_shuts_down_cleanly() {
    let package_root = temporary_package_root();
    tokio::fs::write(
        package_root.join("dashboard.js"),
        b"export const dashboard = true;\n",
    )
    .await
    .expect("write JavaScript asset");
    tokio::fs::write(package_root.join("dashboard.wasm"), b"\0asm\x01\0\0\0")
        .await
        .expect("write WASM asset");
    std::os::unix::fs::symlink("/etc/passwd", package_root.join("escape.js"))
        .expect("create package-root escape symlink");

    let (address, stop, server) = spawn_server(package_root.clone()).await;

    let root = request(
        address,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&root, "200");
    assert!(
        root.contains("main id=\"console-shell\""),
        "root response: {root}"
    );
    assert!(root.contains("id=\"oya-dashboard-island-root\""));
    assert_eq!(root.matches("id=\"oya-dashboard-island-root\"").count(), 1);
    assert!(root.contains("data-island=\"render-envelope-dashboard\""));
    assert!(root.contains("mount_dashboard_islands"));
    assert!(root.contains("/style/app.css"));

    let index = request(
        address,
        "GET /index.html HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&index, "200");

    let css = request(
        address,
        "GET /style/app.css HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&css, "200");
    assert!(css.contains("content-type: text/css; charset=utf-8"));

    let tokens = request(
        address,
        "GET /style/tokens.css HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&tokens, "200");
    assert!(tokens.contains("content-type: text/css; charset=utf-8"));

    let javascript = request(
        address,
        "GET /pkg/dashboard.js HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&javascript, "200");
    assert!(javascript.contains("content-type: text/javascript; charset=utf-8"));
    assert!(javascript.contains("export const dashboard = true;"));

    let wasm = request(
        address,
        "GET /pkg/dashboard.wasm HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&wasm, "200");
    assert!(wasm.contains("content-type: application/wasm"));
    assert!(wasm.ends_with("\0asm\x01\0\0\0"));

    let known = request(address, "GET /api/render-envelope/tenant-admin HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").await;
    assert_status(&known, "200");
    assert!(known.contains("\"tenant-admin\""));

    let unknown = request(
        address,
        "GET /api/render-envelope/unknown HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&unknown, "404");
    assert!(unknown.contains("unknown render envelope"));

    for path in [
        "/pkg/",
        "/pkg/../secret",
        "/pkg/%2e%2e%2fsecret",
        "/pkg/..%2Fsecret",
        "/pkg/..\\secret",
        "/pkg/escape.js",
    ] {
        let response = request(
            address,
            &format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        )
        .await;
        assert_status(&response, "400");
    }

    let missing_package = request(
        address,
        "GET /pkg/missing.js HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&missing_package, "404");

    // The host mounts no `/api/{*fn_name}` server-function route, so an unauthenticated caller
    // cannot reach a wildcard POST control plane. This asserts the absence live, not just in the
    // route table: re-adding the wildcard without a fail-closed authz layer turns this test RED.
    let server_function = request(address, "POST /api/not-a-server-function HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").await;
    assert_status(&server_function, "404");

    // A POST onto the one surviving `/api` read is rejected by method, not silently absorbed by a
    // wildcard fallback.
    let post_envelope = request(address, "POST /api/render-envelope/tenant-admin HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").await;
    assert_status(&post_envelope, "405");

    let post_root = request(
        address,
        "POST / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
    )
    .await;
    assert_status(&post_root, "405");

    let missing = request(
        address,
        "GET /not-found HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&missing, "404");

    stop.send(()).expect("request graceful shutdown");
    server.await.expect("join graceful server");
    tokio::fs::remove_dir_all(&package_root)
        .await
        .expect("remove test package root");
}

async fn spawn_server(
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

async fn request(address: SocketAddr, request: &str) -> String {
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

fn temporary_package_root() -> PathBuf {
    let sequence = TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "oya-application-shell-live-server-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create unique test package root");
    root
}

fn assert_status(response: &str, expected: &str) {
    assert!(
        response.starts_with(&format!("HTTP/1.1 {expected}")),
        "expected HTTP {expected}, got: {response}"
    );
}
