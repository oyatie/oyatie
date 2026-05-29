// PHI access — strictest tier. Reads gated on clinical-roles group;
// writes gated on clinician role; delete forbidden globally.
permit (
    principal in OyaPrincipalGroup::"tenant-clinical-reader",
    action in [Action::"list", Action::"get"],
    resource in OyaResource::"connector-epic-fhir"
)
when {
    principal.tenant == resource.tenant
};

permit (
    principal in OyaPrincipalGroup::"tenant-clinician",
    action in [Action::"create", Action::"update"],
    resource in OyaResource::"connector-epic-fhir"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["MedicationRequest", "Observation", "Encounter"]
};

// Patient is read-only via this adapter.
forbid (
    principal,
    action in [Action::"create", Action::"update"],
    resource in OyaResource::"connector-epic-fhir"
)
when {
    resource.entity_kind == "Patient"
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-epic-fhir"
);
