BEGIN;

CREATE SCHEMA IF NOT EXISTS social_post_composition;
CREATE EXTENSION IF NOT EXISTS citus;

CREATE TABLE IF NOT EXISTS social_post_composition.posts (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'social.post.created',
  post_id TEXT NOT NULL,
  creator_ref TEXT NOT NULL,
  context_kind TEXT NOT NULL CHECK (context_kind IN ('personal', 'work')),
  artifact_kind TEXT NOT NULL CHECK (artifact_kind IN ('feed_post', 'story', 'collaborative_post')),
  media_refs TEXT[] NOT NULL DEFAULT '{}',
  workflow_consent_ref TEXT,
  policy_decision_ref TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  story_expires_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, post_id),
  UNIQUE (tenant_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS social_post_composition.story_purge_targets (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'social.story.purge_target',
  post_id TEXT NOT NULL,
  purge_target TEXT NOT NULL CHECK (purge_target IN ('cdn_object', 'search_index', 'ontology_node')),
  purge_after TIMESTAMPTZ NOT NULL,
  policy_decision_ref TEXT NOT NULL,
  audit_correlation_id TEXT NOT NULL,
  PRIMARY KEY (tenant_id, post_id, purge_target)
);


CREATE TABLE IF NOT EXISTS social_post_composition.protocol_outbox_events (
  tenant_id TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  shard_key TEXT NOT NULL,
  jurisdiction_code TEXT NOT NULL,
  audit_event_class TEXT NOT NULL DEFAULT 'social.protocol.outbox',
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

SELECT create_distributed_table('social_post_composition.posts', 'tenant_id', colocate_with => 'none');
SELECT create_distributed_table('social_post_composition.story_purge_targets', 'tenant_id', colocate_with => 'social_post_composition.posts');
SELECT create_distributed_table('social_post_composition.protocol_outbox_events', 'tenant_id', colocate_with => 'social_post_composition.posts');

ALTER TABLE social_post_composition.posts ENABLE ROW LEVEL SECURITY;
ALTER TABLE social_post_composition.posts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS social_posts_tenant_scope ON social_post_composition.posts;
CREATE POLICY social_posts_tenant_scope
  ON social_post_composition.posts
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE social_post_composition.story_purge_targets ENABLE ROW LEVEL SECURITY;
ALTER TABLE social_post_composition.story_purge_targets FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS social_story_purge_targets_tenant_scope ON social_post_composition.story_purge_targets;
CREATE POLICY social_story_purge_targets_tenant_scope
  ON social_post_composition.story_purge_targets
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

ALTER TABLE social_post_composition.protocol_outbox_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE social_post_composition.protocol_outbox_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS social_protocol_outbox_events_tenant_scope ON social_post_composition.protocol_outbox_events;
CREATE POLICY social_protocol_outbox_events_tenant_scope
  ON social_post_composition.protocol_outbox_events
  FOR ALL
  USING (tenant_id = current_setting('oyatie.tenant_id', true))
  WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));

CREATE INDEX IF NOT EXISTS social_posts_creator_created_idx
  ON social_post_composition.posts (tenant_id, creator_ref, created_at DESC);
CREATE INDEX IF NOT EXISTS social_posts_shard_idx
  ON social_post_composition.posts (tenant_id, home_cell, shard_key);
CREATE INDEX IF NOT EXISTS social_protocol_outbox_pending_idx
  ON social_post_composition.protocol_outbox_events (tenant_id, dispatch_state, created_at);

COMMIT;
