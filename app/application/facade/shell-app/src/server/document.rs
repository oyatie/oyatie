use super::{
    SITE_ROOT, action_inbox, audit_evidence, catalog_navigation, catalog_preview, cloud_resources,
    daily_execution, finance_preview, governance, hr_directory, intelligence_sample, leave_time,
    notifications, passkey_script, policy_navigation, settings_account, workflow_navigation,
    workhub_focus,
};
use leptos::prelude::*;
use std::path::Path;

#[component]
pub(super) fn ServerShell() -> impl IntoView {
    let wasm_package_available = Path::new(SITE_ROOT)
        .join("pkg/application_shell_app.js")
        .exists();
    let island_bootstrap = format!(
        r#"const wasmPackageAvailable = {wasm_package_available};
async function mountDashboardIsland() {{
  if (!wasmPackageAvailable) {{
    console.info('Oyatie console: WASM island package missing from /pkg; serving the server-rendered shell only.');
    return;
  }}
  const wasm = await import('/pkg/application_shell_app.js');
  await wasm.default();
  wasm.mount_dashboard_islands();
}}
mountDashboardIsland();"#
    );

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"Oyatie Cloud/Tenant Control Center"</title>
                <link rel="stylesheet" href="/style/tokens.css" />
                <link rel="stylesheet" href="/style/app.css" />
            </head>
            <body>
                <noscript>"The Oyatie console needs WebAssembly enabled for island hydration; the server-rendered shell remains visible."</noscript>
                <crate::App />
                <script>{catalog_navigation::SCRIPT}</script>
                <script>{action_inbox::SCRIPT}</script>
                <script>{intelligence_sample::script()}</script>
                <script>{policy_navigation::SCRIPT}</script>
                <script>{crate::app::ssr_tab_navigation_script()}</script>
                <script>{hr_directory::SCRIPT}</script>
                <script>{settings_account::script()}</script>
                <script>{notifications::script()}</script>
                <script>{workflow_navigation::SCRIPT}</script>
                <script>{workhub_focus::SCRIPT}</script>
                <script>{daily_execution::SCRIPT}</script>
                <script>{governance::SCRIPT}</script>
                <script>{finance_preview::SCRIPT}</script>
                <script>{leave_time::script()}</script>
                <script>{audit_evidence::SCRIPT}</script>
                <script>{cloud_resources::SCRIPT}</script>
                <script>{catalog_preview::script()}</script>
                <script type="module">{passkey_script::script()}</script>
                <script type="module">{island_bootstrap}</script>
            </body>
        </html>
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use std::path::PathBuf;
    use tower::ServiceExt;

    #[tokio::test]
    async fn served_shell_documents_start_in_html_standards_mode() {
        let routes =
            super::super::router_for_package_root(PathBuf::from("target/document-contract-pkg"));
        for path in ["/", "/index.html"] {
            let response = routes
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/html; charset=utf-8",
                "{path}"
            );
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            let html = std::str::from_utf8(&body).unwrap();
            assert!(
                html.starts_with("<!DOCTYPE html><html lang=\"en\">"),
                "{path} must begin with the HTML doctype before its document element"
            );
            assert_eq!(html.to_ascii_lowercase().matches("<!doctype").count(), 1);
            for unique in [
                "<html lang=\"en\">",
                "<head>",
                "<body>",
                "id=\"oyatie-shell\"",
                "id=\"console-shell\"",
                "id=\"dashboard-island-root\"",
                "href=\"/style/tokens.css\"",
                "href=\"/style/app.css\"",
                "wasm.mount_dashboard_islands();",
            ] {
                assert_eq!(html.matches(unique).count(), 1, "{path}: {unique}");
            }
            assert!(html.contains("id=\"foundry-workspace\""), "{path}");
            assert!(
                html.contains("import('/pkg/application_shell_app.js')"),
                "{path}"
            );
        }
    }
}
