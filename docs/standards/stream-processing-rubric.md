# Stream-Processing Rubric — When MV Suffices vs When Flink Justified

**Authority:** ADR-0195
**Status:** Canonical (2026-05-18)
**Owner:** council-architecture

This rubric translates ADR-0195's two-tier policy into a per-workload decision aid. Use it before introducing any stream-processing concern; cite this rubric in the per-µservice IP that adopts the chosen tier.

## TL;DR

- **Class A → ClickHouse Materialized Views** (default; ~95% of workloads).
- **Class B → Apache Flink** (escalation; requires ADR amendment).

## Decision tree

```
Q1. Does the workload need cross-stream join with state retention beyond a single source table?
   YES → Flink (class B)
   NO  → continue

Q2. Does the workload need exactly-once with external side effects (payments, external API calls)?
   YES → Flink (class B)
   NO  → continue

Q3. Does the workload need Complex Event Processing (multi-event temporal patterns like "A then B within 5min but not C")?
   YES → Flink (class B)
   NO  → continue

Q4. Does the workload need session windows (variable-length gaps)?
   YES → Flink (class B)
   NO  → continue

Q5. Does the workload need per-key state with custom merge logic across hours of state?
   YES → Flink (class B)
   NO  → continue

→ DEFAULT: ClickHouse Materialized Views (class A)
```

## Class A — ClickHouse MV examples

| Workload | MV pattern |
|---|---|
| Per-tenant rolling 1h workflow run counter | `AggregatingMergeTree` + `countState()` |
| Top-10 active tenants per hour | `topKState(10)` |
| p99 request latency per-µservice per-5min window | `quantilesState(0.99)` |
| Per-tenant error burst alert (>100 errors/min) | `AggregatingMergeTree` + `HAVING count > 100` |
| Per-tenant per-axis audit-event rollup | `AggregatingMergeTree` grouped by axis |
| Per-tenant per-day billing rollup | `SummingMergeTree` |

All sub-second freshness; in-OLAP; no separate cluster.

## Class B — Flink examples (each requires ADR amendment)

| Workload | Why MV insufficient |
|---|---|
| Real-time fraud detection joining payment stream with user history stream | Cross-stream join with hour+ state |
| Stream-to-payment-gateway with exactly-once | External side effect |
| "User did login, then changed password, then exported all data within 5min" CEP rule | Multi-event temporal pattern |
| User session activity with 30min idle threshold | Session window |
| Real-time ML feature pipeline with cross-event state | Custom per-key state |

When adopting Flink for a workload, the µservice's adopting IP cites the row above + opens an amendment-of-ADR-0195 with workload-specific Flink job DDL and operational runbook.

## Anti-pattern catalog

### "It feels like a stream so it must be Flink"

Reality: most stream needs are class A. Pre-aggregation in OLAP is cheaper and adequate.

### "MV can't do windowing"

Reality: ClickHouse `toStartOfHour`, `toStartOfMinute`, `toStartOfInterval` produce fixed-window tumbling buckets. Sliding-window aggregates are achievable via overlapping interval expressions.

### "MV doesn't have backpressure"

Reality: ClickHouse `PartsDelayInsert` system metric exposes ingest pressure; Pulsar consumer offsets provide source-side backpressure.

### "I need Materialize / RisingWave"

Reality (per ADR-0195 §"Materialize — DEFERRED"): re-evaluate at v1.0 GA + Apache-2.0 conversion (BSL → Apache after 4 years). For now: MV.

## How to escalate

1. Demonstrate which Q1-Q5 trigger MV-insufficiency for the workload.
2. Author an ADR amendment to ADR-0195 listing `(µservice, workload, Flink job)`.
3. Author the Flink cluster Helm chart at `microservices/<ms>/iac/helm/flink/`.
4. Wire OpenSLO + runbook per ADR-0186 Stage 5 + per-µservice runbooks/.
5. Capacity model entry for the new Flink cluster.

The friction is intentional — class B workloads are real and Flink is the right answer when they exist; the gate prevents accidental adoption.

## References

- ADR-0195 — stream processing tier (parent ADR).
- ADR-0193 — OLAP analytics warehouse (provides MV substrate).
- Apache Flink 2.2 — https://flink.apache.org/
- Materialize (deferred) — https://materialize.com/
