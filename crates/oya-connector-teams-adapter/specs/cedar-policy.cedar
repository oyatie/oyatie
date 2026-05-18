permit (
    principal in OyaPrincipalGroup::"tenant-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-teams"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["channel", "message", "adaptive-card"]
};

forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-teams"
)
unless {
    principal in OyaPrincipalGroup::"tenant-admin"
};
