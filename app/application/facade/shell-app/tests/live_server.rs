#![cfg(unix)]

use std::process::Command;

mod live_server_support;
use live_server_support::*;

#[test]
fn public_component_props_and_builders_remain_available() {
    use application_shell_app::app::{
        AppProps, AppPropsBuilder, DashboardIslandProps, DashboardIslandPropsBuilder,
    };

    let app: AppPropsBuilder = AppProps::builder();
    let _: AppProps = app.build();
    let dashboard: DashboardIslandPropsBuilder = DashboardIslandProps::builder();
    let _: DashboardIslandProps = dashboard.build();
}

#[test]
fn post_bootstrap_mount_contract_preserves_one_host_and_one_island_root() {
    let app_source = include_str!("../src/app/chrome.rs");
    let island_source = include_str!("../src/app/dashboard.rs");
    let bootstrap_source = include_str!("../src/lib.rs");

    // `mount_dashboard_island_by_id` clears this stable host, then pinned Leptos 0.8.19
    // `mount_to(parent, DashboardIsland)` calls `mountable.mount(&parent, None)`: it appends the
    // component root beneath the host rather than replacing the host itself. The two source
    // contracts below prevent duplicate IDs after bootstrap.
    assert!(app_source.contains("<div id=crate::DASHBOARD_MOUNT_HOST_ID>"));
    assert!(bootstrap_source.contains("parent.set_inner_html(\"\");"));
    assert!(bootstrap_source.contains("mount_to(parent, DashboardIsland)"));
    assert!(!island_source.contains("DASHBOARD_MOUNT_HOST_ID"));
}

#[test]
fn native_ssr_request_emits_no_reactive_tracking_warnings() {
    if std::env::var_os("OYATIE_SSR_WARNING_CAPTURE_CHILD").is_some() {
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
                assert!(root.contains("id=\"dashboard-island-root\""));
                assert_eq!(root.matches("id=\"dashboard-island-root\"").count(), 1);
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
        .env("OYATIE_SSR_WARNING_CAPTURE_CHILD", "1")
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

#[path = "live_server_support/routes.rs"]
mod routes;
