BEGIN;

CREATE SCHEMA IF NOT EXISTS messenger_message_stream;
CREATE EXTENSION IF NOT EXISTS citus;

CREATE TABLE IF NOT EXISTS messenger_message_stream.messages (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'messenger.message.sent',
  channel_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  author_ref TEXT NOT NULL,
  envelope_ref TEXT NOT NULL,
  retention_policy_id TEXT NOT NULL,
  legal_hold_ids TEXT[] NOT NULL DEFAULT '{}',
  policy_decision_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, channel_id, message_id),
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS messenger_message_stream.message_receipts (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'messenger.message.receipt',
  idempotency_key TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  receipt_kind TEXT NOT NULL CHECK (receipt_kind IN ('created', 'deduplicated')),
  audit_correlation_id TEXT NOT NULL,
  policy_decision_ref TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, idempotency_key)
);


CREATE TABLE IF NOT EXISTS messenger_message_stream.protocol_outbox_events (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'messenger.protocol.outbox',
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

SELECT create_distributed_table('messenger_message_stream.messages', 'tenant_id', colocate_with => 'none');
SELECT create_distributed_table('messenger_message_stream.message_receipts', 'tenant_id', colocate_with => 'messenger_message_stream.messages');
SELECT create_distributed_table('messenger_message_stream.protocol_outbox_events', 'tenant_id', colocate_with => 'messenger_message_stream.messages');

ALTER TABLE messenger_message_stream.messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE messenger_message_stream.messages FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS messenger_messages_tenant_scope ON messenger_message_stream.messages;
CREATE POLICY messenger_messages_tenant_scope
  ON messenger_message_stream.messages
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE messenger_message_stream.message_receipts ENABLE ROW LEVEL SECURITY;
ALTER TABLE messenger_message_stream.message_receipts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS messenger_message_receipts_tenant_scope ON messenger_message_stream.message_receipts;
CREATE POLICY messenger_message_receipts_tenant_scope
  ON messenger_message_stream.message_receipts
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE messenger_message_stream.protocol_outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE messenger_message_stream.protocol_outbox_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS messenger_protocol_outbox_events_tenant_scope ON messenger_message_stream.protocol_outbox_events;
CREATE POLICY messenger_protocol_outbox_events_tenant_scope
  ON messenger_message_stream.protocol_outbox_events
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

CREATE INDEX IF NOT EXISTS messenger_messages_channel_created_idx
  ON messenger_message_stream.messages (tenant_id, channel_id, created_at DESC);
CREATE INDEX IF NOT EXISTS messenger_messages_shard_idx
  ON messenger_message_stream.messages (tenant_id, home_cell, shard_key);
CREATE INDEX IF NOT EXISTS messenger_protocol_outbox_pending_idx
  ON messenger_message_stream.protocol_outbox_events (tenant_id, dispatch_state, created_at);

COMMIT;
