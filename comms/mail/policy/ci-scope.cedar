// mail µservice — CI-scope policy fragment
// Cedar v4 schema
//
// Purpose: grant CI runners + the oya-governance-promotion-readiness lane + LEAN gates
//          minimal-necessary read access to mail-specific metrics, lane outputs,
//          and OpenSLO manifests. No mailbox content access.
// Owner:   ops-security + axis-mail
// Audit:   every deny emits mail_ci_unauthorized_read_attempt_total
// References: threat-model.md T-S-02, T-E-02
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — slo-engine-worker reads mail SLI metrics (Mimir as oya-self)
// =============================================================================

permit (
  principal in ServiceAccount::"slo-engine-worker",
  action in [
    Action::"read_metrics",
    Action::"emit_metrics"
  ],
  resource in MailSli::?s
)
when {
  resource has metric_namespace &&
  resource.metric_namespace in ["oya_mail_*", "oya_promotion_mail_*", "oya_governance_mail_*"]
};

// =============================================================================
// PERMIT 2 — CI lane reads mail OpenSLO manifests (read-only)
// =============================================================================

permit (
  principal in CiLane::?l,
  action == Action::"read_openslo_manifests",
  resource in OpenSloManifest::?m
)
when {
  resource has microservice &&
  resource.microservice == "mail"
};

// =============================================================================
// PERMIT 3 — CI lane reads mail eligibility verdicts
// =============================================================================

permit (
  principal in CiLane::?l,
  action == Action::"read_eligibility_verdicts",
  resource in EligibilityVerdict::?v
)
when {
  resource has microservice &&
  resource.microservice == "mail"
};

// =============================================================================
// PERMIT 4 — LEAN lanes read mail-related Postgres / Cargo metadata
// =============================================================================
//
// LEAN lanes (oya-check-rls-policy-conformance, oya-check-dkim-key-rotation-conformance,
// oya-check-dual-context-cross-boundary, etc.) read mail-µservice schema + source.
// They do NOT read mail content.

permit (
  principal in CiLane::?l,
  action in [
    Action::"read_postgres_schema",
    Action::"read_cargo_metadata",
    Action::"read_source_code",
    Action::"read_helm_values"
  ],
  resource in MailRepoArtifact::?a
)
when {
  resource has path &&
  resource.path like "comms/mail/**"
};

// =============================================================================
// FORBID — CI lanes NEVER read mail content
// =============================================================================

forbid (
  principal in [CiLane::?l, ServiceAccount::?s],
  action in [
    Action::"read_mailbox",
    Action::"read_thread",
    Action::"read_message",
    Action::"read_mime_blob"
  ],
  resource
);
