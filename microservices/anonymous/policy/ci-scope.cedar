// anonymous µservice — CI-scope policy fragment
// Cedar v4.2 LTS; default-deny.
//
// Purpose: scope what GitHub Actions CI runners can read/write inside the
//          anonymous µservice's domain. Runners may run validators, lint
//          gates, and ledger appends — but NEVER read user content.
// Owner:   axis-anonymous + governance
// Audit:   every CI action emits oya_anonymous_ci_action_total{action,verdict}

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — CI runner reads source code + contracts + manifests
// =============================================================================

permit (
  principal in CIRunner::?p,
  action in [
    Action::"read_source",
    Action::"read_contracts",
    Action::"read_manifests",
    Action::"read_catalog",
    Action::"read_policy_fragments"
  ],
  resource in RepositoryArtifact::?r
)
when {
  principal has ci_lane &&
  resource has microservice &&
  resource.microservice == "anonymous"
};

// =============================================================================
// PERMIT — CI runner appends to JSONL ledgers (union-merge driver per
// .gitattributes)
// =============================================================================

permit (
  principal in CIRunner::?p,
  action == Action::"append_jsonl_ledger",
  resource in JsonlLedger::?r
)
when {
  principal has ci_lane &&
  resource has ledger_kind &&
  resource.ledger_kind in [
    "promotion-eligibility",
    "audit-chain-anonymous",
    "transparency-report-staging"
  ]
};

// =============================================================================
// PERMIT — CI runner reads observability SLO manifests
// =============================================================================

permit (
  principal in CIRunner::?p,
  action == Action::"read_openslo_manifest",
  resource in OpenSLOManifest::?r
)
when {
  resource has microservice &&
  resource.microservice == "anonymous"
};

// =============================================================================
// FORBID — CI runner CANNOT read user content (I1 protection)
// =============================================================================

forbid (
  principal in CIRunner::?p,
  action in [
    Action::"read_post_body",
    Action::"read_comment_body",
    Action::"read_dm_ciphertext",
    Action::"read_attestation_credential",
    Action::"read_blinded_commitment_seed"
  ],
  resource
);

// =============================================================================
// FORBID — CI runner CANNOT execute legal-process disclosure flows
// =============================================================================

forbid (
  principal in CIRunner::?p,
  action in [
    Action::"execute_legal_process_disclosure",
    Action::"sign_dual_control_approval",
    Action::"reveal_post_author"
  ],
  resource
);
