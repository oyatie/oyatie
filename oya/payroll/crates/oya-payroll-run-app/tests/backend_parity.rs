#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_payroll_run_app::prepare_payroll_backend_parity_profile;
use oya_payroll_run_domain::{PayrollBackendParityCapability, PayrollBackendParityProfileInput};

#[test]
fn payroll_app_wraps_backend_parity_profile_in_metadata_envelope() {
    let outcome = prepare_payroll_backend_parity_profile(profile_input()).expect("profile outcome");

    assert_eq!(
        outcome.parity_envelope.topic.value,
        "metadata.payroll.backend-parity.profile"
    );
    assert_eq!(outcome.parity_envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(
        outcome.parity_envelope.idempotency_key.value,
        "ten_acme:audit/payroll/parity/profile:payroll-backend-parity-profile"
    );
    assert_eq!(outcome.parity_envelope.capability_count.value, 11);
    assert_eq!(
        outcome
            .parity_envelope
            .kubernetes_ready_contract_count
            .value,
        1
    );
    assert_eq!(
        outcome.parity_envelope.payload_data_class.value,
        DataClass::InternalOnly
    );
    assert!(
        outcome
            .profile
            .capabilities
            .value
            .iter()
            .any(|capability| capability.capability.value
                == PayrollBackendParityCapability::CloudKubernetesReadiness)
    );
}

fn profile_input() -> PayrollBackendParityProfileInput {
    PayrollBackendParityProfileInput {
        tenant_id: "ten_acme".to_owned(),
        profile_evidence_ref: "audit/payroll/parity/profile".to_owned(),
        source_evidence_refs: vec!["audit/payroll/parity/source/adp-workforce-now".to_owned()],
    }
}
