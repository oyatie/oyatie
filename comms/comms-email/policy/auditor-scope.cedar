// Cedar v4.2 LTS — comms-email auditor scope
// Binding: ADR-0243 (auditor read-only); engagement window-bound.

permit (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action in [
        Action::"view_send_log",
        Action::"view_bounce_log",
        Action::"view_dkim_rotation_history",
        Action::"export_compliance_evidence"
    ],
    resource
) when {
    principal.engagement_active == true
    && principal.engagement_tenant_id == resource.tenant_id
    && principal.engagement_evidence_classes.contains(resource.data_class)
};

forbid (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action,
    resource
) when {
    resource.data_class == "PII_IDENTIFYING"
    && !principal.engagement_evidence_classes.contains("PII_IDENTIFYING")
};

forbid (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action,
    resource
) when {
    principal.engagement_ends_at < context.request_time
};
