permit (
    principal in OyaPrincipalGroup::"tenant-hr-ops",
    action in [Action::"list", Action::"get"],
    resource in OyaResource::"connector-adp"
)
when {
    principal.tenant == resource.tenant
};

permit (
    principal in OyaPrincipalGroup::"tenant-payroll-admin",
    action in [Action::"create", Action::"update"],
    resource in OyaResource::"connector-adp"
)
when {
    principal.tenant == resource.tenant
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-adp"
);
