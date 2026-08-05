//! Managed-Kubernetes SLA observability composition layer.
//!
//! This crate wires the SLA observation store to the settled
//! `ControlPlaneProvisioning` status port from `oya-managed-k8s-control-plane-host-api`.
//! It deliberately exposes no live Prometheus/Kubernetes dependency; live
//! metrics scraping is deferred to a future adapter behind the same port.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_managed_k8s_control_plane_host_api as control_plane_host;

pub use control_plane_host::{ControlPlaneProvisioning, ControlPlaneRef, ProvisioningError};
pub use oya_managed_k8s_sla_observability_adapter_inmemory::InMemorySlaObservabilityStore;
pub use oya_managed_k8s_sla_observability_api::{
    BurnRateEvidenceWindows, ClusterRef, ControlPlaneSlaSnapshot, FleetSlaSummary,
    MANAGED_CLUSTER_AVAILABILITY_OPENSLO_PATH, ObservationWindow,
    PROVISIONING_LATENCY_OPENSLO_PATH, RollbackHoldReason, SlaEvidenceContext,
    SlaEvidenceEmission, SlaEvidenceFreshness, SlaEvidencePlacement, SlaEvidenceSource,
    SlaLiveObservation, SlaObservabilityError, SlaObservabilityPort, StatusWindow,
    TenantSlaReadScope, summarize_burn_rate_alert, summarize_fleet_sla,
};

pub use oya_managed_k8s_sla_observability_kernel::{
    AlertSeverity, AvailabilityState, BurnRatePolicy, ObservedControlPlaneStatus,
    ProvisioningLatencyState, SlaObservation, SlaPolicy, SlaSummary,
};

/// Application service for reading control-plane status and producing SLA DTOs.
pub struct SlaObservabilityService<S> {
    store: S,
}

impl<S> SlaObservabilityService<S> {
    /// Build a service from an observation store.
    #[must_use]
    pub const fn new(store: S) -> Self {
        Self { store }
    }

    /// Access the underlying store for composition tests.
    #[must_use]
    pub const fn store(&self) -> &S {
        &self.store
    }
}

impl<S> SlaObservabilityService<S>
where
    S: SlaObservabilityPort,
{
    /// Ingest a direct snapshot and return the computed deterministic summary.
    ///
    /// # Errors
    /// Returns typed fail-closed errors for malformed/unknown cluster identity or
    /// invalid observation windows.
    pub async fn ingest_snapshot(
        &self,
        snapshot: ControlPlaneSlaSnapshot,
    ) -> Result<SlaSummary, SlaObservabilityError> {
        self.store.ingest_status_snapshot(snapshot).await
    }

    /// Read a summary from the latest ingested observation.
    ///
    /// # Errors
    /// Returns [`SlaObservabilityError::UnknownCluster`] if no observation exists.
    pub async fn summarize_cluster(
        &self,
        cluster_ref: &ClusterRef,
    ) -> Result<SlaSummary, SlaObservabilityError> {
        self.store.summarize_cluster(cluster_ref).await
    }

    /// Read a tenant-scoped summary after default-deny authorization.
    ///
    /// # Errors
    /// Returns [`SlaObservabilityError::TenantScopeDenied`] before loading any
    /// summary when the caller scope does not match the requested tenant.
    pub async fn summarize_cluster_for_scope(
        &self,
        scope: &TenantSlaReadScope,
        cluster_ref: &ClusterRef,
    ) -> Result<SlaSummary, SlaObservabilityError> {
        scope.authorize_cluster_read(cluster_ref)?;
        self.summarize_cluster(cluster_ref).await
    }

    /// Read and aggregate tenant-scoped cluster summaries.
    ///
    /// # Errors
    /// Denies the entire rollup before loading summaries if any requested
    /// cluster is outside the caller's tenant scope.
    pub async fn summarize_fleet_for_scope(
        &self,
        scope: &TenantSlaReadScope,
        cluster_refs: &[ClusterRef],
    ) -> Result<FleetSlaSummary, SlaObservabilityError> {
        for cluster_ref in cluster_refs {
            scope.authorize_cluster_read(cluster_ref)?;
        }

        let mut summaries = Vec::with_capacity(cluster_refs.len());
        for cluster_ref in cluster_refs {
            summaries.push(self.summarize_cluster(cluster_ref).await?);
        }
        summarize_fleet_sla(&summaries).map_err(Into::into)
    }

    /// Ingest normalized live evidence, compute the summary, and attach MWMB evidence.
    ///
    /// # Errors
    /// Returns typed errors for stale/missing evidence, malformed windows,
    /// kernel validation failures, or store failures.
    pub async fn ingest_live_evidence(
        &self,
        observation: SlaLiveObservation,
        burn_rate_windows: BurnRateEvidenceWindows,
        evaluation_time_unix_millis: u64,
    ) -> Result<SlaEvidenceEmission, SlaObservabilityError> {
        observation.validate_at(evaluation_time_unix_millis)?;
        burn_rate_windows.validate(&observation.cluster_ref, observation.evidence.source)?;

        let snapshot = observation.effective_snapshot();
        let summary = self.ingest_snapshot(snapshot.clone()).await?;
        let fast = observation_for_window(&snapshot, &burn_rate_windows.fast);
        let slow = observation_for_window(&snapshot, &burn_rate_windows.slow);
        let alert_verdict = summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default(),
        )?;
        let rollback_holds = rollback_holds_for(&observation, &summary, &alert_verdict);

        Ok(SlaEvidenceEmission {
            snapshot,
            summary,
            alert_verdict,
            evidence: observation.evidence,
            rollback_holds,
        })
    }

    /// Read CPH status, combine optional live status evidence, and emit SLA evidence.
    ///
    /// # Errors
    /// Propagates typed control-plane-host read errors and live-evidence validation errors.
    pub async fn ingest_from_control_plane_live_evidence<P>(
        &self,
        control_plane: &P,
        control_plane_ref: &ControlPlaneRef,
        mut evidence: SlaEvidenceContext,
        live_status: Option<ObservedControlPlaneStatus>,
        status_window: StatusWindow,
        burn_rate_windows: BurnRateEvidenceWindows,
        provisioning_latency_millis: Option<u64>,
        evaluation_time_unix_millis: u64,
    ) -> Result<SlaEvidenceEmission, SlaObservabilityError>
    where
        P: ControlPlaneProvisioning,
    {
        let report = control_plane
            .status(control_plane_ref)
            .await
            .map_err(|err| SlaObservabilityError::ControlPlane {
                detail: err.to_string(),
            })?;
        if evidence.control_plane_tier.is_none() {
            evidence.control_plane_tier = Some(report.control_plane_ref.tier.as_str().to_string());
        }
        self.ingest_live_evidence(
            SlaLiveObservation::new(
                map_cluster_ref(&report.control_plane_ref.cluster_ref),
                map_control_plane_status(report.status),
                live_status,
                status_window,
                provisioning_latency_millis,
                evidence,
            ),
            burn_rate_windows,
            evaluation_time_unix_millis,
        )
        .await
    }

    /// Read the settled control-plane-host status seam, convert it into an SLA
    /// snapshot, ingest it, and return the golden-signal summary.
    ///
    /// `status_window` and `provisioning_latency_millis` are supplied by the
    /// caller/adapter because this lane intentionally does not couple to live
    /// Prometheus or Kubernetes.
    ///
    /// # Errors
    /// Propagates typed control-plane-host read errors and SLA validation errors.
    pub async fn ingest_from_control_plane<P>(
        &self,
        control_plane: &P,
        control_plane_ref: &ControlPlaneRef,
        status_window: StatusWindow,
        provisioning_latency_millis: Option<u64>,
    ) -> Result<SlaSummary, SlaObservabilityError>
    where
        P: ControlPlaneProvisioning,
    {
        let report = control_plane
            .status(control_plane_ref)
            .await
            .map_err(|err| SlaObservabilityError::ControlPlane {
                detail: err.to_string(),
            })?;
        let snapshot = ControlPlaneSlaSnapshot::new(
            map_cluster_ref(&report.control_plane_ref.cluster_ref),
            map_control_plane_status(report.status),
            status_window,
            provisioning_latency_millis,
        );
        self.ingest_snapshot(snapshot).await
    }
}

fn observation_for_window(
    snapshot: &ControlPlaneSlaSnapshot,
    window: &StatusWindow,
) -> SlaObservation {
    SlaObservation::new(
        snapshot.cluster_ref.tenant_id.clone(),
        snapshot.cluster_ref.cluster_name.clone(),
        snapshot.control_plane_status,
        window.total_status_samples,
        window.healthy_status_samples,
        snapshot.provisioning_latency_millis,
    )
}

fn rollback_holds_for(
    observation: &SlaLiveObservation,
    summary: &SlaSummary,
    alert_verdict: &oya_managed_k8s_sla_observability_kernel::AlertVerdict,
) -> Vec<RollbackHoldReason> {
    let mut holds = Vec::new();
    if observation.has_status_disagreement() {
        holds.push(RollbackHoldReason::StatusDisagreement);
    }
    match alert_verdict.severity {
        AlertSeverity::Page => holds.push(RollbackHoldReason::MwmbPage),
        AlertSeverity::Ticket => holds.push(RollbackHoldReason::MwmbTicket),
        AlertSeverity::None => {}
    }
    if summary.provisioning_latency.state == ProvisioningLatencyState::Breached {
        holds.push(RollbackHoldReason::ProvisioningLatencyBreach);
    }
    if summary.availability.state != AvailabilityState::Available {
        holds.push(RollbackHoldReason::AvailabilityNotGreen);
    }
    holds
}

/// Build the default in-memory composition root for tests/local bring-up.
#[must_use]
pub fn build_inmemory_service() -> SlaObservabilityService<InMemorySlaObservabilityStore> {
    SlaObservabilityService::new(InMemorySlaObservabilityStore::new(SlaPolicy::default()))
}

/// Map a sibling control-plane-host cluster ref into this bounded context's DTO.
#[must_use]
pub fn map_cluster_ref(cluster_ref: &control_plane_host::ClusterRef) -> ClusterRef {
    ClusterRef::new(&cluster_ref.tenant_id, &cluster_ref.cluster_name)
}

/// Map the sibling control-plane-host lifecycle enum into the SLA observation enum.
#[must_use]
pub const fn map_control_plane_status(
    status: control_plane_host::ControlPlaneStatus,
) -> ObservedControlPlaneStatus {
    match status {
        control_plane_host::ControlPlaneStatus::Requested => ObservedControlPlaneStatus::Requested,
        control_plane_host::ControlPlaneStatus::DatastoreBound => {
            ObservedControlPlaneStatus::DatastoreBound
        }
        control_plane_host::ControlPlaneStatus::MediaFormed => {
            ObservedControlPlaneStatus::MediaFormed
        }
        control_plane_host::ControlPlaneStatus::Provisioning => {
            ObservedControlPlaneStatus::Provisioning
        }
        control_plane_host::ControlPlaneStatus::EndpointReady => {
            ObservedControlPlaneStatus::EndpointReady
        }
        control_plane_host::ControlPlaneStatus::Active => ObservedControlPlaneStatus::Active,
        control_plane_host::ControlPlaneStatus::Draining => ObservedControlPlaneStatus::Draining,
        control_plane_host::ControlPlaneStatus::Deleted => ObservedControlPlaneStatus::Deleted,
        control_plane_host::ControlPlaneStatus::Failed => ObservedControlPlaneStatus::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_managed_k8s_control_plane_host_adapter_inmemory::InMemoryControlPlaneHost;
    use oya_managed_k8s_control_plane_host_api::{
        ClusterRef as HostClusterRef, ControlPlaneStatus, DatastoreClass, ProvisionRequest,
    };
    use oya_managed_k8s_sla_observability_kernel::{
        AlertSeverity, AvailabilityState, ProvisioningLatencyState,
    };

    fn evidence_context(freshness_deadline_unix_millis: u64) -> SlaEvidenceContext {
        SlaEvidenceContext::new(
            SlaEvidenceSource::PrometheusQuery,
            "managed-k8s-sla-local-collector",
            ObservationWindow::new(1_000, 2_000),
            SlaEvidenceFreshness::new(2_000, freshness_deadline_unix_millis),
            SlaEvidencePlacement::new("us-test-1", "cell-a", Some("hosted_kamaji")),
        )
    }

    fn page_windows() -> BurnRateEvidenceWindows {
        BurnRateEvidenceWindows::new(
            StatusWindow::new(10_000, 9_856),
            StatusWindow::new(10_000, 9_856),
        )
    }

    #[tokio::test]
    async fn reads_control_plane_status_port_and_summarizes_available_cluster() {
        let control_plane = InMemoryControlPlaneHost::new();
        let cp = control_plane
            .provision(&ProvisionRequest::new(
                HostClusterRef::new("ten_acme", "prod-a"),
                Default::default(),
                DatastoreClass::EtcdPerTenant,
            ))
            .await
            .unwrap();
        let service = build_inmemory_service();
        let summary = service
            .ingest_from_control_plane(
                &control_plane,
                &cp,
                StatusWindow::new(1_000, 1_000),
                Some(120_000),
            )
            .await
            .unwrap();
        assert_eq!(summary.availability.state, AvailabilityState::Available);
        assert_eq!(
            summary.provisioning_latency.state,
            ProvisioningLatencyState::Met
        );
    }

    #[tokio::test]
    async fn direct_degraded_snapshot_burns_budget() {
        let service = build_inmemory_service();
        let summary = service
            .ingest_snapshot(ControlPlaneSlaSnapshot::new(
                ClusterRef::new("ten_acme", "prod-a"),
                ObservedControlPlaneStatus::Failed,
                StatusWindow::new(100, 80),
                Some(900_000),
            ))
            .await
            .unwrap();
        assert_eq!(summary.availability.state, AvailabilityState::Unavailable);
        assert!(summary.error_budget.exhausted);
        assert_eq!(
            summary.provisioning_latency.state,
            ProvisioningLatencyState::Breached
        );
    }

    #[tokio::test]
    async fn unknown_cluster_fails_closed() {
        let service = build_inmemory_service();
        let err = service
            .summarize_cluster(&ClusterRef::new("ten_acme", "ghost"))
            .await
            .unwrap_err();
        assert!(matches!(err, SlaObservabilityError::UnknownCluster { .. }));
    }

    #[tokio::test]
    async fn summary_serializes_with_stable_slug() {
        let service = build_inmemory_service();
        let summary = service
            .ingest_snapshot(ControlPlaneSlaSnapshot::new(
                ClusterRef::new("TEN Acme!", "Prod_A.01"),
                ObservedControlPlaneStatus::Active,
                StatusWindow::new(10, 10),
                Some(100),
            ))
            .await
            .unwrap();
        assert_eq!(summary.cluster_slug, "ten-acme--prod-a-01");
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"cluster_slug\":\"ten-acme--prod-a-01\""));
    }

    #[tokio::test]
    async fn stale_live_evidence_fails_before_snapshot_is_stored() {
        let service = build_inmemory_service();
        let err = service
            .ingest_live_evidence(
                SlaLiveObservation::new(
                    ClusterRef::new("ten_acme", "prod-a"),
                    ObservedControlPlaneStatus::Active,
                    None,
                    StatusWindow::new(1_000, 1_000),
                    Some(120_000),
                    evidence_context(2_000),
                ),
                page_windows(),
                2_001,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, SlaObservabilityError::StaleEvidence { .. }));
        let read_after_stale = service
            .summarize_cluster(&ClusterRef::new("ten_acme", "prod-a"))
            .await
            .unwrap_err();
        assert!(matches!(
            read_after_stale,
            SlaObservabilityError::UnknownCluster { .. }
        ));
    }

    #[tokio::test]
    async fn control_plane_active_plus_live_failed_downgrades_summary_and_pages() {
        let control_plane = InMemoryControlPlaneHost::new();
        let cp = control_plane
            .provision(&ProvisionRequest::new(
                HostClusterRef::new("ten_acme", "prod-a"),
                Default::default(),
                DatastoreClass::EtcdPerTenant,
            ))
            .await
            .unwrap();
        let service = build_inmemory_service();
        let emission = service
            .ingest_from_control_plane_live_evidence(
                &control_plane,
                &cp,
                evidence_context(3_000),
                Some(ObservedControlPlaneStatus::Failed),
                StatusWindow::new(100, 80),
                page_windows(),
                Some(900_000),
                2_000,
            )
            .await
            .unwrap();

        assert_eq!(
            emission.summary.control_plane_status,
            ObservedControlPlaneStatus::Failed
        );
        assert_eq!(
            emission.summary.availability.state,
            AvailabilityState::Unavailable
        );
        assert_eq!(emission.alert_verdict.severity, AlertSeverity::Page);
        assert!(
            emission
                .rollback_holds
                .contains(&RollbackHoldReason::StatusDisagreement)
        );
        assert!(
            emission
                .rollback_holds
                .contains(&RollbackHoldReason::MwmbPage)
        );
        assert_eq!(
            emission.evidence.availability_openslo_path,
            MANAGED_CLUSTER_AVAILABILITY_OPENSLO_PATH
        );
    }

    #[tokio::test]
    async fn tenant_scoped_read_denies_cross_tenant_before_lookup() {
        let service = build_inmemory_service();
        service
            .ingest_snapshot(ControlPlaneSlaSnapshot::new(
                ClusterRef::new("ten_acme", "prod-a"),
                ObservedControlPlaneStatus::Active,
                StatusWindow::new(10, 10),
                Some(100),
            ))
            .await
            .unwrap();

        let denied = service
            .summarize_cluster_for_scope(
                &TenantSlaReadScope::new("ten_other"),
                &ClusterRef::new("ten_acme", "prod-a"),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            denied,
            SlaObservabilityError::TenantScopeDenied { .. }
        ));

        let allowed = service
            .summarize_cluster_for_scope(
                &TenantSlaReadScope::new("ten_acme"),
                &ClusterRef::new("ten_acme", "prod-a"),
            )
            .await
            .unwrap();
        assert_eq!(allowed.tenant_id, "ten_acme");
    }

    #[tokio::test]
    async fn tenant_fleet_summary_counts_only_authorized_clusters() {
        let service = build_inmemory_service();
        for (tenant_id, cluster_name, status) in [
            ("ten_acme", "prod-a", ObservedControlPlaneStatus::Active),
            ("ten_acme", "prod-b", ObservedControlPlaneStatus::Failed),
            ("ten_other", "prod-a", ObservedControlPlaneStatus::Active),
        ] {
            service
                .ingest_snapshot(ControlPlaneSlaSnapshot::new(
                    ClusterRef::new(tenant_id, cluster_name),
                    status,
                    StatusWindow::new(10, if status.is_available() { 10 } else { 0 }),
                    Some(100),
                ))
                .await
                .unwrap();
        }

        let fleet = service
            .summarize_fleet_for_scope(
                &TenantSlaReadScope::new("ten_acme"),
                &[
                    ClusterRef::new("ten_acme", "prod-a"),
                    ClusterRef::new("ten_acme", "prod-b"),
                ],
            )
            .await
            .unwrap();
        assert_eq!(fleet.cluster_count, 2);
        assert_eq!(fleet.available_count, 1);
        assert_eq!(fleet.unavailable_count, 1);

        let denied = service
            .summarize_fleet_for_scope(
                &TenantSlaReadScope::new("ten_acme"),
                &[
                    ClusterRef::new("ten_acme", "prod-a"),
                    ClusterRef::new("ten_other", "prod-a"),
                ],
            )
            .await
            .unwrap_err();
        assert!(matches!(
            denied,
            SlaObservabilityError::TenantScopeDenied { .. }
        ));
    }

    #[test]
    fn control_plane_status_mapping_is_complete_and_adapter_neutral() {
        for (source, target) in [
            (
                ControlPlaneStatus::Requested,
                ObservedControlPlaneStatus::Requested,
            ),
            (
                ControlPlaneStatus::DatastoreBound,
                ObservedControlPlaneStatus::DatastoreBound,
            ),
            (
                ControlPlaneStatus::MediaFormed,
                ObservedControlPlaneStatus::MediaFormed,
            ),
            (
                ControlPlaneStatus::Provisioning,
                ObservedControlPlaneStatus::Provisioning,
            ),
            (
                ControlPlaneStatus::EndpointReady,
                ObservedControlPlaneStatus::EndpointReady,
            ),
            (
                ControlPlaneStatus::Active,
                ObservedControlPlaneStatus::Active,
            ),
            (
                ControlPlaneStatus::Draining,
                ObservedControlPlaneStatus::Draining,
            ),
            (
                ControlPlaneStatus::Deleted,
                ObservedControlPlaneStatus::Deleted,
            ),
            (
                ControlPlaneStatus::Failed,
                ObservedControlPlaneStatus::Failed,
            ),
        ] {
            assert_eq!(map_control_plane_status(source), target);
        }
    }
}
