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

/// Result of a batch ceiling evaluation over multiple (surface, tier) pairs.
///
/// Carries per-item [`TenantCeilingVerdict`]s in input order plus an aggregate
/// `most_restrictive_clamp` that is the **minimum** effective ceiling tier among
/// all `Clamped` items. `None` when no item was clamped (including the empty
/// case).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchCeilingVerdict {
    /// Per-request verdict in the same order as the input slice.
    pub results: Vec<TenantCeilingVerdict>,
    /// Lowest effective ceiling tier among clamped items; `None` when all
    /// items are `Permitted` or the input is empty.
    pub most_restrictive_clamp: Option<AutonomyTier>,
}

/// Batch resolver over an ordered slice of `(surface, requested_tier)` pairs.
///
/// Calls [`resolve`] for each entry in order and accumulates:
/// - `results`: one [`TenantCeilingVerdict`] per input request, preserving order.
/// - `most_restrictive_clamp`: the minimum (most restrictive) effective ceiling
///   tier among all [`TenantCeilingVerdict::Clamped`] items; `None` when every
///   item is `Permitted` (or the input is empty).
///
/// T4Actuate-disabled-by-default semantics are inherited from [`resolve`] and
/// [`TenantCeiling::default()`].
pub fn resolve_batch(
    requests: &[(String, AutonomyTier)],
    ceiling: &TenantCeiling,
) -> BatchCeilingVerdict {
    let mut results = Vec::with_capacity(requests.len());
    let mut most_restrictive_clamp: Option<AutonomyTier> = None;

    for (surface, requested) in requests {
        let verdict = resolve(*requested, surface.as_str(), ceiling);
        if let TenantCeilingVerdict::Clamped(effective) = verdict {
            most_restrictive_clamp = Some(match most_restrictive_clamp {
                None => effective,
                Some(prev) => prev.min(effective),
            });
        }
        results.push(verdict);
    }

    BatchCeilingVerdict {
        results,
        most_restrictive_clamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutonomyTier;

    // ── resolve_batch tests ───────────────────────────────────────────────

    #[test]
    fn batch_empty_input_yields_empty_results_and_none_aggregate() {
        let c = TenantCeiling::default();
        let verdict = resolve_batch(&[], &c);
        assert!(verdict.results.is_empty());
        assert_eq!(verdict.most_restrictive_clamp, None);
    }

    #[test]
    fn batch_all_permitted_yields_none_aggregate() {
        let c = TenantCeiling::default();
        let requests = vec![
            ("read".to_string(), AutonomyTier::T1Read),
            ("write".to_string(), AutonomyTier::T2Suggest),
            ("execute".to_string(), AutonomyTier::T3PropAct),
        ];
        let verdict = resolve_batch(&requests, &c);
        assert_eq!(verdict.results.len(), 3);
        assert!(
            verdict
                .results
                .iter()
                .all(|v| *v == TenantCeilingVerdict::Permitted)
        );
        assert_eq!(verdict.most_restrictive_clamp, None);
    }

    #[test]
    fn batch_mixed_permitted_and_clamped_selects_most_restrictive() {
        // Default ceiling is T3. One surface overridden to T2, another to T4.
        let c = TenantCeiling::default()
            .with_surface("restricted", AutonomyTier::T2Suggest)
            .with_surface("elevated", AutonomyTier::T4Actuate);
        let requests = vec![
            // Permitted: T3 within default T3 ceiling
            ("default-surface".to_string(), AutonomyTier::T3PropAct),
            // Clamped: T4 against T2 surface → clamped to T2
            ("restricted".to_string(), AutonomyTier::T4Actuate),
            // Permitted: T4 within T4 surface ceiling
            ("elevated".to_string(), AutonomyTier::T4Actuate),
            // Clamped: T4 against default T3 ceiling → clamped to T3
            ("another-default".to_string(), AutonomyTier::T4Actuate),
        ];
        let verdict = resolve_batch(&requests, &c);
        assert_eq!(verdict.results.len(), 4);
        assert_eq!(verdict.results[0], TenantCeilingVerdict::Permitted);
        assert_eq!(
            verdict.results[1],
            TenantCeilingVerdict::Clamped(AutonomyTier::T2Suggest)
        );
        assert_eq!(verdict.results[2], TenantCeilingVerdict::Permitted);
        assert_eq!(
            verdict.results[3],
            TenantCeilingVerdict::Clamped(AutonomyTier::T3PropAct)
        );
        // Most restrictive clamp is T2 (T2 < T3)
        assert_eq!(
            verdict.most_restrictive_clamp,
            Some(AutonomyTier::T2Suggest)
        );
    }

    #[test]
    fn batch_surface_override_raises_ceiling_item_becomes_permitted() {
        // Without the override, T4 would be clamped. With it, it's Permitted.
        let c = TenantCeiling::default().with_surface("elevated", AutonomyTier::T4Actuate);
        let requests = vec![
            // Permitted via surface override
            ("elevated".to_string(), AutonomyTier::T4Actuate),
            // Clamped: T4 against default T3
            ("default-surface".to_string(), AutonomyTier::T4Actuate),
        ];
        let verdict = resolve_batch(&requests, &c);
        assert_eq!(verdict.results[0], TenantCeilingVerdict::Permitted);
        assert_eq!(
            verdict.results[1],
            TenantCeilingVerdict::Clamped(AutonomyTier::T3PropAct)
        );
        // Aggregate reflects only the clamped item
        assert_eq!(
            verdict.most_restrictive_clamp,
            Some(AutonomyTier::T3PropAct)
        );
    }

    #[test]
    fn batch_aggregate_is_order_independent() {
        // Same items in different order → same most_restrictive_clamp
        let c = TenantCeiling::default().with_surface("low", AutonomyTier::T1Read);
        let requests_a = vec![
            ("default-surface".to_string(), AutonomyTier::T4Actuate), // clamped to T3
            ("low".to_string(), AutonomyTier::T4Actuate),             // clamped to T1
        ];
        let requests_b = vec![
            ("low".to_string(), AutonomyTier::T4Actuate), // clamped to T1
            ("default-surface".to_string(), AutonomyTier::T4Actuate), // clamped to T3
        ];
        let verdict_a = resolve_batch(&requests_a, &c);
        let verdict_b = resolve_batch(&requests_b, &c);
        assert_eq!(verdict_a.most_restrictive_clamp, Some(AutonomyTier::T1Read));
        assert_eq!(verdict_b.most_restrictive_clamp, Some(AutonomyTier::T1Read));
    }

    #[test]
    fn batch_all_clamped_same_tier_yields_that_tier() {
        let c = TenantCeiling::default(); // T3 global
        let requests = vec![
            ("s1".to_string(), AutonomyTier::T4Actuate),
            ("s2".to_string(), AutonomyTier::T4Actuate),
        ];
        let verdict = resolve_batch(&requests, &c);
        assert!(
            verdict
                .results
                .iter()
                .all(|v| *v == TenantCeilingVerdict::Clamped(AutonomyTier::T3PropAct))
        );
        assert_eq!(
            verdict.most_restrictive_clamp,
            Some(AutonomyTier::T3PropAct)
        );
    }

    #[test]
    fn batch_results_len_equals_requests_len() {
        let c = TenantCeiling::default();
        let requests: Vec<(String, AutonomyTier)> = (0..5)
            .map(|i| (format!("surface-{i}"), AutonomyTier::T2Suggest))
            .collect();
        let verdict = resolve_batch(&requests, &c);
        assert_eq!(verdict.results.len(), requests.len());
    }

    // ── default ceiling tests ──────────────────────────────────────────────

    #[test]
    fn default_ceiling_permits_t1() {
        let c = TenantCeiling::default();
        assert_eq!(
            resolve(AutonomyTier::T1Read, "write", &c),
            TenantCeilingVerdict::Permitted
        );
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
