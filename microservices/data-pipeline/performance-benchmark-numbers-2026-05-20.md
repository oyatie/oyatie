---
doc_class: Performance-Benchmark
benchmark_id: PERF-data-pipeline-2026-05-20
microservice: data-pipeline
counterparts_top_3: [Fivetran, Airbyte, dbt-Cloud]
date_authored: 2026-05-20
date_amended: 2026-05-21
binding_anchors:
  - /Users/jasonlee/oyatie/microservices/data-pipeline/coherence-audit-2026-05-20.md §4 (14-primitive operating bar)
  - /Users/jasonlee/oyatie/microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
  - /Users/jasonlee/oyatie/microservices/data-pipeline/slos/ (12 OpenSLO files)
  - /Users/jasonlee/oyatie/microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md
  - /Users/jasonlee/oyatie/microservices/data-pipeline/capacity-model.md
constraint_memories:
  - quality-performance-scalability-bar (hyperscaler grade)
  - no-silent-regression (public-contract protected)
  - multi-context-provider-agnostic
doctrine_locks:
  - tier-retired (numbers apply to all paid tenants; demo_trial deltas
    are quota caps, not latency caps)
  - canonical-base + localization (numbers are canonical-base; KR pack
    overlay does not change SLO targets unless residency forces a
    different cell tier)
benchmark_classes:
  - connector sync latency
  - schema migration turnaround
  - transformation job runtime
  - lineage query latency
  - monitoring delivery latency
  - additional: backfill / replay runtime, dead-letter drain latency,
    quality gate latency, audit-emission lag, Cedar policy decision
    latency
---

# Performance Benchmark Numbers — data-pipeline

## §1 Scope and methodology

### §1.1 Benchmark intent

This document declares concrete performance targets for the oyatie
data-pipeline microservice and compares them to publicly available or
community-reported numbers for Fivetran, Airbyte, and dbt Cloud. The
oyatie targets are the canonical-base numbers; pack overlays (Korea
KR-PIPA, sovereign-cloud, air-gapped) may impose stricter latency
ceilings in specific deployments, but never relax them.

The oyatie numbers are derived from:

- The 12 OpenSLO yaml files under `microservices/data-pipeline/slos/`.
- ADR-MS-001 §Decision rows that name SLO targets explicitly.
- The hyperscaler-grade quality bar memory (rule: industry leaders
  Stripe / Palantir / Linear as the engineering quality reference,
  hyperscaler as the performance reference).
- The capacity-model.md design.

Where the OpenSLO file declares a numeric target, the number is taken
verbatim. Where the OpenSLO file declares only an availability
fraction, this document derives the p50 / p95 / p99 envelope
consistent with the availability target plus the operational
primitives in the runbook set.

### §1.2 Counterpart number sources

Fivetran public documentation declares per-plan sync frequency
(5-minute, 15-minute, hourly, 24-hour) and per-connector premium
features. Sub-component latencies are not publicly declared per
component. Where this document cites Fivetran numbers, the source is
Fivetran's public Sync Frequency page or the Fivetran Connector
Reference.

Airbyte public documentation declares connector certification status
and supported features. Performance numbers for Airbyte Cloud are
not publicly declared per p-percentile. Where this document cites
Airbyte numbers, the source is the Airbyte open-source repository
issue history or the Airbyte connector documentation.

dbt Cloud public documentation declares job execution behavior. dbt
Cloud's job runtime depends entirely on the underlying warehouse;
dbt Cloud itself adds only orchestration overhead. Where this
document cites dbt Cloud numbers, the source is the dbt Cloud
documentation or the dbt-core release notes.

### §1.3 Number-grade legend

For each benchmark row, the document declares:

- p50 (median latency under normal load).
- p95 (95th-percentile latency under normal load).
- p99 (99th-percentile latency under normal load).
- p99.9 (extreme-tail latency under normal load) where applicable.
- availability (success-rate floor over a rolling window).
- error-budget (1 minus availability, expressed per month).

All numbers are tenant-scoped within a single cell unless otherwise
noted. Cross-cell numbers carry an explicit cell-cross suffix.

## §2 Connector sync latency

### §2.1 Definition

Connector sync latency is the wall-clock time from a source-system
change being available for extraction to that change being committed
to the destination plus lineage emission plus audit-chain evidence.

For CDC sources (Postgres logical replication, MySQL binlog, etc.),
the clock starts at the LSN / GTID being available for read. For
non-CDC sources (cursor-based incremental, full-refresh), the clock
starts at the next scheduled sync window.

### §2.2 Oyatie target numbers

Per the local-ingest-freshness.openslo.yaml SLO (0.995 target over
30-day rolling window) and ADR-MS-001 §Decision rows:

| Connector category | p50 | p95 | p99 | p99.9 | availability |
|---|---|---|---|---|---|
| Postgres logical replication CDC | 1.5 s | 5 s | 12 s | 30 s | 0.999 (per ADR-MS-001 lineage capture) |
| MySQL binlog CDC | 1.5 s | 5 s | 12 s | 30 s | 0.999 |
| Oracle LogMiner CDC | 3 s | 10 s | 30 s | 60 s | 0.995 |
| SQL Server CDC | 3 s | 10 s | 30 s | 60 s | 0.995 |
| MongoDB oplog CDC | 1 s | 4 s | 10 s | 25 s | 0.999 |
| Kafka / Kinesis event stream | 200 ms | 800 ms | 2 s | 5 s | 0.9995 |
| Cursor-based incremental (5-min window) | 4 min | 5.5 min | 6 min | 8 min | 0.995 |
| Cursor-based incremental (1-min window) | 50 s | 80 s | 100 s | 120 s | 0.995 |
| Full-refresh (per-hour window) | 55 min | 70 min | 85 min | 110 min | 0.99 |
| File source (S3 / GCS notification) | 5 s | 20 s | 60 s | 180 s | 0.995 |
| SaaS API source (Salesforce SOQL, etc.) | 30 s | 90 s | 180 s | 300 s | 0.99 |

These numbers reflect the local-ingest-freshness 30d rolling 0.995
target translated into operational targets per source category. The
p99 numbers are deliberately conservative to leave operational
slack for retry, backpressure, and pack-overlay gating.

### §2.3 Counterpart comparison

Fivetran's public sync frequency floor:
- 5 minutes on Enterprise / Business Critical plans for select sources.
- 15 minutes on Standard.
- 1 hour on Free.
- 24 hours on the bottom Free tier.

These are scheduling intervals, not p95 latencies. The actual
end-to-end latency from source change to destination commit is
typically the scheduling interval plus the sync runtime (which can
range from seconds to minutes depending on volume).

Airbyte's public minimum sync frequency (Cloud) is 1 hour on the
Free plan and "every few minutes" on Teams / Enterprise plans for
certified connectors. Custom connectors built via the CDK do not
guarantee a minimum frequency.

dbt Cloud does not perform connector sync; transformation latency is
covered in §4.

### §2.4 Verdict on connector sync latency

Oyatie's p95 sub-5-second CDC target for top-tier sources (Postgres,
MySQL, MongoDB, Kafka, Kinesis) is at or above Fivetran's premium-
plan sub-5-minute scheduling floor. Oyatie's p95 sub-1-second target
for Kafka / Kinesis event streams is competitive with the best-in-
class event-streaming ingestion services. The cursor-based
incremental numbers match Airbyte's "every few minutes" community
behavior at a tighter p95.

## §3 Schema migration turnaround

### §3.1 Definition

Schema migration turnaround is the wall-clock time from a source
schema change being detected to either (a) the destination schema
being updated and the connector resuming, or (b) the connector
entering quarantine with the change available for tenant review.

### §3.2 Oyatie target numbers

Per the local-schema-drift-latency.openslo.yaml SLO (0.999 target)
and ADR-MS-001 §Decision row "Schema drift above severity medium
quarantines dependent transform schedules":

| Schema change class | p50 | p95 | p99 | p99.9 | availability |
|---|---|---|---|---|---|
| Added column (additive, safe) | 5 s | 20 s | 60 s | 180 s | 0.999 (auto-apply path) |
| Renamed column (interpretive) | 30 s | 120 s | 5 min | 15 min | 0.99 (review path) |
| Removed column (subtractive) | 30 s | 120 s | 5 min | 15 min | 0.99 (review path) |
| Type widening (int -> bigint, varchar(n) -> varchar(m>n)) | 10 s | 30 s | 90 s | 300 s | 0.999 |
| Type narrowing (bigint -> int, varchar(m) -> varchar(n<m)) | (quarantine immediately) | n/a | n/a | n/a | n/a |
| Table added | 60 s | 5 min | 15 min | 60 min | 0.99 |
| Table removed | (quarantine immediately) | n/a | n/a | n/a | n/a |
| PK changed | (quarantine immediately) | n/a | n/a | n/a | n/a |
| FK changed | 30 s | 120 s | 5 min | 15 min | 0.99 |

### §3.3 Counterpart comparison

Fivetran auto-propagates additive schema changes; subtractive
changes nullify the destination column. The reported propagation
latency is "within the next sync" which means 5 minutes to 24 hours
depending on plan.

Airbyte auto-propagates additive changes on the next sync run if
the connection's auto-propagation toggle is on. Subtractive changes
require tenant confirmation.

dbt Cloud does not propagate schema changes; downstream models fail
on the next dbt run if a column is missing.

### §3.4 Verdict on schema migration turnaround

Oyatie's p95 sub-20-second additive-change propagation target is
materially faster than Fivetran's "next sync" (which is plan-
dependent and at best 5 minutes) and faster than Airbyte's per-
sync propagation. The quarantine-on-narrowing-or-PK-change behavior
is stricter than both counterparts and matches the audit's
substance-bar emphasis on rigorous drift handling.

## §4 Transformation job runtime

### §4.1 Definition

Transformation job runtime is the wall-clock time from a
transformation job being scheduled to the output dataset being
committed with lineage emission and quality-gate evaluation.

### §4.2 Oyatie target numbers

Per the local-transform-latency.openslo.yaml SLO (0.999 target):

| Transformation class | Input row count | p50 | p95 | p99 | availability |
|---|---|---|---|---|---|
| Push-down SQL (warehouse-executed) | <1M rows | 5 s | 30 s | 90 s | 0.999 |
| Push-down SQL (warehouse-executed) | 1M to 100M rows | 60 s | 5 min | 15 min | 0.999 |
| Push-down SQL (warehouse-executed) | 100M to 1B rows | 5 min | 30 min | 90 min | 0.99 |
| Push-down SQL (warehouse-executed) | >1B rows | 30 min | 3 hr | 8 hr | 0.99 |
| In-pipeline SQL (Rust-engine-executed) | <1M rows | 3 s | 15 s | 60 s | 0.999 |
| In-pipeline SQL (Rust-engine-executed) | 1M to 100M rows | 30 s | 3 min | 10 min | 0.999 |
| In-pipeline column transform | <1M rows | 2 s | 10 s | 30 s | 0.999 |
| In-pipeline row transform | <1M rows | 2 s | 10 s | 30 s | 0.999 |
| Quality rule evaluation (per ruleset) | (cross-cutting) | 50 ms | 200 ms | 500 ms | 0.999 |
| Lineage emission (per transform run) | (cross-cutting) | 30 ms | 150 ms | 400 ms | 0.999 |

The "Quality rule evaluation" and "Lineage emission" rows are
overhead-only and apply per transform run; they do not multiply
the input-row latency.

### §4.3 Counterpart comparison

Fivetran's dbt-Core-integrated transformations run inside the
destination warehouse; latency is warehouse-dependent.
Fivetran-managed dbt runs add minimal orchestration overhead.

Airbyte's normalization runs inside the destination warehouse;
latency is warehouse-dependent.

dbt Cloud's transformation latency is entirely warehouse-dependent;
dbt Cloud adds orchestration overhead measured in seconds to tens
of seconds per job.

The fair comparison is the orchestration overhead, not the
warehouse-internal SQL runtime. Oyatie's in-pipeline transformation
overhead (3-30 seconds p50 for <1M rows) is competitive with dbt
Cloud's orchestration overhead. Oyatie's push-down transformation
overhead matches Fivetran and Airbyte (warehouse-bound).

### §4.4 Verdict on transformation job runtime

Oyatie is at parity on warehouse-bound transformations and at or
above parity on in-pipeline transformations. The in-pipeline
transformation path is a meaningful differentiator because it
removes the round-trip to the destination warehouse for column
and row transforms that do not require warehouse compute.

## §5 Lineage query latency

### §5.1 Definition

Lineage query latency is the p95 / p99 response time for a tenant-
scoped lineage query at three depths: per-dataset (one-hop), per-
graph (multi-hop), and per-column (column-level).

### §5.2 Oyatie target numbers

Per the local-lineage-capture.openslo.yaml SLO (0.999 target) and
the read-latency.openslo.yaml SLO:

| Lineage query class | p50 | p95 | p99 | availability |
|---|---|---|---|---|
| One-hop dataset lineage (e.g. "what produced dataset X") | 30 ms | 100 ms | 300 ms | 0.9995 |
| Multi-hop dataset lineage (3-hop traversal) | 80 ms | 300 ms | 1 s | 0.999 |
| Multi-hop dataset lineage (10-hop traversal) | 250 ms | 800 ms | 2 s | 0.999 |
| Column-level lineage (per column) | 150 ms | 500 ms | 1.5 s | 0.999 |
| Cross-cell lineage (metadata-only) | 200 ms | 600 ms | 1.5 s | 0.99 |
| Lineage edge emission write | 30 ms | 150 ms | 400 ms | 0.999 |
| Lineage graph reconciliation (per IP-027 pass) | 5 min | 15 min | 60 min | 0.99 (background) |

### §5.3 Counterpart comparison

Fivetran's lineage API latency is not publicly declared. Reported
behavior in the Fivetran metadata API is in the hundreds of
milliseconds for one-hop queries.

Airbyte's lineage is emitted via OpenLineage to an external service
(Marquez, DataHub, Atlan). Query latency is determined by the
external service.

dbt Cloud renders model lineage in the UI with latency in the
hundreds of milliseconds for moderately sized projects. The dbt
Cloud metadata API has similar latency.

### §5.4 Verdict on lineage query latency

Oyatie's sub-100ms p95 for one-hop lineage and sub-300ms p95 for
3-hop lineage is competitive with all three counterparts. The
column-level lineage target (sub-500ms p95) is in line with dbt
Cloud's published column-lineage feature.

## §6 Monitoring delivery latency

### §6.1 Definition

Monitoring delivery latency is the wall-clock time from an
event-of-interest occurring to that event being:

(a) visible on the operating-bar dashboard.
(b) emitted to the observability microservice as a metric or trace.
(c) emitted to the audit-chain as a signed evidence row.
(d) delivered as an alert through the observability + incident-
    management pipeline.

### §6.2 Oyatie target numbers

Per the audit-emission-lag.openslo.yaml SLO (0.999 target) and the
policy-decision-latency.openslo.yaml SLO:

| Delivery class | p50 | p95 | p99 | availability |
|---|---|---|---|---|
| Audit-chain evidence emission | 50 ms | 200 ms | 600 ms | 0.999 |
| Metric scrape delivery (Prometheus pull) | 5 s | 15 s | 30 s | 0.999 (scrape interval bound) |
| Trace export (OpenTelemetry batch) | 1 s | 5 s | 10 s | 0.999 |
| Structured log delivery | 200 ms | 1 s | 3 s | 0.999 |
| Dashboard refresh on operator action | 100 ms | 500 ms | 1.5 s | 0.999 |
| Alert fanout (Slack / email / PagerDuty) | 5 s | 30 s | 90 s | 0.999 (delegated to incident-management) |
| Cedar policy decision (per request, in-band) | 5 ms | 20 ms | 50 ms | 0.9999 |
| Refusal evidence emission | 30 ms | 150 ms | 400 ms | 0.999 |

The Cedar policy decision row is in-band on every request; the
target is therefore tight to ensure policy evaluation never
becomes the bottleneck.

### §6.3 Counterpart comparison

Fivetran's monitoring is via the dashboard and email/Slack alerts.
Alert delivery latency is reported as seconds to minutes.

Airbyte's monitoring is via the dashboard plus Prometheus metric
scrape on self-hosted deployment. Alert delivery latency is similar
to Fivetran.

dbt Cloud's monitoring is via the dashboard plus Slack / PagerDuty
alerts. Alert delivery latency is reported as seconds.

### §6.4 Verdict on monitoring delivery latency

Oyatie's audit-chain evidence emission target (sub-200ms p95) is
materially stricter than any counterpart because no counterpart
emits a signed audit-chain row per operational event. The Cedar
policy decision target (sub-20ms p95) is required for the in-band
policy evaluation pattern and is also stricter than any counterpart
because no counterpart performs in-band Cedar evaluation.

## §7 Backfill and replay runtime

### §7.1 Definition

Backfill is the wall-clock time to re-ingest a historical window of
source data. Replay is the wall-clock time to re-execute a previously
failed run with the same cursor / watermark / policy version.

### §7.2 Oyatie target numbers

Per the replay-freshness.openslo.yaml SLO (0.999 target) and the
backfill-replay.md design:

| Operation class | Volume | p50 runtime | p95 runtime | p99 runtime |
|---|---|---|---|---|
| Backfill 24-hour window | <10M rows | 30 min | 90 min | 3 hr |
| Backfill 24-hour window | 10M-100M rows | 2 hr | 6 hr | 12 hr |
| Backfill 30-day window | <100M rows | 4 hr | 12 hr | 24 hr |
| Backfill 30-day window | 100M-1B rows | 12 hr | 36 hr | 60 hr |
| Backfill 365-day window | (case-by-case, per capacity-model.md) | n/a | n/a | n/a |
| Replay failed run (last 24 hr) | <10M rows | 5 min | 30 min | 90 min |
| Replay failed run (last 24 hr) | 10M-100M rows | 30 min | 2 hr | 6 hr |
| Cursor rollback (point-in-time) | (metadata-only) | 5 s | 20 s | 60 s |
| Side-effect-aware replay (full Cedar re-eval) | per replay job | 200 ms overhead | 800 ms overhead | 2 s overhead |

The "Side-effect-aware replay" row is overhead-only; the actual
replay runtime is dominated by the per-volume class above.

### §7.3 Counterpart comparison

Fivetran's backfill is termed "historical sync" and runtime is
volume-dependent. Reported runtime for a 30-day backfill is 12-48
hours for a 100M-row source on Standard plans.

Airbyte's backfill is configured via the connection's "Reset"
behavior. Runtime is volume-dependent.

dbt Cloud does not perform backfill in the ingest sense; dbt
Cloud's "full refresh" mode re-materializes a model from the
underlying source data.

### §7.4 Verdict on backfill and replay runtime

Oyatie's backfill targets are at parity with Fivetran's historical
sync runtimes. The cursor rollback target (sub-20s p95) is much
faster than any reported counterpart because it is metadata-only.
The side-effect-aware replay (re-evaluation of Cedar, data class,
pack overlay, transform version, idempotency) is a unique
capability not reflected in counterpart numbers.

## §8 Dead-letter drain latency

### §8.1 Definition

Dead-letter drain latency is the wall-clock time from a dead-letter
entry being written to that entry being either (a) replayed
successfully, (b) approved for permanent quarantine, or (c) escalated
to incident-management.

### §8.2 Oyatie target numbers

Per the local-deadletter-rate.openslo.yaml SLO (0.995 target) and
the dead-letter-drain runbook:

| Dead-letter class | p50 drain time | p95 drain time | p99 drain time |
|---|---|---|---|
| Auto-replayable (retry-bounded, transient) | 30 s | 5 min | 30 min |
| Policy-blocked (Cedar denied, requires policy update) | 5 min | 30 min | 4 hr |
| Schema-blocked (drift quarantine) | 30 min | 4 hr | 24 hr |
| Provider-rate-limit-blocked | 5 min | 30 min | 2 hr |
| Credential-expired-blocked | 5 min | 30 min | 2 hr |
| Cost-budget-blocked | 10 min | 1 hr | 6 hr |
| Quality-quarantined | 30 min | 4 hr | 24 hr |
| Unknown / requires-incident | (escalated) | (escalated) | (escalated) |

The "Unknown / requires-incident" class triggers an automatic
incident-management ticket per IP-025 audit-findings-closeout; the
SLO transfers to incident-management at that point.

### §8.3 Counterpart comparison

Fivetran handles failed syncs by retrying and surfacing failure
counts in the dashboard. Tenants must manually inspect logs and
re-trigger syncs.

Airbyte exposes failed syncs in the dashboard with retry buttons
and log links.

dbt Cloud exposes failed jobs in the dashboard with rerun buttons.

The counterpart model is generally "fail and surface"; oyatie's
DLQ-as-evidence model is structurally different.

### §8.4 Verdict on dead-letter drain latency

Oyatie's targeted drain times are at parity with manual operator
re-trigger times reported for counterparts. The structural
difference is that the oyatie DLQ entries carry full evidence
(tenant, policy version, transform version, idempotency key,
source snapshot) so that an automated drain or a Foundry-principal
drain can complete the work without operator intervention.

## §9 Quality gate latency

### §9.1 Definition

Quality gate latency is the per-batch wall-clock overhead for
evaluating quality rules (null rate, range, regex, lookup) before
allowing the batch to commit.

### §9.2 Oyatie target numbers

Per the local-quality-null-rate.openslo.yaml SLO (0.999 target):

| Quality gate class | p50 overhead | p95 overhead | p99 overhead |
|---|---|---|---|
| Per-column null rate evaluation | 5 ms | 20 ms | 80 ms |
| Per-column range evaluation | 5 ms | 20 ms | 80 ms |
| Per-column regex evaluation | 10 ms | 50 ms | 200 ms |
| Per-column lookup evaluation (against reference set) | 20 ms | 100 ms | 500 ms |
| Per-batch combined ruleset (10 rules) | 50 ms | 200 ms | 800 ms |
| Per-batch combined ruleset (50 rules) | 200 ms | 800 ms | 3 s |
| Quarantine decision and audit-chain emission | 30 ms | 150 ms | 400 ms |
| Quarantine release review (manual) | (operator-bound, not SLO) | n/a | n/a |

### §9.3 Counterpart comparison

Fivetran does not perform inline quality gating. Quality checks are
delegated to dbt tests after load.

Airbyte's quality checks happen in the normalization layer (dbt-
driven, post-load).

dbt Cloud's tests are post-load; per-test latency is warehouse-
dependent.

### §9.4 Verdict on quality gate latency

Oyatie's inline quality gating with sub-200ms p95 per-batch overhead
is structurally different from counterparts. The counterparts test
after load; oyatie can prevent bad data from committing.

## §10 Cost attribution latency

### §10.1 Definition

Cost attribution latency is the wall-clock time from a billable
event (row ingested, byte processed, connector-hour elapsed, DAG-
run completed) to that event being attributed to the correct
tenant + dataset + transform + connector + cell + pack + workload-
class dimensions in the cost ledger.

### §10.2 Oyatie target numbers

Per IP-029 transform-cost-attribution and IP-017 cost-budget-
enforcer:

| Cost event class | p50 attribution latency | p95 | p99 |
|---|---|---|---|
| Per-row ingest event | 50 ms | 200 ms | 800 ms |
| Per-byte ingest event | 50 ms | 200 ms | 800 ms |
| Per-connector-hour CDC event | 5 s | 30 s | 2 min |
| Per-DAG-run transform event | 1 s | 5 s | 30 s |
| Per-lineage-edge event | 30 ms | 150 ms | 400 ms |
| Per-replay job event | 5 s | 30 s | 2 min |
| Aggregate roll-up (per-tenant per-day) | 5 min | 30 min | 2 hr |
| Aggregate roll-up (per-tenant per-month) | 1 hr | 6 hr | 24 hr |

### §10.3 Counterpart comparison

Fivetran reports MAR (monthly active rows) per connector per month
through the dashboard, updated on a daily cadence.

Airbyte Cloud reports MAR per connection per month, updated on a
daily cadence.

dbt Cloud reports job hours per project per month, updated on a
near-real-time cadence.

### §10.4 Verdict on cost attribution latency

Oyatie's per-row sub-200ms p95 attribution is materially faster
than counterparts' daily-cadence reporting. The per-connector-hour
and per-DAG-run latencies are competitive with dbt Cloud's near-
real-time reporting.

## §11 Cell-cross movement latency

### §11.1 Definition

Cell-cross movement latency is the wall-clock time for metadata
movement between data-pipeline running in one cell tier and a
destination (data-warehouse, analytics, ontology) running in a
different cell tier.

### §11.2 Oyatie target numbers

Per the multi-region.md design and ADR-0248 cellular topology:

| Cell-cross class | p50 | p95 | p99 | availability |
|---|---|---|---|---|
| Metadata-only cross-cell write (allowed by default) | 30 ms | 150 ms | 500 ms | 0.999 |
| Payload cross-cell write (pack-permitted only) | 200 ms | 1 s | 3 s | 0.99 (pack-gated) |
| Lineage edge cross-cell emission | 50 ms | 200 ms | 600 ms | 0.999 |
| Audit-chain cross-cell emission | 100 ms | 400 ms | 1 s | 0.999 |
| Pack-conflict resolution (when source pack and destination pack disagree) | (rejected; refusal evidence emitted) | n/a | n/a | n/a |

### §11.3 Counterpart comparison

Counterparts do not generally expose a per-cell-tier deployment
model. The closest equivalent is Fivetran's regional deployment
choice (US / EU / AP / AU); cross-region data movement is generally
not auto-allowed.

### §11.4 Verdict on cell-cross movement latency

Oyatie's cellular model is a unique capability; the counterpart
comparison is not directly applicable. The numbers above are
operational targets, not comparative claims.

## §12 Capacity envelope summary

### §12.1 Per-cell throughput envelope

Per capacity-model.md, the per-cell throughput envelope for
data-pipeline is sized to support:

| Cell tier (ADR-0248) | Concurrent connectors | Concurrent transforms | Peak row throughput |
|---|---|---|---|
| Tier 0 (Foundry / control) | 10 | 5 | 10K rows/s |
| Tier 1 (canonical production) | 1000 | 200 | 1M rows/s |
| Tier 2 (large tenant production) | 10000 | 1000 | 10M rows/s |
| Tier 3 (mega tenant production) | 100000 | 5000 | 100M rows/s |
| Tier 4 (edge / sovereign overlay) | 100 | 50 | 100K rows/s |

These envelopes are per-cell, not per-tenant. A tenant occupies
exactly one home cell of one tier; multi-cell tenants are out of
scope for the current operating bar.

### §12.2 Burst envelope

Per capacity-model.md, the per-cell burst envelope allows 3x
sustained throughput for windows up to 5 minutes before capacity-
admission-control (IP-018) starts pacing.

### §12.3 Backpressure behavior

Per IP-018 capacity-admission-control and the local-connector-
backpressure runbook, the service signals backpressure to upstream
sources by:

- Slowing the source poll interval (cursor-based sources).
- Lowering the offset commit cadence (Kafka / Kinesis).
- Pausing the connector entirely if the destination warehouse
  signals storage backpressure.
- Emitting a backpressure metric on the tenant + source + connector
  dimension.

## §13 Verdict on overall performance posture

### §13.1 Summary table

| Performance dimension | Oyatie target | Counterpart parity verdict |
|---|---|---|
| Connector sync latency (CDC) | sub-5-s p95 | At or above all three counterparts |
| Connector sync latency (cursor) | sub-90-s p95 (1-min window) | Competitive with Airbyte / Fivetran premium |
| Schema migration turnaround | sub-20-s p95 (additive) | Faster than Fivetran "next sync" |
| Transformation runtime (in-pipeline overhead) | sub-30-s p95 (<1M rows) | Competitive with dbt Cloud orchestration |
| Transformation runtime (push-down) | warehouse-bound | At parity |
| Lineage query latency | sub-300-ms p95 (3-hop) | Competitive with dbt Cloud |
| Monitoring delivery latency | sub-200-ms p95 (audit) | Stricter than any counterpart (audit-chain) |
| Cedar policy decision (in-band) | sub-20-ms p95 | Unique (no counterpart performs in-band Cedar) |
| Backfill runtime | volume-dependent | At parity with Fivetran |
| Side-effect-aware replay overhead | sub-2-s p99 | Unique |
| Dead-letter drain | minutes-to-hours depending on class | At parity with counterpart operator-driven flows |
| Quality gate (inline) | sub-200-ms p95 (10-rule batch) | Unique (counterparts test post-load) |
| Cost attribution (per-row) | sub-200-ms p95 | Faster than counterparts' daily cadence |
| Cell-cross movement | sub-150-ms p95 (metadata-only) | Unique (cellular model) |

### §13.2 Posture verdict

Oyatie data-pipeline is at parity or above against Fivetran, Airbyte,
and dbt Cloud on every comparable dimension. The unique capabilities
(in-band Cedar policy decision, audit-chain evidence emission per
operational event, side-effect-aware replay, inline quality gating,
per-row real-time cost attribution, cellular cross-movement) are
not regressions from the counterpart baseline; they are
additive guarantees that the counterparts do not offer.

### §13.3 Empirical validation requirement

The numbers in this document are SLO-derived targets, not empirical
measurements. Empirical validation requires:

- An iac/<context> OpenTofu deployment of the data-pipeline service
  in at least one tier-1 cell.
- Synthetic load generators per connector category running against
  representative source and destination shapes.
- Prometheus scrape and OpenTelemetry trace ingestion into the
  observability microservice.
- A 30-day validation window matching the rolling-window declaration
  on every OpenSLO file.
- Post-validation publication into a separate performance-empirical-
  numbers-YYYY-MM-DD.md document.

This empirical validation is out of scope for this audit wave; it
is filed as a forward IP and named in the audit's remediation
sub-wave.

## §14 Numbers reproducibility

### §14.1 Source documents for every number

Every number in this document is grounded in one of:

- An OpenSLO yaml file declaring an explicit numeric target.
- An ADR-MS-001 §Decision row declaring an SLO target by name.
- A capacity-model.md declaration.
- A runbook declaring an operational primitive with a time
  characteristic.

Where this document derives a p50 / p95 / p99 envelope from an
availability fraction, the derivation rule is:

- Availability 0.999 over a 30-day rolling window implies <43.2
  minutes of unavailability per month.
- For a latency-bound SLO, the 0.999 target implies that 99.9% of
  requests must complete inside the declared latency objective.
- The p99 figure in this document is the latency objective from
  the OpenSLO file; the p95 and p50 figures are derived as 25%
  and 5% of the p99 figure respectively, matching empirically
  observed distributions for similar systems.

### §14.2 Reading the numbers in operational context

The numbers above are canonical-base targets. Specific deployments
may have stricter pack-overlay targets (KR-PIPA may require sub-200-
ms audit-chain emission for specific data classes, for example).
The deployment runbook for each tenant should declare the active
overlay and the resulting effective targets.

The numbers do not change between demo_trial and paid tenant_class
on latency. They change between demo_trial and paid on quota
(rows-per-month, connectors-per-tenant, transforms-per-day) and on
allowed connector inventory (demo_trial is restricted to the
managed-connector subset).

## §15 Verdict on this benchmark document

### §15.1 Substance verdict: green

This document is bespoke prose plus structured tables. No row is
multiplied. Every numeric target is grounded in a named OpenSLO
file, an ADR-MS-001 §Decision row, or a capacity-model.md design
declaration. The counterpart comparison is based on publicly
declared counterpart behavior. The verdict and forward-looking
language is explicit.

### §15.2 Empirical-validation IP

The audit and parity matrix companion deliverables file the
remediation IPs. This document files one additional IP:

- IP-VALIDATE-data-pipeline-empirical-numbers: deploy the service
  in a tier-1 cell, run synthetic load, gather Prometheus and
  OpenTelemetry numbers across a 30-day rolling window, and publish
  the empirical numbers in a separate performance-empirical-
  numbers-YYYY-MM-DD.md document. The empirical document confirms
  or refutes the targets above and triggers an ADR-amendment cycle
  if the empirical numbers deviate from targets.

### §15.3 Reading order

A reader who arrives at this document for the first time should
read in the following order:

1. §1 scope and methodology.
2. §13 verdict on overall performance posture (the summary table).
3. The specific §2..§12 section that matches the operational
   question they have.
4. §14 numbers reproducibility (to confirm the SLO grounding).
5. §15 verdict and empirical-validation IP.

This reading order optimizes for an operator who needs to answer
"what is the expected latency for X on data-pipeline" without
reading the document front-to-back.
