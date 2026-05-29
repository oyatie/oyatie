// audit-chain µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous read of published root manifests + public-key manifests
//          and the anonymous proof-verification endpoint. NO anonymous read of any
//          tenant-attributable AuditEvent payload, ever.
// Owner:   ops-security + axis-audit-chain
// References: threat-model.md T-I-04 (anonymous probe pattern)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — Anonymous read of published root manifests
// =============================================================================
//
// The GitHub-pinned + S3-mirrored evidence/audit-chain-roots/<pack>/<epoch>.json
// manifests are publicly readable. They contain only per-period (root_hash,
// signature, signer_key_fp, period_id, prior_root_hash) tuples — no tenant payload.
//
// Reason for public read: transparency. Anyone can independently verify the chain
// is well-formed without needing oyatie trust.

permit (
  principal == User::"anonymous",
  action == Action::"audit_read_root_manifest",
  resource in PublicArtifact::?p
)
when {
  resource has artifact_type &&
  resource.artifact_type == "root-manifest"
};

// =============================================================================
// PERMIT 2 — Anonymous read of published public-key manifests
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"audit_read_key_manifest",
  resource in PublicArtifact::?p
)
when {
  resource has artifact_type &&
  resource.artifact_type == "key-manifest"
};

// =============================================================================
// PERMIT 3 — Anonymous verification endpoint (proof-only)
// =============================================================================
//
// Caller supplies (event_envelope, claimed_proof, claimed_root, claimed_signature,
// claimed_period_id, claimed_pack); endpoint returns { verified: bool, reason?: string }.
//
// The endpoint does NOT reveal the original event payload; the verification is
// pure-function over the caller's supplied artefacts.
//
// Rate-limited at WAF (10 req/s per IP); each call audit-emitted to detect probing.

permit (
  principal == User::"anonymous",
  action == Action::"audit_verify_anonymous",
  resource in PublicEndpoint::"audit-verification"
);

// =============================================================================
// FORBID — Anonymous never reads AuditEvent payloads
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"audit_query",
    Action::"audit_read_proof",
    Action::"audit_export_request",
    Action::"audit_read_seal_record"
  ],
  resource in AuditEvent::?e
);

// =============================================================================
// FORBID — Anonymous never writes anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"audit_emit",
    Action::"wal_write",
    Action::"seal_record_write",
    Action::"audit_redact",
    Action::"hsm_sign",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — Anonymous never reads tenant-attributable data
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id != "tenant:oya-aggregate"
};

// =============================================================================
// FORBID — Anonymous never reads SECRET
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
