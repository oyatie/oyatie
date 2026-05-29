// Policy: verbal-order-bridge
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: verbal-order entries require provider claim and trigger 24-hour
//         countersign window enforcement. Unsigned past window → block
//         further verbal-order entry until backlog cleared.

permit (
    principal in Oyatie::Role::"rn",
    action == Oyatie::Action::"orderPlaceVerbal",
    resource is Oyatie::Emergency::Order
)
when {
    resource.verbal == true &&
    resource.attestingAttendingId != "" &&
    principal.verbalOrderBacklogCount < 5
};

forbid (
    principal,
    action == Oyatie::Action::"orderPlaceVerbal",
    resource
)
when {
    principal.verbalOrderBacklogCount >= 5
};

forbid (
    principal,
    action == Oyatie::Action::"orderPlaceVerbal",
    resource
)
when {
    resource.attestingAttendingId == ""
};
