#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_payroll_run_api::{
    BackendParityClaimStatusDto, PayrollBackendParityCapabilityDto,
    PayrollBackendParityProfileRequest, PayrollBackendParityProfileResponse,
};
use oya_payroll_run_domain::build_payroll_backend_parity_profile;

#[test]
fn payroll_backend_parity_profile_request_serializes_camel_case_and_converts() {
    let request = PayrollBackendParityProfileRequest {
        tenant_id: "ten_acme".to_owned(),
        profile_evidence_ref: "audit/payroll/parity/profile".to_owned(),
        source_evidence_refs: vec![
            "audit/payroll/parity/gross-to-net-run-controls/sap-successfactors-payroll".to_owned(),
            "audit/payroll/parity/earnings-deductions-tax-model/oracle-payroll".to_owned(),
            "audit/payroll/parity/time-leave-payroll-intake/hr-contracts".to_owned(),
            "audit/payroll/parity/retro-off-cycle-reversal/workday-payroll".to_owned(),
            "audit/payroll/parity/statutory-export-evidence/rulepack".to_owned(),
            "audit/payroll/parity/payslip-disbursement-seam/nonclaim".to_owned(),
            "audit/payroll/parity/accounting-gl-export/journal-contracts".to_owned(),
            "audit/payroll/parity/group-legal-entity-rollup/erp-contracts".to_owned(),
            "audit/payroll/parity/variance-anomaly-rollback/slo-contracts".to_owned(),
            "audit/payroll/parity/audit-idempotency-tenant-residency/governance".to_owned(),
            "audit/payroll/parity/cloud-kubernetes-readiness/service-contract".to_owned(),
        ],
    };
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(body["tenantId"], "ten_acme");
    assert_eq!(body["profileEvidenceRef"], "audit/payroll/parity/profile");
    assert_eq!(
        body["sourceEvidenceRefs"][0],
        "audit/payroll/parity/gross-to-net-run-controls/sap-successfactors-payroll"
    );

    let profile =
        build_payroll_backend_parity_profile(request.into_domain_input()).expect("profile");
    let response = PayrollBackendParityProfileResponse::from_profile(&profile);
    let response_body = serde_json::to_value(&response).expect("serialize response");

    assert_eq!(response_body["schemaVersion"], 1);
    assert!(response_body["tenantId"].is_null());
    assert!(response_body["profileEvidenceRef"].is_null());
    assert!(response_body["sourceEvidenceRefs"].is_null());
    assert_eq!(
        response_body["capabilities"]
            .as_array()
            .expect("capabilities array")
            .len(),
        11
    );
    assert!(response.capabilities.iter().any(|capability| {
        capability.capability == PayrollBackendParityCapabilityDto::CloudKubernetesReadiness
            && capability.claim_status == BackendParityClaimStatusDto::ContractReady
            && capability.evidence_ref_count == 1
            && capability.kubernetes_native_contract_ready
            && !capability.production_runtime_claimed
            && !capability.live_money_movement_claimed
            && !capability.tax_filing_submission_claimed
    }));
    let cloud_response = response_body["capabilities"]
        .as_array()
        .expect("capabilities array")
        .iter()
        .find(|capability| capability["capability"] == "CLOUD_KUBERNETES_READINESS")
        .expect("cloud capability response");
    assert_eq!(cloud_response["evidenceRefCount"], 1);
    assert!(cloud_response["evidenceRefs"].is_null());
    assert_eq!(cloud_response["kubernetesNativeContractReady"], true);
    assert!(
        response
            .nonclaims
            .iter()
            .any(|claim| claim.contains("no tax filing submission"))
    );
}
