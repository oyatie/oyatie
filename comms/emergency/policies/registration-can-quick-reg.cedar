// Policy: registration-can-quick-reg
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: quick-reg requires an ED registration clerk or facility emergency
//         manager, and may only run inside an active ED session.

permit (
    principal in Oyatie::Role::"edRegistrationClerk",
    action == Oyatie::Action::"quickRegister",
    resource is Oyatie::Emergency::Registration
)
when {
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId &&
    principal.edFacilityId == resource.facilityId
};

permit (
    principal in Oyatie::Role::"facilityEmergencyManager",
    action == Oyatie::Action::"quickRegister",
    resource is Oyatie::Emergency::Registration
)
when {
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId
};

forbid (
    principal,
    action == Oyatie::Action::"quickRegister",
    resource
)
unless {
    principal in Oyatie::Role::"edRegistrationClerk" ||
    principal in Oyatie::Role::"facilityEmergencyManager"
};
