// Policy: trauma-alert-bypass-rules
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: trauma alert activation bypasses normal triage queueing,
//         but does NOT bypass audit-chain attestation or BYOK gating.

permit (
    principal in Oyatie::Role::"attending",
    action == Oyatie::Action::"protocolActivate",
    resource is Oyatie::Emergency::ProtocolActivation
)
when {
    resource.protocolType == "trauma" &&
    principal.edPrivilegesActive == true &&
    resource.attestationProvided == true
};

permit (
    principal in Oyatie::Role::"traumaTeamLeader",
    action == Oyatie::Action::"protocolActivate",
    resource is Oyatie::Emergency::ProtocolActivation
)
when {
    resource.protocolType == "trauma" &&
    resource.attestationProvided == true
};

forbid (
    principal,
    action == Oyatie::Action::"protocolActivate",
    resource
)
when {
    resource.protocolType == "trauma" &&
    resource.attestationProvided == false
};
