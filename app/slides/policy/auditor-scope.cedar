// Cedar v4.2 LTS — auditor scope policy for slides µservice
// Auditor principals (compliance officers, DPO office, SOC 2 auditors, HIPAA auditors)
// have scoped read access for audit purposes. No deck content read except under explicit grant.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (principal, action, resource);

// Auditors may read audit-chain seal ledgers + SLO reports + DPIA artifacts within their pack
permit (
    principal is AuditorPrincipal,
    action in [
        Action::"audit_seal:read",
        Action::"slo_report:read",
        Action::"dpia:read",
        Action::"threat_model:read",
        Action::"compliance:read",
        Action::"runbook:read",
        Action::"adr:read"
    ],
    resource
)
when {
    principal.scope == "slides" &&
    principal.pack == resource.pack &&
    principal.bears_audit_engagement_token == true
};

// Auditors may read tenant deck content ONLY when tenant grants explicit audit-grant
permit (
    principal is AuditorPrincipal,
    action in [Action::"deck:read", Action::"slide:read", Action::"comment:read", Action::"version:read"],
    resource
)
when {
    principal.scope == "slides" &&
    principal.pack == resource.pack &&
    resource has audit_grant_for &&
    principal in resource.audit_grant_for
};

// Auditors may NEVER edit deck content
forbid (
    principal is AuditorPrincipal,
    action in [Action::"deck:edit", Action::"slide:edit", Action::"comment:write", Action::"version:restore"],
    resource
);

// Auditors may NEVER start broadcasts
forbid (
    principal is AuditorPrincipal,
    action in [Action::"broadcast:start", Action::"broadcast:join"],
    resource
);

// Auditors may NEVER invoke AI capabilities
forbid (
    principal is AuditorPrincipal,
    action in [Action::"ai_design:invoke", Action::"ai_content_generation:invoke"],
    resource
);

// Auditors MUST stay within their assigned pack
forbid (
    principal is AuditorPrincipal,
    action,
    resource
)
when {
    principal.pack != resource.pack
};

// HIPAA auditors specifically scoped to us-healthcare pack
forbid (
    principal is AuditorPrincipal,
    action,
    resource
)
when {
    principal.role == "hipaa_auditor" &&
    resource.pack != "us-healthcare"
};
