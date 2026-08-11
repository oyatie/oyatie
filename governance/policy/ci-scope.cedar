// Cedar policy fragment: ci-scope.cedar
// Scope: governance µservice — CI runner scopes for lane execution + admission gate
// Purpose: bound lane runners + admission-gate clients to least-privilege actions
// Related: microservices/governance/policy/lane-execution.md
//          microservices/governance/threat-model.md §T-S-01, T-T-02, T-E-02
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMITS
// =============================================================================

// P1 — Lane runner writes Findings for its own scoped runner identity
permit (
    principal in LaneRunner,
    action == Action::"WriteFinding",
    resource is Finding
)
when {
    resource.runner_id == principal.runner_id &&
    resource.lane_id in principal.registered_lanes &&
    resource.severity in ["BLOCKER", "WARN", "INFO"] &&
    resource.signature_valid == true
};

// P2 — Lane runner writes lane-run metadata
permit (
    principal in LaneRunner,
    action == Action::"WriteLaneRun",
    resource is LaneRun
)
when {
    resource.runner_id == principal.runner_id &&
    resource.lane_id in principal.registered_lanes &&
    resource.duration_ms <= 60000     // Invariant 3 bound
};

// P3 — Merge-queue queries admission verdict
permit (
    principal == role::"merge-queue",
    action == Action::"QueryAdmissionVerdict",
    resource is PullRequest
);

// P4 — GitHub webhook ingress posts PullRequestOpened
permit (
    principal == role::"github-webhook-ingress",
    action == Action::"DispatchPullRequestOpened",
    resource is WebhookPayload
)
when {
    resource.signature_valid == true &&
    resource.delivery_id_replay == false &&
    resource.source_ip in context.allow_listed_github_ranges
};

// P5 — Aggregation-indexer reads per-µservice sources
permit (
    principal == role::"aggregation-indexer",
    action in [Action::"ReadPerMicroserviceSource",
               Action::"WriteAggregationIndex"],
    resource is RepoFile
)
when {
    (action == Action::"ReadPerMicroserviceSource" &&
     resource.path matches "microservices/.+/(PRD\\.md|catalog/.+|slos/.+|specs/.+|contracts/.+|policy/.+)") ||
    (action == Action::"WriteAggregationIndex" &&
     resource.path in principal.allow_listed_index_paths)
};

// P6 — Lane-runtime worker dispatches matrix jobs
permit (
    principal == role::"lane-runtime-worker",
    action == Action::"DispatchMatrixJob",
    resource is MatrixJob
)
when {
    resource.runner_count <= 200 &&     // max-replicas cap
    resource.lane_id in context.registered_lanes
};

// P7 — Evidence-emitter signs Findings with scoped Ed25519 key
permit (
    principal == role::"evidence-emitter",
    action == Action::"SealFinding",
    resource is Finding
)
when {
    principal.key_environment == resource.environment &&
    principal.key_rotation_age_days <= 90
};

// =============================================================================
// FORBIDS — defence-in-depth
// =============================================================================

// F1 — Refuse lane runners writing findings for lanes they aren't registered for
forbid (
    principal in LaneRunner,
    action == Action::"WriteFinding",
    resource is Finding
)
unless {
    resource.lane_id in principal.registered_lanes
};

// F2 — Refuse lane-runner severity-degradation: a BLOCKER-registered lane cannot emit WARN
forbid (
    principal in LaneRunner,
    action == Action::"WriteFinding",
    resource is Finding
)
when {
    principal.severity_for(resource.lane_id) != resource.severity
};

// F3 — Refuse rule-pack write attempts from non-PR identities
forbid (
    principal,
    action == Action::"WriteRulePack",
    resource is RulePack
)
unless {
    principal in role::"pr-review-bot" &&
    resource.via_pr == true
};

// F4 — Refuse aggregation-indexer writes outside scoped paths (per T-E-03)
forbid (
    principal == role::"aggregation-indexer",
    action == Action::"WriteAggregationIndex",
    resource is RepoFile
)
unless {
    resource.path in principal.allow_listed_index_paths
};

// F5 — Refuse webhook ingress without valid HMAC + replay-protection
forbid (
    principal == role::"github-webhook-ingress",
    action == Action::"DispatchPullRequestOpened",
    resource is WebhookPayload
)
unless {
    resource.signature_valid == true &&
    resource.delivery_id_replay == false
};

// F6 — Refuse evidence-emitter signing with rotated-out key
forbid (
    principal == role::"evidence-emitter",
    action == Action::"SealFinding",
    resource is Finding
)
when {
    principal.key_rotation_age_days > 90
};

// F7 — Refuse direct DML on findings from non-lane-runner roles
forbid (
    principal,
    action in [Action::"InsertFinding", Action::"UpdateFinding", Action::"DeleteFinding"],
    resource is Finding
)
unless {
    principal in LaneRunner ||
    (principal in role::"evidence-emitter" && action == Action::"InsertFinding")
};

// =============================================================================
// SCHEMA HINTS
// =============================================================================
//
// Entities:
//   - LaneRunner { runner_id: String, registered_lanes: Set<String>,
//                  severity_for(lane: String): String }
//   - WebhookPayload { signature_valid: Boolean, delivery_id_replay: Boolean,
//                      source_ip: String }
//   - RepoFile { path: String }
//   - MatrixJob { runner_count: Long, lane_id: String }
//   - RulePack { via_pr: Boolean }
//
// Actions: WriteFinding, WriteLaneRun, QueryAdmissionVerdict, DispatchPullRequestOpened,
//          ReadPerMicroserviceSource, WriteAggregationIndex, DispatchMatrixJob,
//          SealFinding, InsertFinding, UpdateFinding, DeleteFinding, WriteRulePack

// =============================================================================
// FRAMEWORK MAPPING
// =============================================================================
//   P1/F1 → SLSA Build L3 (build provenance); ISO 27001 A.8.7
//   P3 → ADR-0111 merge-queue projected-state
//   P4/F5 → OWASP API01 broken-authentication; ISO 27001 A.8.20
//   F2 → ADR-0133 axis-5 honesty principle (lane-execution.md Invariant 5)
//   F3 → SOC 2 CC8.1; ISO 27001 A.5.34
//   F4 → T-E-03 mitigation
//   F6 → ISO 27001 A.8.24 cryptographic key management
//   F7 → SOC 2 CC6.1; defence-in-depth
