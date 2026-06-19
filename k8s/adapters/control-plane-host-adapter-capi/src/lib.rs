//! kube-rs adapter for the managed-Kubernetes control-plane-host port (ADR-0376).
//!
//! This adapter satisfies [`ControlPlaneProvisioning`] against the management
//! cluster's Cluster API control-plane providers:
//! - **Hosted tier** → a Kamaji `TenantControlPlane` /
//!   `KamajiControlPlane` on the `controlplane.cluster.x-k8s.io` group (control-plane
//!   pods + a per-tenant datastore inside the management cluster).
//! - **Dedicated tier** → a reference to the per-tenant Talos control plane
//!   (ADR-0375 CABPT/CACPPT spoke).
//!
//! ## HONEST-DEFERRED live reconciliation
//!
//! The LIVE CRD reconciliation — create/read/delete of the `TenantControlPlane`
//! and the Talos control-plane reference — is **not** implemented in this lane.
//! A follow-on ADR owns the real CRD wiring. Per the honest-claims discipline
//! (ADR-0083 + the honest-claims gate), this adapter does **not** fake an
//! `Ok(...)`: every port method returns
//! [`ProvisioningError::Unimplemented`]`(`[`Unimplemented::KamajiProviderLiveIntegration`]`)`,
//! tracked at
//! `registry/placeholder-debt/adr-follow-ups.yaml#kamaji-provider-live-integration`.
//!
//! The adapter DOES hold the kube-rs [`Client`] and the dynamic
//! [`ApiResource`]/[`GroupVersionKind`] descriptors the live path will reconcile
//! against, so when the follow-on ADR lands the production path activates
//! without a caller change. kube-rs + k8s-openapi are isolated to THIS crate
//! (ADR-0092 adapter-only seam; ADR-0376).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use kube::Client;
use kube::api::ApiResource;
use kube::config::{Config, KubeConfigOptions, Kubeconfig};
use kube::core::GroupVersionKind;

use k8s_control_plane_host_api::{
    BoxFuture, ControlPlaneProvisioning, ControlPlaneRef, ControlPlaneStatusReport,
    ProvisionRequest, ProvisioningError, Unimplemented,
};

/// CAPI control-plane provider group the hosted (Kamaji) tier reconciles against.
pub const CAPI_CONTROL_PLANE_GROUP: &str = "controlplane.cluster.x-k8s.io";
/// The Kamaji `TenantControlPlane` kind (hosted tier).
pub const KAMAJI_TENANT_CONTROL_PLANE_KIND: &str = "TenantControlPlane";
/// The Kamaji CAPI control-plane kind (hosted tier, CAPI provider wrapper).
pub const KAMAJI_CONTROL_PLANE_KIND: &str = "KamajiControlPlane";
/// API version the hosted-tier control-plane CRDs are served at.
pub const KAMAJI_CONTROL_PLANE_VERSION: &str = "v1alpha1";

/// kube-rs adapter implementing [`ControlPlaneProvisioning`] against the
/// management cluster's CAPI control-plane providers (ADR-0376).
///
/// Holds the kube [`Client`] + the dynamic [`ApiResource`] descriptor for the
/// Kamaji `TenantControlPlane`. The live reconcile is honest-deferred (see the
/// crate docs); construction wires the seam without performing any I/O.
#[derive(Clone)]
pub struct CapiControlPlaneHost {
    client: Client,
    tenant_control_plane: ApiResource,
}

impl CapiControlPlaneHost {
    /// Build the adapter from a kube [`Client`] connected to the MANAGEMENT
    /// cluster (never a tenant cluster — operational-boundary INV per ADR-0376).
    ///
    /// No I/O is performed here; the live reconcile is honest-deferred. The
    /// dynamic [`ApiResource`] for the Kamaji `TenantControlPlane` is derived
    /// from its [`GroupVersionKind`] so the (eventual) live path can build a
    /// `kube::Api<DynamicObject>` without a compile-time CRD struct.
    #[must_use]
    pub fn new(client: Client) -> Self {
        let gvk = GroupVersionKind::gvk(
            CAPI_CONTROL_PLANE_GROUP,
            KAMAJI_CONTROL_PLANE_VERSION,
            KAMAJI_TENANT_CONTROL_PLANE_KIND,
        );
        let tenant_control_plane = ApiResource::from_gvk(&gvk);
        Self {
            client,
            tenant_control_plane,
        }
    }

    /// Build the adapter from a MANAGEMENT-cluster kubeconfig file at `path`.
    ///
    /// This keeps the kube-rs `Client`/`Config` construction isolated to THIS
    /// crate (ADR-0376) so the composition root never imports kube-rs. The
    /// kubeconfig is loaded and a client is constructed from its current
    /// context; no API call is issued (the live reconcile is honest-deferred),
    /// but an unreadable/invalid kubeconfig fails closed here.
    ///
    /// # Errors
    /// Returns a boxed error if the kubeconfig cannot be read/parsed or a client
    /// cannot be constructed from it.
    pub async fn from_kubeconfig_path(
        path: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let kubeconfig = Kubeconfig::read_from(path)?;
        let config =
            Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default()).await?;
        let client = Client::try_from(config)?;
        Ok(Self::new(client))
    }

    /// Borrow the underlying management-cluster kube client (the live reconcile
    /// path consumes this).
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// The dynamic API resource descriptor for the Kamaji `TenantControlPlane`
    /// the hosted tier reconciles (the live path builds a
    /// `kube::Api<DynamicObject>` from this).
    #[must_use]
    pub fn tenant_control_plane_resource(&self) -> &ApiResource {
        &self.tenant_control_plane
    }

    /// The single honest-deferred boundary every port method currently returns.
    /// Centralised so the live path can be wired in one place later.
    fn deferred() -> ProvisioningError {
        ProvisioningError::Unimplemented(Unimplemented::KamajiProviderLiveIntegration)
    }
}

impl ControlPlaneProvisioning for CapiControlPlaneHost {
    fn provision<'a>(
        &'a self,
        request: &'a ProvisionRequest,
    ) -> BoxFuture<'a, Result<ControlPlaneRef, ProvisioningError>> {
        // Validate the caller input fail-closed BEFORE reporting the deferred
        // boundary, so a malformed request is still rejected honestly.
        Box::pin(async move {
            if !request.cluster_ref.is_well_formed() {
                return Err(ProvisioningError::InvalidClusterRef {
                    cluster_ref: request.cluster_ref.to_string(),
                });
            }
            // HONEST-DEFERRED: the live Kamaji TenantControlPlane / Talos
            // control-plane reconcile is owned by a follow-on ADR. No fake Ok.
            Err(Self::deferred())
        })
    }

    fn status<'a>(
        &'a self,
        _control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<ControlPlaneStatusReport, ProvisioningError>> {
        Box::pin(async move { Err(Self::deferred()) })
    }

    fn teardown<'a>(
        &'a self,
        _control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<(), ProvisioningError>> {
        Box::pin(async move { Err(Self::deferred()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_api_resource_descriptor_is_kamaji_tenant_control_plane() {
        // The dynamic descriptor is built from the GVK without a kube Client
        // (no cluster connection needed to assert the seam shape).
        let gvk = GroupVersionKind::gvk(
            CAPI_CONTROL_PLANE_GROUP,
            KAMAJI_CONTROL_PLANE_VERSION,
            KAMAJI_TENANT_CONTROL_PLANE_KIND,
        );
        let resource = ApiResource::from_gvk(&gvk);
        assert_eq!(resource.group, CAPI_CONTROL_PLANE_GROUP);
        assert_eq!(resource.version, KAMAJI_CONTROL_PLANE_VERSION);
        assert_eq!(resource.kind, KAMAJI_TENANT_CONTROL_PLANE_KIND);
    }

    #[test]
    fn deferred_boundary_cites_placeholder_debt() {
        let err = CapiControlPlaneHost::deferred();
        let rendered = err.to_string();
        assert!(rendered.contains("kamaji-provider-live-integration"));
        assert!(matches!(
            err,
            ProvisioningError::Unimplemented(Unimplemented::KamajiProviderLiveIntegration)
        ));
    }
}
