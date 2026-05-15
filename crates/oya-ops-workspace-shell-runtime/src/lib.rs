//! Composition root for the ops workspace-shell cell.
//!
//! Layer 6 of the hyper foundation per ADR-0090. Wires the REST handlers
//! (`oya-ops-workspace-shell-rest`) through `oya-http-router-kernel` +
//! `oya-http-middleware-kernel` + `oya-http-runtime-hyper-adapter` into a
//! hyper service ready for `tokio::main` boot.
//!
//! Type migration note (per ADR-0092): kernel types renamed from
//! `HyperRequest`/`HyperResponse` to `HttpRequest`/`HttpResponse`. Body
//! type is now `Vec<u8>` end-to-end at this layer; the hyper adapter
//! converts to/from `bytes::Bytes` at its boundary so this runtime no
//! longer needs to depend on `bytes`.

use std::sync::{Arc, RwLock};

use oya_http_middleware_kernel::MiddlewareChain;
use oya_http_router_kernel::{HttpMethod, Router};
use oya_http_runtime_hyper_adapter::{HttpRequest, HttpResponse, SyncHandler};
use oya_ops_workspace_shell_application::{
    ListAllSurfacesUseCase, ListLiveSurfacesUseCase, ShellHealthUseCase,
};
use oya_ops_workspace_shell_kernel::{
    InMemorySurfaceCatalog, SurfaceCatalogPort, SurfaceState, VisibilityTier,
};
use oya_ops_workspace_shell_rest::{
    LIST_ALL_SURFACES_ROUTE, LIST_LIVE_SURFACES_ROUTE, SHELL_HEALTH_ROUTE,
};

/// Shared catalog state. Wrapped in `RwLock` so handlers can clone-snapshot
/// without mutating shared state. Per-cell composition swaps this for a
/// durable port implementation once that crate lands (Layer 4-ish).
pub type SharedCatalog = Arc<RwLock<InMemorySurfaceCatalog>>;

/// Build the Router used by the workspace-shell cell.
pub fn build_router(catalog: SharedCatalog) -> Router<SyncHandler> {
    let mut router: Router<SyncHandler> = Router::new();

    // GET /workspace — Live + principal-tier-scoped.
    let cat_live = catalog.clone();
    router
        .route(
            HttpMethod::Get,
            LIST_LIVE_SURFACES_ROUTE,
            Arc::new(move |_req: HttpRequest| -> HttpResponse {
                let snapshot = cat_live.read().expect("catalog poisoned").clone();
                let response =
                    ListLiveSurfacesUseCase::new(snapshot).execute(VisibilityTier::InternalPublic);
                let surfaces: Vec<oya_ops_workspace_shell_adapter::WireSurface> = response
                    .surfaces
                    .iter()
                    .map(oya_ops_workspace_shell_adapter::WireSurface::from_kernel)
                    .collect();
                let count = surfaces.len();
                let wire =
                    oya_ops_workspace_shell_adapter::WireSurfaceListResponse { surfaces, count };
                json_response(&surface_list_json(&wire))
            }),
        )
        .expect("LIST_LIVE_SURFACES_ROUTE register failed");

    // GET /workspace/api/v1/surfaces — admin-only catalog dump.
    let cat_all = catalog.clone();
    router
        .route(
            HttpMethod::Get,
            LIST_ALL_SURFACES_ROUTE,
            Arc::new(move |_req: HttpRequest| -> HttpResponse {
                let snapshot = cat_all.read().expect("catalog poisoned").clone();
                let response = ListAllSurfacesUseCase::new(snapshot).execute(None, None);
                let surfaces: Vec<oya_ops_workspace_shell_adapter::WireSurface> = response
                    .surfaces
                    .iter()
                    .map(oya_ops_workspace_shell_adapter::WireSurface::from_kernel)
                    .collect();
                let count = surfaces.len();
                let wire =
                    oya_ops_workspace_shell_adapter::WireSurfaceListResponse { surfaces, count };
                json_response(&surface_list_json(&wire))
            }),
        )
        .expect("LIST_ALL_SURFACES_ROUTE register failed");

    // GET /workspace/api/v1/health
    let cat_health = catalog.clone();
    router
        .route(
            HttpMethod::Get,
            SHELL_HEALTH_ROUTE,
            Arc::new(move |_req: HttpRequest| -> HttpResponse {
                let snapshot = cat_health.read().expect("catalog poisoned").clone();
                let response =
                    ShellHealthUseCase::new(snapshot, env!("CARGO_PKG_VERSION"), None).execute();
                json_response(&format!(
                    "{{\"status\":\"{}\",\"surface_count\":{},\"version\":\"{}\"}}",
                    response.status, response.surface_count, response.version
                ))
            }),
        )
        .expect("SHELL_HEALTH_ROUTE register failed");

    router
}

/// Empty middleware chain seed. Cedar / tenant / telemetry / deadline middlewares
/// land in slice K'' and are pushed here before the binary calls serve().
pub fn build_chain() -> MiddlewareChain<HttpRequest, HttpResponse> {
    MiddlewareChain::new()
}

/// Pre-seed a dev catalog with one example surface so the cell can answer
/// queries out of the box. Real composition root reads persistent state.
pub fn build_dev_catalog() -> SharedCatalog {
    let mut catalog = InMemorySurfaceCatalog::new();
    let _ = catalog.register_surface(oya_ops_workspace_shell_kernel::Surface {
        id: oya_ops_workspace_shell_kernel::SurfaceId("docs-portal".into()),
        canonical_route: "/workspace/docs".into(),
        visibility_tier: VisibilityTier::InternalPublic,
        state: SurfaceState::ReservedComingSoon,
        owning_bc_id: "ops/docs-portal".into(),
        cedar_fragments: vec!["ops-internal-public".into()],
        openapi_contract: Some("contracts/ops-docs-v1.openapi.yaml".into()),
        retired_redirects_to: None,
    });
    Arc::new(RwLock::new(catalog))
}

fn json_response(body: &str) -> HttpResponse {
    HttpResponse::new(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string().into_bytes())
}

fn surface_list_json(
    response: &oya_ops_workspace_shell_adapter::WireSurfaceListResponse,
) -> String {
    // Minimal hand-rolled JSON; no serde per 0-to-minimal-deps policy.
    let surfaces = response
        .surfaces
        .iter()
        .map(|s| {
            format!(
                "{{\"id\":\"{}\",\"canonical_route\":\"{}\",\"visibility_tier\":\"{}\",\"state\":\"{}\",\"owning_bc_id\":\"{}\"}}",
                escape_json(&s.id),
                escape_json(&s.canonical_route),
                s.visibility_tier,
                s.state,
                escape_json(&s.owning_bc_id),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"surfaces\":[{}],\"count\":{}}}",
        surfaces, response.count
    )
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_http_router_kernel::HttpMethod;
    use oya_http_runtime_hyper_adapter::dispatch;
    use std::collections::BTreeMap;

    fn mock_request(method: HttpMethod, path: &str) -> HttpRequest {
        HttpRequest {
            method,
            path: path.to_string(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        }
    }

    #[test]
    fn build_router_registers_three_routes() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog);
        assert_eq!(router.count(), 3);
    }

    #[test]
    fn list_live_surfaces_returns_200_json() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog);
        let chain = build_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/workspace"), &router, &chain);
        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        // Seeded surface is ReservedComingSoon → list_live filters Live-only, so 0 results.
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":0"));
    }

    #[test]
    fn list_all_surfaces_includes_seeded_surface() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog);
        let chain = build_chain();
        let response = dispatch(
            mock_request(HttpMethod::Get, "/workspace/api/v1/surfaces"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 200);
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":1"));
        assert!(body_text.contains("docs-portal"));
    }

    #[test]
    fn shell_health_returns_status_healthy() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog);
        let chain = build_chain();
        let response = dispatch(
            mock_request(HttpMethod::Get, "/workspace/api/v1/health"),
            &router,
            &chain,
        );
        assert_eq!(response.status, 200);
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"status\":\"healthy\""));
        assert!(body_text.contains("\"surface_count\":1"));
    }

    #[test]
    fn unknown_route_returns_404() {
        let catalog = build_dev_catalog();
        let router = build_router(catalog);
        let chain = build_chain();
        let response = dispatch(mock_request(HttpMethod::Get, "/nope"), &router, &chain);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn shared_catalog_mutation_visible_to_handlers() {
        let catalog = build_dev_catalog();
        // Register a second surface in the shared catalog.
        catalog
            .write()
            .unwrap()
            .register_surface(oya_ops_workspace_shell_kernel::Surface {
                id: oya_ops_workspace_shell_kernel::SurfaceId("tenant-mgmt".into()),
                canonical_route: "/workspace/tenants".into(),
                visibility_tier: VisibilityTier::InternalPublic,
                state: SurfaceState::Live,
                owning_bc_id: "ops/tenant-mgmt".into(),
                cedar_fragments: vec![],
                openapi_contract: None,
                retired_redirects_to: None,
            })
            .unwrap();
        let router = build_router(catalog);
        let chain = build_chain();
        let response = dispatch(
            mock_request(HttpMethod::Get, "/workspace/api/v1/surfaces"),
            &router,
            &chain,
        );
        let body_text = std::str::from_utf8(&response.body).unwrap();
        assert!(body_text.contains("\"count\":2"));
        assert!(body_text.contains("tenant-mgmt"));
    }
}
