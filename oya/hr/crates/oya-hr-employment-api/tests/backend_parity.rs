#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_api::{
    BackendParityClaimStatusDto, HrBackendParityCapabilityDto, HrBackendParityProfileRequest,
    HrBackendParityProfileResponse,
};
use oya_hr_employment_domain::build_hr_backend_parity_profile;

#[test]
fn hr_backend_parity_profile_request_serializes_camel_case_and_converts() {
    let request = HrBackendParityProfileRequest {
        tenant_id: "ten_acme".to_owned(),
        profile_evidence_ref: "audit/hr/parity/profile".to_owned(),
        source_evidence_refs: vec!["audit/hr/parity/source/oracle-hcm".to_owned()],
    };
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(body["tenantId"], "ten_acme");
    assert_eq!(body["profileEvidenceRef"], "audit/hr/parity/profile");
    assert_eq!(
        body["sourceEvidenceRefs"][0],
        "audit/hr/parity/source/oracle-hcm"
    );

    let profile = build_hr_backend_parity_profile(request.into_domain_input()).expect("profile");
    let response = HrBackendParityProfileResponse::from_profile(&profile);
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
        capability.capability == HrBackendParityCapabilityDto::CloudKubernetesReadiness
            && capability.claim_status == BackendParityClaimStatusDto::ContractReady
            && capability.evidence_ref_count == 1
            && capability.kubernetes_native_contract_ready
            && !capability.production_runtime_claimed
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
            .any(|claim| claim.contains("no live provider integrations"))
    );
}
