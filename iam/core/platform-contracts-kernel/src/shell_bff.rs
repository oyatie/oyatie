//! Shell-BFF contract family: capability registry entry + module route
//! registration.
//!
//! Precedent: the backend-for-frontend pattern (SoundCloud origin, Netflix
//! API-gateway evolution) and micro-frontend route registries: the app shell
//! resolves which product modules a principal may see from a capability
//! registry (visibility is PDP-gated, deny-by-default), and routes each
//! module's traffic to its owning upstream through registered, non-ambiguous
//! route prefixes.

use serde::{Deserialize, Serialize};

use crate::{ContractViolation, MAX_DISPLAY_NAME_LEN, MAX_ID_LEN, check_slug, check_text};

/// Where a capability surfaces in the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationSurface {
    PrimaryNav,
    ContextualPanel,
    CommandPalette,
    /// Reachable by direct route only; never listed.
    Hidden,
}

/// One entry in the shell capability registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRegistryEntry {
    pub capability_id: String, // data_class: INTERNAL_ONLY
    pub display_name: String,  // data_class: INTERNAL_ONLY
    /// The product module that owns this capability.
    pub module_id: String, // data_class: INTERNAL_ONLY
    /// PDP action gating visibility of this capability (deny-by-default: a
    /// capability without an allow decision is invisible, not greyed out).
    pub required_action: String, // data_class: INTERNAL_ONLY
    pub navigation_surface: NavigationSurface, // data_class: INTERNAL_ONLY
}

impl CapabilityRegistryEntry {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug(
            "capability.capability_id",
            &self.capability_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_text(
            "capability.display_name",
            &self.display_name,
            MAX_DISPLAY_NAME_LEN,
            &mut out,
        );
        check_slug(
            "capability.module_id",
            &self.module_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "capability.required_action",
            &self.required_action,
            MAX_ID_LEN,
            &mut out,
        );
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

/// A module's route registration with the shell BFF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRouteRegistration {
    pub module_id: String, // data_class: INTERNAL_ONLY
    /// Route prefix owned by the module, e.g. `/tenancy`. Lowercase slug
    /// segments, leading `/`, no trailing `/`.
    pub route_prefix: String, // data_class: INTERNAL_ONLY
    /// Logical upstream service name (resolved by the platform, never a raw
    /// URL — the BFF owns endpoint resolution).
    pub upstream_service: String, // data_class: INTERNAL_ONLY
    /// Capabilities served under this route (non-empty, unique).
    pub capability_ids: Vec<String>, // data_class: INTERNAL_ONLY
}

impl ModuleRouteRegistration {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug("route.module_id", &self.module_id, MAX_ID_LEN, &mut out);
        check_slug(
            "route.upstream_service",
            &self.upstream_service,
            MAX_ID_LEN,
            &mut out,
        );
        self.check_route_prefix(&mut out);
        if self.capability_ids.is_empty() {
            out.push(ContractViolation::MissingValue {
                field: "route.capability_ids",
            });
        }
        let mut seen = std::collections::BTreeSet::new();
        for capability_id in &self.capability_ids {
            check_slug("route.capability_ids", capability_id, MAX_ID_LEN, &mut out);
            if !seen.insert(capability_id) {
                out.push(ContractViolation::BrokenReference {
                    field: "route.capability_ids",
                    detail: format!("duplicate capability reference {capability_id:?}"),
                });
            }
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }

    fn check_route_prefix(&self, out: &mut Vec<ContractViolation>) {
        let prefix = &self.route_prefix;
        let Some(rest) = prefix.strip_prefix('/') else {
            out.push(ContractViolation::InvalidShape {
                field: "route.route_prefix",
                detail: "route prefix must start with '/'".to_owned(),
            });
            return;
        };
        if rest.is_empty() || rest.ends_with('/') || rest.split('/').any(str::is_empty) {
            out.push(ContractViolation::InvalidShape {
                field: "route.route_prefix",
                detail: "route prefix must be '/<segment>[/<segment>...]' without a trailing '/'"
                    .to_owned(),
            });
            return;
        }
        for segment in rest.split('/') {
            check_slug("route.route_prefix", segment, MAX_ID_LEN, out);
        }
    }
}

/// Whether `outer` is a path-prefix of `inner` (segment-aligned): `/a` covers
/// `/a` and `/a/b`, but not `/ab`.
fn covers(outer: &str, inner: &str) -> bool {
    inner == outer || inner.starts_with(&format!("{outer}/"))
}

/// Registry-level cross-checks (surface-all):
/// - capability ids are unique across the registry;
/// - module route prefixes are unique and non-overlapping (no route may be a
///   path-prefix of another — routing must be unambiguous);
/// - every capability referenced by a route exists in the registry AND is
///   owned by the registering module.
pub fn validate_registry(
    entries: &[CapabilityRegistryEntry],
    routes: &[ModuleRouteRegistration],
) -> Result<(), Vec<ContractViolation>> {
    let mut out = Vec::new();
    let mut by_capability = std::collections::BTreeMap::new();
    for entry in entries {
        if let Err(violations) = entry.validate() {
            out.extend(violations);
        }
        if by_capability
            .insert(entry.capability_id.clone(), entry)
            .is_some()
        {
            out.push(ContractViolation::BrokenReference {
                field: "registry.capability_id",
                detail: format!("duplicate capability id {:?}", entry.capability_id),
            });
        }
    }
    for route in routes {
        if let Err(violations) = route.validate() {
            out.extend(violations);
        }
        for capability_id in &route.capability_ids {
            match by_capability.get(capability_id) {
                None => out.push(ContractViolation::BrokenReference {
                    field: "registry.routes",
                    detail: format!(
                        "route {:?} references unknown capability {capability_id:?}",
                        route.route_prefix
                    ),
                }),
                Some(entry) if entry.module_id != route.module_id => {
                    out.push(ContractViolation::BrokenReference {
                        field: "registry.routes",
                        detail: format!(
                            "route {:?} (module {:?}) references capability {capability_id:?} owned by module {:?}",
                            route.route_prefix, route.module_id, entry.module_id
                        ),
                    });
                }
                Some(_) => {}
            }
        }
    }
    for (i, a) in routes.iter().enumerate() {
        for b in routes.iter().skip(i + 1) {
            if covers(&a.route_prefix, &b.route_prefix) || covers(&b.route_prefix, &a.route_prefix)
            {
                out.push(ContractViolation::BrokenReference {
                    field: "registry.routes",
                    detail: format!(
                        "route prefixes {:?} and {:?} overlap; routing would be ambiguous",
                        a.route_prefix, b.route_prefix
                    ),
                });
            }
        }
    }
    if out.is_empty() { Ok(()) } else { Err(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(capability_id: &str, module_id: &str) -> CapabilityRegistryEntry {
        CapabilityRegistryEntry {
            capability_id: capability_id.to_owned(),
            display_name: "Tenant administration".to_owned(),
            module_id: module_id.to_owned(),
            required_action: "tenancy.administer".to_owned(),
            navigation_surface: NavigationSurface::PrimaryNav,
        }
    }

    fn route(module_id: &str, prefix: &str, capability_ids: &[&str]) -> ModuleRouteRegistration {
        ModuleRouteRegistration {
            module_id: module_id.to_owned(),
            route_prefix: prefix.to_owned(),
            upstream_service: format!("oya-{module_id}"),
            capability_ids: capability_ids.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn valid_registry_passes_and_round_trips() {
        let entries = vec![
            entry("tenant-admin", "tenancy"),
            entry("mail-inbox", "mail"),
        ];
        let routes = vec![
            route("tenancy", "/tenancy", &["tenant-admin"]),
            route("mail", "/mail", &["mail-inbox"]),
        ];
        validate_registry(&entries, &routes).unwrap();
        let json = serde_json::to_string(&routes[0]).unwrap();
        assert_eq!(
            serde_json::from_str::<ModuleRouteRegistration>(&json).unwrap(),
            routes[0]
        );
    }

    #[test]
    fn route_closed_schema_rejects_unknown_fields() {
        let mut value = serde_json::to_value(route("mail", "/mail", &["mail-inbox"])).unwrap();
        value["upstream_url"] = serde_json::json!("https://example.com");
        assert!(serde_json::from_value::<ModuleRouteRegistration>(value).is_err());
    }

    #[test]
    fn malformed_route_prefixes_are_rejected() {
        for bad in [
            "",
            "tenancy",
            "/",
            "/tenancy/",
            "//tenancy",
            "/Tenancy",
            "/tenancy//x",
        ] {
            assert!(
                route("tenancy", bad, &["tenant-admin"]).validate().is_err(),
                "{bad:?} must be rejected"
            );
        }
        route("tenancy", "/tenancy/admin", &["tenant-admin"])
            .validate()
            .unwrap();
    }

    #[test]
    fn overlapping_route_prefixes_are_ambiguous() {
        let entries = vec![
            entry("tenant-admin", "tenancy"),
            entry("mail-inbox", "mail"),
        ];
        let routes = vec![
            route("tenancy", "/tenancy", &["tenant-admin"]),
            route("mail", "/tenancy/mail", &["mail-inbox"]),
        ];
        let violations = validate_registry(&entries, &routes).unwrap_err();
        assert!(violations.iter().any(|v| matches!(
            v,
            ContractViolation::BrokenReference {
                field: "registry.routes",
                ..
            }
        )));
    }

    #[test]
    fn sibling_prefix_is_not_overlap() {
        let entries = vec![
            entry("tenant-admin", "tenancy"),
            entry("mail-inbox", "mail"),
        ];
        let routes = vec![
            route("tenancy", "/tenancy", &["tenant-admin"]),
            route("mail", "/tenancy-reports", &["mail-inbox"]),
        ];
        // `/tenancy` does NOT cover `/tenancy-reports` (segment-aligned check).
        let result = validate_registry(&entries, &routes);
        assert!(
            result
                .as_ref()
                .err()
                .map(|v| v.iter().all(|x| !format!("{x}").contains("overlap")))
                .unwrap_or(true),
            "{result:?}"
        );
    }

    #[test]
    fn dangling_and_foreign_capability_references_are_violations() {
        let entries = vec![entry("tenant-admin", "tenancy")];
        let routes = vec![
            route("mail", "/mail", &["tenant-admin"]),
            route("tenancy", "/tenancy", &["missing-capability"]),
        ];
        let violations = validate_registry(&entries, &routes).unwrap_err();
        assert_eq!(violations.len(), 2, "surface-all: {violations:?}");
    }

    #[test]
    fn duplicate_capability_ids_are_violations() {
        let entries = vec![
            entry("tenant-admin", "tenancy"),
            entry("tenant-admin", "mail"),
        ];
        let violations = validate_registry(&entries, &[]).unwrap_err();
        assert!(matches!(
            violations.as_slice(),
            [ContractViolation::BrokenReference {
                field: "registry.capability_id",
                ..
            }]
        ));
    }
}
