// Cedar policy fragment: cedar-canonical-imports.cedar
// Scope: canonical Cedar entity-type declarations shared across every µservice.
// Authority: ADR-0064 canonical-base + localization-packs; SWEEP-I Slice 1.
// Consumer pattern: every per-µservice policy file (tenant-scope.cedar,
//   ci-scope.cedar, auditor-scope.cedar, public-read.cedar) declares its
//   µservice-specific rules ONLY and references this canonical file in its
//   header comment.
//
// Cedar version: 3.x
// Default: forbid. Permits, then forbids. Explicit deny wins.
//
// ============================================================================
// CANONICAL ENTITY DECLARATIONS (every µservice inherits these)
// ============================================================================
//
// entity TenantOperator {
//   tenant_id: String,
//   subject: String,
//   owned_microservices: Set<String>,
//   residency_pack: String,
//   token_age_seconds: Long
// };
//
// entity InternalEngineer {
//   subject: String,
//   residency_pack: String,
//   roles: Set<String>
// };
//
// entity ExternalAuditor {
//   subject: String,
//   audit_window_id: String,
//   audit_window_start: Long,
//   audit_window_end: Long,
//   token_age_seconds: Long,
//   scope_microservices: Set<String>,
//   scope_packs: Set<String>,
//   baa_acknowledged: Boolean
// };
//
// entity LaneRunner {
//   runner_id: String,
//   registered_lanes: Set<String>
// };
//
// entity PublicAnonymous;   // singleton: PublicAnonymous::"public"
//
// entity Tenant {
//   tenant_id: String,
//   residency_pack: String,
//   created_at: Long
// };
//
// entity Cell {
//   cell_id: String,
//   tenant_id: String,
//   residency_pack: String
// };
//
// entity Capability {
//   tier: String,        // "T0"|"T1"|"T2"
//   eu_ai_act_risk_class: String   // "prohibited"|"high"|"limited"|"minimal"
// };
//
// entity Pack {
//   pack_name: String    // "pack-kr"|"pack-eu"|"pack-us"|"pack-us-healthcare"|...
// };
//
// entity AuditChainSeal {
//   sealed_at: Long,
//   merkle_root: String,
//   scope_microservice: String
// };
//
// ============================================================================
// CANONICAL ACTION DECLARATIONS (subset; per-µservice extends)
// ============================================================================
//
// action ReadFinding, ReadLaneRun, ReadEvidenceBlob, ReadAuditChainSeal,
//        WriteFinding, WriteLaneRun, WriteRulePack, WriteBaselinePin,
//        WriteAggregationIndex, InsertFinding, UpdateFinding, DeleteFinding,
//        DispatchMatrixJob, SealFinding, QueryAdmissionVerdict,
//        DispatchPullRequestOpened, ExportEvidenceBundle,
//        ReadConformancePosture, ReadPublishedSLO, ReadBaselinePinMetadata,
//        ReadCatalogPublicView, ReadTransparencyReportSummary,
//        ReplayPullRequest;
//
// ============================================================================
// CANONICAL CONTEXT TYPE (every evaluation receives)
// ============================================================================
//
// context {
//   now: Long,                                      // current ts
//   allow_listed_github_ranges: Set<String>,        // webhook source CIDRs
//   registered_lanes: Set<String>,                  // registry-driven
//   tenant_id: String,                              // header-extracted
//   residency_pack: String                          // header-extracted
// };
//
// ============================================================================
// CANONICAL FORBID PATTERNS (every µservice MUST embody by purpose)
// ============================================================================
//
// F-CROSS-TENANT: refuse cross-tenant data regardless of permits
// F-CROSS-PACK:   refuse cross-residency-pack data egress
// F-EXPIRED-TOKEN: refuse expired/rotated credentials
// F-LEAST-PRIVILEGE: exhaustive allow-list for anonymous principals
//
// These are STRUCTURAL requirements; each µservice may use different concrete
// wording, but `oya gate cedar-structural-validator` checks that each F-*
// purpose is detectable in the per-µservice .cedar files.
//
// ============================================================================
// CANONICAL DECISION ORDER (every µservice inherits)
// ============================================================================
//
// 1. Cedar evaluates all `permit` statements; if any matches, decision is ALLOW.
// 2. Cedar then evaluates all `forbid` statements; if any matches, decision
//    is overridden to DENY (explicit deny wins).
// 3. If no permit matches, decision defaults to DENY (least-privilege).
//
// ============================================================================
// CANONICAL FRAMEWORK MAPPING (per-µservice files cite specific rules)
// ============================================================================
//
// GDPR Art. 5(1)(f) — integrity and confidentiality
// GDPR Art. 25 — data protection by design
// GDPR Art. 44 — cross-border transfers
// KR PIPA Art. 21, 28, 29
// HIPAA §164.308, §164.316, BAA chain
// ISO 27001 A.5.15, A.5.16, A.5.34, A.5.35, A.5.37, A.8.3, A.8.7, A.8.20, A.8.24
// SOC 2 CC6.1, CC6.2, CC6.3, CC6.6, CC7.4, CC8.1
// SLSA Build L3 (build provenance)
// OWASP API01 broken-authentication
// OWASP API05 broken function-level authorization
// EU AI Act 2024/1689 Annex III (capability-specific)
