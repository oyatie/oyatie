#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_domain::{
    ServiceWriteInput, TenantRbacDomainError, TenantRbacService, TenantRbacWriteKind,
    admit_service_write,
};
use oya_data_boundary_kernel::DataClass;

#[test]
fn test_services_share_tenant_rbac_policy_gateway() {
    let writes = [
        (TenantRbacService::Hr, TenantRbacWriteKind::HrLifecycle),
        (
            TenantRbacService::Payroll,
            TenantRbacWriteKind::PayrollClose,
        ),
        (
            TenantRbacService::Accounting,
            TenantRbacWriteKind::AccountingJournal,
        ),
    ];

    let decisions = writes
        .into_iter()
        .enumerate()
        .map(|(index, (service, write_kind))| {
            admit_service_write(ServiceWriteInput {
                service,
                write_kind,
                tenant_id: "ten_acme".to_owned(),
                legal_entity_id: "le_kr_001".to_owned(),
                payload_data_class: Some(DataClass::Financial),
                audit_evidence_ref: format!("audit/tenant-rbac/write/{index}"),
                policy_gateway_ref: "policy/tenant-rbac/shared-gateway".to_owned(),
                idempotency_key: format!("ten_acme:le_kr_001:{index}"),
                sequence: index as u64 + 1,
            })
            .expect("platform policy decision")
        })
        .collect::<Vec<_>>();

    assert_eq!(decisions.len(), 3);
    for decision in &decisions {
        assert_eq!(decision.tenant_id.value.value, "ten_acme");
        assert_eq!(decision.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(
            decision.policy_gateway_ref.value.value,
            "policy/tenant-rbac/shared-gateway"
        );
        assert_eq!(decision.payload_data_class.value, DataClass::Financial);
        assert_eq!(decision.schema_version.value, 1);
    }
}

#[test]
fn test_service_write_cannot_bypass_tenant_rbac_policy_gateway() {
    let error = admit_service_write(ServiceWriteInput {
        service: TenantRbacService::Payroll,
        write_kind: TenantRbacWriteKind::PayrollClose,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: Some(DataClass::Financial),
        audit_evidence_ref: "audit/payroll/close/001".to_owned(),
        policy_gateway_ref: "policy/payroll/direct".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    })
    .expect_err("direct service policy gateway must be refused");

    assert_eq!(error, TenantRbacDomainError::BypassedPlatformPolicyGateway);
}

#[test]
fn test_service_write_requires_data_class_and_audit_evidence() {
    let error = admit_service_write(ServiceWriteInput {
        service: TenantRbacService::Accounting,
        write_kind: TenantRbacWriteKind::AccountingJournal,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: None,
        audit_evidence_ref: "audit/".to_owned(),
        policy_gateway_ref: "policy/tenant-rbac/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:journal".to_owned(),
        sequence: 1,
    })
    .expect_err("data class must be required first");

    assert_eq!(error, TenantRbacDomainError::MissingPayloadDataClass);
}
