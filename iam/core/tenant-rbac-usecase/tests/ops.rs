#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_usecase::{
    OpsCommandKind, TenantRbacApplicationError, TenantRbacOpsCommandInput, TenantRbacOpsRoute,
    prepare_tenant_rbac_ops_envelope,
};
use data_boundary_kernel::DataClass;

#[test]
fn test_tenant_rbac_ops_contract_has_no_manual_ssh() {
    let envelope = prepare_tenant_rbac_ops_envelope(TenantRbacOpsCommandInput {
        tenant_id: "ten_acme".to_owned(),
        route: TenantRbacOpsRoute::OyaOps,
        command_kind: OpsCommandKind::Day2Change,
        evidence_ref: "audit/tenant-rbac/ops/day2".to_owned(),
        change_plan_ref: "opentofu/tenant-rbac/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    })
    .expect("ops envelope");

    assert_eq!(envelope.topic.value, "audit.tenant-rbac.ops.command");
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(envelope.route.value, TenantRbacOpsRoute::OyaOps);
    assert_eq!(envelope.payload_data_class.value, DataClass::InternalOnly);
    assert_eq!(envelope.schema_version.value, 1);

    let manual_shell_error = prepare_tenant_rbac_ops_envelope(TenantRbacOpsCommandInput {
        tenant_id: "ten_acme".to_owned(),
        route: TenantRbacOpsRoute::ManualSsh,
        command_kind: OpsCommandKind::Day2Change,
        evidence_ref: "audit/tenant-rbac/ops/day2".to_owned(),
        change_plan_ref: "opentofu/tenant-rbac/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:manual".to_owned(),
    })
    .expect_err("manual SSH route must be refused");

    assert_eq!(
        manual_shell_error,
        TenantRbacApplicationError::ManualSshRefused
    );
}
