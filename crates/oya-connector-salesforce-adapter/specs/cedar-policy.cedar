permit (
    principal in OyaPrincipalGroup::"tenant-crm-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-salesforce"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["Account", "Contact", "Opportunity"]
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-salesforce"
)
unless {
    principal in OyaPrincipalGroup::"tenant-crm-admin"
};
