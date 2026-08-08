-- microservices/cloud-secrets/migrations/0001_secret_references_policy_eval_opt_in.sql
-- Adds ADR-0244 §D-3 / ADR-0246-amendment / ADR-0257-amendment library-first opt-in columns
-- to the secret_references table.
--
-- Authority: ADR-0244 §D-3 (Wave-3-A cross-reference wiring, 2026-05-20)
--            ADR-0246-amendment §D-2 (library-first policy-eval network opt-in)
--            ADR-0257-amendment §D-2 (library-first ontology read-path network opt-in)
-- Binding ADR: ADR-0244, ADR-0246-amendment, ADR-0257-amendment
-- Migration class: ADD COLUMN (non-breaking; both columns default to FALSE / empty array)
-- Rollback: DROP COLUMN policy_evaluation_network_opt_in, policy_evaluation_network_opt_in_reasons

-- secret_references holds per-tenant, per-scope credential handles issued to caller µservices.
-- The two new columns allow a tenant's SecretReference configuration to declare that
-- the holder is opted-in to making live network calls to the policy-engine substrate
-- rather than relying solely on the library-first in-process cache, and record the
-- compliance / architectural reasons that necessitate the opt-in.
--
-- Column semantics:
--   policy_evaluation_network_opt_in BOOLEAN (default FALSE)
--     When TRUE, the caller µservice that resolves this SecretReference is permitted to
--     bypass library-first Cedar evaluation cache and make a direct network call to
--     the cell-local policy-engine evaluator for every Cedar decision in the scope of
--     this credential's use. This is the per-SecretReference network opt-in gate
--     mandated by ADR-0246-amendment §D-2 + ADR-0257-amendment §D-2.
--
--   policy_evaluation_network_opt_in_reasons TEXT[] (default '{}')
--     Human-readable + machine-parseable justification codes declaring WHY the opt-in
--     is required. CI lane oya-check-library-first-credential-sidecar (per ADR-0296)
--     rejects any SecretReference row with opt_in=TRUE and an empty reasons array.
--     Canonical reason codes (open enum; register new codes in
--     microservices/cloud-secrets/policy/opt-in-reasons.yaml):
--       'compliance_pack_requires_network_eval'   — active compliance pack mandates live eval
--       'freshness_floor_cannot_be_met_by_cache'  — SLA requires sub-second staleness guarantee
--       'meta_trust_root_attestation_required'    — ADR-0293 attested-fallback threshold exceeded
--       'audit_trail_requires_per_call_decision'  — regulator requires per-call ADR-0263 audit row
--       'sandbox_isolation'                       — sandbox SecretReference; caching suppressed
--       'byok_rotation_in_progress'               — key rotation window; avoid stale cached permits

ALTER TABLE cloud_secrets.secret_references
    ADD COLUMN IF NOT EXISTS policy_evaluation_network_opt_in
        BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS policy_evaluation_network_opt_in_reasons
        TEXT[] NOT NULL DEFAULT '{}';

COMMENT ON COLUMN cloud_secrets.secret_references.policy_evaluation_network_opt_in IS
    'When TRUE, the holder of this SecretReference is permitted to make live network calls '
    'to the cell-local policy-engine evaluator instead of relying on library-first cache. '
    'Per ADR-0246-amendment §D-2 + ADR-0257-amendment §D-2. Requires non-empty reasons array.';

COMMENT ON COLUMN cloud_secrets.secret_references.policy_evaluation_network_opt_in_reasons IS
    'Justification codes for the network opt-in. CI lane oya-check-library-first-credential-sidecar '
    'rejects opt_in=TRUE with empty reasons. Canonical codes in policy/opt-in-reasons.yaml. '
    'Per ADR-0296 §D-3 + ADR-0246-amendment §D-2.';

-- Index: fast lookup of all opted-in references per tenant for the policy-engine evaluator
-- to pre-warm its network-dispatch routing table.
CREATE INDEX IF NOT EXISTS idx_secret_references_policy_eval_opt_in
    ON cloud_secrets.secret_references (tenant_id, policy_evaluation_network_opt_in)
    WHERE policy_evaluation_network_opt_in = TRUE;

-- Constraint: network opt-in TRUE requires at least one reason code.
-- Enforced at application layer too (oya-check-library-first-credential-sidecar CI lane),
-- but belt-and-suspenders at the DB layer prevents silent opt-in with empty reasons.
ALTER TABLE cloud_secrets.secret_references
    ADD CONSTRAINT chk_secret_references_opt_in_reasons
    CHECK (
        policy_evaluation_network_opt_in = FALSE
        OR array_length(policy_evaluation_network_opt_in_reasons, 1) > 0
    );
