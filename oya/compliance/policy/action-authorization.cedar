// Cedar v4.2 LTS — compliance µservice action-authorization (default-deny baseline)
// Binding ADR: ADR-0243 (Cedar universal gate)
// Soak: ≥60s per ADR-0294
// x-fragment-soak-seconds: 60

// Default-deny: nothing is permitted unless an explicit permit fires below.
// Tenant admins may emit evidence + open DSAR + view their own compliance posture.
permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"compliance-admins",
    action in [
        Action::"emit_evidence",
        Action::"open_dsar",
        Action::"view_posture",
        Action::"download_subject_export"
    ],
    resource
) when {
    principal.tenant_id == resource.tenant_id
    && principal.audience_type == "B2B_TENANT_ADMIN"
};

// DSAR subjects may open/track their own DSARs and download portable backups (ADR-0276).
permit (
    principal,
    action in [
        Action::"open_dsar_self",
        Action::"track_dsar_self",
        Action::"download_portability_export"
    ],
    resource
) when {
    principal.audience_type == "B2C_DSAR_SUBJECT"
    && principal.subject_id == resource.subject_id
};

// Substrate principals: evidence collector + DSAR orchestrator + breach notifier.
permit (
    principal in [
        oyatie::Compliance::Principal::"evidence-collector",
        oyatie::Compliance::Principal::"dsar-orchestrator",
        oyatie::Compliance::Principal::"breach-notifier",
        oyatie::Compliance::Principal::"regulator-evidence-emit"
    ],
    action in [
        Action::"emit_audit_event",
        Action::"seal_evidence",
        Action::"notify_authority",
        Action::"emit_regulator_evidence"
    ],
    resource
) when {
    principal.spiffe_attested == true
    && principal.tenant_id == resource.tenant_id
};

// Auditor scope (read-only) — see policy/auditor-scope.cedar for engagement gate.
permit (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action in [Action::"view_evidence", Action::"export_evidence_bundle"],
    resource
) when {
    principal.engagement_active == true
    && principal.engagement_scope == resource.evidence_scope
};

// Forbid any cross-tenant access (defence-in-depth).
forbid (principal, action, resource) when {
    principal.tenant_id != resource.tenant_id
    && !(principal.audience_type == "B2C_DSAR_SUBJECT" && principal.subject_id == resource.subject_id)
};
