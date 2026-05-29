// Cedar policy — T2-tenant-domain-mgmt authorization.
// ADR anchor: ADR-0201, ADR-0183.

// Only tenant administrators (role:tenant-admin scoped to the
// tenant) and substrate-authority operators may onboard or
// rotate from-domains.

permit (
    principal,
    action in [
        Action::"comms-email:onboard-from-domain",
        Action::"comms-email:list-from-domains",
        Action::"comms-email:rotate-dkim",
        Action::"comms-email:revoke-dkim"
    ],
    resource
)
when {
    principal has role &&
    (
        (principal.role == "tenant-admin" &&
         principal has tenant_id &&
         resource has tenant_id &&
         principal.tenant_id == resource.tenant_id) ||
        principal.role == "substrate-authority"
    )
};

// Revocation requires substrate-authority — tenant-admins may
// initiate rotate but only substrate-authority may revoke.
forbid (
    principal,
    action == Action::"comms-email:revoke-dkim",
    resource
)
when {
    principal has role &&
    principal.role == "tenant-admin"
};
