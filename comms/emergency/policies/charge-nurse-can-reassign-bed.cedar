// Policy: charge-nurse-can-reassign-bed
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: only charge nurses or attendings may reassign a patient's bed.

permit (
    principal in Oyatie::Role::"chargeNurse",
    action == Oyatie::Action::"bedReassign",
    resource is Oyatie::Emergency::BoardCell
)
when {
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId &&
    principal.activeShift == true
};

permit (
    principal in Oyatie::Role::"attending",
    action == Oyatie::Action::"bedReassign",
    resource is Oyatie::Emergency::BoardCell
)
when {
    principal.tenantId == resource.tenantId &&
    principal.cellId == resource.cellId &&
    principal.edPrivilegesActive == true
};

forbid (
    principal,
    action == Oyatie::Action::"bedReassign",
    resource
)
unless {
    principal in Oyatie::Role::"chargeNurse" ||
    principal in Oyatie::Role::"attending"
};
