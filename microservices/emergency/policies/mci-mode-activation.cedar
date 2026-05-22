// Policy: mci-mode-activation
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: MCI activation gated to attending or facility-emergency-manager.
//         Drill mode flag must be explicit.

permit (
    principal in Oyatie::Role::"attending",
    action == Oyatie::Action::"mciActivate",
    resource is Oyatie::Emergency::MciActivation
)
when {
    principal.edPrivilegesActive == true &&
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId
};

permit (
    principal in Oyatie::Role::"facilityEmergencyManager",
    action == Oyatie::Action::"mciActivate",
    resource is Oyatie::Emergency::MciActivation
)
when {
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId
};

forbid (
    principal,
    action == Oyatie::Action::"mciActivate",
    resource
)
when {
    resource.drillModeFlagPresent == false
};
