// Policy: ed-only-disposition
// Authority: ADR-0332 (in flight) | ADR-0243 (Cedar universal gate)
// Effect: disposition close requires active ED privileges AND
//         an open Encounter[class=EMER]. EMTALA documentation is
//         enforced for transfer disposition by composing additional
//         policies at request time.

permit (
    principal in Oyatie::Role::"attending",
    action == Oyatie::Action::"dispositionSet",
    resource is Oyatie::Emergency::Disposition
)
when {
    principal.edPrivilegesActive == true &&
    resource.encounterClass == "EMER" &&
    resource.encounterOpen == true
};

permit (
    principal in Oyatie::Role::"delegatedProvider",
    action == Oyatie::Action::"dispositionSet",
    resource is Oyatie::Emergency::Disposition
)
when {
    principal.edPrivilegesActive == true &&
    principal.delegationActive == true &&
    resource.encounterClass == "EMER" &&
    resource.encounterOpen == true
};

forbid (
    principal,
    action == Oyatie::Action::"dispositionSet",
    resource
)
when {
    resource.outcome == "transfer" &&
    resource.emtalaDocumentationComplete == false
};

forbid (
    principal,
    action == Oyatie::Action::"dispositionSet",
    resource
)
when {
    resource.outcome == "ama" &&
    resource.amaFormSigned == false
};
