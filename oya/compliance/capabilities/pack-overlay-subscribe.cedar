// Capability: pack-overlay-subscribe — tenant-admin self-service capability
// Tier: T2 (configuration; reversible)
// Binding: ADR-0251 + policy/pack-overlay-authorization.cedar
//
// Capability schema:
//   id: pack-overlay-subscribe
//   actions: [subscribe_pack, unsubscribe_pack]
//   audit_event_classes:
//     - oya.compliance.pack-subscribed
//     - oya.compliance.pack-unsubscribed
//   risk_class: medium

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"compliance-admins",
    action in [Action::"subscribe_pack", Action::"unsubscribe_pack"],
    resource is oyatie::Compliance::PackOverlay
) when {
    principal.tenant_id == resource.tenant_id
    && resource.pack_status == "published"
};
