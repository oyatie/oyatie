//! Managed-Kubernetes control-plane-host **API / port** layer (ADR-0376).
//!
//! This crate owns the one seam the product layer shares: the
//! [`ControlPlaneProvisioning`] port plus the request/response DTOs that cross
//! it. The sibling product microservices named in ADR-0376 —
//! `oya-managed-k8s-cluster-lifecycle` (the cluster-CRUD API) and
//! `oya-managed-k8s-sla-observability` (control-plane uptime / provisioning
//! latency) — will depend on THIS port, never on a concrete adapter. The
//! adapters ([`oya-managed-k8s-control-plane-host-adapter-capi`] for the live
//! kube-rs/Kamaji path, `...-adapter-inmemory` for the deterministic fake)
//! satisfy this trait.
//!
//! ## Why the port is boxed-future async, not `async fn`
//!
//! The port is object-safe (`dyn ControlPlaneProvisioning`) so the composition
//! root can hold `Arc<dyn ControlPlaneProvisioning>` and swap adapters without
//! a generic blast radius. We therefore return
//! `Pin<Box<dyn Future + Send + '_>>` from each method (the same idiom the
//! intelligence provider-pool transport port uses) rather than `async fn`,
//! which is not yet object-safe across the trait boundary we need.
//!
//! ## Layering invariant (ADR-0105 / ADR-0131)
//!
//! Path-dep inward on the kernel only. NO kube-rs, NO HTTP, NO async runtime —
//! the port names the *shape* of provisioning, and the kernel value types
//! ([`ControlPlaneTier`], [`ControlPlaneStatus`], [`DatastoreClass`]) it
//! exchanges. ADR-0083 Tier-3: every fallible path returns a typed
//! [`ProvisioningError`]; implementations never panic on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use core::pin::Pin;

use serde::{Deserialize, Serialize};

pub use oya_managed_k8s_control_plane_host_kernel::{
    ControlPlaneStatus, ControlPlaneTier, DatastoreClass, IllegalTransition,
};

/// A short alias for the boxed, `Send` future every async port method returns.
pub type BoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + Send + 'a>>;

// =====================================================================
// DTOs
// =====================================================================

/// Stable, tenant-scoped reference to the tenant CLUSTER whose control plane is
/// being provisioned. This is the caller's identity for the cluster as a
/// first-class resource (the CAPI `Cluster` the cluster-lifecycle microservice
/// owns); the control-plane-host concern keys all of its state on it.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ClusterRef {
    /// Tenant that owns the cluster. Tenant-zero (Oyatie dogfood) is an ordinary
    /// value here — there is NO internal-bypass identity (ADR-0376
    /// oyatie-dogfood-tenancy).
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// Tenant-unique cluster name (the CAPI `Cluster` metadata.name the
    /// cluster-lifecycle microservice assigns).
    pub cluster_name: String, // data_class: TENANT_SCOPED
}

impl ClusterRef {
    /// Construct a cluster reference.
    #[must_use]
    pub fn new(tenant_id: impl Into<String>, cluster_name: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
        }
    }

    /// Whether both identity components are non-empty (fail-closed validation
    /// the adapters apply before touching any backend).
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        !self.tenant_id.trim().is_empty() && !self.cluster_name.trim().is_empty()
    }
}

impl fmt::Display for ClusterRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tenant_id, self.cluster_name)
    }
}

/// Request to provision a tenant control plane. The `datastore_class` is
/// meaningful for [`ControlPlaneTier::HostedKamaji`]; for a
/// [`ControlPlaneTier::DedicatedTalosSpoke`] it is advisory (the spoke always
/// carries its own etcd) and adapters MAY ignore it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvisionRequest {
    /// The cluster whose control plane is requested.
    pub cluster_ref: ClusterRef,
    /// The placement tier (ADR-0376; default is hosted).
    pub tier: ControlPlaneTier,
    /// The datastore class for the hosted tier.
    pub datastore_class: DatastoreClass,
}

impl ProvisionRequest {
    /// Construct a provision request.
    #[must_use]
    pub fn new(
        cluster_ref: ClusterRef,
        tier: ControlPlaneTier,
        datastore_class: DatastoreClass,
    ) -> Self {
        Self {
            cluster_ref,
            tier,
            datastore_class,
        }
    }
}

/// Opaque, adapter-issued handle to a provisioned control plane. Returned by
/// [`ControlPlaneProvisioning::provision`] and accepted by
/// [`ControlPlaneProvisioning::status`] / [`ControlPlaneProvisioning::teardown`].
/// Echoes the [`ClusterRef`] + tier so callers (cluster-lifecycle,
/// sla-observability) can correlate without a second lookup.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlPlaneRef {
    /// The cluster this control plane belongs to.
    pub cluster_ref: ClusterRef,
    /// The tier the control plane was provisioned under.
    pub tier: ControlPlaneTier,
    /// Adapter-issued opaque handle (e.g. the Kamaji `TenantControlPlane`
    /// namespaced name, or the Talos spoke id). Treated as opaque by callers.
    pub handle: String, // data_class: TENANT_SCOPED
}

impl ControlPlaneRef {
    /// Construct a control-plane reference.
    #[must_use]
    pub fn new(cluster_ref: ClusterRef, tier: ControlPlaneTier, handle: impl Into<String>) -> Self {
        Self {
            cluster_ref,
            tier,
            handle: handle.into(),
        }
    }
}

/// A point-in-time status report for a tenant control plane, returned by
/// [`ControlPlaneProvisioning::status`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlPlaneStatusReport {
    /// The control plane the report is about.
    pub control_plane_ref: ControlPlaneRef,
    /// Current lifecycle status (kernel state machine).
    pub status: ControlPlaneStatus,
    /// The API-server endpoint, once the control plane reaches
    /// [`ControlPlaneStatus::EndpointReady`] or later. `None` before that.
    pub endpoint: Option<String>, // data_class: TENANT_SCOPED
}

impl ControlPlaneStatusReport {
    /// Construct a status report.
    #[must_use]
    pub fn new(
        control_plane_ref: ControlPlaneRef,
        status: ControlPlaneStatus,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            control_plane_ref,
            status,
            endpoint,
        }
    }
}

// =====================================================================
// Errors
// =====================================================================

/// Typed enumeration of downstream paths an adapter may explicitly defer rather
/// than fake. Retained for compatibility with older/development adapters and
/// honest-claims checks; the live CAPI adapter no longer uses this for its
/// hosted happy path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unimplemented {
    /// Historical live-reconciliation placeholder for a Kamaji
    /// `TenantControlPlane` / dedicated Talos control-plane reference.
    KamajiProviderLiveIntegration,
}

impl Unimplemented {
    /// Stable human-facing slug for this boundary.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::KamajiProviderLiveIntegration => "Unimplemented::KamajiProviderLiveIntegration",
        }
    }

    /// Stable placeholder-debt id this boundary maps to (the YAML registry key
    /// under `registry/placeholder-debt/adr-follow-ups.yaml`).
    #[must_use]
    pub const fn placeholder_debt_id(&self) -> &'static str {
        match self {
            Self::KamajiProviderLiveIntegration => "kamaji-provider-live-integration",
        }
    }
}

impl fmt::Display for Unimplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Failure modes from the [`ControlPlaneProvisioning`] port. ADR-0083 Tier-3:
/// every adapter returns one of these instead of panicking on the request path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvisioningError {
    /// The [`ClusterRef`] was malformed (empty tenant/cluster). Fail-closed.
    InvalidClusterRef {
        /// The offending reference rendered for diagnostics.
        cluster_ref: String,
    },
    /// No control plane is known for the supplied handle (e.g. `status` /
    /// `teardown` of an unknown control plane). Default-deny.
    NotFound {
        /// The handle that was looked up.
        handle: String,
    },
    /// The requested lifecycle move is illegal per the kernel state machine.
    IllegalTransition(IllegalTransition),
    /// The backend (management cluster, datastore, Talos endpoint) could not be
    /// reached or returned an error. Operator-facing detail only — NEVER carries
    /// tenant secrets or kubeconfig material.
    Backend {
        /// Human-facing detail for logs.
        detail: String, // data_class: INTERNAL_ONLY
    },
    /// The path is explicitly deferred: adapters surface a typed boundary rather
    /// than a fake success (see [`Unimplemented`]).
    Unimplemented(Unimplemented),
}

impl ProvisioningError {
    /// Construct a [`ProvisioningError::Backend`] from any displayable detail.
    #[must_use]
    pub fn backend(detail: impl Into<String>) -> Self {
        Self::Backend {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ProvisioningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidClusterRef { cluster_ref } => {
                write!(f, "invalid cluster reference: {cluster_ref:?}")
            }
            Self::NotFound { handle } => write!(f, "control plane not found: {handle:?}"),
            Self::IllegalTransition(transition) => write!(f, "{transition}"),
            Self::Backend { detail } => write!(f, "control-plane-host backend error: {detail}"),
            Self::Unimplemented(boundary) => write!(
                f,
                "{boundary} — explicitly deferred; see registry/placeholder-debt/adr-follow-ups.yaml#{}",
                boundary.placeholder_debt_id()
            ),
        }
    }
}

impl std::error::Error for ProvisioningError {}

impl From<IllegalTransition> for ProvisioningError {
    fn from(value: IllegalTransition) -> Self {
        Self::IllegalTransition(value)
    }
}

// =====================================================================
// The port
// =====================================================================

/// The shared control-plane provisioning port (ADR-0376). Implemented by the
/// kube-rs/Kamaji adapter (live) and the in-memory fake;
/// consumed by the control-plane-host app today and by the cluster-lifecycle +
/// sla-observability microservices in their own lanes.
///
/// Object-safe (`dyn ControlPlaneProvisioning`): every method returns a boxed
/// `Send` future so the composition root can hold
/// `Arc<dyn ControlPlaneProvisioning>` and swap adapters without a generic
/// blast radius.
pub trait ControlPlaneProvisioning: Send + Sync {
    /// Provision a tenant control plane for `request.cluster_ref` under
    /// `request.tier` with `request.datastore_class`. Returns the adapter-issued
    /// [`ControlPlaneRef`] handle.
    ///
    /// # Errors
    /// Returns [`ProvisioningError::InvalidClusterRef`] for a malformed ref,
    /// [`ProvisioningError::Backend`] for a backend failure, or
    /// [`ProvisioningError::Unimplemented`] when an adapter explicitly defers.
    fn provision<'a>(
        &'a self,
        request: &'a ProvisionRequest,
    ) -> BoxFuture<'a, Result<ControlPlaneRef, ProvisioningError>>;

    /// Read the current status of the control plane identified by
    /// `control_plane_ref`.
    ///
    /// # Errors
    /// Returns [`ProvisioningError::NotFound`] if no control plane is known for
    /// the handle, [`ProvisioningError::Backend`] for a backend failure, or
    /// [`ProvisioningError::Unimplemented`] when an adapter explicitly defers.
    fn status<'a>(
        &'a self,
        control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<ControlPlaneStatusReport, ProvisioningError>>;

    /// Tear down the control plane identified by `control_plane_ref` (drain then
    /// delete). Idempotent: tearing down an already-deleted control plane is not
    /// an error.
    ///
    /// # Errors
    /// Returns [`ProvisioningError::NotFound`] if no control plane is known for
    /// the handle, [`ProvisioningError::Backend`] for a backend failure, or
    /// [`ProvisioningError::Unimplemented`] when an adapter explicitly defers.
    fn teardown<'a>(
        &'a self,
        control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<(), ProvisioningError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_ref_wellformed_rejects_empty() {
        assert!(ClusterRef::new("ten_zero", "dogfood-a").is_well_formed());
        assert!(!ClusterRef::new("", "c").is_well_formed());
        assert!(!ClusterRef::new("t", "  ").is_well_formed());
    }

    #[test]
    fn provision_request_serde_roundtrip() {
        let req = ProvisionRequest::new(
            ClusterRef::new("ten_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            DatastoreClass::EtcdPerTenant,
        );
        let json = serde_json::to_string(&req).expect("serialize");
        let back: ProvisionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req, back);
        assert!(json.contains("hosted_kamaji"));
        assert!(json.contains("etcd_per_tenant"));
    }

    #[test]
    fn unimplemented_maps_to_stable_debt_id() {
        assert_eq!(
            Unimplemented::KamajiProviderLiveIntegration.placeholder_debt_id(),
            "kamaji-provider-live-integration"
        );
    }

    #[test]
    fn provisioning_error_display_cites_placeholder_debt() {
        let err = ProvisioningError::Unimplemented(Unimplemented::KamajiProviderLiveIntegration);
        let rendered = err.to_string();
        assert!(rendered.contains("kamaji-provider-live-integration"));
        assert!(rendered.contains("registry/placeholder-debt/adr-follow-ups.yaml"));
    }

    #[test]
    fn illegal_transition_converts_into_provisioning_error() {
        let transition = ControlPlaneStatus::Requested
            .transition(ControlPlaneStatus::Active)
            .expect_err("illegal");
        let err: ProvisioningError = transition.into();
        assert!(matches!(err, ProvisioningError::IllegalTransition(_)));
    }
}
