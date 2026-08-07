-- MV template — hourly workflow execution rollup per tenant.
-- Authority: ADR-AN-005-materialized-view-cadence, IP-005.
-- Convention: mv_${cadence}_${entity}_${dimension}; target ${entity}_${cadence}.
-- Renders per-tenant via the bootstrap controller at onboard time.

CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_hour_workflow_per_tenant
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.workflow_hour
AS
SELECT
    toStartOfHour(emitted_at) AS hour,
    tenant_id,
    countState() AS run_count,
    sumState(duration_ms) AS total_duration_ms,
    quantilesState(0.5, 0.95, 0.99)(duration_ms) AS duration_percentiles
FROM oya_events_kafka_source
WHERE event_type = 'workflow.executed' AND tenant_id = '${tid}'
GROUP BY hour, tenant_id;

-- Target table.
CREATE TABLE IF NOT EXISTS tenant_${tid}.workflow_hour
ON CLUSTER analytics-clickhouse-1
(
    hour DateTime,
    tenant_id String,
    run_count AggregateFunction(count, UInt64),
    total_duration_ms AggregateFunction(sum, UInt64),
    duration_percentiles AggregateFunction(quantiles(0.5, 0.95, 0.99), UInt32)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (tenant_id, hour)
TTL hour + INTERVAL 90 DAY TO DISK 's3_cold',
    hour + INTERVAL 365 DAY DELETE
SETTINGS storage_policy = 'hot_cold';
