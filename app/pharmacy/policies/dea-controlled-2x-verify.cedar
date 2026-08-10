// DEA-controlled-2x-verify Cedar policy
// Authority: ADR-0243 (Cedar universal gate); ADR-0332 (pharmacy substrate); 21 CFR §1306
// Effect: Schedule II prescriptions MUST be dual-verified by two distinct active pharmacists.

@id("dea-controlled-2x-verify")
forbid (
    principal,
    action == Action::"pharmacy.dispense",
    resource is PharmacyOrder
)
when {
    resource has schedule &&
    resource.schedule == "CII" &&
    (
        !(resource has primary_verifier_id) ||
        !(resource has witness_verifier_id) ||
        resource.primary_verifier_id == "" ||
        resource.witness_verifier_id == "" ||
        resource.primary_verifier_id == resource.witness_verifier_id
    )
};
