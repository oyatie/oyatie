-- MV template — monthly billing rollup, chained from daily.
-- Authority: ADR-AN-005-materialized-view-cadence cadence L5 (chain depth 2; max), IP-009.

CREATE MATERIALIZED VIEW IF NOT EXISTS tenant_${tid}.mv_month_billing_per_resource
ON CLUSTER analytics-clickhouse-1
TO tenant_${tid}.billing_month
AS
SELECT
    toStartOfMonth(day) AS month,
    tenant_id,
    resource_type,
    sumState(sumMerge(count_state)) AS count_state
FROM tenant_${tid}.billing_day
GROUP BY month, tenant_id, resource_type;

-- Target table.
CREATE TABLE IF NOT EXISTS tenant_${tid}.billing_month
ON CLUSTER analytics-clickhouse-1
(
    month Date,
    tenant_id String,
    resource_type String,
    count_state AggregateFunction(sum, Int64)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYY(month)
ORDER BY (tenant_id, resource_type, month)
TTL month + INTERVAL 30 DAY TO DISK 's3_cold',
    month + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
