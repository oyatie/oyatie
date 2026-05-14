//! Ops workspace-shell application — orchestration layer per Clean
//! Architecture: kernel ← domain ← app ← {api, worker, adapter}.
//!
//! Use cases wrap a kernel `SurfaceCatalogPort` impl + project results
//! through the adapter wire DTOs. Pure std-only; no I/O, no framework deps.

use oya_ops_workspace_shell_adapter::{WireHealthResponse, WireSurfaceListResponse};
use oya_ops_workspace_shell_kernel::{
    Surface, SurfaceCatalogError, SurfaceCatalogPort, SurfaceId, SurfaceState, VisibilityTier,
};

/// GET /workspace — Live + principal-tier-scoped surfaces.
pub struct ListLiveSurfacesUseCase<P: SurfaceCatalogPort> {
    catalog: P, // data_class: INTERNAL_ONLY
}

impl<P: SurfaceCatalogPort> ListLiveSurfacesUseCase<P> {
    pub fn new(catalog: P) -> Self {
        Self { catalog }
    }

    pub fn execute(&self, principal_tier: VisibilityTier) -> WireSurfaceListResponse {
        WireSurfaceListResponse::live_visible(&self.catalog, principal_tier)
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
    ) -> WireSurfaceListResponse {
        let surfaces: Vec<oya_ops_workspace_shell_adapter::WireSurface> = self
            .catalog
            .list_surfaces()
            .into_iter()
            .filter(|s| state_filter.map(|st| s.state == st).unwrap_or(true))
            .filter(|s| tier_filter.map(|t| s.visibility_tier == t).unwrap_or(true))
            .map(oya_ops_workspace_shell_adapter::WireSurface::from_kernel)
            .collect();
        let count = surfaces.len();
        WireSurfaceListResponse { surfaces, count }
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
    ) -> Result<(), SurfaceCatalogError> {
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

    pub fn execute(&self) -> WireHealthResponse {
        WireHealthResponse::from_catalog(&self.catalog, &*self.version, self.cell_id.clone())
    }
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
        // Public principal: only the live-pub surface (Live + Public tier).
        assert_eq!(response.count, 1);
        assert_eq!(response.surfaces[0].id, "live-pub");
    }

    #[test]
    fn list_live_internal_principal_sees_more() {
        let use_case = ListLiveSurfacesUseCase::new(populated_catalog());
        let response = use_case.execute(VisibilityTier::InternalPublic);
        // Internal-public principal: live-pub + live-internal (both Live).
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
    fn flip_state_use_case_promotes_to_live() {
        let mut use_case = FlipSurfaceStateUseCase::new(populated_catalog());
        use_case
            .execute(&SurfaceId("soon".into()), SurfaceState::Live)
            .unwrap();
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
