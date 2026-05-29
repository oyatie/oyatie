-- foundry-guardrails Postgres schema — initial migration
-- Per ADR-0131 + policy/tenant-isolation.md TI-01..TI-06.
-- All tenant-scoped tables enable Row-Level Security; mutation log is append-only.

BEGIN;

-- =============================================================================
-- Schema setup
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS foundry_guardrails;
SET search_path TO foundry_guardrails;

-- =============================================================================
-- Table: rule_definitions
-- =============================================================================
-- Per-pack + optional per-tenant rule overlay for content-safety + ai-slop + etc.

CREATE TABLE rule_definitions (
  rule_id          TEXT NOT NULL,
  version          INTEGER NOT NULL,
  category         TEXT NOT NULL CHECK (category IN (
                     'toxicity', 'self_harm', 'sexual', 'violence',
                     'minors', 'hate', 'weapons', 'illegal',
                     'pii', 'phi', 'jailbreak', 'ai_slop'
                   )),
  pack             TEXT NOT NULL CHECK (pack IN (
                     'pack-kr', 'pack-eu', 'pack-us', 'pack-us-healthcare',
                     'pack-jp', 'pack-sg', 'pack-au', 'pack-in',
                     'pack-br', 'pack-ae', 'pack-ksa'
                   )),
  tenant_id        TEXT,                                  -- NULL = pack-default
  threshold        DOUBLE PRECISION NOT NULL CHECK (threshold >= 0 AND threshold <= 1),
  status           TEXT NOT NULL CHECK (status IN ('shadow', 'enforce', 'sunsetted')),
  author_spiffe    TEXT NOT NULL,
  commit_sha       TEXT NOT NULL,
  pr_id            TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  signature        TEXT NOT NULL,                          -- Ed25519 author signature
  PRIMARY KEY (rule_id, version)
);

CREATE INDEX rule_definitions_pack_tenant_idx
  ON rule_definitions (pack, tenant_id, category);
CREATE INDEX rule_definitions_status_idx
  ON rule_definitions (status) WHERE status = 'enforce';

-- =============================================================================
-- Table: cedar_fragments
-- =============================================================================
-- Per-pack and per-tenant Cedar policy fragment registry.

CREATE TABLE cedar_fragments (
  fragment_id      TEXT NOT NULL,
  version          INTEGER NOT NULL,
  pack             TEXT NOT NULL,
  tenant_id        TEXT,                                  -- NULL = pack-default
  fragment_text    TEXT NOT NULL,
  fragment_sha     TEXT NOT NULL,
  status           TEXT NOT NULL CHECK (status IN ('shadow', 'enforce', 'sunsetted')),
  author_spiffe    TEXT NOT NULL,
  commit_sha       TEXT NOT NULL,
  pr_id            TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  signature        TEXT NOT NULL,
  PRIMARY KEY (fragment_id, version)
);

CREATE INDEX cedar_fragments_pack_tenant_idx
  ON cedar_fragments (pack, tenant_id);

-- =============================================================================
-- Table: audit_mutation_log (append-only)
-- =============================================================================

CREATE TABLE audit_mutation_log (
  log_id           BIGSERIAL PRIMARY KEY,
  table_name       TEXT NOT NULL,
  row_pk           JSONB NOT NULL,
  action           TEXT NOT NULL CHECK (action IN ('created', 'modified', 'sunsetted')),
  pack             TEXT NOT NULL,
  tenant_id        TEXT,
  prior_version    INTEGER,
  current_version  INTEGER NOT NULL,
  author_spiffe    TEXT NOT NULL,
  commit_sha       TEXT,
  pr_id            TEXT,
  occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  signature        TEXT NOT NULL
);

CREATE INDEX audit_mutation_log_table_idx
  ON audit_mutation_log (table_name, occurred_at DESC);
CREATE INDEX audit_mutation_log_pack_tenant_idx
  ON audit_mutation_log (pack, tenant_id, occurred_at DESC);

-- Append-only trigger: refuse UPDATE / DELETE on audit_mutation_log
CREATE OR REPLACE FUNCTION audit_mutation_log_append_only()
  RETURNS TRIGGER AS $$
BEGIN
  RAISE EXCEPTION 'audit_mutation_log is append-only; UPDATE/DELETE refused (TG_OP=%)', TG_OP;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_mutation_log_no_update
  BEFORE UPDATE ON audit_mutation_log
  FOR EACH ROW EXECUTE FUNCTION audit_mutation_log_append_only();

CREATE TRIGGER audit_mutation_log_no_delete
  BEFORE DELETE ON audit_mutation_log
  FOR EACH ROW EXECUTE FUNCTION audit_mutation_log_append_only();

-- =============================================================================
-- Table: classifier_model_versions
-- =============================================================================

CREATE TABLE classifier_model_versions (
  model_id              TEXT NOT NULL,
  version               TEXT NOT NULL,
  sha                   TEXT NOT NULL,
  cosign_signature_sha  TEXT NOT NULL,
  pack                  TEXT NOT NULL,
  status                TEXT NOT NULL CHECK (status IN ('shadow', 'enforce', 'sunsetted')),
  prior_version         TEXT,
  shadow_vs_enforce_delta DOUBLE PRECISION,
  deployed_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
  deployer_spiffe       TEXT NOT NULL,
  PRIMARY KEY (model_id, version)
);

CREATE INDEX classifier_model_versions_pack_status_idx
  ON classifier_model_versions (pack, status);

-- =============================================================================
-- Table: false_positive_escalations
-- =============================================================================

CREATE TABLE false_positive_escalations (
  escalation_id    TEXT PRIMARY KEY,                     -- ULID
  decision_id      TEXT NOT NULL,
  tenant_id        TEXT NOT NULL,
  pack             TEXT NOT NULL,
  reason           TEXT NOT NULL CHECK (length(reason) >= 10),
  budget_remaining INTEGER NOT NULL,
  occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  signature        TEXT NOT NULL
);

CREATE INDEX false_positive_escalations_tenant_idx
  ON false_positive_escalations (tenant_id, occurred_at DESC);
CREATE INDEX false_positive_escalations_pack_idx
  ON false_positive_escalations (pack, occurred_at DESC);

-- =============================================================================
-- Table: tenant_overlay_registry
-- =============================================================================

CREATE TABLE tenant_overlay_registry (
  overlay_id       TEXT NOT NULL,
  tenant_id        TEXT NOT NULL,
  pack             TEXT NOT NULL,
  fragment_ids     TEXT[] NOT NULL,        -- references cedar_fragments(fragment_id)
  status           TEXT NOT NULL CHECK (status IN ('active', 'sunsetted')),
  registered_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  signature        TEXT NOT NULL,
  PRIMARY KEY (overlay_id)
);

CREATE INDEX tenant_overlay_registry_tenant_idx
  ON tenant_overlay_registry (tenant_id, status);

-- =============================================================================
-- Application role
-- =============================================================================

CREATE ROLE foundry_guardrails_app NOLOGIN;
GRANT USAGE ON SCHEMA foundry_guardrails TO foundry_guardrails_app;
GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA foundry_guardrails TO foundry_guardrails_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA foundry_guardrails TO foundry_guardrails_app;
-- NO UPDATE/DELETE grant on rule_definitions / cedar_fragments / audit_mutation_log
-- (append-only via new-row inserts; status transitions via new rows)

COMMIT;

-- Next: 002-rls-policies.sql (Row-Level Security)
-- Next: 003-indexes.sql (additional indexes)
