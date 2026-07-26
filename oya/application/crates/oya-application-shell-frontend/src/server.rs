//! Native Axum host for the server-rendered shell.

use std::{
    env,
    future::Future,
    io,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use axum::{
    Router,
    extract::{Path as AxumPath, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use leptos::prelude::*;

use crate::render_envelope_json;

const TOKENS_CSS: &str = include_str!("../style/tokens.css");
const APP_CSS: &str = include_str!("../style/app.css");
const SITE_ROOT: &str = "target/site";

#[derive(Clone)]
struct ServerState {
    package_root: Arc<PathBuf>,
}

/// Runs the local SSR host at `LEPTOS_SITE_ADDR`, `SITE_ADDR`, or `127.0.0.1:3000`.
///
/// # Errors
/// Returns the listener or serving error reported by Tokio/Axum.
pub async fn run() -> io::Result<()> {
    let addr = env::var("LEPTOS_SITE_ADDR")
        .or_else(|_| env::var("SITE_ADDR"))
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Oyatie console shell local dev server listening on http://{addr}");
    serve(listener).await
}

/// Serves the production route graph on an already-bound listener.
///
/// Keeping binding separate lets the bounded smoke test exercise the same serving path without
/// reserving the developer port or leaving a process behind.
///
/// # Errors
/// Returns the serving error reported by Axum.
pub async fn serve(listener: tokio::net::TcpListener) -> io::Result<()> {
    axum::serve(listener, router()).await
}

/// Serves the production route graph until a caller-owned graceful shutdown completes.
///
/// This is the same host path used by the bounded live-server integration test; production
/// callers normally use [`serve`], which intentionally has no synthetic shutdown signal.
pub async fn serve_until_shutdown<F>(
    listener: tokio::net::TcpListener,
    shutdown: F,
) -> io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    serve_router_until_shutdown(listener, router(), shutdown).await
}

/// Serves an explicitly constructed route graph until graceful shutdown.
///
/// This makes the host's bounded integration smoke independent from local build artifacts while
/// retaining the exact Axum/Tokio serving primitive used by [`serve_until_shutdown`].
pub async fn serve_router_until_shutdown<F>(
    listener: tokio::net::TcpListener,
    route_graph: Router,
    shutdown: F,
) -> io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, route_graph)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Builds the native route graph.
///
/// Every route is an explicit, non-mutating read so the SSR shell and its deny-by-default
/// envelope are easy to audit. There is deliberately no `/api/{*fn_name}` server-function route:
/// this crate — and the workspace as a whole — declares zero `#[server]` functions, so mounting
/// `leptos_axum::handle_server_fns` would publish an unauthenticated wildcard POST control plane
/// backed by an empty registry. Re-adding it requires a fail-closed authz layer (verified
/// principal + server-side PDP `decide()`) landed in the same change as the first server function.
#[must_use]
pub fn router() -> Router {
    router_for_package_root(PathBuf::from(SITE_ROOT).join("pkg"))
}

/// Builds the route graph with an explicitly configured, trusted package root.
///
/// The host uses [`router`] in production. This constructor keeps filesystem-confinement
/// coverage independent from the developer's local `target/site` contents.
#[must_use]
pub fn router_for_package_root(package_root: PathBuf) -> Router {
    // Streaming SSR spawns through `any_spawner`, so the native Tokio executor must be installed
    // before the renderer is constructed. Re-initialization is a no-op, hence the discarded result.
    let _ = any_spawner::Executor::init_tokio();
    let render_shell = leptos_axum::render_app_to_stream(ServerShell);

    Router::new()
        .route("/", get(render_shell.clone()))
        .route("/index.html", get(render_shell))
        .route("/style/tokens.css", get(tokens_css))
        .route("/style/app.css", get(app_css))
        .route("/api/render-envelope/{context_id}", get(render_envelope))
        .route("/pkg/", get(package_asset_empty))
        .route("/pkg/{*asset}", get(package_asset))
        .with_state(ServerState {
            package_root: Arc::new(package_root),
        })
}

async fn tokens_css() -> impl IntoResponse {
    css_response(TOKENS_CSS)
}

async fn app_css() -> impl IntoResponse {
    css_response(APP_CSS)
}

async fn render_envelope(AxumPath(context_id): AxumPath<String>) -> Response {
    match render_envelope_json(&context_id) {
        Some(body) => json_response(StatusCode::OK, body),
        None => json_response(
            StatusCode::NOT_FOUND,
            r#"{"error":"unknown render envelope"}"#.to_owned(),
        ),
    }
}

async fn package_asset(
    State(state): State<ServerState>,
    AxumPath(asset): AxumPath<String>,
) -> Response {
    let Ok(path) = confined_package_path(&state.package_root, &asset).await else {
        return plain_response(StatusCode::BAD_REQUEST, "bad request");
    };

    match tokio::fs::read(&path).await {
        Ok(bytes) => binary_response(StatusCode::OK, content_type(&path), bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            plain_response(StatusCode::NOT_FOUND, "not found")
        }
        Err(error) => {
            eprintln!(
                "console shell package asset read failed for {}: {error}",
                path.display()
            );
            plain_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}

async fn package_asset_empty() -> Response {
    plain_response(StatusCode::BAD_REQUEST, "bad request")
}

#[component]
fn ServerShell() -> impl IntoView {
    let wasm_package_available = Path::new(SITE_ROOT)
        .join("pkg/oya_application_shell_frontend.js")
        .exists();
    let island_bootstrap = format!(
        r#"const wasmPackageAvailable = {wasm_package_available};
async function mountDashboardIsland() {{
  if (!wasmPackageAvailable) {{
    console.info('Oyatie console: WASM island package missing from /pkg; serving the server-rendered shell only.');
    return;
  }}
  const wasm = await import('/pkg/oya_application_shell_frontend.js');
  await wasm.default();
  wasm.mount_dashboard_islands();
}}
mountDashboardIsland();"#
    );

    view! {
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
                <script type="module">{island_bootstrap}</script>
            </body>
        </html>
    }
}

fn safe_site_package_path(asset: &str) -> Option<PathBuf> {
    safe_package_path(&PathBuf::from(SITE_ROOT).join("pkg"), asset)
}

fn safe_package_path(package_root: &Path, asset: &str) -> Option<PathBuf> {
    if asset.is_empty() || asset.contains(['%', '\\']) {
        return None;
    }

    let mut path = PathBuf::from(package_root);
    for component in Path::new(asset).components() {
        match component {
            Component::Normal(part) => path.push(part),
            _ => return None,
        }
    }
    Some(path)
}

/// Resolves a lexically-safe asset name only through non-symlink child components.
///
/// The configured package root is trusted host configuration; user input can only select normal
/// descendants. Refusing every existing symlink component keeps a package request from escaping
/// that root even when a developer's output directory contains a hostile or stale link.
async fn confined_package_path(package_root: &Path, asset: &str) -> Result<PathBuf, ()> {
    let path = safe_package_path(package_root, asset).ok_or(())?;
    let mut component_path = package_root.to_path_buf();

    for component in Path::new(asset).components() {
        let Component::Normal(component) = component else {
            return Err(());
        };
        component_path.push(component);
        match tokio::fs::symlink_metadata(&component_path).await {
            Ok(metadata) if metadata.file_type().is_symlink() => return Err(()),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            // Let the subsequent read preserve its useful 500-vs-404 distinction for ordinary
            // filesystem faults; only input traversal and symlink escapes are client errors.
            Err(_) => break,
        }
    }

    Ok(path)
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn css_response(body: &'static str) -> Response {
    text_response(StatusCode::OK, "text/css; charset=utf-8", body)
}

fn json_response(status: StatusCode, body: String) -> Response {
    text_response(status, "application/json; charset=utf-8", body)
}

fn plain_response(status: StatusCode, body: &'static str) -> Response {
    text_response(status, "text/plain; charset=utf-8", body)
}

fn binary_response(status: StatusCode, content_type: &'static str, body: Vec<u8>) -> Response {
    (status, [(header::CONTENT_TYPE, content_type)], body).into_response()
}

fn text_response(
    status: StatusCode,
    content_type: &'static str,
    body: impl Into<String>,
) -> Response {
    (status, [(header::CONTENT_TYPE, content_type)], body.into()).into_response()
}

#[cfg(test)]
mod tests {
    use super::safe_site_package_path;

    #[test]
    fn package_paths_reject_traversal_and_only_accept_normal_components() {
        assert!(safe_site_package_path("oya_application_shell_frontend.js").is_some());
        assert!(safe_site_package_path("nested/module.wasm").is_some());
        assert!(safe_site_package_path("../secret").is_none());
        assert!(safe_site_package_path("..%2Fsecret").is_none());
        assert!(safe_site_package_path("%2e%2e%2fsecret").is_none());
        assert!(safe_site_package_path(r"..\secret").is_none());
        assert!(safe_site_package_path("/etc/passwd").is_none());
        assert!(safe_site_package_path(".").is_none());
        assert!(safe_site_package_path("").is_none());
    }
}
