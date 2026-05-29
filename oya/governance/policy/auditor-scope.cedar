// Cedar policy fragment: auditor-scope.cedar
// Scope: governance µservice — external-auditor JIT-scoped read access
// Purpose: bound external auditors (SOC 2 / ISO 27001 / SLSA / GDPR DPAs) to least-privilege
//          read-only access during a time-boxed audit window
// Related: microservices/governance/dpia.md §3 Right of Access §"External auditor"
//          microservices/governance/runbooks/evidence-replay.md
//          microservices/governance/threat-model.md §P-D-02
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMITS
// =============================================================================

// P1 — External auditor reads Findings within scoped microservices + window
permit (
    principal in ExternalAuditor,
    action == Action::"ReadFinding",
    resource is Finding
)
when {
    resource.microservice in principal.scope_microservices &&
    resource.created_at >= principal.audit_window_start &&
    resource.created_at <= principal.audit_window_end &&
    principal.token_age_seconds <= 3600
};

// P2 — External auditor reads lane runs within scope
permit (
    principal in ExternalAuditor,
    action == Action::"ReadLaneRun",
    resource is LaneRun
)
when {
    resource.microservice in principal.scope_microservices &&
    resource.started_at >= principal.audit_window_start &&
    resource.started_at <= principal.audit_window_end &&
    principal.token_age_seconds <= 3600
};

// P3 — External auditor reads evidence blobs within scope
permit (
    principal in ExternalAuditor,
    action == Action::"ReadEvidenceBlob",
    resource is EvidenceBlob
)
when {
    resource.linked_finding.microservice in principal.scope_microservices &&
    resource.linked_finding.created_at >= principal.audit_window_start &&
    resource.linked_finding.created_at <= principal.audit_window_end &&
    principal.token_age_seconds <= 3600 &&
    resource.data_residency_pack in principal.scope_packs
};

// P4 — External auditor reads audit-chain seals within scope
permit (
    principal in ExternalAuditor,
    action == Action::"ReadAuditChainSeal",
    resource is AuditChainSeal
)
when {
    resource.sealed_at >= principal.audit_window_start &&
    resource.sealed_at <= principal.audit_window_end &&
    principal.token_age_seconds <= 3600
};

// P5 — External auditor exports evidence bundle (canonical-JSON + Ed25519 manifest)
permit (
    principal in ExternalAuditor,
    action == Action::"ExportEvidenceBundle",
    resource is EvidenceExport
)
when {
    resource.scope_microservices == principal.scope_microservices &&
    resource.window_start == principal.audit_window_start &&
    resource.window_end == principal.audit_window_end &&
    principal.token_age_seconds <= 3600 &&
    resource.export_size_bytes <= 5368709120     // 5 GB per export
};

// P6 — External auditor queries baseline-pin history (read-only)
permit (
    principal in ExternalAuditor,
    action == Action::"ReadBaselinePinHistory",
    resource is BaselinePin
);

// =============================================================================
// FORBIDS — defence-in-depth
// =============================================================================

// F1 — Refuse external auditor outside their audit window
forbid (
    principal in ExternalAuditor,
    action,
    resource
)
unless {
    principal.audit_window_start <= context.now &&
    context.now <= principal.audit_window_end
};

// F2 — Refuse external auditor with expired token (>1h)
forbid (
    principal in ExternalAuditor,
    action,
    resource
)
when {
    principal.token_age_seconds > 3600
};

// F3 — Refuse external auditor any write action
forbid (
    principal in ExternalAuditor,
    action in [Action::"WriteFinding", Action::"WriteLaneRun",
               Action::"WriteRulePack", Action::"WriteBaselinePin",
               Action::"WriteAggregationIndex", Action::"InsertFinding",
               Action::"UpdateFinding", Action::"DeleteFinding",
               Action::"DispatchMatrixJob", Action::"SealFinding"],
    resource
);

// F4 — Refuse external auditor cross-pack read where not scope-listed
forbid (
    principal in ExternalAuditor,
    action == Action::"ReadEvidenceBlob",
    resource is EvidenceBlob
)
unless {
    resource.data_residency_pack in principal.scope_packs
};

// F5 — Refuse external auditor reading Findings outside scope microservices
forbid (
    principal in ExternalAuditor,
    action == Action::"ReadFinding",
    resource is Finding
)
unless {
    resource.microservice in principal.scope_microservices
};

// F6 — Refuse external auditor export larger than 5 GB
forbid (
    principal in ExternalAuditor,
    action == Action::"ExportEvidenceBundle",
    resource is EvidenceExport
)
when {
    resource.export_size_bytes > 5368709120
};

// F7 — Refuse pack-us-healthcare PHI read by auditors lacking BAA acknowledgement claim
forbid (
    principal in ExternalAuditor,
    action,
    resource is EvidenceBlob
)
when {
    resource.data_residency_pack == "pack-us-healthcare" &&
    !principal.baa_acknowledged
};

// =============================================================================
// SCHEMA HINTS
// =============================================================================
//
// Entities:
//   - ExternalAuditor { subject: String, audit_window_id: String,
//                       audit_window_start: Long, audit_window_end: Long,
//                       token_age_seconds: Long, scope_microservices: Set<String>,
//                       scope_packs: Set<String>, baa_acknowledged: Boolean }
//   - AuditChainSeal { sealed_at: Long, merkle_root: String,
//                      scope_microservice: String }
//   - EvidenceExport { scope_microservices: Set<String>, window_start: Long,
//                      window_end: Long, export_size_bytes: Long }
//   - BaselinePin { axis: String, source_url: String, pinned_sha: String,
//                   effective_at: Long }
//
// Actions: ReadFinding, ReadLaneRun, ReadEvidenceBlob, ReadAuditChainSeal,
//          ExportEvidenceBundle, ReadBaselinePinHistory

// =============================================================================
// FRAMEWORK MAPPING
// =============================================================================
//   P1-P6 → SOC 2 CC7.4 (audit evidence); ISO 27001 A.5.34, A.5.35, A.5.37
//   F1/F2 → least-privilege; JIT-scope per `feedback_openbao_secrets.md`
//   F3 → read-only auditor posture; SOC 2 CC8.1
//   F4-F5 → ADR-0117 data-residency
//   F7 → HIPAA §164.308 BAA chain; pack-us-healthcare isolation
