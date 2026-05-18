-- foundry-guardrails Postgres — additional indexes for hot-path queries
-- Per capacity-model.md §"Postgres rule-store sizing" + access-pattern analysis.

BEGIN;

SET search_path TO foundry_guardrails;

-- =============================================================================
-- Hot path 1: rule fetch for (pack, tenant, category) on every invocation
-- =============================================================================
-- Covered by rule_definitions_pack_tenant_idx + rule_definitions_status_idx
-- (001-init-schema.sql). Add covering index for the "all enforce rules for
-- (pack, tenant)" hot read path.

CREATE INDEX IF NOT EXISTS rule_definitions_enforce_lookup_idx
  ON rule_definitions (pack, tenant_id, category, status)
  INCLUDE (rule_id, version, threshold)
  WHERE status = 'enforce';

-- =============================================================================
-- Hot path 2: Cedar fragment composition for (pack, tenant)
-- =============================================================================

CREATE INDEX IF NOT EXISTS cedar_fragments_enforce_lookup_idx
  ON cedar_fragments (pack, tenant_id, status)
  INCLUDE (fragment_id, version, fragment_sha)
  WHERE status = 'enforce';

-- =============================================================================
-- Hot path 3: FP budget consumption count per (tenant, month)
-- =============================================================================

CREATE INDEX IF NOT EXISTS fp_escalations_tenant_month_idx
  ON false_positive_escalations (tenant_id, date_trunc('month', occurred_at));

-- =============================================================================
-- Hot path 4: active classifier-model version per (model_id, pack)
-- =============================================================================

CREATE INDEX IF NOT EXISTS classifier_model_active_idx
  ON classifier_model_versions (model_id, pack, status)
  INCLUDE (version, sha, cosign_signature_sha)
  WHERE status = 'enforce';

-- =============================================================================
-- Hot path 5: audit-mutation log recent reads per (table_name, pack)
-- =============================================================================

CREATE INDEX IF NOT EXISTS audit_mutation_log_recent_idx
  ON audit_mutation_log (table_name, pack, occurred_at DESC);

-- =============================================================================
-- Optimisation: cluster rule_definitions by (pack, tenant_id) for cache locality
-- =============================================================================

CLUSTER rule_definitions USING rule_definitions_pack_tenant_idx;

-- =============================================================================
-- Statistics targets for query planner (rule_definitions sees high read QPS)
-- =============================================================================

ALTER TABLE rule_definitions ALTER COLUMN pack SET STATISTICS 1000;
ALTER TABLE rule_definitions ALTER COLUMN tenant_id SET STATISTICS 1000;
ALTER TABLE rule_definitions ALTER COLUMN category SET STATISTICS 1000;

ANALYZE rule_definitions;
ANALYZE cedar_fragments;
ANALYZE false_positive_escalations;
ANALYZE classifier_model_versions;

COMMIT;
