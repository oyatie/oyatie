permit (
    principal in OyaPrincipalGroup::"tenant-hr-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-rippling"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["employee", "device", "transaction"]
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-rippling"
)
unless {
    principal in OyaPrincipalGroup::"tenant-hr-admin"
};
