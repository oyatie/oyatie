BEGIN;

CREATE SCHEMA IF NOT EXISTS community_post_store;
CREATE EXTENSION IF NOT EXISTS citus;

CREATE TABLE IF NOT EXISTS community_post_store.posts (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'community.post.created',
  space_id TEXT NOT NULL,
  thread_id TEXT NOT NULL,
  post_id TEXT NOT NULL,
  mode TEXT NOT NULL CHECK (mode IN ('reddit', 'teamblind', 'handshake', 'knowledge_base')),
  routine_display_ref TEXT NOT NULL,
  audit_author_ref TEXT NOT NULL,
  disclosure_policy_ref TEXT,
  body_ref TEXT NOT NULL,
  retention_policy_id TEXT NOT NULL,
  policy_decision_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, post_id),
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS community_post_store.votes (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'community.vote.cast',
  post_id TEXT NOT NULL,
  vote_id TEXT NOT NULL,
  voter_ref TEXT NOT NULL,
  direction TEXT NOT NULL CHECK (direction IN ('up', 'down')),
  policy_decision_ref TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, post_id, vote_id),
  UNIQUE (tenant_id, post_id, voter_ref)
);

CREATE TABLE IF NOT EXISTS community_post_store.moderation_actions (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'community.moderation.actioned',
  post_id TEXT NOT NULL,
  evidence_ref TEXT NOT NULL,
  policy_ref TEXT NOT NULL,
  verb TEXT NOT NULL CHECK (verb IN ('allow', 'hide', 'remove')),
  policy_decision_ref TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, post_id, evidence_ref)
);


CREATE TABLE IF NOT EXISTS community_post_store.protocol_outbox_events (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'community.protocol.outbox',
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

SELECT create_distributed_table('community_post_store.posts', 'tenant_id', colocate_with => 'none');
SELECT create_distributed_table('community_post_store.votes', 'tenant_id', colocate_with => 'community_post_store.posts');
SELECT create_distributed_table('community_post_store.moderation_actions', 'tenant_id', colocate_with => 'community_post_store.posts');
SELECT create_distributed_table('community_post_store.protocol_outbox_events', 'tenant_id', colocate_with => 'community_post_store.posts');

ALTER TABLE community_post_store.posts ENABLE ROW LEVEL SECURITY;
ALTER TABLE community_post_store.posts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS community_posts_tenant_scope ON community_post_store.posts;
CREATE POLICY community_posts_tenant_scope
  ON community_post_store.posts
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE community_post_store.votes ENABLE ROW LEVEL SECURITY;
ALTER TABLE community_post_store.votes FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS community_votes_tenant_scope ON community_post_store.votes;
CREATE POLICY community_votes_tenant_scope
  ON community_post_store.votes
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE community_post_store.moderation_actions ENABLE ROW LEVEL SECURITY;
ALTER TABLE community_post_store.moderation_actions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS community_moderation_actions_tenant_scope ON community_post_store.moderation_actions;
CREATE POLICY community_moderation_actions_tenant_scope
  ON community_post_store.moderation_actions
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE community_post_store.protocol_outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE community_post_store.protocol_outbox_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS community_protocol_outbox_events_tenant_scope ON community_post_store.protocol_outbox_events;
CREATE POLICY community_protocol_outbox_events_tenant_scope
  ON community_post_store.protocol_outbox_events
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

CREATE INDEX IF NOT EXISTS community_posts_space_created_idx
  ON community_post_store.posts (tenant_id, space_id, created_at DESC);
CREATE INDEX IF NOT EXISTS community_posts_shard_idx
  ON community_post_store.posts (tenant_id, home_cell, shard_key);
CREATE INDEX IF NOT EXISTS community_votes_tally_idx
  ON community_post_store.votes (tenant_id, post_id, direction);
CREATE INDEX IF NOT EXISTS community_protocol_outbox_pending_idx
  ON community_post_store.protocol_outbox_events (tenant_id, dispatch_state, created_at);

COMMIT;
