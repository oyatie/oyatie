//! Ops workspace shell kernel — port traits + types for ops.oyatie.com's
//! workspace shell BC per ralplan-ops-portal v7 §6(a) + ralplan-docs-portal v7.
//!
//! The workspace shell mounts every ops µservice's surface (docs, dashboards,
//! tenant-mgmt, capacity, on-call, ...). 14 surface slots per ADR-0067; each
//! slot declares Live | ReservedComingSoon | Retired state + Cedar visibility
//! tier + canonical route.
//!
//! Pure std-only kernel layer per ADR-0015: no outbound I/O, no framework
//! dependencies. Adapter + runtime crates implement the ports.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

/// Cedar visibility tier per ralplan-ops-portal v7 §6(d) — 6-tier model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VisibilityTier {
    Public,
    TenantPublic,
    TenantPrivate,
    InternalPublic,
    InternalPrivate,
    SystemOnly,
}

impl VisibilityTier {
    pub fn name(self) -> &'static str {
        match self {
            VisibilityTier::Public => "public",
            VisibilityTier::TenantPublic => "tenant-public",
            VisibilityTier::TenantPrivate => "tenant-private",
            VisibilityTier::InternalPublic => "internal-public",
            VisibilityTier::InternalPrivate => "internal-private",
            VisibilityTier::SystemOnly => "system-only",
        }
    }
}

/// Surface lifecycle state per ADR-0067 SURFACE_CATALOG protocol.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SurfaceState {
    /// Registered but not yet live; renders a placeholder.
    ReservedComingSoon,
    /// Live in production; Cedar fragment + OpenAPI contract green.
    Live,
    /// Retired; redirects to canonical replacement (if any).
    Retired,
}

impl SurfaceState {
    pub fn name(self) -> &'static str {
        match self {
            SurfaceState::ReservedComingSoon => "reserved-coming-soon",
            SurfaceState::Live => "live",
            SurfaceState::Retired => "retired",
        }
    }
}

/// Stable surface identifier (e.g., `docs-portal`, `tenant-mgmt`,
/// `deployments`). Distinct from `route` because routes can be aliased.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SurfaceId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Surface {
    pub id: SurfaceId,                           // data_class: INTERNAL_ONLY
    pub canonical_route: String, // data_class: INTERNAL_ONLY (e.g., "/workspace/docs")
    pub visibility_tier: VisibilityTier, // data_class: INTERNAL_ONLY
    pub state: SurfaceState,     // data_class: INTERNAL_ONLY
    pub owning_bc_id: String,    // data_class: INTERNAL_ONLY (e.g., "ops/docs-portal")
    pub cedar_fragments: Vec<String>, // data_class: INTERNAL_ONLY
    pub openapi_contract: Option<String>, // data_class: INTERNAL_ONLY
    pub retired_redirects_to: Option<SurfaceId>, // data_class: INTERNAL_ONLY
}

/// Typed result of a surface lifecycle request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceStateTransitionOutcome {
    Changed {
        from: SurfaceState,
        to: SurfaceState,
    },
    Unchanged {
        state: SurfaceState,
    },
}

/// Port trait: registers and queries surfaces in the workspace catalog.
/// Adapter implementations write to durable storage; tests use in-memory.
pub trait SurfaceCatalogPort {
    fn register_surface(&mut self, surface: Surface) -> Result<(), SurfaceCatalogError>;
    fn get_surface(&self, id: &SurfaceId) -> Option<&Surface>;
    fn list_surfaces(&self) -> Vec<&Surface>;
    fn flip_state(
        &mut self,
        id: &SurfaceId,
        new_state: SurfaceState,
    ) -> Result<SurfaceStateTransitionOutcome, SurfaceCatalogError>;
    fn count(&self) -> usize;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceCatalogError {
    DuplicateId(SurfaceId),
    UnknownId(SurfaceId),
    InvalidStateTransition {
        id: SurfaceId,
        from: SurfaceState,
        to: SurfaceState,
    },
    RouteCollision {
        existing_id: SurfaceId,
        attempted_id: SurfaceId,
        route: String,
    },
    RetiredWithoutRedirect(SurfaceId),
}

/// In-memory `SurfaceCatalogPort` implementation suitable for tests and
/// the runtime composition root.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemorySurfaceCatalog {
    by_id: BTreeMap<SurfaceId, Surface>, // data_class: INTERNAL_ONLY
    by_route: BTreeMap<String, SurfaceId>, // data_class: INTERNAL_ONLY
}

impl InMemorySurfaceCatalog {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SurfaceCatalogPort for InMemorySurfaceCatalog {
    fn register_surface(&mut self, surface: Surface) -> Result<(), SurfaceCatalogError> {
        if self.by_id.contains_key(&surface.id) {
            return Err(SurfaceCatalogError::DuplicateId(surface.id));
        }
        if let Some(existing) = self.by_route.get(&surface.canonical_route) {
            return Err(SurfaceCatalogError::RouteCollision {
                existing_id: existing.clone(),
                attempted_id: surface.id.clone(),
                route: surface.canonical_route.clone(),
            });
        }
        if matches!(surface.state, SurfaceState::Retired) && surface.retired_redirects_to.is_none()
        {
            return Err(SurfaceCatalogError::RetiredWithoutRedirect(surface.id));
        }
        self.by_route
            .insert(surface.canonical_route.clone(), surface.id.clone());
        self.by_id.insert(surface.id.clone(), surface);
        Ok(())
    }

    fn get_surface(&self, id: &SurfaceId) -> Option<&Surface> {
        self.by_id.get(id)
    }

    fn list_surfaces(&self) -> Vec<&Surface> {
        self.by_id.values().collect()
    }

    fn flip_state(
        &mut self,
        id: &SurfaceId,
        new_state: SurfaceState,
    ) -> Result<SurfaceStateTransitionOutcome, SurfaceCatalogError> {
        let surface = self
            .by_id
            .get_mut(id)
            .ok_or_else(|| SurfaceCatalogError::UnknownId(id.clone()))?;
        // Allowed transitions: ReservedComingSoon → Live; Live → Retired.
        // Backwards transitions are explicit policy violations.
        match (surface.state, new_state) {
            (from @ SurfaceState::ReservedComingSoon, to @ SurfaceState::Live) => {
                surface.state = new_state;
                Ok(SurfaceStateTransitionOutcome::Changed { from, to })
            }
            (from @ SurfaceState::Live, to @ SurfaceState::Retired) => {
                if surface.retired_redirects_to.is_none() {
                    return Err(SurfaceCatalogError::RetiredWithoutRedirect(id.clone()));
                }
                surface.state = new_state;
                Ok(SurfaceStateTransitionOutcome::Changed { from, to })
            }
            (state, requested) if state == requested => {
                Ok(SurfaceStateTransitionOutcome::Unchanged { state })
            }
            (from, to) => Err(SurfaceCatalogError::InvalidStateTransition {
                id: id.clone(),
                from,
                to,
            }),
        }
    }

    fn count(&self) -> usize {
        self.by_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(id: &str, route: &str, state: SurfaceState) -> Surface {
        Surface {
            id: SurfaceId(id.into()),
            canonical_route: route.into(),
            visibility_tier: VisibilityTier::InternalPrivate,
            state,
            owning_bc_id: format!("ops/{id}"),
            cedar_fragments: vec![],
            openapi_contract: None,
            retired_redirects_to: None,
        }
    }

    #[test]
    fn register_and_retrieve() {
        let mut catalog = InMemorySurfaceCatalog::new();
        let s = surface(
            "docs-portal",
            "/workspace/docs",
            SurfaceState::ReservedComingSoon,
        );
        catalog.register_surface(s.clone()).unwrap();
        assert_eq!(catalog.count(), 1);
        assert_eq!(
            catalog.get_surface(&SurfaceId("docs-portal".into())),
            Some(&s)
        );
    }

    #[test]
    fn duplicate_id_errors() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        let result = catalog.register_surface(surface(
            "a",
            "/workspace/b",
            SurfaceState::ReservedComingSoon,
        ));
        assert!(matches!(result, Err(SurfaceCatalogError::DuplicateId(_))));
    }

    #[test]
    fn route_collision_errors() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/x",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        let result = catalog.register_surface(surface(
            "b",
            "/workspace/x",
            SurfaceState::ReservedComingSoon,
        ));
        assert!(matches!(
            result,
            Err(SurfaceCatalogError::RouteCollision { .. })
        ));
    }

    #[test]
    fn retired_without_redirect_errors_on_register() {
        let mut catalog = InMemorySurfaceCatalog::new();
        let result = catalog.register_surface(surface("a", "/workspace/a", SurfaceState::Retired));
        assert!(matches!(
            result,
            Err(SurfaceCatalogError::RetiredWithoutRedirect(_))
        ));
    }

    #[test]
    fn reserved_to_live_reports_changed_from_and_to() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        let outcome = catalog
            .flip_state(&SurfaceId("a".into()), SurfaceState::Live)
            .unwrap();
        assert_eq!(
            outcome,
            SurfaceStateTransitionOutcome::Changed {
                from: SurfaceState::ReservedComingSoon,
                to: SurfaceState::Live,
            }
        );
        assert_eq!(
            catalog.get_surface(&SurfaceId("a".into())).unwrap().state,
            SurfaceState::Live
        );
    }

    #[test]
    fn flip_state_live_to_reserved_errors() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        catalog
            .flip_state(&SurfaceId("a".into()), SurfaceState::Live)
            .unwrap();
        let result = catalog.flip_state(&SurfaceId("a".into()), SurfaceState::ReservedComingSoon);
        assert!(matches!(
            result,
            Err(SurfaceCatalogError::InvalidStateTransition { .. })
        ));
    }

    #[test]
    fn flip_state_live_to_retired_requires_redirect() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        catalog
            .flip_state(&SurfaceId("a".into()), SurfaceState::Live)
            .unwrap();
        let result = catalog.flip_state(&SurfaceId("a".into()), SurfaceState::Retired);
        assert!(matches!(
            result,
            Err(SurfaceCatalogError::RetiredWithoutRedirect(_))
        ));
    }

    #[test]
    fn same_state_reports_unchanged() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        let outcome = catalog
            .flip_state(&SurfaceId("a".into()), SurfaceState::ReservedComingSoon)
            .unwrap();
        assert_eq!(
            outcome,
            SurfaceStateTransitionOutcome::Unchanged {
                state: SurfaceState::ReservedComingSoon,
            }
        );
    }

    #[test]
    fn list_surfaces_returns_all() {
        let mut catalog = InMemorySurfaceCatalog::new();
        catalog
            .register_surface(surface(
                "a",
                "/workspace/a",
                SurfaceState::ReservedComingSoon,
            ))
            .unwrap();
        catalog
            .register_surface(surface("b", "/workspace/b", SurfaceState::Live))
            .unwrap();
        assert_eq!(catalog.list_surfaces().len(), 2);
    }

    #[test]
    fn visibility_tier_names_round_trip() {
        let tiers = [
            VisibilityTier::Public,
            VisibilityTier::TenantPublic,
            VisibilityTier::TenantPrivate,
            VisibilityTier::InternalPublic,
            VisibilityTier::InternalPrivate,
            VisibilityTier::SystemOnly,
        ];
        let names: Vec<&str> = tiers.iter().map(|t| t.name()).collect();
        assert_eq!(names.len(), 6);
        assert!(names.iter().all(|n| !n.is_empty()));
    }
}
