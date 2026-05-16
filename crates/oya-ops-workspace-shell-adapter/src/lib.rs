//! Ops workspace-shell adapter — projects kernel types onto the OpenAPI 3.2
//! wire schema declared in `contracts/ops-workspace-shell-v1.openapi.yaml`.
//!
//! Pure std-only adapter per ADR-0015: no framework dependencies, no I/O.
//! Owns the kernel→wire projection; rest-layer crate owns transport binding.
//!
//! Wire DTOs mirror OpenAPI `#/components/schemas/*` 1:1 by name and shape
//! so the contract is enforceable: changing the kernel type without changing
//! the projection breaks the build.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_ops_workspace_shell_kernel::{
    Surface, SurfaceCatalogPort, SurfaceId, SurfaceState, VisibilityTier,
};

/// Wire shape mirroring `#/components/schemas/Surface` in
/// `contracts/ops-workspace-shell-v1.openapi.yaml`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireSurface {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub canonical_route: String,              // data_class: INTERNAL_ONLY
    pub visibility_tier: String,              // data_class: INTERNAL_ONLY
    pub state: String,                        // data_class: INTERNAL_ONLY
    pub owning_bc_id: String,                 // data_class: INTERNAL_ONLY
    pub cedar_fragments: Vec<String>,         // data_class: INTERNAL_ONLY
    pub openapi_contract: Option<String>,     // data_class: INTERNAL_ONLY
    pub retired_redirects_to: Option<String>, // data_class: INTERNAL_ONLY
}

impl WireSurface {
    pub fn from_kernel(surface: &Surface) -> Self {
        Self {
            id: surface.id.0.clone(),
            canonical_route: surface.canonical_route.clone(),
            visibility_tier: surface.visibility_tier.name().to_string(),
            state: surface.state.name().to_string(),
            owning_bc_id: surface.owning_bc_id.clone(),
            cedar_fragments: surface.cedar_fragments.clone(),
            openapi_contract: surface.openapi_contract.clone(),
            retired_redirects_to: surface.retired_redirects_to.as_ref().map(|id| id.0.clone()),
        }
    }
}

/// Wire shape mirroring `#/components/schemas/SurfaceListResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireSurfaceListResponse {
    pub surfaces: Vec<WireSurface>, // data_class: INTERNAL_ONLY
    pub count: usize,               // data_class: INTERNAL_ONLY
}

impl WireSurfaceListResponse {
    pub fn from_catalog<P: SurfaceCatalogPort>(catalog: &P) -> Self {
        let surfaces: Vec<WireSurface> = catalog
            .list_surfaces()
            .into_iter()
            .map(WireSurface::from_kernel)
            .collect();
        let count = surfaces.len();
        Self { surfaces, count }
    }

    /// Filter to surfaces visible at or below the principal's tier.
    /// Models the GET /workspace endpoint (Live-only, principal-scoped).
    pub fn live_visible<P: SurfaceCatalogPort>(
        catalog: &P,
        principal_tier: VisibilityTier,
    ) -> Self {
        let surfaces: Vec<WireSurface> = catalog
            .list_surfaces()
            .into_iter()
            .filter(|s| matches!(s.state, SurfaceState::Live))
            .filter(|s| tier_allows(principal_tier, s.visibility_tier))
            .map(WireSurface::from_kernel)
            .collect();
        let count = surfaces.len();
        Self { surfaces, count }
    }
}

/// Wire shape mirroring `#/components/schemas/HealthResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireHealthResponse {
    pub status: String, // healthy | degraded | unhealthy
    pub surface_count: usize,
    pub version: String,
    pub cell_id: Option<String>,
}

impl WireHealthResponse {
    pub fn from_catalog<P: SurfaceCatalogPort>(
        catalog: &P,
        version: impl Into<String>,
        cell_id: Option<String>,
    ) -> Self {
        Self {
            status: "healthy".to_string(),
            surface_count: catalog.count(),
            version: version.into(),
            cell_id,
        }
    }
}

/// Wire shape mirroring `#/components/schemas/CedarDenyResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireCedarDenyResponse {
    pub error: String,
    pub principal_role: String,
    pub resource: String,
    pub cedar_fragment_denied_by: Option<String>,
}

impl WireCedarDenyResponse {
    pub fn from_surface_id(
        id: &SurfaceId,
        principal_role: impl Into<String>,
        cedar_fragment_denied_by: Option<String>,
    ) -> Self {
        Self {
            error: "cedar-deny".into(),
            principal_role: principal_role.into(),
            resource: format!("surface:{}", id.0),
            cedar_fragment_denied_by,
        }
    }
}

/// Principal-tier visibility ordering: principal sees their tier and every
/// tier strictly less restrictive than theirs.
///
/// Order (most → least restrictive):
///   SystemOnly > InternalPrivate > InternalPublic
///                > TenantPrivate > TenantPublic > Public
fn tier_rank(tier: VisibilityTier) -> u8 {
    match tier {
        VisibilityTier::Public => 0,
        VisibilityTier::TenantPublic => 1,
        VisibilityTier::TenantPrivate => 2,
        VisibilityTier::InternalPublic => 3,
        VisibilityTier::InternalPrivate => 4,
        VisibilityTier::SystemOnly => 5,
    }
}

fn tier_allows(principal: VisibilityTier, resource: VisibilityTier) -> bool {
    tier_rank(principal) >= tier_rank(resource)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ops_workspace_shell_kernel::InMemorySurfaceCatalog;

    fn surface(id: &str, route: &str, state: SurfaceState, tier: VisibilityTier) -> Surface {
        Surface {
            id: SurfaceId(id.into()),
            canonical_route: route.into(),
            visibility_tier: tier,
            state,
            owning_bc_id: format!("ops/{id}"),
            cedar_fragments: vec!["ops-internal-public".into()],
            openapi_contract: Some(format!("contracts/{id}.openapi.yaml")),
            retired_redirects_to: None,
        }
    }

    #[test]
    fn wire_surface_round_trip_names() {
        let s = surface(
            "docs-portal",
            "/workspace/docs",
            SurfaceState::Live,
            VisibilityTier::InternalPublic,
        );
        let wire = WireSurface::from_kernel(&s);
        assert_eq!(wire.id, "docs-portal");
        assert_eq!(wire.canonical_route, "/workspace/docs");
        assert_eq!(wire.visibility_tier, "internal-public");
        assert_eq!(wire.state, "live");
        assert_eq!(wire.owning_bc_id, "ops/docs-portal");
        assert_eq!(wire.cedar_fragments, vec!["ops-internal-public"]);
        assert_eq!(
            wire.openapi_contract.as_deref(),
            Some("contracts/docs-portal.openapi.yaml")
        );
        assert!(wire.retired_redirects_to.is_none());
    }

    #[test]
    fn wire_surface_serializes_all_six_tiers() {
        let tiers = [
            (VisibilityTier::Public, "public"),
            (VisibilityTier::TenantPublic, "tenant-public"),
            (VisibilityTier::TenantPrivate, "tenant-private"),
            (VisibilityTier::InternalPublic, "internal-public"),
            (VisibilityTier::InternalPrivate, "internal-private"),
            (VisibilityTier::SystemOnly, "system-only"),
        ];
        for (tier, expected) in tiers {
            let s = surface("x", "/workspace/x", SurfaceState::Live, tier);
            assert_eq!(WireSurface::from_kernel(&s).visibility_tier, expected);
        }
    }

    #[test]
    fn wire_surface_serializes_all_three_states() {
        let states = [
            (SurfaceState::ReservedComingSoon, "reserved-coming-soon"),
            (SurfaceState::Live, "live"),
            (SurfaceState::Retired, "retired"),
        ];
        for (state, expected) in states {
            let s = surface("x", "/workspace/x", state, VisibilityTier::InternalPublic);
            assert_eq!(WireSurface::from_kernel(&s).state, expected);
        }
    }

    #[test]
    fn list_response_count_matches_surfaces_len() {
        let mut catalog = InMemorySurfaceCatalog::new();
        for id in ["a", "b", "c"] {
            catalog
                .register_surface(surface(
                    id,
                    &format!("/workspace/{id}"),
                    SurfaceState::ReservedComingSoon,
                    VisibilityTier::InternalPublic,
                ))
                .unwrap();
        }
        let response = WireSurfaceListResponse::from_catalog(&catalog);
        assert_eq!(response.count, 3);
        assert_eq!(response.surfaces.len(), 3);
    }

    #[test]
    fn live_visible_filters_to_live_only() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::Live,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "b",
                "/workspace/b",
                SurfaceState::ReservedComingSoon,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        let response =
            WireSurfaceListResponse::live_visible(&catalog, VisibilityTier::InternalPublic);
        assert_eq!(response.count, 1);
        assert_eq!(response.surfaces[0].id, "a");
    }

    #[test]
    fn live_visible_excludes_higher_tier() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "low",
                "/workspace/low",
                SurfaceState::Live,
                VisibilityTier::Public,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "high",
                "/workspace/high",
                SurfaceState::Live,
                VisibilityTier::SystemOnly,
            ))
            .unwrap();
        let response = WireSurfaceListResponse::live_visible(&catalog, VisibilityTier::Public);
        // Public principal sees only Public-tier surfaces, not SystemOnly.
        assert_eq!(response.count, 1);
        assert_eq!(response.surfaces[0].id, "low");
    }

    #[test]
    fn live_visible_allows_principal_tier_and_below() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "pub",
                "/workspace/pub",
                SurfaceState::Live,
                VisibilityTier::Public,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "tp",
                "/workspace/tp",
                SurfaceState::Live,
                VisibilityTier::TenantPublic,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "ip",
                "/workspace/ip",
                SurfaceState::Live,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        let response =
            WireSurfaceListResponse::live_visible(&catalog, VisibilityTier::InternalPublic);
        // Internal-public principal sees public, tenant-public, internal-public surfaces.
        // Skips tenant-private (rank 2 < principal rank 3 — actually yes it sees lower).
        // Wait: tenant-private (rank 2) ≤ internal-public (rank 3) → visible.
        // Recheck logic: principal_rank=3, tenant-public rank=1, internal-public rank=3.
        assert_eq!(response.count, 3);
    }

    #[test]
    fn health_response_status_healthy() {
        let catalog = InMemorySurfaceCatalog::new();
        let h = WireHealthResponse::from_catalog(&catalog, "v0.1.0", None);
        assert_eq!(h.status, "healthy");
        assert_eq!(h.surface_count, 0);
        assert_eq!(h.version, "v0.1.0");
        assert!(h.cell_id.is_none());
    }

    #[test]
    fn cedar_deny_response_shape() {
        let resp = WireCedarDenyResponse::from_surface_id(
            &SurfaceId("docs-portal".into()),
            "anonymous",
            Some("ops-internal-public".into()),
        );
        assert_eq!(resp.error, "cedar-deny");
        assert_eq!(resp.principal_role, "anonymous");
        assert_eq!(resp.resource, "surface:docs-portal");
        assert_eq!(
            resp.cedar_fragment_denied_by.as_deref(),
            Some("ops-internal-public")
        );
    }

    #[test]
    fn tier_rank_strict_ordering() {
        assert!(tier_rank(VisibilityTier::Public) < tier_rank(VisibilityTier::TenantPublic));
        assert!(tier_rank(VisibilityTier::TenantPublic) < tier_rank(VisibilityTier::TenantPrivate));
        assert!(
            tier_rank(VisibilityTier::TenantPrivate) < tier_rank(VisibilityTier::InternalPublic)
        );
        assert!(
            tier_rank(VisibilityTier::InternalPublic) < tier_rank(VisibilityTier::InternalPrivate)
        );
        assert!(tier_rank(VisibilityTier::InternalPrivate) < tier_rank(VisibilityTier::SystemOnly));
    }

    #[test]
    fn tier_allows_reflexive() {
        for tier in [
            VisibilityTier::Public,
            VisibilityTier::TenantPublic,
            VisibilityTier::TenantPrivate,
            VisibilityTier::InternalPublic,
            VisibilityTier::InternalPrivate,
            VisibilityTier::SystemOnly,
        ] {
            assert!(tier_allows(tier, tier));
        }
    }
}
