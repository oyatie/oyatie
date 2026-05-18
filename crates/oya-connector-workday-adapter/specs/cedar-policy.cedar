permit (
    principal in OyaPrincipalGroup::"tenant-hr-ops",
    action in [Action::"list", Action::"get"],
    resource in OyaResource::"connector-workday"
)
when {
    principal.tenant == resource.tenant
};

permit (
    principal in OyaPrincipalGroup::"tenant-hr-admin",
    action in [Action::"create", Action::"update"],
    resource in OyaResource::"connector-workday"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["worker", "job-profile", "compensation"]
};

// Hard-delete is unsupported by Workday; deny just in case caller asks.
forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-workday"
);
