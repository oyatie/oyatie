-- oya-data outbox events: Postgres/RLS durable schema for the CDC
-- change-stream (ADR-0510 transitional impl of the data-outbox-kernel
-- ChangeStreamSource port; ADR-0536 D-10 change streams / D-13 messaging).
--
-- This is the table the kernel's INSERT_OUTBOX_EVENT_SQL targets
-- (oya_data_outbox.outbox_events) and the table the transitional
-- SqlxChangeStreamSource polls. The outbox row commits in the SAME port
-- WriteBatch transaction as the business rows it announces (transactional
-- outbox), so an at-least-once relay over this CDC poll yields
-- effectively-once delivery.
--
-- CDC STREAM ORDER: the W5 engine-native changefeed orders records by a
-- monotone resolved-offset cursor; the transitional adapter synthesizes the
-- same strict total order DB-side from `commit_logical`, a global
-- GENERATED ALWAYS AS IDENTITY (bigint) sequence. A single global IDENTITY is
-- already a strict, unique, monotone total order over committed rows — so it
-- IS the CDC stream position the poll's checkpoint advances over (the kernel
-- StreamPosition), sufficient on its own. `commit_wall_nanos` defaults from
-- clock_timestamp() and is the informational physical commit instant ONLY: it
-- is deliberately NOT in the ORDER BY / WHERE / index ordering, because
-- clock_timestamp() is non-monotone (NTP step-back or a long transaction can
-- emit a later row with a smaller wall) and using it as the ordering key would
-- silently skip a later row past the strictly-after filter (under-delivery,
-- violating outbox at-least-once). The bigint sequence is carried WITHOUT
-- narrowing as the kernel's u64 StreamPosition (never an HLC u32 logical tie
-- counter, which would wrap a global sequence past ~4.3B rows).
--
-- Tenant isolation is enforced by ENABLE + FORCE ROW LEVEL SECURITY under TWO
-- policies, keyed on the canonical session GUC
-- current_setting('oyatie.tenant_id', true) — the SAME GUC the shared command
-- kernel sets via `SELECT set_config('oyatie.tenant_id', $1, true)` before any
-- tenant-scoped statement (libs/oya-shared-postgres-command-kernel
-- SET_LOCAL_TENANT_SQL):
--
--   1. A PERMISSIVE policy (FOR ALL) that ADMITS a row only when its tenant_id
--      equals the session GUC (USING + WITH CHECK). Postgres requires at least
--      one PERMISSIVE policy to admit a row to a forced-RLS table. With none,
--      the table is deny-all (no permissive grant), even for the row's own
--      tenant — a RESTRICTIVE-only policy set is deny-all.
--   2. A RESTRICTIVE policy (FOR ALL) that hard-DENIES any access when the GUC
--      is unset or empty, so a missing per-tx SET_LOCAL_TENANT_SQL can never
--      fall back to an open scan. RESTRICTIVE intersects the permissive grant —
--      a row is visible only if the permissive policy admits it AND every
--      restrictive policy admits it.
--
-- The runtime role MUST NOT carry BYPASSRLS or RLS would be silently skipped.
-- A CHECK (tenant_id <> '') on the tenant_id column is defense-in-depth so a
-- blank tenant can never be persisted (and the empty-GUC restrictive guard can
-- never be satisfied by a blank stored value). Producer retries are a no-op via
-- ON CONFLICT (tenant_id, idempotency_key) DO NOTHING (the kernel insert SQL).
CREATE SCHEMA IF NOT EXISTS oya_data_outbox;

CREATE TABLE IF NOT EXISTS oya_data_outbox.outbox_events (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    event_id text NOT NULL,
    event_kind text NOT NULL,
    aggregate_id text NOT NULL,
    schema_version text NOT NULL,
    idempotency_key text NOT NULL,
    payload bytea NOT NULL,
    commit_wall_nanos bigint NOT NULL DEFAULT (extract(epoch from clock_timestamp()) * 1000000000)::bigint,
    commit_logical bigint NOT NULL GENERATED ALWAYS AS IDENTITY,
    PRIMARY KEY (tenant_id, idempotency_key)
);
ALTER TABLE oya_data_outbox.outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE oya_data_outbox.outbox_events FORCE ROW LEVEL SECURITY;
CREATE POLICY outbox_events_tenant_isolation ON oya_data_outbox.outbox_events AS PERMISSIVE FOR ALL TO oya_data_outbox_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
CREATE POLICY outbox_events_require_tenant_guc ON oya_data_outbox.outbox_events AS RESTRICTIVE FOR ALL TO oya_data_outbox_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');

-- Table privileges for the RLS-subject runtime role provisioned by
-- 0000_runtime_role.sql: the CDC adapter only polls (SELECT) and producers only
-- append (INSERT); outbox rows are immutable, so no UPDATE/DELETE is granted.
GRANT SELECT, INSERT ON oya_data_outbox.outbox_events TO oya_data_outbox_runtime;

-- The CDC poll scans strictly-after the commit_logical stream-position
-- checkpoint, tenant-scoped, ordered solely by commit_logical (the monotone
-- sequence IS the stream order); this index serves exactly that access path.
CREATE INDEX IF NOT EXISTS outbox_events_commit_order
    ON oya_data_outbox.outbox_events (tenant_id, commit_logical);

COMMENT ON TABLE oya_data_outbox.outbox_events IS 'oya-data transactional-outbox / CDC change-stream events; tenant-scoped under oyatie.tenant_id, ordered by the monotone commit_logical stream position (commit_wall_nanos is informational only).';
