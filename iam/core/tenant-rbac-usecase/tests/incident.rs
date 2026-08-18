#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_domain::{
    IncidentFirstAction, IncidentPlanStatus, IncidentRemediationRoute, IncidentRollbackInput,
    IncidentTrigger, plan_incident_rollback,
};
use iam_tenant_rbac_usecase::prepare_incident_rollback_envelope;
use oya_data_boundary_kernel::DataClass;

#[test]
fn incident_envelope_is_metadata_only() {
    let plan = plan_incident_rollback(incident_input()).expect("incident rollback plan");
    let envelope = prepare_incident_rollback_envelope(&plan);

    assert_eq!(envelope.topic.value, "incident.tenant-rbac.rollback.plan");
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(envelope.incident_id.value.value, "inc_canary_slo_001");
    assert_eq!(envelope.trigger.value, IncidentTrigger::CanarySloBreach);
    assert_eq!(envelope.first_action.value, IncidentFirstAction::Rollback);
    assert_eq!(
        envelope.remediation_route.value,
        IncidentRemediationRoute::OpenTofu
    );
    assert_eq!(
        envelope.incident_evidence_ref.value.value,
        "audit/tenant-rbac/incidents/inc_canary_slo_001.json"
    );
    assert_eq!(
        envelope.rollback_evidence_ref.value.value,
        "audit/tenant-rbac/incidents/rollback-first"
    );
    assert_eq!(
        envelope.convergence_ref.value,
        "opentofu/tenant-rbac/fixes/inc_canary_slo_001"
    );
    assert_eq!(
        envelope.plan_status.value,
        IncidentPlanStatus::RollbackFirstAccepted
    );
    assert_eq!(envelope.payload_data_class.value, DataClass::InternalOnly);
    assert_eq!(envelope.schema_version.value, 1);
}

fn incident_input() -> IncidentRollbackInput {
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
