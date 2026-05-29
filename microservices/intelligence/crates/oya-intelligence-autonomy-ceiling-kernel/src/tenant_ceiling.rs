//! Tenant ceiling policy — per-surface autonomy tier caps.
//!
//! A [`TenantCeiling`] carries a global fallback ceiling plus optional
//! per-surface overrides. The pure [`resolve`] function compares a requested
//! tier against the effective ceiling for a given surface and returns a
//! [`TenantCeilingVerdict`].
//!
//! T4Actuate is disabled by default: [`TenantCeiling::default()`] sets the
//! global ceiling to T3PropAct and has no per-surface T4 overrides.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;

use crate::AutonomyTier;

/// Verdict returned by [`resolve`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantCeilingVerdict {
    /// The requested tier is within the effective ceiling for the surface.
    Permitted,
    /// The requested tier exceeds the ceiling; the request is clamped to this
    /// effective ceiling tier instead.
    Clamped(AutonomyTier),
    /// Reserved: the effective ceiling is below T1Read (unreachable with the
    /// current [`AutonomyTier`] enum, kept for forward-compatibility).
    Denied,
}

/// Per-surface tenant ceiling value type.
///
/// Holds a `global` fallback ceiling (default: [`AutonomyTier::T3PropAct`])
/// and an optional map of `surface -> AutonomyTier` overrides.
///
/// Construct via [`TenantCeiling::default()`] for the standard T3 ceiling,
/// or use [`TenantCeiling::builder()`] / [`TenantCeiling::with_surface`] to
/// configure custom surfaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCeiling {
    /// Global fallback when no surface-specific ceiling is set.
    pub global: AutonomyTier,
    /// Per-surface ceiling overrides.
    pub surfaces: HashMap<String, AutonomyTier>,
}

impl Default for TenantCeiling {
    /// Default ceiling: T3PropAct globally, no surface overrides.
    /// T4Actuate is therefore disabled by default.
    fn default() -> Self {
        Self {
            global: AutonomyTier::T3PropAct,
            surfaces: HashMap::new(),
        }
    }
}

impl TenantCeiling {
    /// Create a new ceiling with the given global tier and no surface overrides.
    pub fn new(global: AutonomyTier) -> Self {
        Self {
            global,
            surfaces: HashMap::new(),
        }
    }

    /// Builder-style: add (or replace) a per-surface ceiling, returning `Self`.
    pub fn with_surface(mut self, surface: impl Into<String>, tier: AutonomyTier) -> Self {
        self.surfaces.insert(surface.into(), tier);
        self
    }

    /// Set a per-surface ceiling in-place.
    pub fn set_surface(&mut self, surface: impl Into<String>, tier: AutonomyTier) {
        self.surfaces.insert(surface.into(), tier);
    }

    /// Effective ceiling for the given surface.
    ///
    /// Returns the surface-specific tier if present, otherwise the global ceiling.
    pub fn effective_ceiling(&self, surface: &str) -> AutonomyTier {
        self.surfaces.get(surface).copied().unwrap_or(self.global)
    }
}

/// Resolve a requested tier against a surface ceiling.
///
/// # Algorithm
///
/// 1. Compute `effective = ceiling.effective_ceiling(surface)`.
/// 2. If `requested <= effective` → [`TenantCeilingVerdict::Permitted`].
/// 3. Otherwise → [`TenantCeilingVerdict::Clamped(effective)`].
///
/// [`TenantCeilingVerdict::Denied`] is never returned by this function with the
/// current [`AutonomyTier`] enum (all variants are >= T1Read); it is reserved
/// for forward-compatibility.
pub fn resolve(
    requested: AutonomyTier,
    surface: &str,
    ceiling: &TenantCeiling,
) -> TenantCeilingVerdict {
    let effective = ceiling.effective_ceiling(surface);
    if requested <= effective {
        TenantCeilingVerdict::Permitted
    } else {
        TenantCeilingVerdict::Clamped(effective)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutonomyTier;

    // ── default ceiling tests ──────────────────────────────────────────────

    #[test]
    fn default_ceiling_permits_t1() {
        let c = TenantCeiling::default();
        assert_eq!(resolve(AutonomyTier::T1Read, "write", &c), TenantCeilingVerdict::Permitted);
    }

    #[test]
    fn default_ceiling_permits_t3() {
        let c = TenantCeiling::default();
        assert_eq!(
            resolve(AutonomyTier::T3PropAct, "execute", &c),
            TenantCeilingVerdict::Permitted
        );
    }

    #[test]
    fn default_ceiling_clamps_t4() {
        let c = TenantCeiling::default();
        assert_eq!(
            resolve(AutonomyTier::T4Actuate, "write", &c),
            TenantCeilingVerdict::Clamped(AutonomyTier::T3PropAct)
        );
    }

    // ── surface override tests ─────────────────────────────────────────────

    #[test]
    fn surface_override_permits_t4() {
        let c = TenantCeiling::default().with_surface("write", AutonomyTier::T4Actuate);
        assert_eq!(
            resolve(AutonomyTier::T4Actuate, "write", &c),
            TenantCeilingVerdict::Permitted
        );
    }

    #[test]
    fn surface_override_clamps_to_surface_tier() {
        let c = TenantCeiling::default().with_surface("execute", AutonomyTier::T2Suggest);
        assert_eq!(
            resolve(AutonomyTier::T3PropAct, "execute", &c),
            TenantCeilingVerdict::Clamped(AutonomyTier::T2Suggest)
        );
    }

    #[test]
    fn unknown_surface_falls_back_to_global() {
        // "read" is not in the surface map; falls back to global T3PropAct.
        let c = TenantCeiling::default().with_surface("write", AutonomyTier::T1Read);
        assert_eq!(
            resolve(AutonomyTier::T3PropAct, "read", &c),
            TenantCeilingVerdict::Permitted
        );
    }

    #[test]
    fn t4_global_override_permits_t4() {
        let c = TenantCeiling::new(AutonomyTier::T4Actuate);
        assert_eq!(
            resolve(AutonomyTier::T4Actuate, "any-surface", &c),
            TenantCeilingVerdict::Permitted
        );
    }

    // ── Clamped carries effective tier ────────────────────────────────────

    #[test]
    fn clamped_carries_effective_tier() {
        let c = TenantCeiling::default().with_surface("write", AutonomyTier::T2Suggest);
        match resolve(AutonomyTier::T4Actuate, "write", &c) {
            TenantCeilingVerdict::Clamped(tier) => {
                assert_eq!(tier, AutonomyTier::T2Suggest);
            }
            other => panic!("expected Clamped, got {other:?}"),
        }
    }

    // ── value-type properties ─────────────────────────────────────────────

    #[test]
    fn tenant_ceiling_clone_eq() {
        let a = TenantCeiling::default().with_surface("write", AutonomyTier::T4Actuate);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn builder_with_surface() {
        let c = TenantCeiling::new(AutonomyTier::T1Read)
            .with_surface("read", AutonomyTier::T2Suggest)
            .with_surface("write", AutonomyTier::T4Actuate);
        assert_eq!(c.effective_ceiling("read"), AutonomyTier::T2Suggest);
        assert_eq!(c.effective_ceiling("write"), AutonomyTier::T4Actuate);
        assert_eq!(c.effective_ceiling("other"), AutonomyTier::T1Read); // global fallback
    }
}
