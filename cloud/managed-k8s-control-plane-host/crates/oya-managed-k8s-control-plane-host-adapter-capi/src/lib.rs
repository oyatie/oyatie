//! kube-rs adapter for the managed-Kubernetes control-plane-host port (ADR-0376).
//!
//! This adapter satisfies [`ControlPlaneProvisioning`] against the management
//! cluster's Cluster API control-plane providers:
//! - **Hosted tier** → a Kamaji `TenantControlPlane` on the
//!   `kamaji.clastix.io/v1alpha1` group (control-plane pods + a per-tenant
//!   datastore inside the management cluster).
//! - **Dedicated tier** → a reference to the per-tenant Talos control plane
//!   (ADR-0375 CABPT/CACPPT spoke).
//!
//! ## Live reconciliation boundary
//!
//! Hosted-tier requests create/read/delete a Kamaji `TenantControlPlane` through
//! kube-rs's dynamic [`ApiResource`]/[`DynamicObject`] path. Dedicated-tier
//! requests resolve an opaque Talos/CAPI reference and deliberately do not create
//! a hosted Kamaji object or hold tenant kubeconfig material. kube-rs +
//! k8s-openapi remain isolated to THIS crate (ADR-0092 adapter-only seam;
//! ADR-0376).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use kube::api::{Api, ApiResource, DeleteParams, DynamicObject, PostParams};
use kube::config::{Config, KubeConfigOptions, Kubeconfig};
use kube::core::GroupVersionKind;
use kube::{Client, Error as KubeError};
use serde_json::{Value, json};

use oya_managed_k8s_control_plane_host_api::{
    BoxFuture, ClusterRef, ControlPlaneProvisioning, ControlPlaneRef, ControlPlaneStatus,
    ControlPlaneStatusReport, ControlPlaneTier, DatastoreClass, ProvisionRequest,
    ProvisioningError, Unimplemented,
};

/// Kamaji TenantControlPlane CRD group the hosted tier reconciles against.
pub const CAPI_CONTROL_PLANE_GROUP: &str = "kamaji.clastix.io";
/// The Kamaji `TenantControlPlane` kind (hosted tier).
pub const KAMAJI_TENANT_CONTROL_PLANE_KIND: &str = "TenantControlPlane";
/// The Kamaji CAPI control-plane kind (hosted tier, CAPI provider wrapper).
pub const KAMAJI_CONTROL_PLANE_KIND: &str = "KamajiControlPlane";
/// API version the hosted-tier control-plane CRDs are served at.
pub const KAMAJI_CONTROL_PLANE_VERSION: &str = "v1alpha1";
/// Explicit plural for the dynamic kube-rs API; avoids guessed CRD pluralization.
pub const KAMAJI_TENANT_CONTROL_PLANE_PLURAL: &str = "tenantcontrolplanes";
/// Oyatie ownership label required before status/teardown trusts a dynamic object.
pub const OWNER_LABEL: &str = "oya.io/owner";
/// Value written to [`OWNER_LABEL`] for objects this adapter owns.
pub const OWNER_LABEL_VALUE: &str = "managed-k8s-control-plane-host";
const TIER_LABEL: &str = "oya.io/tier";
const TENANT_HASH_LABEL: &str = "oya.io/tenant-hash";
const CLUSTER_LABEL: &str = "oya.io/cluster";
const FINALIZER: &str = "control-plane-host.oya.io/finalizer";
const HOSTED_HANDLE_PREFIX: &str = "kamaji";
const DEDICATED_HANDLE_PREFIX: &str = "talos-capi";

/// Rollback policy for the live adapter. Disabling new provisioning is narrower
/// than disabling the adapter: status and teardown remain available so existing
/// live objects can be observed and removed during recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveProvisioningPolicy {
    allow_new_provisioning: bool,
}

impl LiveProvisioningPolicy {
    /// Normal live mode: new provisions, status reads, and teardown are allowed.
    #[must_use]
    pub const fn new_provisions_enabled() -> Self {
        Self {
            allow_new_provisioning: true,
        }
    }

    /// Rollback mode: block new live provisions but keep status/teardown live.
    #[must_use]
    pub const fn new_provisions_disabled() -> Self {
        Self {
            allow_new_provisioning: false,
        }
    }

    /// Whether the adapter may create new hosted control-plane objects.
    #[must_use]
    pub const fn allows_new_provisioning(&self) -> bool {
        self.allow_new_provisioning
    }

    /// Status and teardown stay enabled in both modes for recovery.
    #[must_use]
    pub const fn allows_status_and_teardown(&self) -> bool {
        true
    }
}

impl Default for LiveProvisioningPolicy {
    fn default() -> Self {
        Self::new_provisions_enabled()
    }
}

/// Parsed hosted-control-plane handle. Kept private so callers treat the handle
/// as opaque while the adapter can address the dynamic Kubernetes object.
#[derive(Clone, Debug, Eq, PartialEq)]
struct HostedHandle {
    namespace: String,
    name: String,
}

/// Result of mapping provider status fields to the kernel lifecycle status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStatusMapping {
    /// Kernel lifecycle status derived from provider conditions/phase.
    pub status: ControlPlaneStatus,
    /// Provider-published API endpoint. Never synthesized from names.
    pub endpoint: Option<String>,
}

/// kube-rs adapter implementing [`ControlPlaneProvisioning`] against the
/// management cluster's CAPI control-plane providers (ADR-0376).
///
/// Holds the kube [`Client`] + the dynamic [`ApiResource`] descriptor for the
/// Kamaji `TenantControlPlane`. The hosted tier creates/reads/deletes that CRD
/// through kube-rs's dynamic API; the dedicated tier returns an opaque Talos/CAPI
/// reference path without creating a hosted Kamaji object.
#[derive(Clone)]
pub struct CapiControlPlaneHost {
    client: Client,
    tenant_control_plane: ApiResource,
    policy: LiveProvisioningPolicy,
}

impl CapiControlPlaneHost {
    /// Build the adapter from a kube [`Client`] connected to the MANAGEMENT
    /// cluster (never a tenant cluster — operational-boundary INV per ADR-0376).
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self::with_policy(client, LiveProvisioningPolicy::default())
    }

    /// Build the adapter with an explicit live-provisioning policy.
    #[must_use]
    pub fn with_policy(client: Client, policy: LiveProvisioningPolicy) -> Self {
        Self {
            client,
            tenant_control_plane: Self::tenant_control_plane_api_resource(),
            policy,
        }
    }

    /// Return this adapter with a different rollback/live-provisioning policy.
    #[must_use]
    pub fn with_live_provisioning_policy(mut self, policy: LiveProvisioningPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Build the adapter from a MANAGEMENT-cluster kubeconfig file at `path`.
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

    /// Borrow the underlying management-cluster kube client.
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// The dynamic API resource descriptor for the Kamaji `TenantControlPlane`.
    #[must_use]
    pub fn tenant_control_plane_resource(&self) -> &ApiResource {
        &self.tenant_control_plane
    }

    /// The current rollback/live-provisioning policy.
    #[must_use]
    pub const fn policy(&self) -> LiveProvisioningPolicy {
        self.policy
    }

    /// Stable dynamic API resource descriptor for the LTS-pinned Kamaji CRD GVK.
    #[must_use]
    pub fn tenant_control_plane_api_resource() -> ApiResource {
        let gvk = GroupVersionKind::gvk(
            CAPI_CONTROL_PLANE_GROUP,
            KAMAJI_CONTROL_PLANE_VERSION,
            KAMAJI_TENANT_CONTROL_PLANE_KIND,
        );
        ApiResource::from_gvk_with_plural(&gvk, KAMAJI_TENANT_CONTROL_PLANE_PLURAL)
    }

    /// Build the hosted Kamaji dynamic object without performing Kubernetes I/O.
    ///
    /// # Errors
    /// Returns a typed error if the cluster ref is malformed or the request is
    /// not for the hosted Kamaji tier.
    pub fn hosted_control_plane_object(
        request: &ProvisionRequest,
    ) -> Result<DynamicObject, ProvisioningError> {
        validate_cluster_ref(&request.cluster_ref)?;
        if request.tier != ControlPlaneTier::HostedKamaji {
            return Err(ProvisioningError::backend(
                "hosted Kamaji object requested for non-hosted tier",
            ));
        }

        let handle = Self::hosted_handle_for(&request.cluster_ref);
        let mut labels = BTreeMap::new();
        labels.insert(OWNER_LABEL.to_string(), OWNER_LABEL_VALUE.to_string());
        labels.insert(TIER_LABEL.to_string(), request.tier.as_str().to_string());
        labels.insert(
            TENANT_HASH_LABEL.to_string(),
            stable_hash_hex(&request.cluster_ref.tenant_id),
        );
        labels.insert(
            CLUSTER_LABEL.to_string(),
            sanitize_k8s_segment(&request.cluster_ref.cluster_name, 40),
        );

        let mut annotations = BTreeMap::new();
        annotations.insert(
            "oya.io/tenant-id-hash".to_string(),
            stable_hash_hex(&request.cluster_ref.tenant_id),
        );
        annotations.insert(
            "oya.io/control-plane-host-boundary".to_string(),
            "management-cluster-only-no-tenant-kubeconfig".to_string(),
        );

        let mut object =
            DynamicObject::new(&handle.name, &Self::tenant_control_plane_api_resource())
                .within(&handle.namespace)
                .data(json!({
                    "spec": {
                        "clusterName": request.cluster_ref.cluster_name,
                        "datastore": { "class": request.datastore_class.as_str() },
                        "tenantRef": { "hash": stable_hash_hex(&request.cluster_ref.tenant_id) },
                        "managementBoundary": {
                            "tenantKubeconfigMaterial": false,
                            "ownedBy": OWNER_LABEL_VALUE
                        }
                    }
                }));
        object.metadata.labels = Some(labels);
        object.metadata.annotations = Some(annotations);
        object.metadata.finalizers = Some(vec![FINALIZER.to_string()]);
        Ok(object)
    }

    /// Resolve a dedicated Talos/CAPI reference without building a hosted object.
    ///
    /// # Errors
    /// Returns a typed error if the cluster ref is malformed or the request is
    /// not for the dedicated Talos spoke tier.
    pub fn dedicated_talos_reference(
        request: &ProvisionRequest,
    ) -> Result<ControlPlaneRef, ProvisioningError> {
        validate_cluster_ref(&request.cluster_ref)?;
        if request.tier != ControlPlaneTier::DedicatedTalosSpoke {
            return Err(ProvisioningError::backend(
                "dedicated Talos reference requested for non-dedicated tier",
            ));
        }
        let namespace = tenant_namespace(&request.cluster_ref.tenant_id);
        let cluster = sanitize_k8s_segment(&request.cluster_ref.cluster_name, 40);
        Ok(ControlPlaneRef::new(
            request.cluster_ref.clone(),
            request.tier,
            format!(
                "{DEDICATED_HANDLE_PREFIX}/{namespace}/{cluster}-{}",
                stable_hash_hex(&format!(
                    "{}:{}",
                    request.cluster_ref.tenant_id, request.cluster_ref.cluster_name
                ))
            ),
        ))
    }

    /// Map provider conditions/status into the kernel lifecycle state.
    ///
    /// # Errors
    /// Returns a typed backend error if the object is not owned by this adapter.
    pub fn status_from_dynamic_object(
        object: &DynamicObject,
    ) -> Result<ProviderStatusMapping, ProvisioningError> {
        if !Self::is_owned_control_plane(object) {
            return Err(ProvisioningError::backend(
                "refusing to map foreign Kamaji TenantControlPlane status",
            ));
        }
        if object.metadata.deletion_timestamp.is_some() {
            return Ok(ProviderStatusMapping {
                status: ControlPlaneStatus::Draining,
                endpoint: published_endpoint(&object.data),
            });
        }
        let data_status = &object.data["status"];
        let endpoint = published_endpoint(&object.data);
        let status = if condition_true(data_status, &["Failed", "Failure", "Error"]) {
            ControlPlaneStatus::Failed
        } else if condition_true(data_status, &["Ready", "Available"]) {
            ControlPlaneStatus::Active
        } else if endpoint.is_some()
            || condition_true(data_status, &["EndpointReady", "ControlPlaneEndpointReady"])
        {
            ControlPlaneStatus::EndpointReady
        } else if condition_true(data_status, &["DatastoreReady", "DatastoreBound"])
            || phase_is(data_status, &["DatastoreBound", "datastore_bound"])
        {
            ControlPlaneStatus::DatastoreBound
        } else if condition_true(data_status, &["Reconciling", "Provisioning"])
            || phase_is(data_status, &["Provisioning", "provisioning"])
        {
            ControlPlaneStatus::Provisioning
        } else {
            ControlPlaneStatus::Requested
        };
        Ok(ProviderStatusMapping { status, endpoint })
    }

    /// Whether this dynamic object carries the required Oyatie owner label.
    #[must_use]
    pub fn is_owned_control_plane(object: &DynamicObject) -> bool {
        object
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get(OWNER_LABEL))
            .is_some_and(|owner| owner == OWNER_LABEL_VALUE)
    }

    /// Error returned when the rollback switch blocks new live provisioning.
    #[must_use]
    pub fn rollback_disabled_error() -> ProvisioningError {
        ProvisioningError::backend(
            "live provisioning disabled by rollback switch; status/teardown remain enabled for existing live control planes",
        )
    }

    /// The single historical explicit-deferral boundary retained for registry
    /// compatibility tests; live methods no longer return it on the happy path.
    fn deferred() -> ProvisioningError {
        ProvisioningError::Unimplemented(Unimplemented::KamajiProviderLiveIntegration)
    }

    fn hosted_handle_for(cluster_ref: &ClusterRef) -> HostedHandle {
        let tenant_hash = stable_hash_hex(&cluster_ref.tenant_id);
        let cluster = sanitize_k8s_segment(&cluster_ref.cluster_name, 36);
        HostedHandle {
            namespace: tenant_namespace(&cluster_ref.tenant_id),
            name: format!(
                "tcp-{cluster}-{}",
                stable_hash_hex(&format!(
                    "{}:{}",
                    cluster_ref.tenant_id, cluster_ref.cluster_name
                ))
            ),
        }
        .with_short_name(tenant_hash)
    }

    fn hosted_ref_for(cluster_ref: ClusterRef) -> ControlPlaneRef {
        let handle = Self::hosted_handle_for(&cluster_ref);
        ControlPlaneRef::new(
            cluster_ref,
            ControlPlaneTier::HostedKamaji,
            format!(
                "{HOSTED_HANDLE_PREFIX}/{}/{}",
                handle.namespace, handle.name
            ),
        )
    }

    fn parse_hosted_handle(reference: &ControlPlaneRef) -> Result<HostedHandle, ProvisioningError> {
        let parts = reference.handle.split('/').collect::<Vec<_>>();
        if parts.len() == 3 && parts[0] == HOSTED_HANDLE_PREFIX {
            return Ok(HostedHandle {
                namespace: parts[1].to_string(),
                name: parts[2].to_string(),
            });
        }
        Err(ProvisioningError::NotFound {
            handle: reference.handle.clone(),
        })
    }

    fn api_for(&self, handle: &HostedHandle) -> Api<DynamicObject> {
        Api::namespaced_with(
            self.client.clone(),
            &handle.namespace,
            &self.tenant_control_plane,
        )
    }

    fn map_kube_error(error: KubeError, context: &str) -> ProvisioningError {
        match error {
            KubeError::Api(status) if status.code == 404 && context == "teardown-get" => {
                ProvisioningError::NotFound {
                    handle: "already-deleted hosted Kamaji TenantControlPlane".to_string(),
                }
            }
            KubeError::Api(status) if status.code == 404 => ProvisioningError::NotFound {
                handle: status.message,
            },
            KubeError::Api(status) if status.code == 409 => ProvisioningError::backend(format!(
                "Kubernetes conflict during {context}: {status}"
            )),
            other => ProvisioningError::backend(format!("Kubernetes {context} failed: {other}")),
        }
    }
}

impl HostedHandle {
    fn with_short_name(mut self, tenant_hash: String) -> Self {
        if self.name.len() > 63 {
            self.name = format!(
                "tcp-{}-{}",
                &tenant_hash[..10],
                &stable_hash_hex(&self.name)[..10]
            );
        }
        self
    }
}

impl ControlPlaneProvisioning for CapiControlPlaneHost {
    fn provision<'a>(
        &'a self,
        request: &'a ProvisionRequest,
    ) -> BoxFuture<'a, Result<ControlPlaneRef, ProvisioningError>> {
        Box::pin(async move {
            validate_cluster_ref(&request.cluster_ref)?;
            match request.tier {
                ControlPlaneTier::DedicatedTalosSpoke => Self::dedicated_talos_reference(request),
                ControlPlaneTier::HostedKamaji => {
                    if !self.policy.allows_new_provisioning() {
                        return Err(Self::rollback_disabled_error());
                    }
                    let desired = Self::hosted_control_plane_object(request)?;
                    let reference = Self::hosted_ref_for(request.cluster_ref.clone());
                    let handle = Self::parse_hosted_handle(&reference)?;
                    let api = self.api_for(&handle);
                    match api.create(&PostParams::default(), &desired).await {
                        Ok(_) => Ok(reference),
                        Err(KubeError::Api(status)) if status.code == 409 => {
                            let existing = api.get(&handle.name).await.map_err(|error| {
                                Self::map_kube_error(error, "duplicate-provision-get")
                            })?;
                            if Self::is_owned_control_plane(&existing) {
                                Ok(reference)
                            } else {
                                Err(ProvisioningError::backend(
                                    "refusing duplicate provision over foreign Kamaji TenantControlPlane",
                                ))
                            }
                        }
                        Err(error) => Err(Self::map_kube_error(error, "create")),
                    }
                }
            }
        })
    }

    fn status<'a>(
        &'a self,
        control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<ControlPlaneStatusReport, ProvisioningError>> {
        Box::pin(async move {
            validate_cluster_ref(&control_plane_ref.cluster_ref)?;
            if control_plane_ref.tier == ControlPlaneTier::DedicatedTalosSpoke {
                return Ok(ControlPlaneStatusReport::new(
                    control_plane_ref.clone(),
                    ControlPlaneStatus::MediaFormed,
                    None,
                ));
            }
            let handle = Self::parse_hosted_handle(control_plane_ref)?;
            let object = self
                .api_for(&handle)
                .get(&handle.name)
                .await
                .map_err(|error| Self::map_kube_error(error, "get"))?;
            let mapped = Self::status_from_dynamic_object(&object)?;
            Ok(ControlPlaneStatusReport::new(
                control_plane_ref.clone(),
                mapped.status,
                mapped.endpoint,
            ))
        })
    }

    fn teardown<'a>(
        &'a self,
        control_plane_ref: &'a ControlPlaneRef,
    ) -> BoxFuture<'a, Result<(), ProvisioningError>> {
        Box::pin(async move {
            validate_cluster_ref(&control_plane_ref.cluster_ref)?;
            if control_plane_ref.tier == ControlPlaneTier::DedicatedTalosSpoke {
                return Ok(());
            }
            let handle = Self::parse_hosted_handle(control_plane_ref)?;
            let api = self.api_for(&handle);
            let object = match api.get(&handle.name).await {
                Ok(object) => object,
                Err(KubeError::Api(status)) if status.code == 404 => return Ok(()),
                Err(error) => return Err(Self::map_kube_error(error, "teardown-get")),
            };
            if !Self::is_owned_control_plane(&object) {
                return Err(ProvisioningError::backend(
                    "refusing to delete foreign Kamaji TenantControlPlane without Oyatie owner label",
                ));
            }
            api.delete(&handle.name, &DeleteParams::default())
                .await
                .map(|_| ())
                .or_else(|error| match error {
                    KubeError::Api(status) if status.code == 404 => Ok(()),
                    other => Err(Self::map_kube_error(other, "delete")),
                })
        })
    }
}

fn validate_cluster_ref(cluster_ref: &ClusterRef) -> Result<(), ProvisioningError> {
    if cluster_ref.is_well_formed() {
        Ok(())
    } else {
        Err(ProvisioningError::InvalidClusterRef {
            cluster_ref: cluster_ref.to_string(),
        })
    }
}

fn tenant_namespace(tenant_id: &str) -> String {
    format!("oya-cph-{}", &stable_hash_hex(tenant_id)[..12])
}

fn stable_hash_hex(value: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

fn sanitize_k8s_segment(value: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if mapped == '-' {
            if !last_dash && !out.is_empty() {
                out.push(mapped);
                last_dash = true;
            }
        } else {
            out.push(mapped);
            last_dash = false;
        }
        if out.len() >= max_len {
            break;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "cluster".to_string()
    } else {
        out
    }
}

fn condition_true(status: &Value, names: &[&str]) -> bool {
    status
        .get("conditions")
        .and_then(Value::as_array)
        .is_some_and(|conditions| {
            conditions.iter().any(|condition| {
                let condition_type = condition.get("type").and_then(Value::as_str);
                let condition_status = condition.get("status").and_then(Value::as_str);
                condition_status == Some("True")
                    && condition_type.is_some_and(|ty| names.contains(&ty))
            })
        })
}

fn phase_is(status: &Value, phases: &[&str]) -> bool {
    status
        .get("phase")
        .and_then(Value::as_str)
        .is_some_and(|phase| phases.contains(&phase))
}

fn published_endpoint(data: &Value) -> Option<String> {
    let status = data.get("status")?;
    if let Some(endpoint) = status.get("endpoint").and_then(Value::as_str) {
        return Some(endpoint.to_string());
    }
    let control_plane_endpoint = status.get("controlPlaneEndpoint")?;
    let host = control_plane_endpoint.get("host").and_then(Value::as_str)?;
    let port = control_plane_endpoint
        .get("port")
        .and_then(Value::as_u64)
        .unwrap_or(6443);
    Some(format!("https://{host}:{port}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_managed_k8s_control_plane_host_api::{ClusterRef, ProvisionRequest};
    use oya_managed_k8s_control_plane_host_kernel::{
        ControlPlaneStatus, ControlPlaneTier, DatastoreClass,
    };
    use serde_json::json;

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

    #[test]
    fn hosted_control_plane_object_uses_lts_gvk_and_safe_tenant_scope() {
        let request = ProvisionRequest::new(
            ClusterRef::new("tenant_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            DatastoreClass::EtcdPerTenant,
        );

        let planned = CapiControlPlaneHost::hosted_control_plane_object(&request)
            .expect("hosted object planned without kube API I/O");

        assert_eq!(
            planned.types.as_ref().map(|t| t.api_version.as_str()),
            Some("kamaji.clastix.io/v1alpha1")
        );
        assert_eq!(
            planned.types.as_ref().map(|t| t.kind.as_str()),
            Some(KAMAJI_TENANT_CONTROL_PLANE_KIND)
        );
        assert_eq!(
            planned
                .metadata
                .labels
                .as_ref()
                .and_then(|labels| labels.get("oya.io/owner")),
            Some(&"managed-k8s-control-plane-host".to_string())
        );
        assert_eq!(
            planned
                .metadata
                .labels
                .as_ref()
                .and_then(|labels| labels.get("oya.io/tier")),
            Some(&"hosted_kamaji".to_string())
        );
        assert!(
            planned
                .metadata
                .namespace
                .as_deref()
                .unwrap_or_default()
                .starts_with("oya-cph-")
        );
        assert_eq!(
            planned.data["spec"]["datastore"]["class"],
            json!("etcd_per_tenant")
        );
    }

    #[test]
    fn dedicated_talos_reference_does_not_build_hosted_kamaji_object() {
        let request = ProvisionRequest::new(
            ClusterRef::new("tenant_zero", "sovereign-a"),
            ControlPlaneTier::DedicatedTalosSpoke,
            DatastoreClass::EtcdPerTenant,
        );

        let reference = CapiControlPlaneHost::dedicated_talos_reference(&request)
            .expect("dedicated reference is resolved locally");

        assert_eq!(reference.tier, ControlPlaneTier::DedicatedTalosSpoke);
        assert!(reference.handle.starts_with("talos-capi/"));
        assert!(CapiControlPlaneHost::hosted_control_plane_object(&request).is_err());
    }

    #[test]
    fn provider_conditions_map_to_kernel_status_and_endpoint_only_when_reported() {
        let mut object = CapiControlPlaneHost::hosted_control_plane_object(&ProvisionRequest::new(
            ClusterRef::new("tenant_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            DatastoreClass::EtcdPerTenant,
        ))
        .expect("hosted object");

        object.data["status"] = json!({
            "conditions": [
                {"type": "DatastoreReady", "status": "True"},
                {"type": "Ready", "status": "False"}
            ]
        });
        let mapped = CapiControlPlaneHost::status_from_dynamic_object(&object)
            .expect("status maps without endpoint synthesis");
        assert_eq!(mapped.status, ControlPlaneStatus::DatastoreBound);
        assert_eq!(mapped.endpoint, None);

        object.data["status"] = json!({
            "conditions": [{"type": "Ready", "status": "True"}],
            "controlPlaneEndpoint": {"host": "api.dogfood-a.example", "port": 6443}
        });
        let mapped = CapiControlPlaneHost::status_from_dynamic_object(&object)
            .expect("ready condition maps active");
        assert_eq!(mapped.status, ControlPlaneStatus::Active);
        assert_eq!(
            mapped.endpoint.as_deref(),
            Some("https://api.dogfood-a.example:6443")
        );
    }

    #[test]
    fn teardown_guard_refuses_foreign_dynamic_objects() {
        let request = ProvisionRequest::new(
            ClusterRef::new("tenant_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            DatastoreClass::EtcdPerTenant,
        );
        let mut object =
            CapiControlPlaneHost::hosted_control_plane_object(&request).expect("hosted object");
        assert!(CapiControlPlaneHost::is_owned_control_plane(&object));

        object.metadata.labels = None;
        assert!(!CapiControlPlaneHost::is_owned_control_plane(&object));
    }

    #[test]
    fn rollback_switch_blocks_new_live_provisioning_only() {
        let policy = LiveProvisioningPolicy::new_provisions_disabled();
        assert!(!policy.allows_new_provisioning());
        assert!(policy.allows_status_and_teardown());
        assert!(
            CapiControlPlaneHost::rollback_disabled_error()
                .to_string()
                .contains("status/teardown remain enabled")
        );
    }
}
