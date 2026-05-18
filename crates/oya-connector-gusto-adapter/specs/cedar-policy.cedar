permit (
    principal in OyaPrincipalGroup::"tenant-payroll-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-gusto"
)
when {
    principal.tenant == resource.tenant
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-gusto"
);
