// Cedar policy fragment: tenant-scope.cedar
// Scope: governance µservice
// Purpose: ABAC tenant-isolation for Finding + evidence reads
// Related: microservices/governance/policy/data-residency.md
//          microservices/governance/threat-model.md §T-I-01
//          ADR-0117 (data-residency); ADR-0131 (per-microservice)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)
//
// Cedar version: 3.x
// Evaluation point: API gateway (Envoy ext_authz) on every read of
//   GET /findings, GET /lane-runs, GET /evidence/{blob_id}, GET /aggregation-indices
//
// Decision order: permits (below) then forbids (data-residency.cedar +
//   ci-scope.cedar + auditor-scope.cedar). Cedar default is forbid.

// =============================================================================
// PERMITS
// =============================================================================

// P1 — Tenant operator reads own µservice's Findings
permit (
    principal in TenantOperator,
    action == Action::"ReadFinding",
    resource is Finding
)
when {
    resource.microservice in principal.owned_microservices &&
    resource.tenant_id == principal.tenant_id
};

// P2 — Tenant operator reads own µservice's lane runs
permit (
    principal in TenantOperator,
    action == Action::"ReadLaneRun",
    resource is LaneRun
)
when {
    resource.microservice in principal.owned_microservices &&
    resource.tenant_id == principal.tenant_id
};

// P3 — Tenant operator reads evidence blobs for own µservice's Findings
permit (
    principal in TenantOperator,
    action == Action::"ReadEvidenceBlob",
    resource is EvidenceBlob
)
when {
    resource.linked_finding.microservice in principal.owned_microservices &&
    resource.linked_finding.tenant_id == principal.tenant_id &&
    resource.data_residency_pack == principal.residency_pack
};

// P4 — Tenant operator self-service: replay own µservice's PR
permit (
    principal in TenantOperator,
    action == Action::"ReplayPullRequest",
    resource is PullRequestEvidence
)
when {
    resource.microservice in principal.owned_microservices &&
    resource.pr_author == principal.subject &&
    resource.tenant_id == principal.tenant_id
};

// P5 — Internal engineer reads any non-tenant-attributable lane run
permit (
    principal in InternalEngineer,
    action == Action::"ReadLaneRun",
    resource is LaneRun
)
when {
    resource.tenant_id == "internal" ||
    principal in role::"axis-foundry-on-call"
};

// P6 — Internal engineer reads any non-AUDIT Finding view
permit (
    principal in InternalEngineer,
    action == Action::"ReadFinding",
    resource is Finding
)
when {
    resource.view == "non-audit"
};

// =============================================================================
// FORBIDS — defence-in-depth (refuse-all-cross-tenant)
// =============================================================================

// F1 — Refuse cross-tenant Finding read regardless of permits
forbid (
    principal,
    action == Action::"ReadFinding",
    resource is Finding
)
when {
    principal has tenant_id &&
    resource has tenant_id &&
    principal.tenant_id != resource.tenant_id &&
    !(principal in InternalEngineer) &&
    !(principal in ExternalAuditor)
};

// F2 — Refuse cross-residency-pack evidence read regardless of permits
forbid (
    principal,
    action == Action::"ReadEvidenceBlob",
    resource is EvidenceBlob
)
when {
    principal has residency_pack &&
    resource.data_residency_pack != principal.residency_pack &&
    !(principal in role::"council-privacy-cross-pack-admin")
};

// F3 — Refuse non-axis-foundry-on-call from AUDIT-view Findings except for
//      ops-security or external auditors during a JIT window
forbid (
    principal,
    action == Action::"ReadFinding",
    resource is Finding
)
when {
    resource.view == "audit" &&
    !(principal in role::"ops-security") &&
    !(principal in role::"council-privacy") &&
    !(principal in ExternalAuditor && resource.in_audit_window == true)
};

// F4 — Refuse evidence replay outside permitted retention window
forbid (
    principal,
    action == Action::"ReplayPullRequest",
    resource is PullRequestEvidence
)
when {
    resource.archived_at < (context.now - duration("2555d"))   // 7y baseline; pack-specific override at iac/kustomize/overlays/pack-<pack>/
};

// =============================================================================
// SCHEMA HINTS (informative; full schema at microservices/governance/specs/cedar-schema.json)
// =============================================================================
//
// Entities:
//   - TenantOperator { tenant_id: String, subject: String,
//                      owned_microservices: Set<String>,
//                      residency_pack: String }
//   - InternalEngineer { subject: String, residency_pack: String }
//   - ExternalAuditor { subject: String, audit_window_id: String,
//                       scope_microservices: Set<String> }
//   - Finding { microservice: String, tenant_id: String, view: String /* "audit"|"non-audit" */,
//               severity: String, lane_id: String, sha: String, in_audit_window: Boolean }
//   - LaneRun { microservice: String, tenant_id: String, lane_id: String, sha: String }
//   - EvidenceBlob { linked_finding: Finding, data_residency_pack: String }
//   - PullRequestEvidence { microservice: String, tenant_id: String, pr_author: String,
//                           archived_at: Long }
//
// Actions: ReadFinding, ReadLaneRun, ReadEvidenceBlob, ReplayPullRequest

// =============================================================================
// FRAMEWORK MAPPING (informative)
// =============================================================================
//   F1 → GDPR Art. 25 (data-protection by design); ISO 27001 A.8.3; SOC 2 CC6.1, CC6.6
//   F2 → ADR-0117 data-residency; pack-specific laws (KR PIPA Art. 28; HIPAA §164.308; GDPR Art. 44)
//   F3 → SOC 2 CC6.3 separation-of-duties; ISO 27001 A.5.16
//   F4 → GDPR Art. 5(1)(e); KR PIPA Art. 21; HIPAA §164.316(b)(2)
//   P5/P6 → least-privilege; SOC 2 CC6.2; ISO 27001 A.5.15
