// Cedar v4.2 LTS — compliance µservice auditor scope
// Binding ADR: ADR-0243 + ADR-compliance-003-auditor-access-cedar-policy

// Auditor permit fires only inside an active engagement window with explicit scope.
permit (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action in [
        Action::"view_evidence",
        Action::"export_evidence_bundle",
        Action::"view_control_mapping"
    ],
    resource
) when {
    principal.engagement_active == true
    && principal.engagement_scope == resource.evidence_scope
    && resource.evidence_class in principal.engagement_evidence_classes
    && resource.tenant_id == principal.engagement_tenant_id
};

// Auditor CANNOT view PII/PHI evidence unless engagement scope explicitly permits.
forbid (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action,
    resource
) when {
    resource.data_class in ["PII_IDENTIFYING", "PHI"]
    && !principal.engagement_evidence_classes.contains(resource.data_class)
};

// Auditor scope auto-expires.
forbid (
    principal in oyatie::Compliance::AuditorGroup::"engaged",
    action,
    resource
) when {
    principal.engagement_ends_at < context.request_time
};
