// Nurse-can-administer Cedar policy
// Authority: ADR-0243 (Cedar universal gate); ADR-0332 (pharmacy substrate)
// Effect: only an RN or LPN with active state license, on active shift at the relevant facility,
// can record a medication administration; license-residency enforced by identity.

@id("nurse-can-administer")
permit (
    principal in Group::"pharmacy.nurses",
    action == Action::"pharmacy.administer",
    resource is MedicationAdministration
)
when {
    principal has license &&
    principal.license.status == "active" &&
    principal.license.kind in ["RN", "LPN", "LVN", "APRN"] &&
    principal.license.state == resource.administration_state &&
    principal has shift &&
    principal.shift.active == true &&
    principal.shift.facility_id == resource.facility_id &&
    resource.tenant_id == principal.tenant_id
};

// Override path: any failed right requires explicit override + reason + pharmacist callback within 5 min.
@id("nurse-five-rights-override")
permit (
    principal in Group::"pharmacy.nurses",
    action == Action::"pharmacy.administer.override",
    resource is MedicationAdministration
)
when {
    resource has override_reason &&
    resource.override_reason != "" &&
    resource has pharmacist_callback_within_minutes &&
    resource.pharmacist_callback_within_minutes <= 5
};
