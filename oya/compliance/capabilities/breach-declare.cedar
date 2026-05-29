// Capability: breach-declare — high-risk capability with 72h SLA
// Tier: T3 (irreversible audit event with regulatory clock)
// Binding: ADR-0251 + IP-019
//
// Capability schema:
//   id: breach-declare
//   actions: [declare_breach]
//   audit_event_classes: [oya.compliance.breach-declared]
//   risk_class: high

permit (
    principal in [
        oyatie::Tenancy::TenantAdminGroup::"compliance-admins",
        oyatie::Compliance::Principal::"breach-notifier"
    ],
    action == Action::"declare_breach",
    resource is oyatie::Compliance::BreachRecord
) when {
    principal.tenant_id == resource.tenant_id
};

// Defence-in-depth: never permit declare if the principal isn't trained / authorized.
forbid (
    principal,
    action == Action::"declare_breach",
    resource
) when {
    principal.breach_declaration_authorized != true
};
