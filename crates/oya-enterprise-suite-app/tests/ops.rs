#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_enterprise_suite_app::{
    EnterpriseOpsCommandInput, EnterpriseOpsRoute, EnterpriseSuiteAppError, OpsCommandKind,
    prepare_enterprise_ops_envelope,
};

#[test]
fn test_enterprise_ops_contract_has_no_manual_ssh() {
    let envelope = prepare_enterprise_ops_envelope(EnterpriseOpsCommandInput {
        tenant_id: "ten_acme".to_owned(),
        route: EnterpriseOpsRoute::OyaOps,
        command_kind: OpsCommandKind::Day2Change,
        evidence_ref: "audit/enterprise-suite/ops/day2".to_owned(),
        change_plan_ref: "opentofu/enterprise-suite/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    })
    .expect("ops envelope");

    assert_eq!(envelope.topic.value, "audit.enterprise-suite.ops.command");
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(envelope.route.value, EnterpriseOpsRoute::OyaOps);
    assert_eq!(envelope.payload_data_class.value, DataClass::InternalOnly);
    assert_eq!(envelope.schema_version.value, 1);

    let manual_shell_error = prepare_enterprise_ops_envelope(EnterpriseOpsCommandInput {
        tenant_id: "ten_acme".to_owned(),
        route: EnterpriseOpsRoute::ManualSsh,
        command_kind: OpsCommandKind::Day2Change,
        evidence_ref: "audit/enterprise-suite/ops/day2".to_owned(),
        change_plan_ref: "opentofu/enterprise-suite/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:manual".to_owned(),
    })
    .expect_err("manual SSH route must be refused");

    assert_eq!(
        manual_shell_error,
        EnterpriseSuiteAppError::ManualSshRefused
    );
}
