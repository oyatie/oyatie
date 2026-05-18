---
dashboard: brownout-degradation
authored: 2026-05-18
canonical_authority: ADR-0176
related_specs:
  - /specs/brownout-degradation-signal.json
related_adrs:
  - ADR-0176
status: dashboard-schema
owner_team: ops-sre-reliability
---

# Observability dashboard — brown-out + degradation

## Purpose

Visualizes per-µservice + per-cell degradation class and state
transition history per ADR-0176.

## Panels

### Panel 1: portfolio degradation heatmap

- Type: heatmap.
- X axis: µservice (32 entries).
- Y axis: cell_id (top 50 cells by traffic).
- Cell color: max(`oya_degradation_class`) over last 5 minutes
  (0 nominal green → 3 outage red).

### Panel 2: class transitions (last 24h)

- Type: state-transition table from audit-chain query.
- Columns: timestamp, microservice, cell_id, from_class, to_class,
  contributing_factors.
- Source: audit-chain query for `DegradationStateChange` rows.

### Panel 3: per-µservice degradation duration

- Type: stacked bar chart.
- X axis: µservice.
- Y axis: cumulative minutes in each class over last 24h.
- Series: `degraded`, `brownout`, `outage`.

### Panel 4: mesh retry budget impact

- Type: line chart.
- X axis: time.
- Y axis: mesh outbound retry-attempt rate.
- Series: per `oya-degradation-class` of the downstream.

### Panel 5: SLO burn rate vs degradation class

- Type: scatter.
- X axis: max SLO burn rate (across the µservice's SLOs).
- Y axis: degradation class (0..3).
- Annotation: expected relationship line (burn 1.0 ↔ nominal,
  burn 14 ↔ outage).

## Alerts

```yaml
groups:
  - name: brownout-degradation
    rules:
      - alert: MicroserviceInBrownout
        expr: oya_degradation_class >= 2
        for: 5m
        labels:
          severity: SEV-3
        annotations:
          summary: "{{ $labels.microservice }} in {{ $labels.cell_id }} is brown-out (class >= 2)"

      - alert: MicroserviceInOutage
        expr: oya_degradation_class >= 3
        for: 1m
        labels:
          severity: SEV-2

      - alert: ManyMicroservicesDegraded
        expr: count(oya_degradation_class >= 1) > 10
        for: 5m
        labels:
          severity: SEV-2

      - alert: DependencyBrownoutCascade
        expr: |
          (count by (cell_id) (oya_degradation_class{microservice="audit-chain"} >= 2)) > 0
          AND (count by (cell_id) (oya_degradation_class >= 1) > 5)
        for: 5m
        labels:
          severity: SEV-1
        annotations:
          summary: "Cell {{ $labels.cell_id }} cascade — audit-chain brown-out AND > 5 µservices degraded"
```

## Owners

- Dashboard owner: ops-sre-reliability.
- Per-cell alert routing: ops-dr-capacity (the cell owner).
- Per-µservice alert routing: the µservice's `owner_team`.
