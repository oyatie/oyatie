// Cedar v4.2 LTS — compliance µservice CI scope
// Binding ADR: ADR-0295 (bootstrap CI SPIFFE)
// CI workloads may emit synthetic evidence + run validators; nothing else.

permit (
    principal in oyatie::Foundry::CIPrincipal::"compliance-validators",
    action in [
        Action::"emit_synthetic_evidence",
        Action::"run_validator",
        Action::"verify_seal_chain"
    ],
    resource
) when {
    principal.spiffe_attested == true
    && resource.environment == "ci"
    && resource.synthetic == true
};

forbid (
    principal in oyatie::Foundry::CIPrincipal::"compliance-validators",
    action,
    resource
) when {
    resource.environment != "ci"
    || resource.contains_production_data == true
};
