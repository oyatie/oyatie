---
dashboard: dr-business-continuity
authored: 2026-05-18
canonical_authority: ADR-0180
related_specs:
  - /specs/dr-business-continuity.json
related_adrs:
  - ADR-0180
status: dashboard-schema
owner_team: ops-dr-capacity
---

# Observability dashboard — DR + business-continuity

## Purpose

Visualizes per-µservice DR tier declaration, last-drill timestamp,
next-drill due date, and tabletop scenario history.

## Panels

### Panel 1: per-µservice DR tier summary

- Type: table.
- Columns: microservice, dr_tier, replication_shape, last_drill_at,
  last_drill_success, next_drill_due_at, days_overdue.
- Sort: days_overdue DESC.
- Source: query against `registry/dr/per-microservice-tier.yaml` joined
  with audit-chain `DrDrillReceipt` rows.

### Panel 2: drill cadence compliance heatmap

- Type: heatmap.
- X axis: time (last 12 months).
- Y axis: µservice.
- Cell color:
  - green = drill completed successfully within cadence window
  - yellow = drill completed but with findings
  - red = drill missed
  - grey = N/A (µservice didn't exist yet)

### Panel 3: tabletop scenario coverage

- Type: bar chart.
- X axis: scenario (7 scenarios from ADR-0180).
- Y axis: months since last tabletop.
- Annotation: cadence ceiling (annual = 12 months).

### Panel 4: cross-provider failover drill outcomes

- Type: stacked bar chart.
- X axis: quarter.
- Y axis: count.
- Series: succeeded / succeeded-with-findings / failed.
- Source: audit-chain query for cross-provider failover drill rows.

### Panel 5: replication lag (per T1/T2 µservice)

- Type: line chart.
- X axis: time.
- Y axis: replication lag (seconds).
- Series: per T1/T2 µservice.
- Threshold lines: T1 lag should be ≤ 0 (zero data loss); T2 lag ≤ 60s.

## Alerts

```yaml
groups:
  - name: dr-business-continuity
    rules:
      - alert: DrDrillOverdue
        expr: |
          (time() - oya_dr_last_drill_success_timestamp{tier=~"T1|T2"})
          > 2 * 90 * 86400
        for: 1d
        labels:
          severity: SEV-3
        annotations:
          summary: "{{ $labels.microservice }} drill > 2x cadence overdue"

      - alert: DrDrillFailureRateHigh
        expr: |
          sum by (microservice) (
            rate(oya_dr_drill_failures_total[90d])
            / rate(oya_dr_drill_attempts_total[90d])
          ) > 0.05
        for: 6h
        labels:
          severity: SEV-2

      - alert: ReplicationLagBreachT1
        expr: oya_dr_replication_lag_seconds{tier="T1"} > 1
        for: 1m
        labels:
          severity: SEV-2

      - alert: ReplicationLagBreachT2
        expr: oya_dr_replication_lag_seconds{tier="T2"} > 60
        for: 5m
        labels:
          severity: SEV-3
```

## Owners

- Dashboard owner: ops-dr-capacity.
- Quarterly review: ops-dr-capacity + ops-compliance + council-architecture.
