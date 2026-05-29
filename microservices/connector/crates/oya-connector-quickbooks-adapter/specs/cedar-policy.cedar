permit (
    principal in OyaPrincipalGroup::"tenant-finance-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-quickbooks"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["Customer", "Vendor", "Invoice", "Bill"]
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-quickbooks"
)
unless {
    principal in OyaPrincipalGroup::"tenant-finance-admin"
};
