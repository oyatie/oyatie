use super::*;

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
    assert!(root.contains("id=\"dashboard-island-root\""));
    assert_eq!(root.matches("id=\"dashboard-island-root\"").count(), 1);
    assert!(root.contains("data-island=\"render-envelope-dashboard\""));
    assert!(root.contains("mount_dashboard_islands"));
    assert!(root.contains("/style/app.css"));
    assert!(root.contains("CAT-OYATIE-ONTOLOGY"));
    let markup = root.split("<script").next().expect("SSR shell markup");
    assert!(markup.contains("Open ontology sample"));
    assert!(!markup.contains("data-sidepeek-title=\"Ontology\""));
    assert!(markup.contains("ontology-command-console"));
    assert!(markup.contains("data-ontology-action=\"inspect-fact\""));
    assert!(root.contains("panel.focus({ preventScroll: true })"));
    assert!(root.contains("candidate.setAttribute('aria-pressed'"));
    assert!(markup.contains("no tenant or role is verified"));
    assert!(markup.contains(
        "Live review, execution, attestation, sealing and receipt creation are unavailable"
    ));
    assert!(markup.contains("data-governance-unavailable=\"true\" disabled"));
    assert!(markup.contains("id=\"shell-account-title\">Sample workspace</strong>"));
    assert!(markup.contains("id=\"console-title\">Operations samples</h1>"));
    assert_eq!(markup.matches("href=\"#catalog-sample-search\"").count(), 2);
    let form = markup
        .split_once("<form id=\"passkey-form\"")
        .expect("native passkey form")
        .1
        .split_once("</form>")
        .unwrap()
        .0;
    for control in [
        "passkey-sign-in",
        "session-restore",
        "session-logout",
        "passkey-enroll",
        "passkey-invitation",
    ] {
        let tag = form
            .split_once(&format!("id=\"{control}\""))
            .unwrap()
            .1
            .split_once('>')
            .unwrap()
            .0;
        assert!(
            tag.contains(" disabled"),
            "anonymous initial control {control}"
        );
    }
    let live_card = markup
        .split_once("<section id=\"verified-ontology-card\"")
        .expect("separate verified live card")
        .1;
    assert!(live_card.split_once('>').unwrap().0.contains(" hidden"));
    for fact in [
        "policyVersion",
        "servedTenants",
        "projectionLag",
        "poisonedEntries",
    ] {
        assert!(live_card.contains(&format!("data-live-fact=\"{fact}\"></dd>")));
    }

    let availability = request(
        address,
        "GET /api/identity-config HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_status(&availability, "200");
    assert!(availability.contains("\"enabled\":false"));
    for path in [
        "/api/identity/ten_acme/session",
        "/api/ontology-status/ten_acme",
    ] {
        let anonymous = request(address, &format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nCookie: __Host-oya_session=ssn_fixture\r\nConnection: close\r\n\r\n")).await;
        assert_status(&anonymous, "503");
        assert!(anonymous.contains("cache-control: no-store"));
        assert!(!anonymous.contains("\"subject\"") && !anonymous.contains("\"policyVersion\""));
    }
    let compatibility_markup = application_shell_app::app::static_dashboard_html();
    for rendered in [markup, compatibility_markup.as_str()] {
        for destination in ["finance", "workflow", "mail", "evidence"] {
            let target = format!("payroll-{destination}-preview");
            assert_eq!(rendered.matches(&format!("id=\"{target}\"")).count(), 1);
            assert!(rendered.contains(&format!("href=\"#{target}\"")));
        }
        assert_eq!(
            rendered
                .matches("data-payroll-preview-context=\"sample-april-close\"")
                .count(),
            4
        );
        assert!(rendered.contains("sample close progress"));
        assert!(rendered.contains("118 sample employees; not validated"));
        assert!(rendered.contains("Example only; not created"));
        assert!(!rendered.contains("118 employees validated"));
        assert!(!rendered.contains("Receipt REC-PAY-2026-04 staged"));
    }
    let ids = attribute_values(markup, "id")
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(!ids.is_empty(), "SSR shell must expose destination IDs");
    for fragment in attribute_values(markup, "href")
        .into_iter()
        .filter_map(|href| href.strip_prefix('#'))
        .filter(|fragment| !fragment.is_empty())
    {
        assert!(
            ids.contains(fragment),
            "SSR fragment #{fragment} has no destination"
        );
    }
    for label in attribute_values(markup, "aria-labelledby")
        .into_iter()
        .flat_map(str::split_whitespace)
    {
        assert!(
            ids.contains(label),
            "SSR label reference {label} has no destination"
        );
    }

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
