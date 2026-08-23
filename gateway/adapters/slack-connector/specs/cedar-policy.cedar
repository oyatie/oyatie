// Cedar policy template — tenant→Slack adapter access.
// One policy per tenant; deployed by policy-cedar.

// Allow ops principals in tenant T to use the slack adapter
// for conversation + message entity kinds.
permit (
    principal in OyaPrincipalGroup::"tenant-ops",
    action in [Action::"list", Action::"get", Action::"create", Action::"update"],
    resource in OyaResource::"connector-slack"
)
when {
    principal.tenant == resource.tenant &&
    resource.entity_kind in ["conversation", "message", "file"]
};

// Deny non-admin from delete; Slack delete is destructive.
forbid (
    principal,
    action == Action::"delete",
    resource in OyaResource::"connector-slack"
)
unless {
    principal in OyaPrincipalGroup::"tenant-admin"
};
