// Cedar policy — T0-transactional-send authorization.
// ADR anchor: ADR-0201, ADR-0183.

// Allow a µservice principal to send on behalf of a tenant
// only when (a) the tenant binding's from-domain matches the
// message from address domain, (b) the tenant is in `active`
// state for the targeted from-domain, and (c) the principal
// holds the `comms-email:send` capability for the tenant.

permit (
    principal,
    action == Action::"comms-email:send",
    resource
)
when {
    resource has tenant_id &&
    resource has from_domain &&
    principal has capabilities &&
    principal.capabilities.contains("comms-email:send:" + resource.tenant_id) &&
    resource.from_domain_state == "active"
};

// Sovereign-pack tenants reject any non-Postal provider at
// policy time (defense-in-depth on top of IP-014 config-load
// enforcement).
forbid (
    principal,
    action == Action::"comms-email:send",
    resource
)
when {
    resource has pack_tag &&
    (resource.pack_tag == "ksa" || resource.pack_tag == "uae") &&
    resource has provider &&
    resource.provider != "postal"
};
