// Allergy-override-requires-justification Cedar policy
// Authority: ADR-0243 (Cedar universal gate); ADR-0332 (pharmacy substrate)
// Effect: overriding an allergy alert at severity ≥ severe requires:
//   (a) a structured reason code from the canonical override-reason vocabulary,
//   (b) attestation by an authorized clinician,
//   (c) two-step confirmation (e.g., re-entered phrase or step-up auth).

@id("allergy-override-severe-requires-two-step")
forbid (
    principal,
    action == Action::"pharmacy.allergy-override",
    resource is AllergyOverrideRequest
)
when {
    resource has severity &&
    resource.severity in ["severe", "contraindicated"] &&
    (
        !(resource has reason_code) ||
        resource.reason_code == "" ||
        !(resource has two_step_confirmed) ||
        resource.two_step_confirmed == false
    )
};

@id("allergy-override-attestor-must-be-authorized-clinician")
forbid (
    principal,
    action == Action::"pharmacy.allergy-override",
    resource is AllergyOverrideRequest
)
when {
    !(principal in Group::"pharmacy.authorized-override-clinicians")
};
