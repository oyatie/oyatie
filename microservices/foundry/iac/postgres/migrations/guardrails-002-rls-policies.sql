-- foundry-guardrails Row-Level Security policies
-- Per policy/tenant-isolation.md TI-01 (RLS on every tenant-scoped table).
-- Per Bominal ADR-0028 (data-class enforcement).

BEGIN;

SET search_path TO foundry_guardrails;

-- =============================================================================
-- Enable RLS on every tenant-scoped table
-- =============================================================================

ALTER TABLE rule_definitions             ENABLE ROW LEVEL SECURITY;
ALTER TABLE cedar_fragments              ENABLE ROW LEVEL SECURITY;
ALTER TABLE false_positive_escalations   ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_overlay_registry      ENABLE ROW LEVEL SECURITY;
-- classifier_model_versions: NOT tenant-scoped (pack-scoped only); no RLS.
-- audit_mutation_log: append-only by trigger; reads scoped by pack via app layer.

-- Default-deny RLS posture: refuse any read or write that doesn't match
-- the application's current_setting('app.tenant_id') OR system-admin role.

-- =============================================================================
-- rule_definitions RLS
-- =============================================================================

CREATE POLICY rule_definitions_tenant_scope
  ON rule_definitions
  FOR ALL
  USING (
    -- pack-default rules (tenant_id IS NULL) visible to all tenants in pack
    (tenant_id IS NULL AND pack = current_setting('app.pack', true))
    -- OR per-tenant rules visible only to that tenant
    OR (tenant_id = current_setting('app.tenant_id', true))
    -- OR system-admin (rule-store-writer SA) sees all
    OR current_setting('app.role', true) = 'rule_store_writer'
  );

-- =============================================================================
-- cedar_fragments RLS (same shape as rule_definitions)
-- =============================================================================

CREATE POLICY cedar_fragments_tenant_scope
  ON cedar_fragments
  FOR ALL
  USING (
    (tenant_id IS NULL AND pack = current_setting('app.pack', true))
    OR (tenant_id = current_setting('app.tenant_id', true))
    OR current_setting('app.role', true) = 'rule_store_writer'
  );

-- =============================================================================
-- false_positive_escalations RLS
-- =============================================================================
-- Tenants see only their own; rule-author SA sees all.

CREATE POLICY fp_escalations_tenant_scope
  ON false_positive_escalations
  FOR ALL
  USING (
    tenant_id = current_setting('app.tenant_id', true)
    OR current_setting('app.role', true) IN ('rule_store_writer', 'rule_author_dashboard')
  );

-- =============================================================================
-- tenant_overlay_registry RLS
-- =============================================================================

CREATE POLICY tenant_overlay_registry_tenant_scope
  ON tenant_overlay_registry
  FOR ALL
  USING (
    tenant_id = current_setting('app.tenant_id', true)
    OR current_setting('app.role', true) = 'rule_store_writer'
  );

-- =============================================================================
-- Force RLS (no bypass even for table-owner)
-- =============================================================================

ALTER TABLE rule_definitions             FORCE ROW LEVEL SECURITY;
ALTER TABLE cedar_fragments              FORCE ROW LEVEL SECURITY;
ALTER TABLE false_positive_escalations   FORCE ROW LEVEL SECURITY;
ALTER TABLE tenant_overlay_registry      FORCE ROW LEVEL SECURITY;

COMMIT;
