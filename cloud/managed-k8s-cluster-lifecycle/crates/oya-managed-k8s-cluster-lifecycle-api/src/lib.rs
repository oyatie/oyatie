#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use oya_managed_k8s_cluster_lifecycle_kernel::{
    ClusterLifecycleState, DesiredTier, DrainAdmission, LifecycleRequest, LifecycleValidationError,
    evaluate_drain_admission,
};
use oya_managed_k8s_control_plane_host_api::{
    ClusterRef, ControlPlaneProvisioning, ControlPlaneRef, ControlPlaneTier, DatastoreClass,
    ProvisionRequest as ControlPlaneProvisionRequest, ProvisioningError,
};
use oya_managed_k8s_tenant_quota_api::{
    ProvisionRequest as QuotaProvisionRequest, QuotaDecision, QuotaDecisionPort, QuotaPortError,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub use oya_managed_k8s_cluster_lifecycle_kernel::{
    ClusterResourceRequest, DesiredTier as LifecycleTier,
};

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
    InvalidOperation(String),
    LedgerUnavailable(String),
    QuotaDenied(String),
    QuotaUnavailable(String),
    ProvisioningFailed(ProvisioningError),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error) => write!(f, "invalid lifecycle request: {error}"),
            Self::InvalidOperation(error) => write!(f, "invalid lifecycle operation: {error}"),
            Self::LedgerUnavailable(error) => write!(f, "operation ledger unavailable: {error}"),
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManagedClusterIdentity {
    pub organization_id: String,   // data_class: TENANT_SCOPED
    pub account_id: String,        // data_class: TENANT_SCOPED
    pub project_id: String,        // data_class: TENANT_SCOPED
    pub tenant_id: String,         // data_class: TENANT_SCOPED
    pub region: String,            // data_class: TENANT_SCOPED
    pub cell: String,              // data_class: TENANT_SCOPED
    pub resource_group_id: String, // data_class: TENANT_SCOPED
    pub cluster_name: String,      // data_class: TENANT_SCOPED
    pub cluster_orn: String,       // data_class: TENANT_SCOPED
}

impl ManagedClusterIdentity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: impl Into<String>,
        account_id: impl Into<String>,
        project_id: impl Into<String>,
        tenant_id: impl Into<String>,
        region: impl Into<String>,
        cell: impl Into<String>,
        resource_group_id: impl Into<String>,
        cluster_name: impl Into<String>,
    ) -> Result<Self, LifecycleError> {
        let organization_id = require_non_empty("organization_id", organization_id.into())?;
        let account_id = require_non_empty("account_id", account_id.into())?;
        let project_id = require_non_empty("project_id", project_id.into())?;
        let tenant_id = require_non_empty("tenant_id", tenant_id.into())?;
        let region = require_non_empty("region", region.into())?;
        let cell = require_non_empty("cell", cell.into())?;
        let resource_group_id = require_non_empty("resource_group_id", resource_group_id.into())?;
        let cluster_name = require_non_empty("cluster_name", cluster_name.into())?;
        let cluster_orn =
            format!("orn:oya:{region}:{account_id}:managed-k8s:cluster/{cluster_name}");
        Ok(Self {
            organization_id,
            account_id,
            project_id,
            tenant_id,
            region,
            cell,
            resource_group_id,
            cluster_name,
            cluster_orn,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Create,
    Update,
    Scale,
    Delete,
}

impl OperationKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Scale => "scale",
            Self::Delete => "delete",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationLifecycleState {
    Pending,
    QuotaDenied,
    Running,
    Succeeded,
    Failed,
    RollbackRunning,
    RolledBack,
    HoldManualReview,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneAction {
    None,
    Provision,
    Status,
    Teardown,
    HonestDeferredUpdate,
    HonestDeferredScale,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaDecisionSnapshot {
    Allow,
    Deny { reason: String },
    NotFound { tenant_id: String },
    Unavailable { reason: String },
    NotRequiredForReleaseOnly,
}

impl QuotaDecisionSnapshot {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny { .. } => "deny",
            Self::NotFound { .. } => "not_found",
            Self::Unavailable { .. } => "unavailable",
            Self::NotRequiredForReleaseOnly => "not_required_for_release_only",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SloEvidence {
    pub admission_latency_ms: Option<u64>, // data_class: INTERNAL_ONLY
    pub backend_actuation_latency_ms: Option<u64>, // data_class: INTERNAL_ONLY
    pub reconciliation_lag_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub error_class: Option<String>,       // data_class: INTERNAL_ONLY
    pub burn_or_hold_marker: Option<String>, // data_class: INTERNAL_ONLY
}

impl SloEvidence {
    #[must_use]
    pub fn deterministic_admission() -> Self {
        Self {
            admission_latency_ms: Some(0),
            backend_actuation_latency_ms: None,
            reconciliation_lag_seconds: None,
            error_class: None,
            burn_or_hold_marker: None,
        }
    }

    fn with_error(mut self, error_class: impl Into<String>) -> Self {
        let error_class = error_class.into();
        self.error_class = Some(error_class.clone());
        self.burn_or_hold_marker = Some(error_class);
        self
    }

    fn with_backend_latency(mut self) -> Self {
        self.backend_actuation_latency_ms = Some(0);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: String,                       // data_class: TENANT_SCOPED
    pub idempotency_key: String,                    // data_class: TENANT_SCOPED
    pub operation_kind: OperationKind,              // data_class: TENANT_SCOPED
    pub identity: ManagedClusterIdentity,           // data_class: TENANT_SCOPED
    pub requested_by_principal: String,             // data_class: TENANT_SCOPED
    pub requested_desired_state: String,            // data_class: TENANT_SCOPED
    pub prior_known_state: Option<String>,          // data_class: TENANT_SCOPED
    pub quota_decision: QuotaDecisionSnapshot,      // data_class: TENANT_SCOPED
    pub control_plane_action: ControlPlaneAction,   // data_class: TENANT_SCOPED
    pub control_plane_ref: Option<ControlPlaneRef>, // data_class: TENANT_SCOPED
    pub lifecycle_state: OperationLifecycleState,   // data_class: TENANT_SCOPED
    pub retry_count: u32,                           // data_class: INTERNAL_ONLY
    pub next_retry_epoch_seconds: Option<u64>,      // data_class: INTERNAL_ONLY
    pub last_error_class: Option<String>,           // data_class: INTERNAL_ONLY
    pub rollback_compensation_action: Option<String>, // data_class: INTERNAL_ONLY
    pub audit_id: String,                           // data_class: AUDIT
    pub audit_events: Vec<String>,                  // data_class: AUDIT
    pub slo_evidence: SloEvidence,                  // data_class: INTERNAL_ONLY
    pub metrics: Vec<String>,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterCreateOperationRequest {
    pub identity: ManagedClusterIdentity,
    pub idempotency_key: String,
    pub requested_by_principal: String,
    pub desired_tier: DesiredTier,
    pub resources: ClusterResourceRequest,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterUpdateOperationRequest {
    pub identity: ManagedClusterIdentity,
    pub idempotency_key: String,
    pub requested_by_principal: String,
    pub desired_tier: DesiredTier,
    pub prior_resources: ClusterResourceRequest,
    pub target_resources: ClusterResourceRequest,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterScaleOperationRequest {
    pub identity: ManagedClusterIdentity,
    pub idempotency_key: String,
    pub requested_by_principal: String,
    pub desired_tier: DesiredTier,
    pub current_resources: ClusterResourceRequest,
    pub target_resources: ClusterResourceRequest,
    pub observed_state_fresh: bool,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterDeleteOperationRequest {
    pub identity: ManagedClusterIdentity,
    pub idempotency_key: String,
    pub requested_by_principal: String,
    pub control_plane_ref: ControlPlaneRef,
    pub prior_state: ClusterLifecycleState,
    pub audit_correlation_id: String,
}

#[derive(Default)]
pub struct InMemoryOperationLedger {
    records: Mutex<BTreeMap<String, OperationRecord>>, // data_class: TENANT_SCOPED
}

impl InMemoryOperationLedger {
    #[must_use]
    pub fn len(&self) -> usize {
        self.records
            .lock()
            .map(|records| records.len())
            .unwrap_or(0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, key: &str) -> Result<Option<OperationRecord>, LifecycleError> {
        self.records
            .lock()
            .map_err(|_| {
                LifecycleError::LedgerUnavailable("operation ledger mutex poisoned".into())
            })
            .map(|records| records.get(key).cloned())
    }

    fn upsert(
        &self,
        key: String,
        record: OperationRecord,
    ) -> Result<OperationRecord, LifecycleError> {
        self.records
            .lock()
            .map_err(|_| {
                LifecycleError::LedgerUnavailable("operation ledger mutex poisoned".into())
            })?
            .insert(key, record.clone());
        Ok(record)
    }
}

pub struct ClusterLifecycleOperationService<'a, Q, P>
where
    Q: QuotaDecisionPort + ?Sized,
    P: ControlPlaneProvisioning + ?Sized,
{
    quota: &'a Q,
    provisioning: &'a P,
    ledger: &'a InMemoryOperationLedger,
}

impl<'a, Q, P> ClusterLifecycleOperationService<'a, Q, P>
where
    Q: QuotaDecisionPort + ?Sized,
    P: ControlPlaneProvisioning + ?Sized,
{
    #[must_use]
    pub const fn new(
        quota: &'a Q,
        provisioning: &'a P,
        ledger: &'a InMemoryOperationLedger,
    ) -> Self {
        Self {
            quota,
            provisioning,
            ledger,
        }
    }

    pub async fn create_cluster(
        &self,
        request: ClusterCreateOperationRequest,
    ) -> Result<OperationRecord, LifecycleError> {
        validate_operation_boundary(
            &request.identity,
            &request.idempotency_key,
            &request.requested_by_principal,
            &request.audit_correlation_id,
        )?;
        validate_resources(&request.resources)?;
        let key = ledger_key(
            OperationKind::Create,
            &request.identity,
            &request.idempotency_key,
        );
        if let Some(record) = self.ledger.get(&key)? {
            return Ok(record);
        }

        let mut record = base_record(
            OperationKind::Create,
            request.identity.clone(),
            request.idempotency_key.clone(),
            request.requested_by_principal.clone(),
            format!(
                "create:tier={}:nodes={}:vcpu={}:ram_gib={}",
                request.desired_tier.as_str(),
                request.resources.nodes,
                request.resources.vcpu,
                request.resources.ram_gib
            ),
            None,
            request.audit_correlation_id.clone(),
        );
        self.ledger.upsert(key.clone(), record.clone())?;

        record.quota_decision =
            self.check_quota_for_resources(&request.identity, &request.resources);
        match &record.quota_decision {
            QuotaDecisionSnapshot::Allow => {
                record.lifecycle_state = OperationLifecycleState::Running;
                record.control_plane_action = ControlPlaneAction::Provision;
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.quota_allowed".into());
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.control_plane_call_started".into());
                let control_plane_request = ControlPlaneProvisionRequest::new(
                    ClusterRef::new(
                        request.identity.tenant_id.clone(),
                        request.identity.cluster_name.clone(),
                    ),
                    map_tier(request.desired_tier),
                    DatastoreClass::EtcdPerTenant,
                );
                match self.provisioning.provision(&control_plane_request).await {
                    Ok(control_plane_ref) => {
                        record.lifecycle_state = OperationLifecycleState::Succeeded;
                        record.control_plane_ref = Some(control_plane_ref);
                        record.slo_evidence = record.slo_evidence.with_backend_latency();
                        record
                            .audit_events
                            .push("managed_k8s.cluster.create.control_plane_call_succeeded".into());
                        record
                            .audit_events
                            .push("managed_k8s.cluster.create.succeeded".into());
                    }
                    Err(error) => {
                        record.lifecycle_state = hold_or_failed_for_provisioning_error(&error);
                        record.last_error_class = Some(provisioning_error_class(&error));
                        record.rollback_compensation_action =
                            Some("rollback:no_control_plane_ref_to_teardown".into());
                        record.slo_evidence = record
                            .slo_evidence
                            .with_error(record.last_error_class.clone().unwrap_or_default());
                        record
                            .audit_events
                            .push("managed_k8s.cluster.create.control_plane_call_failed".into());
                        record
                            .audit_events
                            .push("managed_k8s.cluster.create.rollback_started".into());
                        if record.lifecycle_state == OperationLifecycleState::HoldManualReview {
                            record
                                .audit_events
                                .push("managed_k8s.cluster.create.hold_manual_review".into());
                        }
                    }
                }
            }
            QuotaDecisionSnapshot::Deny { .. } => {
                record.lifecycle_state = OperationLifecycleState::QuotaDenied;
                record.control_plane_action = ControlPlaneAction::None;
                record.last_error_class = Some("quota_denied".into());
                record.slo_evidence = record.slo_evidence.with_error("quota_denied");
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.quota_denied".into());
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.quota_deny".into());
            }
            QuotaDecisionSnapshot::NotFound { .. } => {
                record.lifecycle_state = OperationLifecycleState::HoldManualReview;
                record.control_plane_action = ControlPlaneAction::None;
                record.last_error_class = Some("quota_not_found".into());
                record.slo_evidence = record.slo_evidence.with_error("quota_not_found");
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.quota_not_found".into());
            }
            QuotaDecisionSnapshot::Unavailable { .. } => {
                record.lifecycle_state = OperationLifecycleState::HoldManualReview;
                record.control_plane_action = ControlPlaneAction::None;
                record.last_error_class = Some("quota_unavailable".into());
                record.slo_evidence = record.slo_evidence.with_error("quota_unavailable");
                record
                    .audit_events
                    .push("managed_k8s.cluster.create.quota_unavailable".into());
            }
            QuotaDecisionSnapshot::NotRequiredForReleaseOnly => {}
        }
        self.ledger.upsert(key, record)
    }

    pub async fn update_cluster(
        &self,
        request: ClusterUpdateOperationRequest,
    ) -> Result<OperationRecord, LifecycleError> {
        validate_operation_boundary(
            &request.identity,
            &request.idempotency_key,
            &request.requested_by_principal,
            &request.audit_correlation_id,
        )?;
        validate_resources(&request.prior_resources)?;
        validate_resources(&request.target_resources)?;
        let key = ledger_key(
            OperationKind::Update,
            &request.identity,
            &request.idempotency_key,
        );
        if let Some(record) = self.ledger.get(&key)? {
            return Ok(record);
        }
        let mut record = base_record(
            OperationKind::Update,
            request.identity.clone(),
            request.idempotency_key.clone(),
            request.requested_by_principal.clone(),
            format!(
                "update:tier={}:nodes={}:vcpu={}:ram_gib={}",
                request.desired_tier.as_str(),
                request.target_resources.nodes,
                request.target_resources.vcpu,
                request.target_resources.ram_gib
            ),
            Some(format!(
                "nodes={}:vcpu={}:ram_gib={}",
                request.prior_resources.nodes,
                request.prior_resources.vcpu,
                request.prior_resources.ram_gib
            )),
            request.audit_correlation_id,
        );
        self.ledger.upsert(key.clone(), record.clone())?;
        record.quota_decision =
            if positive_delta(&request.prior_resources, &request.target_resources) {
                self.check_quota_for_resources(&request.identity, &request.target_resources)
            } else {
                QuotaDecisionSnapshot::NotRequiredForReleaseOnly
            };
        finalize_deferred_record(
            &mut record,
            ControlPlaneAction::HonestDeferredUpdate,
            "update",
        );
        self.ledger.upsert(key, record)
    }

    pub async fn scale_cluster(
        &self,
        request: ClusterScaleOperationRequest,
    ) -> Result<OperationRecord, LifecycleError> {
        validate_operation_boundary(
            &request.identity,
            &request.idempotency_key,
            &request.requested_by_principal,
            &request.audit_correlation_id,
        )?;
        validate_resources(&request.current_resources)?;
        validate_resources(&request.target_resources)?;
        let key = ledger_key(
            OperationKind::Scale,
            &request.identity,
            &request.idempotency_key,
        );
        if let Some(record) = self.ledger.get(&key)? {
            return Ok(record);
        }
        let mut record = base_record(
            OperationKind::Scale,
            request.identity.clone(),
            request.idempotency_key.clone(),
            request.requested_by_principal.clone(),
            format!(
                "scale:tier={}:target_nodes={}",
                request.desired_tier.as_str(),
                request.target_resources.nodes
            ),
            Some(format!("current_nodes={}", request.current_resources.nodes)),
            request.audit_correlation_id,
        );
        self.ledger.upsert(key.clone(), record.clone())?;

        if request.target_resources.nodes > request.current_resources.nodes {
            record.quota_decision =
                self.check_quota_for_resources(&request.identity, &request.target_resources);
            finalize_deferred_record(
                &mut record,
                ControlPlaneAction::HonestDeferredScale,
                "scale",
            );
        } else {
            record.quota_decision = QuotaDecisionSnapshot::NotRequiredForReleaseOnly;
            record.control_plane_action = ControlPlaneAction::HonestDeferredScale;
            record.lifecycle_state = OperationLifecycleState::HoldManualReview;
            if !request.observed_state_fresh {
                record.last_error_class = Some("stale_observation_hold".into());
                record.slo_evidence = record.slo_evidence.with_error("stale_observation_hold");
                record
                    .audit_events
                    .push("managed_k8s.cluster.scale.hold_manual_review".into());
            } else if request.target_resources.nodes < request.current_resources.nodes {
                let drain_target = request.current_resources.nodes - request.target_resources.nodes;
                match evaluate_drain_admission(
                    request.current_resources.nodes,
                    drain_target,
                    request.desired_tier,
                ) {
                    DrainAdmission::Allow => {
                        record.last_error_class = Some("control_plane_scale_port_missing".into());
                        record.slo_evidence = record
                            .slo_evidence
                            .with_error("control_plane_scale_port_missing");
                    }
                    DrainAdmission::Deny { reason } => {
                        record.last_error_class = Some(format!("drain_denied:{reason}"));
                        record.slo_evidence = record.slo_evidence.with_error("drain_denied");
                    }
                }
            } else {
                record.last_error_class = Some("scale_noop_observation_hold".into());
                record.slo_evidence = record
                    .slo_evidence
                    .with_error("scale_noop_observation_hold");
            }
        }
        self.ledger.upsert(key, record)
    }

    pub async fn delete_cluster(
        &self,
        request: ClusterDeleteOperationRequest,
    ) -> Result<OperationRecord, LifecycleError> {
        validate_operation_boundary(
            &request.identity,
            &request.idempotency_key,
            &request.requested_by_principal,
            &request.audit_correlation_id,
        )?;
        let key = ledger_key(
            OperationKind::Delete,
            &request.identity,
            &request.idempotency_key,
        );
        if let Some(record) = self.ledger.get(&key)? {
            return Ok(record);
        }
        let mut record = base_record(
            OperationKind::Delete,
            request.identity,
            request.idempotency_key,
            request.requested_by_principal,
            "delete:target_state=deleted".to_string(),
            Some(request.prior_state.as_str().to_string()),
            request.audit_correlation_id,
        );
        record.quota_decision = QuotaDecisionSnapshot::NotRequiredForReleaseOnly;
        record.control_plane_action = ControlPlaneAction::Teardown;
        record.lifecycle_state = OperationLifecycleState::Running;
        record.control_plane_ref = Some(request.control_plane_ref.clone());
        record
            .audit_events
            .push("managed_k8s.cluster.delete.control_plane_call_started".into());
        self.ledger.upsert(key.clone(), record.clone())?;

        match self.provisioning.teardown(&request.control_plane_ref).await {
            Ok(()) => {
                record.lifecycle_state = OperationLifecycleState::Succeeded;
                record.slo_evidence = record.slo_evidence.with_backend_latency();
                record
                    .audit_events
                    .push("managed_k8s.cluster.delete.control_plane_call_succeeded".into());
                record
                    .audit_events
                    .push("managed_k8s.cluster.delete.succeeded".into());
            }
            Err(error) => {
                record.lifecycle_state = OperationLifecycleState::HoldManualReview;
                record.last_error_class = Some(provisioning_error_class(&error));
                record.rollback_compensation_action =
                    Some("hold:teardown_failed_preserve_registry_and_ledger".into());
                record.slo_evidence = record
                    .slo_evidence
                    .with_error(record.last_error_class.clone().unwrap_or_default());
                record
                    .audit_events
                    .push("managed_k8s.cluster.delete.control_plane_call_failed".into());
                record
                    .audit_events
                    .push("managed_k8s.cluster.delete.hold_manual_review".into());
            }
        }
        self.ledger.upsert(key, record)
    }

    fn check_quota_for_resources(
        &self,
        identity: &ManagedClusterIdentity,
        resources: &ClusterResourceRequest,
    ) -> QuotaDecisionSnapshot {
        match QuotaProvisionRequest::new(
            identity.tenant_id.clone(),
            1,
            resources.nodes,
            resources.vcpu,
            resources.ram_gib,
        ) {
            Ok(request) => match self.quota.check_quota(&request) {
                Ok(QuotaDecision::Allow) => QuotaDecisionSnapshot::Allow,
                Ok(QuotaDecision::Deny(reason)) => QuotaDecisionSnapshot::Deny {
                    reason: reason.to_string(),
                },
                Err(QuotaPortError::NotFound(tenant_id)) => {
                    QuotaDecisionSnapshot::NotFound { tenant_id }
                }
                Err(error) => QuotaDecisionSnapshot::Unavailable {
                    reason: error.to_string(),
                },
            },
            Err(error) => QuotaDecisionSnapshot::Unavailable {
                reason: error.to_string(),
            },
        }
    }
}

fn base_record(
    operation_kind: OperationKind,
    identity: ManagedClusterIdentity,
    idempotency_key: String,
    requested_by_principal: String,
    requested_desired_state: String,
    prior_known_state: Option<String>,
    audit_id: String,
) -> OperationRecord {
    let operation_id = operation_id_for(operation_kind, &identity, &idempotency_key);
    OperationRecord {
        operation_id,
        idempotency_key,
        operation_kind,
        identity,
        requested_by_principal,
        requested_desired_state,
        prior_known_state,
        quota_decision: QuotaDecisionSnapshot::Unavailable {
            reason: "not_evaluated".into(),
        },
        control_plane_action: ControlPlaneAction::None,
        control_plane_ref: None,
        lifecycle_state: OperationLifecycleState::Pending,
        retry_count: 0,
        next_retry_epoch_seconds: None,
        last_error_class: None,
        rollback_compensation_action: None,
        audit_id,
        audit_events: vec![format!(
            "managed_k8s.cluster.{}.requested",
            operation_kind.as_str()
        )],
        slo_evidence: SloEvidence::deterministic_admission(),
        metrics: vec![
            "managed_k8s_cluster_lifecycle_operation_started_total".into(),
            "managed_k8s_cluster_lifecycle_admission_duration_seconds".into(),
        ],
    }
}

fn finalize_deferred_record(
    record: &mut OperationRecord,
    action: ControlPlaneAction,
    kind: &'static str,
) {
    match &record.quota_decision {
        QuotaDecisionSnapshot::Allow | QuotaDecisionSnapshot::NotRequiredForReleaseOnly => {
            record.control_plane_action = action;
            record.lifecycle_state = OperationLifecycleState::HoldManualReview;
            record.last_error_class = Some(format!("control_plane_{kind}_port_missing"));
            record.slo_evidence = record
                .slo_evidence
                .clone()
                .with_error(format!("control_plane_{kind}_port_missing"));
            record
                .audit_events
                .push(format!("managed_k8s.cluster.{kind}.hold_manual_review"));
        }
        QuotaDecisionSnapshot::Deny { .. } => {
            record.control_plane_action = ControlPlaneAction::None;
            record.lifecycle_state = OperationLifecycleState::QuotaDenied;
            record.last_error_class = Some("quota_denied".into());
            record.slo_evidence = record.slo_evidence.clone().with_error("quota_denied");
            record
                .audit_events
                .push(format!("managed_k8s.cluster.{kind}.quota_denied"));
        }
        QuotaDecisionSnapshot::NotFound { .. } => {
            record.control_plane_action = ControlPlaneAction::None;
            record.lifecycle_state = OperationLifecycleState::HoldManualReview;
            record.last_error_class = Some("quota_not_found".into());
            record.slo_evidence = record.slo_evidence.clone().with_error("quota_not_found");
            record
                .audit_events
                .push(format!("managed_k8s.cluster.{kind}.quota_not_found"));
        }
        QuotaDecisionSnapshot::Unavailable { .. } => {
            record.control_plane_action = ControlPlaneAction::None;
            record.lifecycle_state = OperationLifecycleState::HoldManualReview;
            record.last_error_class = Some("quota_unavailable".into());
            record.slo_evidence = record.slo_evidence.clone().with_error("quota_unavailable");
            record
                .audit_events
                .push(format!("managed_k8s.cluster.{kind}.quota_unavailable"));
        }
    }
}

fn validate_operation_boundary(
    identity: &ManagedClusterIdentity,
    idempotency_key: &str,
    requested_by_principal: &str,
    audit_correlation_id: &str,
) -> Result<(), LifecycleError> {
    require_non_empty_ref("cluster_orn", &identity.cluster_orn)?;
    require_non_empty_ref("idempotency_key", idempotency_key)?;
    require_non_empty_ref("requested_by_principal", requested_by_principal)?;
    require_non_empty_ref("audit_correlation_id", audit_correlation_id)?;
    Ok(())
}

fn validate_resources(resources: &ClusterResourceRequest) -> Result<(), LifecycleError> {
    if resources.nodes == 0 {
        return Err(LifecycleError::InvalidOperation("nodes must be > 0".into()));
    }
    if resources.vcpu == 0 {
        return Err(LifecycleError::InvalidOperation("vcpu must be > 0".into()));
    }
    if resources.ram_gib == 0 {
        return Err(LifecycleError::InvalidOperation(
            "ram_gib must be > 0".into(),
        ));
    }
    Ok(())
}

fn positive_delta(prior: &ClusterResourceRequest, target: &ClusterResourceRequest) -> bool {
    target.nodes > prior.nodes || target.vcpu > prior.vcpu || target.ram_gib > prior.ram_gib
}

fn require_non_empty(field: &'static str, value: String) -> Result<String, LifecycleError> {
    require_non_empty_ref(field, &value)?;
    Ok(value)
}

fn require_non_empty_ref(field: &'static str, value: &str) -> Result<(), LifecycleError> {
    if value.trim().is_empty() {
        return Err(LifecycleError::InvalidOperation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn ledger_key(
    kind: OperationKind,
    identity: &ManagedClusterIdentity,
    idempotency_key: &str,
) -> String {
    format!(
        "{}|{}|{}|{}",
        kind.as_str(),
        identity.tenant_id,
        identity.cluster_orn,
        idempotency_key
    )
}

fn operation_id_for(
    kind: OperationKind,
    identity: &ManagedClusterIdentity,
    idempotency_key: &str,
) -> String {
    format!(
        "op_{}_{}_{}_{}",
        kind.as_str(),
        sanitize_id(&identity.tenant_id),
        sanitize_id(&identity.cluster_name),
        sanitize_id(idempotency_key)
    )
}

fn sanitize_id(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    sanitized.trim_matches('_').to_string()
}

fn hold_or_failed_for_provisioning_error(error: &ProvisioningError) -> OperationLifecycleState {
    match error {
        ProvisioningError::Unimplemented(_) | ProvisioningError::Backend { .. } => {
            OperationLifecycleState::HoldManualReview
        }
        ProvisioningError::InvalidClusterRef { .. }
        | ProvisioningError::NotFound { .. }
        | ProvisioningError::IllegalTransition(_) => OperationLifecycleState::Failed,
    }
}

fn provisioning_error_class(error: &ProvisioningError) -> String {
    match error {
        ProvisioningError::InvalidClusterRef { .. } => "invalid_cluster_ref".into(),
        ProvisioningError::NotFound { .. } => "control_plane_not_found".into(),
        ProvisioningError::IllegalTransition(_) => "illegal_transition".into(),
        ProvisioningError::Backend { .. } => "backend_unavailable".into(),
        ProvisioningError::Unimplemented(boundary) => {
            format!("unimplemented:{}", boundary.placeholder_debt_id())
        }
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
    use oya_managed_k8s_tenant_quota_api::DenyReason;
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
        provision_calls: AtomicUsize,
        teardown_calls: AtomicUsize,
        result: Result<ControlPlaneRef, ProvisioningError>,
    }
    impl SpyProvisioning {
        fn new(result: Result<ControlPlaneRef, ProvisioningError>) -> Self {
            Self {
                provision_calls: AtomicUsize::new(0),
                teardown_calls: AtomicUsize::new(0),
                result,
            }
        }
        fn calls(&self) -> usize {
            self.provision_calls()
        }
        fn provision_calls(&self) -> usize {
            self.provision_calls.load(Ordering::SeqCst)
        }
        fn teardown_calls(&self) -> usize {
            self.teardown_calls.load(Ordering::SeqCst)
        }
    }
    impl ControlPlaneProvisioning for SpyProvisioning {
        fn provision<'b>(
            &'b self,
            _request: &'b ControlPlaneProvisionRequest,
        ) -> oya_managed_k8s_control_plane_host_api::BoxFuture<
            'b,
            Result<ControlPlaneRef, ProvisioningError>,
        > {
            Box::pin(async move {
                self.provision_calls.fetch_add(1, Ordering::SeqCst);
                self.result.clone()
            })
        }
        fn status<'b>(
            &'b self,
            _control_plane_ref: &'b ControlPlaneRef,
        ) -> oya_managed_k8s_control_plane_host_api::BoxFuture<
            'b,
            Result<
                oya_managed_k8s_control_plane_host_api::ControlPlaneStatusReport,
                ProvisioningError,
            >,
        > {
            Box::pin(async move { Err(ProvisioningError::backend("unused")) })
        }
        fn teardown<'b>(
            &'b self,
            _control_plane_ref: &'b ControlPlaneRef,
        ) -> oya_managed_k8s_control_plane_host_api::BoxFuture<'b, Result<(), ProvisioningError>>
        {
            Box::pin(async move {
                self.teardown_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
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

    fn identity() -> ManagedClusterIdentity {
        ManagedClusterIdentity::new(
            "org_oya",
            "acct_dogfood",
            "proj_k8s",
            "ten_zero",
            "kr-central-1",
            "cell-a",
            "rg-platform",
            "dogfood-a",
        )
        .expect("valid managed cluster identity")
    }

    fn create_operation_request(idempotency_key: &str) -> ClusterCreateOperationRequest {
        ClusterCreateOperationRequest {
            identity: identity(),
            idempotency_key: idempotency_key.to_string(),
            requested_by_principal: "sp:gateway:dogfood".to_string(),
            desired_tier: DesiredTier::Hosted,
            resources: ClusterResourceRequest::default_small(),
            audit_correlation_id: "audit-create-dogfood-a".to_string(),
        }
    }

    fn update_operation_request(idempotency_key: &str) -> ClusterUpdateOperationRequest {
        ClusterUpdateOperationRequest {
            identity: identity(),
            idempotency_key: idempotency_key.to_string(),
            requested_by_principal: "sp:gateway:dogfood".to_string(),
            desired_tier: DesiredTier::Hosted,
            prior_resources: ClusterResourceRequest::default_small(),
            target_resources: ClusterResourceRequest::new(4, 12, 48),
            audit_correlation_id: "audit-update-dogfood-a".to_string(),
        }
    }

    fn scale_operation_request(
        idempotency_key: &str,
        target_nodes: u32,
    ) -> ClusterScaleOperationRequest {
        ClusterScaleOperationRequest {
            identity: identity(),
            idempotency_key: idempotency_key.to_string(),
            requested_by_principal: "sp:gateway:dogfood".to_string(),
            desired_tier: DesiredTier::Dedicated,
            current_resources: ClusterResourceRequest::new(4, 16, 64),
            target_resources: ClusterResourceRequest::new(target_nodes, 16, 64),
            observed_state_fresh: true,
            audit_correlation_id: "audit-scale-dogfood-a".to_string(),
        }
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

    #[tokio::test]
    async fn create_operation_writes_identity_ledger_and_idempotently_calls_provision_once() {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let ledger = InMemoryOperationLedger::default();
        let service = ClusterLifecycleOperationService::new(&quota, &provisioning, &ledger);

        let first = service
            .create_cluster(create_operation_request("idem-create-dogfood-a"))
            .await
            .expect("quota-allowed create is accepted into the operation ledger");
        let replay = service
            .create_cluster(create_operation_request("idem-create-dogfood-a"))
            .await
            .expect("matching idempotency key replays existing operation");

        assert_eq!(first, replay);
        assert_eq!(provisioning.provision_calls(), 1);
        assert_eq!(ledger.len(), 1);
        assert_eq!(first.operation_kind, OperationKind::Create);
        assert_eq!(first.lifecycle_state, OperationLifecycleState::Succeeded);
        assert_eq!(first.control_plane_action, ControlPlaneAction::Provision);
        assert!(matches!(first.quota_decision, QuotaDecisionSnapshot::Allow));
        assert_eq!(first.identity.account_id, "acct_dogfood");
        assert_eq!(first.identity.project_id, "proj_k8s");
        assert_eq!(first.identity.region, "kr-central-1");
        assert_eq!(first.identity.cell, "cell-a");
        assert_eq!(first.identity.resource_group_id, "rg-platform");
        assert_eq!(
            first.identity.cluster_orn,
            "orn:oya:kr-central-1:acct_dogfood:managed-k8s:cluster/dogfood-a"
        );
        assert_eq!(first.idempotency_key, "idem-create-dogfood-a");
        assert_eq!(first.audit_id, "audit-create-dogfood-a");
        assert!(first.slo_evidence.admission_latency_ms.is_some());
        assert!(
            first
                .audit_events
                .iter()
                .any(|event| event == "managed_k8s.cluster.create.requested")
        );
        assert!(
            first
                .metrics
                .iter()
                .any(|metric| metric == "managed_k8s_cluster_lifecycle_operation_started_total")
        );
    }

    #[tokio::test]
    async fn create_operation_quota_denied_notfound_and_persistence_never_call_backend() {
        for (quota_result, expected_decision, expected_state) in [
            (
                Ok(QuotaDecision::Deny(DenyReason::ClusterLimitExceeded {
                    current: 1,
                    requested: 1,
                    limit: 1,
                })),
                "deny",
                OperationLifecycleState::QuotaDenied,
            ),
            (
                Err(QuotaPortError::NotFound("ten_zero".to_string())),
                "not_found",
                OperationLifecycleState::HoldManualReview,
            ),
            (
                Err(QuotaPortError::Persistence(
                    "quota store unavailable".to_string(),
                )),
                "unavailable",
                OperationLifecycleState::HoldManualReview,
            ),
        ] {
            let quota = StaticQuota(quota_result);
            let provisioning = SpyProvisioning::new(Ok(cp_ref()));
            let ledger = InMemoryOperationLedger::default();
            let service = ClusterLifecycleOperationService::new(&quota, &provisioning, &ledger);

            let record = service
                .create_cluster(create_operation_request(&format!(
                    "idem-create-{expected_decision}"
                )))
                .await
                .expect("quota failure is recorded as a ledger operation, not a backend call");

            assert_eq!(provisioning.provision_calls(), 0, "{expected_decision}");
            assert_eq!(
                record.lifecycle_state, expected_state,
                "{expected_decision}"
            );
            assert_eq!(
                record.control_plane_action,
                ControlPlaneAction::None,
                "{expected_decision}"
            );
            assert_eq!(record.quota_decision.as_str(), expected_decision);
            assert!(record.control_plane_ref.is_none(), "{expected_decision}");
            assert!(
                record
                    .audit_events
                    .iter()
                    .any(|event| event.contains(expected_decision)),
                "{expected_decision}"
            );
        }
    }

    #[tokio::test]
    async fn delete_operation_tears_down_idempotently_and_preserves_ledger_state() {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let ledger = InMemoryOperationLedger::default();
        let service = ClusterLifecycleOperationService::new(&quota, &provisioning, &ledger);
        let created = service
            .create_cluster(create_operation_request("idem-create-before-delete"))
            .await
            .expect("create produces a control-plane ref");
        let control_plane_ref = created.control_plane_ref.clone().expect("cp ref");

        let delete = ClusterDeleteOperationRequest {
            identity: identity(),
            idempotency_key: "idem-delete-dogfood-a".to_string(),
            requested_by_principal: "sp:gateway:dogfood".to_string(),
            control_plane_ref,
            prior_state: ClusterLifecycleState::Ready,
            audit_correlation_id: "audit-delete-dogfood-a".to_string(),
        };
        let first = service
            .delete_cluster(delete.clone())
            .await
            .expect("delete accepted");
        let replay = service
            .delete_cluster(delete)
            .await
            .expect("delete replay accepted");

        assert_eq!(first, replay);
        assert_eq!(provisioning.teardown_calls(), 1);
        assert_eq!(
            ledger.len(),
            2,
            "create and delete records are both preserved"
        );
        assert_eq!(first.operation_kind, OperationKind::Delete);
        assert_eq!(first.lifecycle_state, OperationLifecycleState::Succeeded);
        assert_eq!(first.control_plane_action, ControlPlaneAction::Teardown);
        assert!(matches!(
            first.quota_decision,
            QuotaDecisionSnapshot::NotRequiredForReleaseOnly
        ));
        assert_eq!(first.prior_known_state.as_deref(), Some("ready"));
        assert!(first.requested_desired_state.contains("deleted"));
    }

    #[tokio::test]
    async fn update_and_scale_record_honest_deferred_or_hold_semantics_without_fake_backend_success()
     {
        let quota = StaticQuota(Ok(QuotaDecision::Allow));
        let provisioning = SpyProvisioning::new(Ok(cp_ref()));
        let ledger = InMemoryOperationLedger::default();
        let service = ClusterLifecycleOperationService::new(&quota, &provisioning, &ledger);

        let update = service
            .update_cluster(update_operation_request("idem-update-dogfood-a"))
            .await
            .expect("update ledger record accepted");
        assert_eq!(update.operation_kind, OperationKind::Update);
        assert_eq!(
            update.control_plane_action,
            ControlPlaneAction::HonestDeferredUpdate
        );
        assert_eq!(
            update.lifecycle_state,
            OperationLifecycleState::HoldManualReview
        );
        assert!(matches!(
            update.quota_decision,
            QuotaDecisionSnapshot::Allow
        ));

        let scale_up = service
            .scale_cluster(scale_operation_request("idem-scale-up-dogfood-a", 6))
            .await
            .expect("scale-up ledger record accepted");
        assert_eq!(scale_up.operation_kind, OperationKind::Scale);
        assert_eq!(
            scale_up.control_plane_action,
            ControlPlaneAction::HonestDeferredScale
        );
        assert!(matches!(
            scale_up.quota_decision,
            QuotaDecisionSnapshot::Allow
        ));
        assert_eq!(
            scale_up.lifecycle_state,
            OperationLifecycleState::HoldManualReview
        );

        let stale_scale_down = ClusterScaleOperationRequest {
            observed_state_fresh: false,
            ..scale_operation_request("idem-stale-scale-down-dogfood-a", 3)
        };
        let stale = service
            .scale_cluster(stale_scale_down)
            .await
            .expect("stale scale-down is held, not released");
        assert_eq!(
            stale.lifecycle_state,
            OperationLifecycleState::HoldManualReview
        );
        assert_eq!(
            stale.quota_decision,
            QuotaDecisionSnapshot::NotRequiredForReleaseOnly
        );
        assert!(
            stale
                .last_error_class
                .as_deref()
                .unwrap_or_default()
                .contains("stale_observation")
        );

        let below_floor = service
            .scale_cluster(scale_operation_request("idem-scale-down-below-floor", 2))
            .await
            .expect("below-floor scale-down is recorded as a hold");
        assert_eq!(
            below_floor.lifecycle_state,
            OperationLifecycleState::HoldManualReview
        );
        assert!(
            below_floor
                .last_error_class
                .as_deref()
                .unwrap_or_default()
                .contains("drain_denied")
        );
        assert_eq!(provisioning.provision_calls(), 0);
        assert_eq!(provisioning.teardown_calls(), 0);
    }
}
