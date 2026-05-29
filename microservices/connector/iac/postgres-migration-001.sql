-- connect schema migration 001
-- Binding ADR: ADR-0244 (tenant scoping), ADR-0263 (audit events)
-- Every table carries tenant_id (UUIDv7). RLS enforced via tenancy µservice.
-- Hyperscaler precedent: Stripe per-account data isolation; Salesforce multi-tenant schema.

BEGIN;

-- Extension: pgcrypto for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ─── oauth_grants ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS connector.oauth_grants (
    grant_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,           -- ADR-0244 §D-2 UUIDv7
    connector_name    TEXT NOT NULL,
    principal_id      UUID NOT NULL,
    scopes            TEXT[] NOT NULL DEFAULT '{}',
    issued_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at        TIMESTAMPTZ,
    refresh_token_ref TEXT NOT NULL,           -- OpenBao SecretReference; never raw token
    access_token_ref  TEXT,                    -- short-lived; written by sidecar
    status            TEXT NOT NULL DEFAULT 'active'
                         CHECK (status IN ('active','revoked','expired','soft_disabled')),
    revoked_at        TIMESTAMPTZ,
    revocation_reason TEXT,
    created_by        UUID NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_oauth_grants_tenant_connector
    ON connector.oauth_grants (tenant_id, connector_name)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS idx_oauth_grants_expires_at
    ON connector.oauth_grants (expires_at)
    WHERE status = 'active';

-- RLS: tenant isolation enforced by generated JWT claim (set by tenancy µservice)
ALTER TABLE connector.oauth_grants ENABLE ROW LEVEL SECURITY;
CREATE POLICY oauth_grants_tenant_isolation ON connector.oauth_grants
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- ─── webhook_endpoints ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS connect.webhook_endpoints (
    endpoint_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            UUID NOT NULL,
    connector_name       TEXT NOT NULL,
    endpoint_url         TEXT NOT NULL,
    signing_secret_ref   TEXT NOT NULL,        -- OpenBao SecretReference
    signing_algorithm    TEXT NOT NULL DEFAULT 'hmac-sha256',
    registered_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    deregistered_at      TIMESTAMPTZ,
    status               TEXT NOT NULL DEFAULT 'active'
                            CHECK (status IN ('active','deregistered')),
    event_types          TEXT[] NOT NULL DEFAULT '{}',
    idempotency_window_s INTEGER NOT NULL DEFAULT 300
);

CREATE INDEX IF NOT EXISTS idx_webhook_endpoints_tenant_connector
    ON connect.webhook_endpoints (tenant_id, connector_name)
    WHERE status = 'active';

ALTER TABLE connect.webhook_endpoints ENABLE ROW LEVEL SECURITY;
CREATE POLICY webhook_endpoints_tenant_isolation ON connect.webhook_endpoints
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- ─── dlq_entries ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS connector.dlq_entries (
    entry_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,
    wiring_id         UUID,
    connector_name    TEXT NOT NULL,
    action_name       TEXT NOT NULL,
    payload_digest    TEXT NOT NULL,           -- SHA-256 of original payload; not PII
    error_class       TEXT NOT NULL,
    error_message     TEXT,
    last_tried_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    retry_count       INTEGER NOT NULL DEFAULT 0,
    next_retry_at     TIMESTAMPTZ,
    status            TEXT NOT NULL DEFAULT 'pending'
                         CHECK (status IN ('pending','replaying','success','abandoned','quarantined')),
    quarantine_reason TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_dlq_entries_tenant_status
    ON connector.dlq_entries (tenant_id, status, next_retry_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_dlq_entries_connector
    ON connector.dlq_entries (connector_name, status);

ALTER TABLE connector.dlq_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY dlq_entries_tenant_isolation ON connector.dlq_entries
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- ─── idempotency_keys ─────────────────────────────────────────────────────────
-- Per-tenant idempotency window for webhook deduplication
CREATE TABLE IF NOT EXISTS connect.idempotency_keys (
    key_hash      TEXT NOT NULL,               -- SHA-256 of idempotency_key + tenant_id
    tenant_id     UUID NOT NULL,
    connector_name TEXT NOT NULL,
    received_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at    TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (key_hash, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires
    ON connect.idempotency_keys (expires_at);

-- Auto-purge expired idempotency keys (background job; not RLS-needed)

COMMIT;
