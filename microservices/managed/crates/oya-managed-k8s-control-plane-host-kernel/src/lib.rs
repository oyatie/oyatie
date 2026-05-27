//! Managed-Kubernetes control-plane-host kernel (ADR-0376, ADR-0083 Tier-3).
//!
//! This is the **pure** layer for the hosted-tier control-plane-host concern:
//! the abstract tier / provisioning-status / datastore-class value types a
//! tenant control plane moves through, with NO knowledge of Kamaji CRD fields,
//! kube-rs, Talos, HTTP, or any I/O. Per ADR-0376 the product is two-tier —
//! [`ControlPlaneTier::HostedKamaji`] (the dense default: control planes as pods
//! in Oyatie's management cluster) and [`ControlPlaneTier::DedicatedTalosSpoke`]
//! (the sovereign premium SKU: a full Talos spoke per tenant). This kernel
//! abstracts *only* the lifecycle shape both tiers share; the live CRD wiring is
//! an adapter concern, honest-deferred per
//! `registry/placeholder-debt/adr-follow-ups.yaml#kamaji-provider-live-integration`.
//!
//! ## State machine (ADR-0376 hosted/dedicated provisioning)
//!
//! ```text
//!                     ┌─ datastore_bound ─┐  (hosted: Kamaji etcd/relational bound)
//!  requested ─────────┤                    ├─▶ provisioning ─▶ endpoint_ready ─▶ active
//!                     └─ media_formed ────┘  (dedicated: Talos control-plane media formed)
//!                                                                                   │
//!                                                          draining ◀───────────────┘
//!                                                             │
//!                                                             ▼
//!                                                          deleted
//!
//!  (any non-terminal state) ─▶ failed
//! ```
//!
//! The branch after `requested` is tier-determined: a [`ControlPlaneTier::HostedKamaji`]
//! control plane binds a datastore ([`ControlPlaneStatus::DatastoreBound`]); a
//! [`ControlPlaneTier::DedicatedTalosSpoke`] forms its control-plane installation
//! media ([`ControlPlaneStatus::MediaFormed`]). Both converge on `provisioning`.
//! `deleted` is the sole success-terminal; `failed` is the fault-terminal,
//! reachable from any non-terminal state.
//!
//! ## Hot-path posture (ADR-0083 Tier-3 — panic-free)
//!
//! Every fallible operation returns an explicit `Result`/`Option`; the kernel
//! never `unwrap`/`expect`/`panic!`s on a caller-supplied value. Parsing an
//! unknown enum string returns `None`, and an illegal state transition returns
//! a typed [`IllegalTransition`] error rather than panicking.

// ADR-0083 Tier-3: production stays panic-free (deny in release). Inline tests
// may use unwrap/expect/panic under cfg(test).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;

use serde::{Deserialize, Serialize};

// =====================================================================
// Tier
// =====================================================================

/// The control-plane placement tier a tenant cluster is provisioned under
/// (ADR-0376). The tenant picks the tier; the product default is
/// [`ControlPlaneTier::HostedKamaji`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneTier {
    /// DEFAULT tier: the tenant control plane runs as pods inside Oyatie's
    /// shared management cluster (the Kamaji hosted-control-plane model). Dense,
    /// provisions in seconds, collapses the per-tenant standing control-plane
    /// tax. The control plane reaches its own API server but never the
    /// management cluster or a peer tenant.
    HostedKamaji,
    /// PREMIUM tier: a full dedicated Talos spoke cluster per tenant (its own
    /// etcd + three control-plane nodes), the ADR-0375 spoke promoted to a
    /// product SKU. For sovereign / air-gapped / strongest-isolation tenants.
    DedicatedTalosSpoke,
}

impl ControlPlaneTier {
    /// The product default tier (ADR-0376: hosted is default).
    #[must_use]
    pub const fn default_tier() -> Self {
        Self::HostedKamaji
    }

    /// Stable wire/log slug for this tier.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::HostedKamaji => "hosted_kamaji",
            Self::DedicatedTalosSpoke => "dedicated_talos_spoke",
        }
    }

    /// Parse a tier from its stable slug. Returns `None` for any unknown value
    /// (fail-closed; no panic).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hosted_kamaji" => Some(Self::HostedKamaji),
            "dedicated_talos_spoke" => Some(Self::DedicatedTalosSpoke),
            _ => None,
        }
    }

    /// Whether this tier hosts the control plane inside the shared management
    /// cluster (true for [`ControlPlaneTier::HostedKamaji`]).
    #[must_use]
    pub const fn is_hosted(&self) -> bool {
        matches!(self, Self::HostedKamaji)
    }
}

impl Default for ControlPlaneTier {
    fn default() -> Self {
        Self::default_tier()
    }
}

impl fmt::Display for ControlPlaneTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// Datastore class
// =====================================================================

/// The datastore backing a hosted-tier tenant control plane (ADR-0376). Abstract
/// only — the kernel does not know whether this is a Kamaji-managed etcd
/// StatefulSet or a pooled relational datastore connection; it records the
/// CHOICE so the lifecycle + audit chain can reason about isolation posture.
///
/// This is meaningful for [`ControlPlaneTier::HostedKamaji`]; a
/// [`ControlPlaneTier::DedicatedTalosSpoke`] always carries its own etcd on its
/// control-plane nodes and does not consult this class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatastoreClass {
    /// A dedicated etcd datastore per tenant control plane (strongest hosted
    /// isolation; each tenant control plane owns its own keyspace).
    EtcdPerTenant,
    /// A pooled relational datastore shared across hosted tenant control planes
    /// (denser; the Kamaji `datastore` model where many tenant control planes
    /// share one relational backend with per-tenant logical separation).
    PooledRelational,
}

impl DatastoreClass {
    /// Stable wire/log slug for this datastore class.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EtcdPerTenant => "etcd_per_tenant",
            Self::PooledRelational => "pooled_relational",
        }
    }

    /// Parse a datastore class from its stable slug. Returns `None` for any
    /// unknown value (fail-closed; no panic).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "etcd_per_tenant" => Some(Self::EtcdPerTenant),
            "pooled_relational" => Some(Self::PooledRelational),
            _ => None,
        }
    }

    /// Whether this class gives each tenant control plane its own physical
    /// datastore (true for [`DatastoreClass::EtcdPerTenant`]).
    #[must_use]
    pub const fn is_per_tenant(&self) -> bool {
        matches!(self, Self::EtcdPerTenant)
    }
}

impl fmt::Display for DatastoreClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =====================================================================
// Status state machine
// =====================================================================

/// The lifecycle status of a tenant control plane (ADR-0376). The legal
/// transition graph is enforced by [`ControlPlaneStatus::can_transition_to`] /
/// [`ControlPlaneStatus::transition`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneStatus {
    /// Provisioning requested; nothing materialized yet.
    Requested,
    /// HOSTED branch: the tenant control plane's datastore (etcd or pooled
    /// relational) has been bound. Reached only from [`Self::Requested`] for a
    /// [`ControlPlaneTier::HostedKamaji`] control plane.
    DatastoreBound,
    /// DEDICATED branch: the Talos control-plane installation media has been
    /// formed for the spoke. Reached only from [`Self::Requested`] for a
    /// [`ControlPlaneTier::DedicatedTalosSpoke`] control plane.
    MediaFormed,
    /// Control-plane components are coming up (both tiers converge here).
    Provisioning,
    /// The API-server endpoint is reachable but the control plane is not yet
    /// fully ready for tenant workloads to be scheduled against it.
    EndpointReady,
    /// The control plane is fully active and serving the tenant.
    Active,
    /// The control plane is being torn down (graceful drain before delete).
    Draining,
    /// Terminal success: the control plane has been deleted.
    Deleted,
    /// Terminal fault: provisioning or operation failed. Reachable from any
    /// non-terminal state.
    Failed,
}

impl ControlPlaneStatus {
    /// The initial status of a freshly requested control plane.
    #[must_use]
    pub const fn initial() -> Self {
        Self::Requested
    }

    /// Stable wire/log slug for this status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::DatastoreBound => "datastore_bound",
            Self::MediaFormed => "media_formed",
            Self::Provisioning => "provisioning",
            Self::EndpointReady => "endpoint_ready",
            Self::Active => "active",
            Self::Draining => "draining",
            Self::Deleted => "deleted",
            Self::Failed => "failed",
        }
    }

    /// Parse a status from its stable slug. Returns `None` for any unknown value
    /// (fail-closed; no panic).
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "requested" => Some(Self::Requested),
            "datastore_bound" => Some(Self::DatastoreBound),
            "media_formed" => Some(Self::MediaFormed),
            "provisioning" => Some(Self::Provisioning),
            "endpoint_ready" => Some(Self::EndpointReady),
            "active" => Some(Self::Active),
            "draining" => Some(Self::Draining),
            "deleted" => Some(Self::Deleted),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    /// Whether this status is terminal (no outgoing transition).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Deleted | Self::Failed)
    }

    /// Whether a control plane in this status is serving its tenant.
    #[must_use]
    pub const fn is_serving(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether `next` is a legal successor of `self` per the ADR-0376 graph.
    ///
    /// `Failed` is reachable from any non-terminal status. The hosted branch
    /// (`Requested -> DatastoreBound`) and dedicated branch
    /// (`Requested -> MediaFormed`) both converge on `Provisioning`.
    #[must_use]
    pub const fn can_transition_to(&self, next: Self) -> bool {
        // Any non-terminal state may fault to Failed.
        if matches!(next, Self::Failed) {
            return !self.is_terminal();
        }
        matches!(
            (self, next),
            (Self::Requested, Self::DatastoreBound)
                | (Self::Requested, Self::MediaFormed)
                | (Self::DatastoreBound, Self::Provisioning)
                | (Self::MediaFormed, Self::Provisioning)
                | (Self::Provisioning, Self::EndpointReady)
                | (Self::EndpointReady, Self::Active)
                | (Self::Active, Self::Draining)
                | (Self::Draining, Self::Deleted)
        )
    }

    /// Attempt to transition `self` to `next`, validating the move against the
    /// ADR-0376 graph.
    ///
    /// # Errors
    /// Returns [`IllegalTransition`] when `next` is not a legal successor of
    /// `self` (including any outgoing move from a terminal state). The kernel
    /// never panics on an illegal transition — callers fail closed.
    pub fn transition(self, next: Self) -> Result<Self, IllegalTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(IllegalTransition {
                from: self,
                to: next,
            })
        }
    }

    /// The status a freshly-`Requested` control plane of `tier` moves to next:
    /// the tier-determined branch ([`Self::DatastoreBound`] for hosted,
    /// [`Self::MediaFormed`] for dedicated).
    #[must_use]
    pub const fn next_after_request(tier: ControlPlaneTier) -> Self {
        match tier {
            ControlPlaneTier::HostedKamaji => Self::DatastoreBound,
            ControlPlaneTier::DedicatedTalosSpoke => Self::MediaFormed,
        }
    }
}

impl fmt::Display for ControlPlaneStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An attempted control-plane status transition that the ADR-0376 state machine
/// forbids. Carries the offending `from`/`to` pair for fail-closed diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IllegalTransition {
    /// The status the control plane was in.
    pub from: ControlPlaneStatus, // data_class: INTERNAL_ONLY
    /// The illegal target status.
    pub to: ControlPlaneStatus, // data_class: INTERNAL_ONLY
}

impl fmt::Display for IllegalTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "illegal control-plane status transition: {} -> {}",
            self.from.as_str(),
            self.to.as_str()
        )
    }
}

impl std::error::Error for IllegalTransition {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_roundtrips_through_slug() {
        for tier in [
            ControlPlaneTier::HostedKamaji,
            ControlPlaneTier::DedicatedTalosSpoke,
        ] {
            assert_eq!(ControlPlaneTier::parse(tier.as_str()), Some(tier));
        }
        assert_eq!(ControlPlaneTier::parse("gardener"), None);
    }

    #[test]
    fn tier_default_is_hosted() {
        assert_eq!(ControlPlaneTier::default(), ControlPlaneTier::HostedKamaji);
        assert!(ControlPlaneTier::HostedKamaji.is_hosted());
        assert!(!ControlPlaneTier::DedicatedTalosSpoke.is_hosted());
    }

    #[test]
    fn datastore_class_roundtrips_through_slug() {
        for class in [
            DatastoreClass::EtcdPerTenant,
            DatastoreClass::PooledRelational,
        ] {
            assert_eq!(DatastoreClass::parse(class.as_str()), Some(class));
        }
        assert_eq!(DatastoreClass::parse("sqlite"), None);
        assert!(DatastoreClass::EtcdPerTenant.is_per_tenant());
        assert!(!DatastoreClass::PooledRelational.is_per_tenant());
    }

    #[test]
    fn status_roundtrips_through_slug() {
        for status in [
            ControlPlaneStatus::Requested,
            ControlPlaneStatus::DatastoreBound,
            ControlPlaneStatus::MediaFormed,
            ControlPlaneStatus::Provisioning,
            ControlPlaneStatus::EndpointReady,
            ControlPlaneStatus::Active,
            ControlPlaneStatus::Draining,
            ControlPlaneStatus::Deleted,
            ControlPlaneStatus::Failed,
        ] {
            assert_eq!(ControlPlaneStatus::parse(status.as_str()), Some(status));
        }
        assert_eq!(ControlPlaneStatus::parse("paused"), None);
    }

    #[test]
    fn hosted_happy_path_is_legal() {
        let mut s = ControlPlaneStatus::initial();
        assert_eq!(s, ControlPlaneStatus::Requested);
        for next in [
            ControlPlaneStatus::DatastoreBound,
            ControlPlaneStatus::Provisioning,
            ControlPlaneStatus::EndpointReady,
            ControlPlaneStatus::Active,
            ControlPlaneStatus::Draining,
            ControlPlaneStatus::Deleted,
        ] {
            s = s.transition(next).expect("legal hosted transition");
        }
        assert!(s.is_terminal());
    }

    #[test]
    fn dedicated_happy_path_is_legal() {
        let mut s = ControlPlaneStatus::initial();
        for next in [
            ControlPlaneStatus::MediaFormed,
            ControlPlaneStatus::Provisioning,
            ControlPlaneStatus::EndpointReady,
            ControlPlaneStatus::Active,
            ControlPlaneStatus::Draining,
            ControlPlaneStatus::Deleted,
        ] {
            s = s.transition(next).expect("legal dedicated transition");
        }
        assert!(s.is_terminal());
    }

    #[test]
    fn next_after_request_is_tier_determined() {
        assert_eq!(
            ControlPlaneStatus::next_after_request(ControlPlaneTier::HostedKamaji),
            ControlPlaneStatus::DatastoreBound
        );
        assert_eq!(
            ControlPlaneStatus::next_after_request(ControlPlaneTier::DedicatedTalosSpoke),
            ControlPlaneStatus::MediaFormed
        );
    }

    #[test]
    fn hosted_branch_cannot_form_media() {
        // A hosted control plane binds a datastore; it must NOT take the
        // dedicated media-forming branch.
        assert!(!ControlPlaneStatus::Requested.can_transition_to(ControlPlaneStatus::Provisioning));
        let err = ControlPlaneStatus::Requested
            .transition(ControlPlaneStatus::Active)
            .expect_err("requested cannot jump to active");
        assert_eq!(err.from, ControlPlaneStatus::Requested);
        assert_eq!(err.to, ControlPlaneStatus::Active);
    }

    #[test]
    fn any_non_terminal_can_fail() {
        for s in [
            ControlPlaneStatus::Requested,
            ControlPlaneStatus::DatastoreBound,
            ControlPlaneStatus::MediaFormed,
            ControlPlaneStatus::Provisioning,
            ControlPlaneStatus::EndpointReady,
            ControlPlaneStatus::Active,
            ControlPlaneStatus::Draining,
        ] {
            assert!(
                s.can_transition_to(ControlPlaneStatus::Failed),
                "{s} should be able to fail"
            );
        }
    }

    #[test]
    fn terminal_states_have_no_exit() {
        for terminal in [ControlPlaneStatus::Deleted, ControlPlaneStatus::Failed] {
            assert!(terminal.is_terminal());
            for next in [
                ControlPlaneStatus::Requested,
                ControlPlaneStatus::Provisioning,
                ControlPlaneStatus::Active,
                ControlPlaneStatus::Failed,
                ControlPlaneStatus::Deleted,
            ] {
                assert!(
                    !terminal.can_transition_to(next),
                    "{terminal} must not transition to {next}"
                );
            }
        }
    }

    #[test]
    fn serde_status_uses_snake_case_slug() {
        let json = serde_json::to_string(&ControlPlaneStatus::EndpointReady).expect("serialize");
        assert_eq!(json, "\"endpoint_ready\"");
        let back: ControlPlaneStatus = serde_json::from_str("\"datastore_bound\"").expect("de");
        assert_eq!(back, ControlPlaneStatus::DatastoreBound);
    }
}
