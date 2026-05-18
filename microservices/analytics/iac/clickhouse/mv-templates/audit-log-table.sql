-- Per-tenant audit-log table (not an MV — raw insertion target from Kafka engine).
-- Authority: IP-008, ADR-AN-001-ttl-policy, ADR-AN-002-partition-strategy.

CREATE TABLE IF NOT EXISTS tenant_${tid}.audit_events
ON CLUSTER analytics-clickhouse-1
(
    event_id UUID,
    emitted_at DateTime64(3, 'UTC'),
    tenant_id String,
    axis String,
    event_type String,
    principal_id String,
    evidence_ref String,
    payload_hash FixedString(64)
)
ENGINE = ReplacingMergeTree(emitted_at)
PARTITION BY toYYYYMM(emitted_at)
ORDER BY (tenant_id, axis, emitted_at, event_id)
TTL emitted_at + INTERVAL 90 DAY TO DISK 's3_cold',
    emitted_at + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
