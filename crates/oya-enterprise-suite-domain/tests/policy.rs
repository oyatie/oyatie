#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_enterprise_suite_domain::{
    ChildWriteInput, EnterpriseChildProduct, EnterpriseSuiteDomainError, SuiteWriteKind,
    admit_child_write,
};

#[test]
fn test_children_share_suite_policy_gateway() {
    let writes = [
        (EnterpriseChildProduct::Hr, SuiteWriteKind::HrLifecycle),
        (
            EnterpriseChildProduct::Payroll,
            SuiteWriteKind::PayrollClose,
        ),
        (
            EnterpriseChildProduct::Accounting,
            SuiteWriteKind::AccountingJournal,
        ),
    ];

    let decisions = writes
        .into_iter()
        .enumerate()
        .map(|(index, (child_product, write_kind))| {
            admit_child_write(ChildWriteInput {
                child_product,
                write_kind,
                tenant_id: "ten_acme".to_owned(),
                legal_entity_id: "le_kr_001".to_owned(),
                payload_data_class: Some(DataClass::Financial),
                audit_evidence_ref: format!("audit/enterprise-suite/write/{index}"),
                policy_gateway_ref: "policy/enterprise-suite/shared-gateway".to_owned(),
                idempotency_key: format!("ten_acme:le_kr_001:{index}"),
                sequence: index as u64 + 1,
            })
            .expect("suite policy decision")
        })
        .collect::<Vec<_>>();

    assert_eq!(decisions.len(), 3);
    for decision in &decisions {
        assert_eq!(decision.tenant_id.value.value, "ten_acme");
        assert_eq!(decision.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(
            decision.policy_gateway_ref.value.value,
            "policy/enterprise-suite/shared-gateway"
        );
        assert_eq!(decision.payload_data_class.value, DataClass::Financial);
        assert_eq!(decision.schema_version.value, 1);
    }
}

#[test]
fn test_child_write_cannot_bypass_suite_policy_gateway() {
    let error = admit_child_write(ChildWriteInput {
        child_product: EnterpriseChildProduct::Payroll,
        write_kind: SuiteWriteKind::PayrollClose,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: Some(DataClass::Financial),
        audit_evidence_ref: "audit/payroll/close/001".to_owned(),
        policy_gateway_ref: "policy/payroll/direct".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    })
    .expect_err("direct child policy gateway must be refused");

    assert_eq!(
        error,
        EnterpriseSuiteDomainError::BypassedSuitePolicyGateway
    );
}

#[test]
fn test_child_write_requires_data_class_and_audit_evidence() {
    let error = admit_child_write(ChildWriteInput {
        child_product: EnterpriseChildProduct::Accounting,
        write_kind: SuiteWriteKind::AccountingJournal,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: None,
        audit_evidence_ref: "audit/".to_owned(),
        policy_gateway_ref: "policy/enterprise-suite/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:journal".to_owned(),
        sequence: 1,
    })
    .expect_err("data class must be required first");

    assert_eq!(error, EnterpriseSuiteDomainError::MissingPayloadDataClass);
}
