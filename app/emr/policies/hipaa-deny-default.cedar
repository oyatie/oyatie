// Cedar policy — hipaa.deny-default
// Owning µservice: emr
// Authored Wave 15M-B 2026-05-21
// Per ADR-0243 default-deny + ADR-0251 HIPAA-2024 pack

// Default-deny on every PHI-touching action unless an explicit permit fires.
// Cedar default-deny is implicit, but this policy adds tenant-scoped, pack-scoped
// safeguard rails that the policy engine must satisfy first.

// Refuse cross-tenant access
forbid (
    principal,
    action,
    resource
)
when {
    principal.tenant_id != resource.tenant_id
};

// Refuse PHI access for tenants without HIPAA-2024 pack installed
forbid (
    principal,
    action,
    resource
)
when {
    resource.data_class == "phi-protected-health-information" &&
    !("HIPAA-2024" in context.installed_packs)
};

// Refuse access when patient has not consented and pack requires explicit consent
forbid (
    principal,
    action,
    resource
)
when {
    resource.data_class == "phi-protected-health-information" &&
    context.consent_required == true &&
    context.consent_state != "GRANTED"
};

// Refuse access from non-AAL2 sessions (HIPAA Security Rule §164.312(d))
forbid (
    principal,
    action,
    resource
)
when {
    resource.data_class == "phi-protected-health-information" &&
    !(context.session_assurance_level in ["AAL2", "AAL3"])
};

// Refuse access when cell's certification level does not match pack's eligibility
forbid (
    principal,
    action,
    resource
)
when {
    !(context.cell_certification_level in [
        "hipaa-certified",
        "hipaa-pci-certified",
        "healthcare-sovereign"
    ])
};
