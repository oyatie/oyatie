//! Framework-free REST boundary for the ops workspace-shell BC.
//!
//! Hyper service bindings are intentionally deferred to the runtime composition
//! root per the LTS-dependency-enforcement directive (2026-05-12). Hyper is
//! the canonical workspace HTTP backbone (user-issued 2026-05-14: "hyper
//! everywhere; thats our backbone"). Keeping this crate std-only means we
//! own the OpenAPI-aligned request/response shapes + handler functions today
//! and bind hyper services in the runtime crate.
//!
//! Route constants here MUST stay 1:1 with paths in
//! `contracts/ops-workspace-shell-v1.openapi.yaml`. Future lane
//! `lean-a-openapi-rest-route-parity` will enforce this.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_workspace_shell_adapter::{WireHealthResponse, WireSurface, WireSurfaceListResponse};
use console_workspace_shell_kernel::{SurfaceCatalogPort, SurfaceState, VisibilityTier};
use console_workspace_shell_usecase::{
    ListAllSurfacesUseCase, ListLiveSurfacesUseCase, ShellHealthUseCase,
};

pub const LIST_LIVE_SURFACES_ROUTE: &str = "/workspace";
pub const LIST_ALL_SURFACES_ROUTE: &str = "/workspace/api/v1/surfaces";
pub const SHELL_HEALTH_ROUTE: &str = "/workspace/api/v1/health";

/// HTTP-method constants matching OpenAPI operations.
pub const LIST_LIVE_SURFACES_METHOD: &str = "GET";
pub const LIST_ALL_SURFACES_METHOD: &str = "GET";
pub const SHELL_HEALTH_METHOD: &str = "GET";

/// Request shape for GET /workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListLiveSurfacesRequest {
    pub principal_tier: VisibilityTier, // data_class: INTERNAL_ONLY
}

/// Request shape for GET /workspace/api/v1/surfaces.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ListAllSurfacesRequest {
    pub state: Option<SurfaceState>, // data_class: INTERNAL_ONLY
    pub visibility_tier: Option<VisibilityTier>, // data_class: INTERNAL_ONLY
}

/// Request shape for GET /workspace/api/v1/health.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellHealthRequest;

/// REST handler: GET /workspace (Live + principal-tier-scoped).
pub fn list_live_surfaces<P: SurfaceCatalogPort>(
    catalog: P,
    request: ListLiveSurfacesRequest,
) -> WireSurfaceListResponse {
    let result = ListLiveSurfacesUseCase::new(catalog).execute(request.principal_tier);
    let surfaces: Vec<WireSurface> = result
        .surfaces
        .iter()
        .map(WireSurface::from_kernel)
        .collect();
    let count = surfaces.len();
    WireSurfaceListResponse { surfaces, count }
}

/// REST handler: GET /workspace/api/v1/surfaces (admin-only).
pub fn list_all_surfaces<P: SurfaceCatalogPort>(
    catalog: P,
    request: ListAllSurfacesRequest,
) -> WireSurfaceListResponse {
    let result =
        ListAllSurfacesUseCase::new(catalog).execute(request.state, request.visibility_tier);
    let surfaces: Vec<WireSurface> = result
        .surfaces
        .iter()
        .map(WireSurface::from_kernel)
        .collect();
    let count = surfaces.len();
    WireSurfaceListResponse { surfaces, count }
}

/// REST handler: GET /workspace/api/v1/health.
pub fn shell_health<P: SurfaceCatalogPort>(
    catalog: P,
    _request: ShellHealthRequest,
    version: impl Into<String>,
    cell_id: Option<String>,
) -> WireHealthResponse {
    let result = ShellHealthUseCase::new(catalog, version, cell_id).execute();
    WireHealthResponse {
        status: result.status,
        surface_count: result.surface_count,
        version: result.version,
        cell_id: result.cell_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console_workspace_shell_kernel::{InMemorySurfaceCatalog, Surface, SurfaceId};

    fn surface(id: &str, route: &str, state: SurfaceState, tier: VisibilityTier) -> Surface {
        Surface {
            id: SurfaceId(id.into()),
            canonical_route: route.into(),
            visibility_tier: tier,
            state,
            owning_bc_id: format!("ops/{id}"),
            cedar_fragments: vec![],
            openapi_contract: None,
            retired_redirects_to: None,
        }
    }

    fn populated() -> InMemorySurfaceCatalog {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "live-pub",
                "/workspace/live-pub",
                SurfaceState::Live,
                VisibilityTier::Public,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "live-int",
                "/workspace/live-int",
                SurfaceState::Live,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "soon",
                "/workspace/soon",
                SurfaceState::ReservedComingSoon,
                VisibilityTier::Public,
            ))
            .unwrap();
        catalog
    }

    #[test]
    fn routes_match_openapi_paths() {
        // These must stay 1:1 with `contracts/ops-workspace-shell-v1.openapi.yaml`.
        assert_eq!(LIST_LIVE_SURFACES_ROUTE, "/workspace");
        assert_eq!(LIST_ALL_SURFACES_ROUTE, "/workspace/api/v1/surfaces");
        assert_eq!(SHELL_HEALTH_ROUTE, "/workspace/api/v1/health");
    }

    #[test]
    fn http_methods_match_openapi() {
        assert_eq!(LIST_LIVE_SURFACES_METHOD, "GET");
        assert_eq!(LIST_ALL_SURFACES_METHOD, "GET");
        assert_eq!(SHELL_HEALTH_METHOD, "GET");
    }

    #[test]
    fn list_live_surfaces_public_tier() {
        let response = list_live_surfaces(
            populated(),
            ListLiveSurfacesRequest {
                principal_tier: VisibilityTier::Public,
            },
        );
        assert_eq!(response.count, 1);
        assert_eq!(response.surfaces[0].id, "live-pub");
    }

    #[test]
    fn list_live_surfaces_internal_tier_sees_more() {
        let response = list_live_surfaces(
            populated(),
            ListLiveSurfacesRequest {
                principal_tier: VisibilityTier::InternalPublic,
            },
        );
        assert_eq!(response.count, 2);
    }

    #[test]
    fn list_all_surfaces_no_filter() {
        let response = list_all_surfaces(populated(), ListAllSurfacesRequest::default());
        assert_eq!(response.count, 3);
    }

    #[test]
    fn list_all_surfaces_state_filter() {
        let response = list_all_surfaces(
            populated(),
            ListAllSurfacesRequest {
                state: Some(SurfaceState::Live),
                visibility_tier: None,
            },
        );
        assert_eq!(response.count, 2);
    }

    #[test]
    fn list_all_surfaces_tier_filter() {
        let response = list_all_surfaces(
            populated(),
            ListAllSurfacesRequest {
                state: None,
                visibility_tier: Some(VisibilityTier::Public),
            },
        );
        assert_eq!(response.count, 2);
    }

    #[test]
    fn shell_health_includes_surface_count() {
        let response = shell_health(populated(), ShellHealthRequest, "v0.1.0", None);
        assert_eq!(response.status, "healthy");
        assert_eq!(response.surface_count, 3);
        assert_eq!(response.version, "v0.1.0");
    }
}
