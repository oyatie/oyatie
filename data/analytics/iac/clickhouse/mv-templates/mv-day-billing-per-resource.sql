-- MV template — daily billing rollup per resource type.
-- Authority: ADR-AN-005-materialized-view-cadence cadence L4, IP-009.

CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_day_billing_per_resource
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.billing_day
AS
SELECT
    toDate(emitted_at) AS day,
    tenant_id,
    resource_type,
    sumState(increment) AS count_state
FROM oya_usage_counters_kafka_source
WHERE tenant_id = '${tid}'
GROUP BY day, tenant_id, resource_type;

-- Target table.
CREATE TABLE IF NOT EXISTS tenant_${tid}.billing_day
ON CLUSTER analytics-clickhouse-1
(
    day Date,
    tenant_id String,
    resource_type String,
    count_state AggregateFunction(sum, Int64)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(day)
ORDER BY (tenant_id, resource_type, day)
TTL day + INTERVAL 30 DAY TO DISK 's3_cold',
    day + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
