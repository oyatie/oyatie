use oya_tenant_rbac_slo_evidence::{
    TenantRbacSloEvidenceError, TenantRbacSloKind, openslo_manifest_paths,
    tenant_rbac_slo_evidence_plan, validate_tenant_rbac_slo_evidence_plan,
};

#[test]
fn slo_evidence_plan_validates_error_budget_otel_and_nonclaims() {
    let plan = tenant_rbac_slo_evidence_plan();
    validate_tenant_rbac_slo_evidence_plan(&plan).expect("SLO evidence plan validates");

    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.objectives.len(), 4);
    assert!(plan.error_budget_release_gate_required);
    assert!(plan.multi_window_burn_rate_alert_required);
    assert!(plan.openslo_manifests_required);
    assert!(plan.otel_metric_streams_required);
    assert!(plan.dashboard_ref.starts_with("dashboards/tenant-rbac/"));
    assert!(plan.otel_collector_ref.starts_with("otel/collector/"));
    assert!(!plan.runtime_otel_export_attached);
    assert!(!plan.metrics_backend_attached);
    assert!(!plan.alert_manager_attached);
    assert!(!plan.canary_runtime_attached);
    assert!(!plan.rollback_automation_attached);
    assert!(!plan.production_slo_evidence_attached);
    assert!(!plan.multi_region_slo_evidence_attached);
}

#[test]
fn slo_evidence_plan_preserves_openslo_manifest_paths_and_objective_mix() {
    let plan = tenant_rbac_slo_evidence_plan();
    let paths = openslo_manifest_paths(&plan);

    assert_eq!(paths.len(), 4);
    assert!(
        paths
            .iter()
            .all(|path| path.starts_with("microservices/tenant-rbac/slos/"))
    );
    assert!(paths.iter().all(|path| path.ends_with(".openslo.yaml")));
    assert!(
        plan.objectives
            .iter()
            .any(|objective| objective.kind == TenantRbacSloKind::Availability)
    );
    assert!(
        plan.objectives
            .iter()
            .any(|objective| objective.kind == TenantRbacSloKind::Latency)
    );
    assert!(
        plan.objectives
            .iter()
            .any(|objective| objective.kind == TenantRbacSloKind::Freshness)
    );
    assert!(
        plan.objectives
            .iter()
            .any(|objective| objective.kind == TenantRbacSloKind::Correctness)
    );
}

#[test]
fn slo_evidence_plan_rejects_missing_objectives_or_unsafe_refs() {
    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.objectives.truncate(1);
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::MissingObjectives)
    );

    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.objectives[0].openslo_manifest_path = "../secret.openslo.yaml";
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::InvalidManifestPath)
    );

    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.objectives[0].sli_metric_name = "secret.token";
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::InvalidMetricName)
    );
}

#[test]
fn slo_evidence_plan_rejects_runtime_or_production_overclaims() {
    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.production_slo_evidence_attached = true;
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::RuntimeAttachmentOverclaim)
    );

    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.metrics_backend_attached = true;
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::RuntimeAttachmentOverclaim)
    );
}

#[test]
fn slo_evidence_plan_requires_actionable_burn_rate_windows() {
    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.burn_rate_policy.slow_window_minutes = plan.burn_rate_policy.fast_window_minutes;
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::InvalidBurnRateWindow)
    );

    let mut plan = tenant_rbac_slo_evidence_plan();
    plan.burn_rate_policy.page_burn_rate_threshold = 1;
    assert_eq!(
        validate_tenant_rbac_slo_evidence_plan(&plan),
        Err(TenantRbacSloEvidenceError::InvalidBurnRateThreshold)
    );
}
