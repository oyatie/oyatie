permit (
    principal in OyaPrincipalGroup::"tenant-finance-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-netsuite"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["customer", "vendor", "salesOrder", "journalEntry"]
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-netsuite"
)
unless {
    principal in OyaPrincipalGroup::"tenant-finance-admin"
};
