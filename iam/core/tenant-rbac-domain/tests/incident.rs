#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_domain::{
    IncidentFirstAction, IncidentPlanStatus, IncidentRemediationRoute, IncidentRollbackInput,
    IncidentTrigger, TenantRbacDomainError, plan_incident_rollback,
};

#[test]
fn test_rollback_first_no_manual_ssh() {
    let plan = plan_incident_rollback(valid_input()).expect("incident rollback plan");

    assert_eq!(plan.tenant_id.value.value, "ten_acme");
    assert_eq!(plan.incident_id.value.value, "inc_canary_slo_001");
    assert_eq!(plan.trigger.value, IncidentTrigger::CanarySloBreach);
    assert_eq!(plan.first_action.value, IncidentFirstAction::Rollback);
    assert_eq!(
        plan.remediation_route.value,
        IncidentRemediationRoute::OpenTofu
    );
    assert_eq!(
        plan.canary_evidence_ref.value.value,
        "audit/tenant-rbac/incidents/canary-slo"
    );
    assert_eq!(
        plan.incident_evidence_ref.value.value,
        "audit/tenant-rbac/incidents/inc_canary_slo_001.json"
    );
    assert_eq!(
        plan.rollback_evidence_ref.value.value,
        "audit/tenant-rbac/incidents/rollback-first"
    );
    assert_eq!(
        plan.convergence_ref.value.value,
        "opentofu/tenant-rbac/fixes/inc_canary_slo_001"
    );
    assert_eq!(
        plan.plan_status.value,
        IncidentPlanStatus::RollbackFirstAccepted
    );
    assert_eq!(plan.schema_version.value, 1);
}

#[test]
fn test_manual_ssh_incident_route_is_refused() {
    let error = plan_incident_rollback(IncidentRollbackInput {
        remediation_route: IncidentRemediationRoute::ManualSsh,
        ..valid_input()
    })
    .expect_err("manual SSH cannot be an incident remediation route");

    assert_eq!(error, TenantRbacDomainError::ManualSshRefused);
}

#[test]
fn test_remediation_before_rollback_is_refused() {
    let error = plan_incident_rollback(IncidentRollbackInput {
        first_action: IncidentFirstAction::Remediate,
        ..valid_input()
    })
    .expect_err("remediation cannot happen before rollback or quarantine");

    assert_eq!(
        error,
        TenantRbacDomainError::RollbackOrQuarantineMustBeFirst
    );
}

#[test]
fn test_missing_incident_evidence_is_refused() {
    let error = plan_incident_rollback(IncidentRollbackInput {
        incident_evidence_ref: "audit/".to_owned(),
        ..valid_input()
    })
    .expect_err("incident JSON/evidence pointer is mandatory");

    assert_eq!(error, TenantRbacDomainError::InvalidAuditEvidenceRef);
}

#[test]
fn test_missing_opentofu_or_ops_convergence_ref_is_refused() {
    let error = plan_incident_rollback(IncidentRollbackInput {
        convergence_ref: "ssh/tenant-rbac/manual-hotfix".to_owned(),
        ..valid_input()
    })
    .expect_err("incident fixes must converge through OpenTofu or ops");

    assert_eq!(error, TenantRbacDomainError::InvalidConvergenceRef);
}

#[test]
fn test_quarantine_first_can_converge_through_ops() {
    let plan = plan_incident_rollback(IncidentRollbackInput {
        trigger: IncidentTrigger::ProductionIncident,
        first_action: IncidentFirstAction::Quarantine,
        remediation_route: IncidentRemediationRoute::OyaOps,
        convergence_ref: "ops/tenant-rbac/quarantine/inc_prod_002".to_owned(),
        incident_id: "inc_prod_002".to_owned(),
        idempotency_key: "ten_acme:incident:inc_prod_002".to_owned(),
        ..valid_input()
    })
    .expect("quarantine-first incident plan");

    assert_eq!(plan.first_action.value, IncidentFirstAction::Quarantine);
    assert_eq!(
        plan.remediation_route.value,
        IncidentRemediationRoute::OyaOps
    );
    assert_eq!(
        plan.convergence_ref.value.value,
        "ops/tenant-rbac/quarantine/inc_prod_002"
    );
}

fn valid_input() -> IncidentRollbackInput {
    IncidentRollbackInput {
        tenant_id: "ten_acme".to_owned(),
        incident_id: "inc_canary_slo_001".to_owned(),
        trigger: IncidentTrigger::CanarySloBreach,
        first_action: IncidentFirstAction::Rollback,
        remediation_route: IncidentRemediationRoute::OpenTofu,
        canary_evidence_ref: "audit/tenant-rbac/incidents/canary-slo".to_owned(),
        incident_evidence_ref: "audit/tenant-rbac/incidents/inc_canary_slo_001.json".to_owned(),
        rollback_evidence_ref: "audit/tenant-rbac/incidents/rollback-first".to_owned(),
        convergence_ref: "opentofu/tenant-rbac/fixes/inc_canary_slo_001".to_owned(),
        idempotency_key: "ten_acme:incident:inc_canary_slo_001".to_owned(),
    }
}
