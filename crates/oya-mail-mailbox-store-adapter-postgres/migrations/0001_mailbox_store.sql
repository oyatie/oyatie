BEGIN;

CREATE SCHEMA IF NOT EXISTS mail_mailbox_store;
CREATE EXTENSION IF NOT EXISTS citus;

CREATE TABLE IF NOT EXISTS mail_mailbox_store.mail_messages (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'mail.message.submitted',
  mailbox_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  subject_ref TEXT NOT NULL,
  envelope_ref TEXT NOT NULL,
  retention_policy_id TEXT NOT NULL,
  dmarc_action TEXT NOT NULL CHECK (dmarc_action IN ('accept', 'quarantine')),
  policy_decision_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, mailbox_id, message_id),
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS mail_mailbox_store.mail_submission_receipts (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'mail.submission.receipt',
  idempotency_key TEXT NOT NULL,
  mailbox_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  dmarc_action TEXT NOT NULL CHECK (dmarc_action IN ('accept', 'quarantine')),
  audit_correlation_id TEXT NOT NULL,
  policy_decision_ref TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, idempotency_key)
);


CREATE TABLE IF NOT EXISTS mail_mailbox_store.protocol_outbox_events (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'mail.protocol.outbox',
  service_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  event_kind TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  asyncapi_operation_id TEXT NOT NULL,
  asyncapi_channel_address TEXT NOT NULL,
  asyncapi_message_name TEXT NOT NULL,
  proto_package TEXT NOT NULL,
  proto_service TEXT NOT NULL,
  proto_rpc TEXT NOT NULL,
  schema_version TEXT NOT NULL,
  idempotency_key TEXT,
  policy_decision_ref TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  dispatch_state TEXT NOT NULL DEFAULT 'pending' CHECK (dispatch_state IN ('pending', 'publishing', 'published', 'dead_letter')),
  attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, event_id),
  UNIQUE (tenant_id, idempotency_key, event_kind)
);

SELECT create_distributed_table('mail_mailbox_store.mail_messages', 'tenant_id', colocate_with => 'none');
SELECT create_distributed_table('mail_mailbox_store.mail_submission_receipts', 'tenant_id', colocate_with => 'mail_mailbox_store.mail_messages');
SELECT create_distributed_table('mail_mailbox_store.protocol_outbox_events', 'tenant_id', colocate_with => 'mail_mailbox_store.mail_messages');

ALTER TABLE mail_mailbox_store.mail_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE mail_mailbox_store.mail_messages FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS mail_messages_tenant_scope ON mail_mailbox_store.mail_messages;
CREATE POLICY mail_messages_tenant_scope
  ON mail_mailbox_store.mail_messages
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE mail_mailbox_store.mail_submission_receipts ENABLE ROW LEVEL SECURITY;
ALTER TABLE mail_mailbox_store.mail_submission_receipts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS mail_submission_receipts_tenant_scope ON mail_mailbox_store.mail_submission_receipts;
CREATE POLICY mail_submission_receipts_tenant_scope
  ON mail_mailbox_store.mail_submission_receipts
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE mail_mailbox_store.protocol_outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE mail_mailbox_store.protocol_outbox_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS mail_protocol_outbox_events_tenant_scope ON mail_mailbox_store.protocol_outbox_events;
CREATE POLICY mail_protocol_outbox_events_tenant_scope
  ON mail_mailbox_store.protocol_outbox_events
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

CREATE INDEX IF NOT EXISTS mail_messages_mailbox_submitted_idx
  ON mail_mailbox_store.mail_messages (tenant_id, mailbox_id, submitted_at DESC);
CREATE INDEX IF NOT EXISTS mail_messages_shard_idx
  ON mail_mailbox_store.mail_messages (tenant_id, home_cell, shard_key);
CREATE INDEX IF NOT EXISTS mail_protocol_outbox_pending_idx
  ON mail_mailbox_store.protocol_outbox_events (tenant_id, dispatch_state, created_at);

COMMIT;
