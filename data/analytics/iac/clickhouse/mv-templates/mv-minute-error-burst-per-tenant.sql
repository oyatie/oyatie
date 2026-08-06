-- MV template — anomaly window for error-burst detection.
-- Authority: ADR-AN-005-materialized-view-cadence cadence L1, IP-005.

CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_minute_error_burst_per_tenant
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.error_burst_minute
AS
SELECT
    toStartOfMinute(emitted_at) AS minute,
    tenant_id,
    count() AS error_count
FROM oya_events_kafka_source
WHERE event_type = 'workflow.failed' AND tenant_id = '${tid}'
GROUP BY minute, tenant_id
HAVING error_count > 100;

-- Target table.
CREATE TABLE IF NOT EXISTS tenant_${tid}.error_burst_minute
ON CLUSTER analytics-clickhouse-1
(
    minute DateTime,
    tenant_id String,
    error_count UInt64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(minute)
ORDER BY (tenant_id, minute)
TTL minute + INTERVAL 7 DAY DELETE;
