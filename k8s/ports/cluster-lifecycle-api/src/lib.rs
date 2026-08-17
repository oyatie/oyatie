#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use k8s_cluster_lifecycle_kernel::{DesiredTier, LifecycleRequest, LifecycleValidationError};
use k8s_control_plane_host_api::{
    ClusterRef, ControlPlaneProvisioning, ControlPlaneRef, ControlPlaneTier, DatastoreClass,
    ProvisionRequest as ControlPlaneProvisionRequest, ProvisioningError,
};
use k8s_tenant_quota_api::{
    ProvisionRequest as QuotaProvisionRequest, QuotaDecision, QuotaDecisionPort, QuotaPortError,
};
use serde::{Deserialize, Serialize};

pub use k8s_cluster_lifecycle_kernel::{ClusterResourceRequest, DesiredTier as LifecycleTier};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LifecycleProvisioningResult {
    pub tenant_id: String,                    // data_class: TENANT_SCOPED
    pub cluster_name: String,                 // data_class: TENANT_SCOPED
    pub desired_tier: DesiredTier,            // data_class: TENANT_SCOPED
    pub control_plane_tier: ControlPlaneTier, // data_class: TENANT_SCOPED
    pub control_plane_ref: ControlPlaneRef,   // data_class: TENANT_SCOPED
}

impl LifecycleProvisioningResult {
    fn from_control_plane(request: &LifecycleRequest, control_plane_ref: ControlPlaneRef) -> Self {
        Self {
            tenant_id: request.tenant_id.clone(),
            cluster_name: request.cluster_name.clone(),
            desired_tier: request.desired_tier,
            control_plane_tier: control_plane_ref.tier,
            control_plane_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    InvalidRequest(LifecycleValidationError),
    QuotaDenied(String),
    QuotaUnavailable(String),
    ProvisioningFailed(ProvisioningError),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error) => write!(f, "invalid lifecycle request: {error}"),
            Self::QuotaDenied(reason) => write!(f, "quota denied lifecycle request: {reason}"),
            Self::QuotaUnavailable(error) => write!(f, "quota unavailable; fail-closed: {error}"),
            Self::ProvisioningFailed(error) => {
                write!(f, "control-plane provisioning failed: {error}")
            }
        }
    }
}
impl std::error::Error for LifecycleError {}

pub struct ClusterLifecycle<'a, Q, P>
where
    Q: QuotaDecisionPort + ?Sized,
    P: ControlPlaneProvisioning + ?Sized,
{
    quota: &'a Q,
    provisioning: &'a P,
}

impl<'a, Q, P> ClusterLifecycle<'a, Q, P>
where
    Q: QuotaDecisionPort + ?Sized,
    P: ControlPlaneProvisioning + ?Sized,
{
    #[must_use]
    pub const fn new(quota: &'a Q, provisioning: &'a P) -> Self {
        Self {
            quota,
            provisioning,
        }
    }

    pub async fn provision_cluster(
        &self,
        request: &LifecycleRequest,
    ) -> Result<LifecycleProvisioningResult, LifecycleError> {
        request.validate().map_err(LifecycleError::InvalidRequest)?;
        let quota_request = QuotaProvisionRequest::new(
            request.tenant_id.clone(),
            1,
            request.resources.nodes,
            request.resources.vcpu,
            request.resources.ram_gib,
        )
        .map_err(|error| LifecycleError::QuotaUnavailable(error.to_string()))?;
        match self.quota.check_quota(&quota_request) {
            Ok(QuotaDecision::Allow) => {}
            Ok(QuotaDecision::Deny(reason)) => {
                return Err(LifecycleError::QuotaDenied(reason.to_string()));
            }
            Err(QuotaPortError::NotFound(tenant)) => {
                return Err(LifecycleError::QuotaUnavailable(format!(
                    "quota record not found for tenant {tenant}"
                )));
            }
            Err(error) => return Err(LifecycleError::QuotaUnavailable(error.to_string())),
        }
        let control_plane_request = ControlPlaneProvisionRequest::new(
            ClusterRef::new(request.tenant_id.clone(), request.cluster_name.clone()),
            map_tier(request.desired_tier),
            DatastoreClass::EtcdPerTenant,
        );
        let control_plane_ref = self
            .provisioning
            .provision(&control_plane_request)
            .await
            .map_err(LifecycleError::ProvisioningFailed)?;
        Ok(LifecycleProvisioningResult::from_control_plane(
            request,
            control_plane_ref,
        ))
    }
}

#[must_use]
pub const fn map_tier(tier: DesiredTier) -> ControlPlaneTier {
    match tier {
        DesiredTier::Hosted => ControlPlaneTier::HostedKamaji,
        DesiredTier::Dedicated => ControlPlaneTier::DedicatedTalosSpoke,
    }
}

impl From<LifecycleValidationError> for LifecycleError {
    fn from(value: LifecycleValidationError) -> Self {
        Self::InvalidRequest(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_tenant_quota_api::DenyReason;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct StaticQuota(Result<QuotaDecision, QuotaPortError>);
    impl QuotaDecisionPort for StaticQuota {
        fn check_quota(
            &self,
            _request: &QuotaProvisionRequest,
        ) -> Result<QuotaDecision, QuotaPortError> {
            self.0.clone()
        }
    }

    struct SpyProvisioning {
        calls: AtomicUsize,
        result: Result<ControlPlaneRef, ProvisioningError>,
    }
    impl SpyProvisioning {
        fn new(result: Result<ControlPlaneRef, ProvisioningError>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                result,
            }
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }
    impl ControlPlaneProvisioning for SpyProvisioning {
        fn provision<'b>(
            &'b self,
            _request: &'b ControlPlaneProvisionRequest,
        ) -> k8s_control_plane_host_api::BoxFuture<'b, Result<ControlPlaneRef, ProvisioningError>>
        {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.result.clone()
            })
        }
        fn status<'b>(
            &'b self,
            _control_plane_ref: &'b ControlPlaneRef,
        ) -> k8s_control_plane_host_api::BoxFuture<
            'b,
            Result<k8s_control_plane_host_api::ControlPlaneStatusReport, ProvisioningError>,
        > {
            Box::pin(async move { Err(ProvisioningError::backend("unused")) })
        }
        fn teardown<'b>(
            &'b self,
            _control_plane_ref: &'b ControlPlaneRef,
        ) -> k8s_control_plane_host_api::BoxFuture<'b, Result<(), ProvisioningError>> {
            Box::pin(async move { Ok(()) })
        }
    }

    fn request() -> LifecycleRequest {
        LifecycleRequest::new(
            "ten_zero",
            "dogfood-a",
            DesiredTier::Hosted,
            ClusterResourceRequest::default_small(),
        )
        .unwrap()
    }
    fn cp_ref() -> ControlPlaneRef {
        ControlPlaneRef::new(
            ClusterRef::new("ten_zero", "dogfood-a"),
            ControlPlaneTier::HostedKamaji,
            "hosted_kamaji:ten_zero:dogfood-a",
        )
    }

    #[tokio::test]
    async fn allowed_hosted_default_flow_invokes_provisioning() {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let service = ClusterLifecycle::new(&quota, &provisioning);
        let result = service.provision_cluster(&request()).await.unwrap();
        assert_eq!(result.desired_tier, DesiredTier::Hosted);
        assert_eq!(result.control_plane_tier, ControlPlaneTier::HostedKamaji);
        assert_eq!(provisioning.calls(), 1);
    }

    #[tokio::test]
    async fn denied_quota_does_not_invoke_provisioning() {
        let quota = StaticQuota(Ok(QuotaDecision::Deny(DenyReason::ClusterLimitExceeded {
            current: 1,
            requested: 1,
            limit: 1,
        })));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let service = ClusterLifecycle::new(&quota, &provisioning);
        let err = service.provision_cluster(&request()).await.unwrap_err();
        assert!(matches!(err, LifecycleError::QuotaDenied(_)));
        assert_eq!(provisioning.calls(), 0);
    }

    #[tokio::test]
    async fn quota_not_found_fails_closed_without_provisioning() {
        let quota = StaticQuota(Err(QuotaPortError::NotFound("ten_zero".to_string())));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let service = ClusterLifecycle::new(&quota, &provisioning);
        let err = service.provision_cluster(&request()).await.unwrap_err();
        assert!(matches!(err, LifecycleError::QuotaUnavailable(_)));
        assert_eq!(provisioning.calls(), 0);
    }

    #[tokio::test]
    async fn malformed_request_fails_closed_without_provisioning() {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let service = ClusterLifecycle::new(&quota, &provisioning);
        let malformed = LifecycleRequest {
            tenant_id: "".to_string(),
            cluster_name: "dogfood-a".to_string(),
            desired_tier: DesiredTier::Hosted,
            resources: ClusterResourceRequest::default_small(),
        };
        let err = service.provision_cluster(&malformed).await.unwrap_err();
        assert!(matches!(err, LifecycleError::InvalidRequest(_)));
        assert_eq!(provisioning.calls(), 0);
    }

    #[tokio::test]
    async fn provisioning_failure_maps_after_quota_allow() {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Err(ProvisioningError::backend(
            "management cluster unavailable",
        )));
        let service = ClusterLifecycle::new(&quota, &provisioning);
        let err = service.provision_cluster(&request()).await.unwrap_err();
        assert!(matches!(
            err,
            LifecycleError::ProvisioningFailed(ProvisioningError::Backend { .. })
        ));
        assert_eq!(provisioning.calls(), 1);
    }
}
