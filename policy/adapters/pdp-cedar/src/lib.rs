//! # iam-pdp-cedar
//!
//! Embedded cedar-policy PDP adapter for FD-001 (story G004, ADR-0536 D-2).
//!
//! ## Posture
//! Implements the [`PolicyDecisionPoint`] port from `shared-pdp-kernel`
//! over the upstream, formally-verified `cedar-policy` engine (arXiv
//! 2403.04651): default-deny, forbid-overrides-permit, order-independent.
//! Evaluation is strictly in-process — an authorization decision never takes
//! a network hop (ADR-0536 D-2; precedent: Cedar / Amazon Verified
//! Permissions embedded evaluator).
//!
//! Cedar is the TERMINAL engine decision, not a transitional impl: ADR-0536
//! D-2 retires the hand-rolled `policy-cedar-*` evaluator in favor of
//! this crate (two decision algorithms must never coexist, ADR-0243).
//!
//! ## Behavior
//! - Bundles are parsed, template-linked, and STRICT-validated before load;
//!   a rejected bundle never replaces a serving one (fail closed).
//! - Zookie freshness: a caller-pinned `min_policy_version` that does not
//!   match the loaded bundle version is a refusal, never a stale answer.
//! - Decision cache keyed `(request-fingerprint, policy-version)`: a bundle
//!   swap changes the version and structurally invalidates every prior
//!   entry, so revocation latency reduces to bundle propagation
//!   (sub-60s revocation SLO, G004).
//! - One audit record per decision — allow or deny, cached or evaluated —
//!   with a freshly minted decision id every time.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use cedar_policy::{Authorizer, EntityUid, PolicySet, Schema};
use shared_pdp_kernel::{DecisionCache, PdpError, PolicyBundle};
use shared_platform_contracts_kernel::pdp::PolicyVersion;
use shared_ulid_id_kernel::IdGenerator;

mod admission;
mod audit;
mod entity;
mod evaluation;
mod overlay;

use admission::compile;
pub use admission::validate_bundle;
pub use audit::{
    AuditChainCedarPdp, PDP_DECISION_AUDIT_SURFACE, PdpAuditChainError, PdpDecisionAuditChainLogger,
};

struct LoadedBundle {
    version: PolicyVersion,
    source_identity: Vec<u8>,
    schema: Schema,
    /// The global policy set (structural forbid + RBAC/ABAC/PBAC). Served to
    /// any request whose tenant has no overlay.
    policy_set: PolicySet,
    /// Per-tenant MERGED policy sets: `tenant_id` -> (global ∪ that tenant's
    /// overlay). Built at compile so the request path stays a single
    /// `is_authorized` over one `PolicySet` (ADR-0243: one decision
    /// algorithm). A tenant absent from this map falls back to `policy_set`.
    /// The merge is per-tenant, so one tenant's overlay can NEVER appear in
    /// another tenant's set (overlay SELECTION is keyed by the owning tenant).
    /// Note: cross-tenant GRANT isolation is a separate, stronger guarantee
    /// enforced at runtime by the global `structural-tenant-isolation` forbid
    /// present in every merged set — not by this selection keying.
    tenant_policy_sets: BTreeMap<String, PolicySet>,
    action_map: BTreeMap<String, EntityUid>,
}

impl LoadedBundle {
    /// The policy set to decide `tenant_id` against: the per-tenant merged set
    /// (global ∪ that tenant's overlay) when one exists, else the global set.
    /// A tenant NEVER sees another tenant's overlay — the BTreeMap is keyed by
    /// the owning tenant, so selection is structural.
    fn policy_set_for(&self, tenant_id: &str) -> &PolicySet {
        self.tenant_policy_sets
            .get(tenant_id)
            .unwrap_or(&self.policy_set)
    }
}

/// The embedded Cedar PDP. One instance per process; the policy-store
/// delivery fabric swaps bundles in place via [`CedarPdp::swap_bundle`].
pub struct CedarPdp {
    state: RwLock<LoadedBundle>,
    cache: Mutex<DecisionCache>,
    cache_capacity: usize,
    id_gen: Arc<dyn IdGenerator>,
    authorizer: Authorizer,
}

impl CedarPdp {
    /// Compile and strict-validate `bundle`, then serve from it.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when any part of the bundle fails to
    /// parse, link, or strict-validate — nothing is loaded in that case.
    pub fn load(
        bundle: &PolicyBundle,
        id_gen: Arc<dyn IdGenerator>,
        cache_capacity: usize,
    ) -> Result<Self, PdpError> {
        let state = compile(bundle)?;
        Ok(Self {
            state: RwLock::new(state),
            cache: Mutex::new(DecisionCache::new(cache_capacity)),
            cache_capacity,
            id_gen,
            authorizer: Authorizer::new(),
        })
    }

    /// Atomically replace the serving bundle (the revocation path). The new
    /// bundle is fully compiled and strict-validated BEFORE the swap; on any
    /// error the current bundle keeps serving (fail closed, static
    /// stability). A replacement invalidates cached content, including content
    /// from a previous use of its opaque version. An identical reload is a no-op.
    ///
    /// # Errors
    /// [`PdpError::BundleRejected`] when admission fails or the current version
    /// is redefined; [`PdpError::Evaluation`] when a serving-state lock is poisoned.
    pub fn swap_bundle(&self, bundle: &PolicyBundle) -> Result<(), PdpError> {
        let next = compile(bundle)?;
        let mut state = self.state.write().map_err(|_| PdpError::Evaluation {
            detail: "policy state lock poisoned".to_owned(),
        })?;
        if state.source_identity == next.source_identity {
            return Ok(());
        }
        if state.version == next.version {
            return Err(PdpError::BundleRejected {
                detail: format!(
                    "serving policy version {} cannot be redefined",
                    state.version.as_str()
                ),
            });
        }
        let mut cache = self.cache.lock().map_err(|_| PdpError::Evaluation {
            detail: "decision cache lock poisoned".to_owned(),
        })?;
        // Authorization retains the state read lock through cache insertion.
        // Holding the write lock makes this invalidation and replacement atomic.
        *cache = DecisionCache::new(self.cache_capacity);
        *state = next;
        Ok(())
    }
}
