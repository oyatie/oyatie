//! Ops workspace-shell application — use-case orchestration layer.
//!
//! Per ADR-0056 this crate depends only inward on the kernel. REST/OpenAPI
//! wire projection stays in the presentation/adapter boundary, not here.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_workspace_shell_kernel::{
    Surface, SurfaceCatalogError, SurfaceCatalogPort, SurfaceId, SurfaceState,
    SurfaceStateTransitionOutcome, VisibilityTier,
};

/// Application response for surface-list use cases before wire projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceList {
    pub surfaces: Vec<Surface>, // data_class: INTERNAL_ONLY
    pub count: usize,           // data_class: INTERNAL_ONLY
}

impl SurfaceList {
    fn new(surfaces: Vec<Surface>) -> Self {
        let count = surfaces.len();
        Self { surfaces, count }
    }
}

/// Application health response before wire projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellHealth {
    pub status: String,          // data_class: INTERNAL_ONLY
    pub surface_count: usize,    // data_class: INTERNAL_ONLY
    pub version: String,         // data_class: INTERNAL_ONLY
    pub cell_id: Option<String>, // data_class: INTERNAL_ONLY
}

/// GET /workspace — Live + principal-tier-scoped surfaces.
pub struct ListLiveSurfacesUseCase<P: SurfaceCatalogPort> {
    catalog: P, // data_class: INTERNAL_ONLY
}

impl<P: SurfaceCatalogPort> ListLiveSurfacesUseCase<P> {
    pub fn new(catalog: P) -> Self {
        Self { catalog }
    }

    pub fn execute(&self, principal_tier: VisibilityTier) -> SurfaceList {
        let surfaces = self
            .catalog
            .list_surfaces()
            .into_iter()
            .filter(|s| matches!(s.state, SurfaceState::Live))
            .filter(|s| tier_allows(principal_tier, s.visibility_tier))
            .cloned()
            .collect();
        SurfaceList::new(surfaces)
    }

    pub fn into_inner(self) -> P {
        self.catalog
    }
}

/// GET /workspace/api/v1/surfaces — full catalog (admin-only at route level).
pub struct ListAllSurfacesUseCase<P: SurfaceCatalogPort> {
    catalog: P, // data_class: INTERNAL_ONLY
}

impl<P: SurfaceCatalogPort> ListAllSurfacesUseCase<P> {
    pub fn new(catalog: P) -> Self {
        Self { catalog }
    }

    pub fn execute(
        &self,
        state_filter: Option<SurfaceState>,
        tier_filter: Option<VisibilityTier>,
    ) -> SurfaceList {
        let surfaces = self
            .catalog
            .list_surfaces()
            .into_iter()
            .filter(|s| state_filter.map(|st| s.state == st).unwrap_or(true))
            .filter(|s| tier_filter.map(|t| s.visibility_tier == t).unwrap_or(true))
            .cloned()
            .collect();
        SurfaceList::new(surfaces)
    }
}

/// Surface registration (mutating; admin-only at route level).
pub struct RegisterSurfaceUseCase<P: SurfaceCatalogPort> {
    catalog: P, // data_class: INTERNAL_ONLY
}

impl<P: SurfaceCatalogPort> RegisterSurfaceUseCase<P> {
    pub fn new(catalog: P) -> Self {
        Self { catalog }
    }

    pub fn execute(&mut self, surface: Surface) -> Result<(), SurfaceCatalogError> {
        self.catalog.register_surface(surface)
    }

    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.catalog
    }
}

/// Surface state transition (mutating; internal-sre+).
pub struct FlipSurfaceStateUseCase<P: SurfaceCatalogPort> {
    catalog: P, // data_class: INTERNAL_ONLY
}

impl<P: SurfaceCatalogPort> FlipSurfaceStateUseCase<P> {
    pub fn new(catalog: P) -> Self {
        Self { catalog }
    }

    pub fn execute(
        &mut self,
        id: &SurfaceId,
        new_state: SurfaceState,
    ) -> Result<SurfaceStateTransitionOutcome, SurfaceCatalogError> {
        self.catalog.flip_state(id, new_state)
    }
}

/// GET /workspace/api/v1/health.
pub struct ShellHealthUseCase<P: SurfaceCatalogPort> {
    catalog: P,
    version: String,
    cell_id: Option<String>,
}

impl<P: SurfaceCatalogPort> ShellHealthUseCase<P> {
    pub fn new(catalog: P, version: impl Into<String>, cell_id: Option<String>) -> Self {
        Self {
            catalog,
            version: version.into(),
            cell_id,
        }
    }

    pub fn execute(&self) -> ShellHealth {
        ShellHealth {
            status: "healthy".to_string(),
            surface_count: self.catalog.count(),
            version: self.version.clone(),
            cell_id: self.cell_id.clone(),
        }
    }
}

/// Principal-tier visibility ordering: principal sees their tier and every
/// tier strictly less restrictive than theirs.
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
    use console_workspace_shell_kernel::InMemorySurfaceCatalog;

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

    fn populated_catalog() -> InMemorySurfaceCatalog {
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
                "live-internal",
                "/workspace/live-internal",
                SurfaceState::Live,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        catalog
            .register_surface(surface(
                "soon",
                "/workspace/soon",
                SurfaceState::ReservedComingSoon,
                VisibilityTier::InternalPublic,
            ))
            .unwrap();
        catalog
    }

    #[test]
    fn list_live_filters_by_state_and_tier() {
        let use_case = ListLiveSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(VisibilityTier::Public);
        assert_eq!(response.count, 1);
        assert_eq!(response.surfaces[0].id, SurfaceId("live-pub".into()));
    }

    #[test]
    fn list_live_internal_principal_sees_more() {
        let use_case = ListLiveSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(VisibilityTier::InternalPublic);
        assert_eq!(response.count, 2);
    }

    #[test]
    fn list_all_no_filter_returns_everything() {
        let use_case = ListAllSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(None, None);
        assert_eq!(response.count, 3);
    }

    #[test]
    fn list_all_state_filter() {
        let use_case = ListAllSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(Some(SurfaceState::Live), None);
        assert_eq!(response.count, 2);
    }

    #[test]
    fn list_all_tier_filter() {
        let use_case = ListAllSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(None, Some(VisibilityTier::InternalPublic));
        assert_eq!(response.count, 2);
    }

    #[test]
    fn register_use_case_inserts() {
        let mut use_case = RegisterSurfaceUseCase::new(InMemorySurfaceCatalog::new());
        let s = surface(
            "x",
            "/workspace/x",
            SurfaceState::ReservedComingSoon,
            VisibilityTier::InternalPublic,
        );
        assert!(use_case.execute(s).is_ok());
        assert_eq!(use_case.inner_mut().count(), 1);
    }

    #[test]
    fn register_duplicate_errors() {
        let mut use_case = RegisterSurfaceUseCase::new(populated_catalog());
        let s = surface(
            "live-pub",
            "/workspace/dup",
            SurfaceState::ReservedComingSoon,
            VisibilityTier::Public,
        );
        let result = use_case.execute(s);
        assert!(matches!(result, Err(SurfaceCatalogError::DuplicateId(_))));
    }

    #[test]
    fn flip_state_use_case_propagates_changed() {
        let mut use_case = FlipSurfaceStateUseCase::new(populated_catalog());
        let outcome = use_case
            .execute(&SurfaceId("soon".into()), SurfaceState::Live)
            .unwrap();
        assert_eq!(
            outcome,
            console_workspace_shell_kernel::SurfaceStateTransitionOutcome::Changed {
                from: SurfaceState::ReservedComingSoon,
                to: SurfaceState::Live,
            }
        );
    }

    #[test]
    fn flip_state_use_case_propagates_unchanged() {
        let mut use_case = FlipSurfaceStateUseCase::new(populated_catalog());
        let outcome = use_case
            .execute(&SurfaceId("soon".into()), SurfaceState::ReservedComingSoon)
            .unwrap();
        assert_eq!(
            outcome,
            console_workspace_shell_kernel::SurfaceStateTransitionOutcome::Unchanged {
                state: SurfaceState::ReservedComingSoon,
            }
        );
    }

    #[test]
    fn flip_state_invalid_errors() {
        let mut use_case = FlipSurfaceStateUseCase::new(populated_catalog());
        let result = use_case.execute(
            &SurfaceId("live-pub".into()),
            SurfaceState::ReservedComingSoon,
        );
        assert!(matches!(
            result,
            Err(SurfaceCatalogError::InvalidStateTransition { .. })
        ));
    }

    #[test]
    fn shell_health_returns_count() {
        let use_case =
            ShellHealthUseCase::new(populated_catalog(), "v0.1.0", Some("cell-a".into()));
        let response = use_case.execute();
        assert_eq!(response.status, "healthy");
        assert_eq!(response.surface_count, 3);
        assert_eq!(response.version, "v0.1.0");
        assert_eq!(response.cell_id.as_deref(), Some("cell-a"));
    }
}
