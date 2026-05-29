// Cedar policy fragment: public-read.cedar
// Scope: governance µservice — public read surfaces (status page, conformance posture)
// Purpose: explicit allow-list of fields readable without authentication for tenant-trust signals
// Related: microservices/governance/PRD.md §"Tenant Outcome 2" (industry-benchmarked conformance)
//          microservices/observability/policy/public-read.cedar (shape reference)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMITS
// =============================================================================

// P1 — Public reads aggregate conformance posture (per-axis pass rate; no per-PR detail)
permit (
    principal == PublicAnonymous::"public",
    action == Action::"ReadConformancePosture",
    resource is ConformancePostureAggregate
)
when {
    resource.published_at != null &&
    resource.contains_pii == false
};

// P2 — Public reads per-µservice published SLO posture (the µservice's own SLO target, NOT realtime metrics)
permit (
    principal == PublicAnonymous::"public",
    action == Action::"ReadPublishedSLO",
    resource is PublishedSLO
)
when {
    resource.published_at != null
};

// P3 — Public reads industry-baseline pin metadata (axis + source URL + pinned SHA; not the full diff)
permit (
    principal == PublicAnonymous::"public",
    action == Action::"ReadBaselinePinMetadata",
    resource is BaselinePin
);

// P4 — Public reads list of µservices the platform tracks (catalog metadata; not Findings)
permit (
    principal == PublicAnonymous::"public",
    action == Action::"ReadCatalogPublicView",
    resource is CatalogPublicEntry
)
when {
    resource.public_listing == true
};

// P5 — Public reads quarterly transparency report summaries
permit (
    principal == PublicAnonymous::"public",
    action == Action::"ReadTransparencyReportSummary",
    resource is TransparencyReportSummary
)
when {
    resource.contains_pii == false &&
    resource.published_at != null
};

// =============================================================================
// FORBIDS — defence-in-depth (public surface is exhaustive allow-list)
// =============================================================================

// F1 — Refuse any non-allow-listed action from PublicAnonymous
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource
)
unless {
    action in [Action::"ReadConformancePosture",
               Action::"ReadPublishedSLO",
               Action::"ReadBaselinePinMetadata",
               Action::"ReadCatalogPublicView",
               Action::"ReadTransparencyReportSummary"]
};

// F2 — Refuse public reads of any resource carrying PII
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource
)
when {
    resource has contains_pii && resource.contains_pii == true
};

// F3 — Refuse public reads of any resource not explicitly published
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource
)
unless {
    resource has published_at && resource.published_at != null
};

// F4 — Refuse public reads of Findings (regardless of severity)
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource is Finding
);

// F5 — Refuse public reads of lane runs (regardless of microservice)
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource is LaneRun
);

// F6 — Refuse public reads of evidence blobs (always restricted)
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource is EvidenceBlob
);

// F7 — Refuse public reads of rule packs (security-sensitive)
forbid (
    principal == PublicAnonymous::"public",
    action,
    resource is RulePack
);

// =============================================================================
// SCHEMA HINTS
// =============================================================================
//
// Entities:
//   - PublicAnonymous (a singleton)
//   - ConformancePostureAggregate { axis: String, pass_rate: Double,
//                                   period: String, published_at: Long,
//                                   contains_pii: Boolean }
//   - PublishedSLO { microservice: String, sli: String, target: Double,
//                    window: String, published_at: Long }
//   - CatalogPublicEntry { microservice: String, public_listing: Boolean,
//                          tier: String, sales_segment: String }
//   - TransparencyReportSummary { period: String, finding_count_buckets: ...,
//                                 contains_pii: Boolean, published_at: Long }
//
// Actions: ReadConformancePosture, ReadPublishedSLO, ReadBaselinePinMetadata,
//          ReadCatalogPublicView, ReadTransparencyReportSummary

// =============================================================================
// FRAMEWORK MAPPING
// =============================================================================
//   P1-P5 → public trust signal per ADR-0123 hyperscaler-maturity-claim-gate
//   F1 → exhaustive allow-list; explicit-deny-by-default per OWASP API05
//   F2 → GDPR Art. 5(1)(f) integrity-and-confidentiality; KR PIPA Art. 29
//   F3 → "draft" content never reachable publicly; reduces surprise
//   F4-F7 → defence-in-depth; tenant-evidence never publicly readable per
//           threat-model.md §T-I-01 + T-I-02
