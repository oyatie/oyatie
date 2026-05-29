// Pharmacist-can-verify Cedar policy
// Authority: ADR-0243 (Cedar universal gate); ADR-0332 (pharmacy substrate)
// Effect: only licensed pharmacist of correct state can verify an order;
// for CII controlled substances, the pharmacist MUST also have an active DEA registration.

@id("pharmacist-can-verify")
permit (
    principal in Group::"pharmacy.pharmacists",
    action == Action::"pharmacy.verify",
    resource is PharmacyOrder
)
when {
    principal has license &&
    principal.license.state == resource.dispense_state &&
    principal.license.status == "active" &&
    principal.license.kind == "RPh" &&
    resource.tenant_id == principal.tenant_id
};

// CII orders additionally require active DEA registration on the pharmacist.
@id("pharmacist-can-verify-cii")
forbid (
    principal,
    action == Action::"pharmacy.verify",
    resource is PharmacyOrder
)
when {
    resource has schedule &&
    resource.schedule == "CII" &&
    (!(principal has dea) || principal.dea.status != "active")
};
