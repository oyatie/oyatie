---
id: ADR-0195
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-analytics
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0005, ADR-0042, ADR-0131-per-microservice-flat-layout, ADR-0145, ADR-0153, ADR-0184, ADR-0186, ADR-0192, ADR-0193, ADR-0194]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0195 — Stream processing tier: ClickHouse Materialized Views + Kafka Engine default; Apache Flink 2.2 escalation under explicit ADR amendment

## Status

Accepted (2026-05-18). Establishes a **two-tier stream-processing policy**: ClickHouse Materialized Views + Kafka Engine are the default for the overwhelming majority of stream workloads (per-tenant rolling aggregates, percentile rollups, top-K ranking, anomaly windows, dashboard freshness); Apache Flink 2.2.x is reserved as escalation only for workloads that demonstrably exceed ClickHouse MV capability — gated behind ADR amendment for each Flink adoption.

The decision is asymmetric on purpose. **Default = cheap, in-OLAP, sub-second freshness.** Escalation = heavy stream-processing infrastructure only when the workload truly requires it.

## Context

Stream processing means transforming a continuous event stream into derived streams or rolling state. oyatie's stream workloads break into two classes:

### Class A — "ClickHouse MV–shaped" (~95% of workloads)

| Workload | Shape | Example |
|---|---|---|
| Per-tenant rolling aggregates | INSERT-triggered MV with `AggregatingMergeTree` target | Tenant's "this hour's workflow runs" counter |
| Percentile dashboards | MV with `quantilesState` aggregate | Per-µservice request-latency p99 over 5-min windows |
| Top-K ranking | MV with `topK` aggregate | Top-10 most-active tenants in the last hour |
| Anomaly windows | MV with sliding-window aggregate + threshold check | "Tenant X exceeded 10K errors/min" alert source |
| Dashboard freshness | MV materializes pre-aggregated state for query latency | Sub-100ms dashboard rendering vs multi-second raw-row scans |
| Audit trail rollup | MV grouping audit events by tenant+axis+day | Per-tenant compliance posture summary |

All of these are well-served by ClickHouse Materialized Views + Kafka Engine. Sub-second freshness; one substrate; no separate stream-processing cluster.

### Class B — "Flink-shaped" (~5% of workloads — escalation)

| Workload | Why ClickHouse MV insufficient |
|---|---|
| Multi-stream joins with state retention beyond a single table | MV is single-source-triggered; multi-stream joins require Flink's state backend |
| Exactly-once with external side effects (e.g., posting a payment side-effect on stream condition) | ClickHouse MV does not provide exactly-once with external side effects; Flink's checkpointing + transactional sinks do |
| Complex Event Processing (CEP) — multi-event temporal patterns (e.g., "user did A, then B within 5min, but not C") | Stateful CEP exceeds MV's per-row trigger model |
| Long-windowed stateful computations with session windows | ClickHouse MV windowing is fixed/sliding; session windows require Flink |
| Per-key state with custom merge logic across hours of state | Beyond MV `AggregateFunction` capability |

For class B, ClickHouse MV is genuinely insufficient; the workload escalates to Apache Flink.

Hyperscaler practice for stream processing:

- **Uber** — Flink heavily for real-time pricing + ML feature pipelines. But Uber also has internal Apache Pinot + materialized-view-class processing for the rest.
- **Stripe** — internally uses a mix; the vast majority of "stream rollup" work is materialized-view-class (in-OLAP) rather than Flink.
- **Cloudflare** — ClickHouse Materialized Views for the analytics-dashboard slot (matching ADR-0193); Flink only at the edge for specific CEP workloads.
- **Materialize / RisingWave** — newer entrants demonstrate that materialized-view-class stream processing covers the majority shape.

Anti-patterns this ADR forecloses:

1. Defaulting to Flink for every stream workload — Flink adds JVM + state-backend + checkpoint storage + ops surface that 95% of workloads don't need.
2. Per-µservice stream-processing-engine choice (Kafka Streams here, ksqlDB there, Flink elsewhere) — multi-runtime lock-in.
3. Confluent / proprietary stream-processing platforms — vendor lock-in conflicts with ADR-0014.

## Decision

### Default: ClickHouse Materialized Views + Kafka Engine

For class A workloads (~95%), the canonical stream-processing path is:

1. **Source.** Events land in the log-broker substrate (Apache Pulsar 4.2.x; supports Kafka wire protocol via Pulsar's Kafka-on-Pulsar proxy).
2. **Ingest.** ClickHouse `Kafka` engine connects to Pulsar's Kafka-protocol endpoint as a consumer.
3. **Materialized View.** `MATERIALIZED VIEW <view> TO <target_table> AS SELECT ... FROM <kafka_engine_table>` — runs on every batch of consumed messages.
4. **Target.** `AggregatingMergeTree` or `ReplacingMergeTree` target table.
5. **Query.** Dashboards / API queries read the target table; freshness ≤ 5 seconds for the default ingest cadence.

No separate stream-processing cluster. ClickHouse cluster (per ADR-0193) handles ingest + transform + serve in one engine.

### Escalation: Apache Flink 2.2.x

For class B workloads that demonstrably exceed ClickHouse MV, the canonical escalation is **Apache Flink 2.2.x** (current stable: 2.2.1 released 2026-05-15; Apache-2.0).

**Adoption gate.** Every Flink deployment requires:

1. Concrete workload description showing why ClickHouse MV is insufficient (one of the class B criteria above).
2. An **ADR amendment** (this ADR or a successor) listing the specific µservice + workload + Flink job.
3. Capacity model entry for the new Flink cluster (JVM heap, state backend size, checkpoint storage capacity, parallelism).
4. Runbook entry for the Flink job (failure modes, restart procedure, savepoint cadence).

This gate exists because Flink's ops surface is materially larger than ClickHouse MV's. Defaulting to Flink without the gate produces engine sprawl.

**Flink deployment shape.**

- JobManager (HA via 2 instances + ZK / Kubernetes leader election — Kubernetes leader election preferred to avoid ZK dependency).
- TaskManager pool (per-job parallelism × slots).
- State backend: RocksDB on local NVMe (fast); incremental checkpoints to object store (SeaweedFS S3-compat).
- Source connector: Flink Kafka connector (against Pulsar's Kafka-on-Pulsar) or Flink Pulsar connector (native).
- Sink connector: per-workload — ClickHouse, Postgres, OpenBao-secret-fronted external API, etc.

### Standards doc — when MV suffices vs when Flink justified

A standards doc lives at `docs/standards/stream-processing-rubric.md` (created in this batch's follow-on) and provides:

1. **Decision tree** — given a workload description, which tier?
2. **Per-class examples** — concrete workload-to-tier mappings.
3. **Anti-patterns** — workloads that have looked like Flink-shaped but were actually MV-shaped (and vice versa).
4. **Escalation procedure** — exactly how to amend this ADR when a Flink workload is genuinely required.

### Rejected stream-processing options

#### Kafka Streams — REJECTED (JVM lock-in + library, not service)

- **Pros:** ubiquitous in Kafka shops; library model lets it run inside any JVM app.
- **Cons:** JVM-only — clashes with oyatie's Rust-first fleet; library-model means stream state lives inside the µservice's own JVM, complicating per-µservice scaling decisions; Kafka-Streams-specific semantics don't generalize to Pulsar; smaller ecosystem than ClickHouse MV or Flink for the workload shape.
- **Rejected** on JVM lock-in + library shape.

#### ksqlDB — REJECTED (Confluent lock-in)

- **Pros:** SQL-native; embedded in Confluent stack.
- **Cons:** Confluent commercial licensing for production-grade use; vendor lock-in; cannot run on Pulsar (Confluent platform–specific).
- **Rejected** on vendor lock-in.

#### Materialize (Rust-native + Postgres-wire-compatible) — DEFERRED (revisit at v1.0 GA)

- **Pros:** Rust-native; Postgres-wire-compatible; differential dataflow primitive is genuinely state-of-the-art; positions as "real-time materialized views in the open."
- **Cons:** young (sub-1.0 at decision time); BSL-licensed (Business Source License, not OSI-OSS — auto-converts to Apache-2.0 after 4 years per the BSL grant); single-node primary architecture in current GA; multi-region / multi-cell story immature.
- **Deferred** — Materialize is the most plausible future-supplanter of ClickHouse MV for class A; re-evaluate at v1.0 GA + Apache-2.0 conversion + multi-node maturity.

#### RisingWave — DEFERRED

- **Pros:** distributed SQL streaming database; Apache-2.0; younger Materialize alternative.
- **Cons:** community + production-reference set still growing; ClickHouse MV is the safer 2026 bet.
- **Deferred** for future re-evaluation.

#### Apache Spark Streaming / Structured Streaming — REJECTED

- **Pros:** mature; broad ecosystem.
- **Cons:** batch-oriented under the hood; "structured streaming" is micro-batch; latency floor incompatible with sub-second freshness for the class A workload; JVM-heavy.
- **Rejected** on latency floor + batch-orientation.

#### Apache Beam — REJECTED (abstraction, not engine)

- **Pros:** portability across runners.
- **Cons:** runs on top of Flink/Spark/Dataflow; adds a layer without adding engine capability; the runtime is still Flink (or Spark or Dataflow). When oyatie reaches Flink-shaped workloads, going direct to Flink is simpler than layering Beam.
- **Rejected** on layer-without-value.

## Consequences

### Positive

1. **One substrate covers 95% of workloads.** ClickHouse MV + Kafka Engine = no new stream-processing engine for the dominant workload class.
2. **Escalation is gated.** Flink adoption requires explicit ADR amendment; prevents engine sprawl.
3. **All-permissive-license.** ClickHouse Apache-2.0; Flink Apache-2.0; Pulsar Apache-2.0.
4. **Sub-second freshness for the default tier.** Cloudflare-class dashboard freshness without a separate stream-processing cluster.
5. **Clear rubric.** `docs/standards/stream-processing-rubric.md` ends per-team re-litigation.

### Negative

1. **ClickHouse MV is single-source-triggered.** Multi-source joins escalate to Flink. Mitigation: documented in rubric; the 95% / 5% split is empirical from Cloudflare / Stripe / Uber reference workloads.
2. **Flink adoption gate adds process overhead.** Mitigation: the gate is exactly the right friction — Flink is genuinely heavy and the gate prevents accidental adoption.

### Operational

1. Per-µservice manifest declares `data.stream_processing.tier` as one of `clickhouse_mv` (default) | `flink` (with ADR amendment reference).
2. ClickHouse MV deployment piggybacks on ADR-0193's ClickHouse cluster — no separate operational artifact for class A.
3. Flink (if adopted) deploys at `microservices/<ms>/iac/helm/flink/` per µservice that owns the Flink job; canonical Flink Helm chart authored when the first Flink workload is adopted.
4. Rubric doc at `docs/standards/stream-processing-rubric.md` is the canonical decision aid.

## In-house roadmap

Per the in-house tech stack policy (user directive 2026-05-18) — "wherever possible, support in-house tech stack like AWS / Google / Microsoft / Oracle do" — stream-processing's in-house posture inherits from ADR-0193's ClickHouse in-house roadmap:

### Phase 0 — ClickHouse MV default + Flink escalation (current, this ADR)

- ClickHouse Materialized Views deployed inside the ADR-0193 ClickHouse cluster (no separate stream cluster for class A workloads).
- Apache Flink 2.2.x adopted only when ADR-amended for a specific class B workload.

### Phase 2 — In-house MV substrate inherits ADR-0193 Phase 2

When ADR-0193's Phase 2 in-house OLAP warehouse (`oya-olap-warehouse-server`) ships, its Materialized View capability **is** the in-house stream-processing default. Materialized View semantics in DataFusion-based OLAP engines are an open-research area (incremental view maintenance, differential dataflow integration); oyatie's Phase 2 plan tracks this as a follow-on after the core OLAP warehouse ships.

- **Phase 2 trigger.** Same as ADR-0193 Phase 2 (when oyatie's OLAP warehouse moves in-house, its MV layer comes with it).
- **MV equivalence.** The Phase 2 in-house OLAP warehouse's MV layer covers the class A workload set; consumer µservices repoint the `oya-shared-olap-client-kernel` adapter — no consumer-side code change.

### Apache Flink — KEEP (community standard, no in-house replacement planned)

Flink is the community standard for class B (stateful streaming) workloads. AWS, Google, Microsoft, Oracle all offer managed Flink (AWS Managed Service for Apache Flink, Google Cloud Dataflow built on Apache Beam + Flink runner, Azure Stream Analytics is Microsoft-specific but Azure also offers managed Flink, Oracle GoldenGate Stream Analytics). No hyperscaler has rebuilt Flink in-house; oyatie's posture follows the same trajectory — keep Flink for the escalation tier.

### Industry parallels for stream-processing

- **AWS Kinesis Data Streams + Kinesis Data Analytics** — in-house at AWS for the streaming substrate, but Kinesis Data Analytics actually runs Apache Flink.
- **Google Cloud Dataflow** — runs Apache Beam atop Flink/Dataflow runner.
- **Microsoft Azure Stream Analytics** — proprietary engine; in-house at Microsoft.
- **Oracle GoldenGate Stream Analytics** — proprietary engine; in-house at Oracle for the in-CDC streaming slot.
- **Cloudflare Workers Analytics Engine** — built on ClickHouse + custom edge-side ingest; in-house at Cloudflare for the analytics slot.

The convergent practice — the stream-processing **default** is the materialized-view layer of the OLAP warehouse (which goes in-house in Phase 2 per ADR-0193), while the **escalation** is community-standard Flink (which stays community per the hyperscaler-pattern reading above).

## Rollback

- **MV rollback** — `DROP MATERIALIZED VIEW <view>` reverts to raw-row scans against the source table; degraded query latency until the MV is recreated.
- **Flink rollback** (if adopted) — drop the Flink Helm release; savepoints persist in object store; resume from savepoint on next deployment.

## References

- ClickHouse Materialized Views — https://clickhouse.com/docs/en/sql-reference/statements/create/view
- ClickHouse Kafka Engine — https://clickhouse.com/docs/en/engines/table-engines/integrations/kafka
- Apache Flink — https://flink.apache.org/ ; Apache 2.0.
- Apache Flink 2.2.1 release — https://flink.apache.org/2026/05/15/... (2026-05-15 release line)
- Apache Pulsar Kafka-on-Pulsar — https://pulsar.apache.org/docs/io-kop/
- Materialize — https://materialize.com/ (Business Source License; deferred per this ADR).
- RisingWave — https://www.risingwave.com/ ; Apache 2.0 (deferred per this ADR).
- ADR-0005 — eventing backbone outbox pattern.
- ADR-0042 — observability stack.
- ADR-0131-per-microservice-flat-layout — flat layout.
- ADR-0145 — inter-microservice communication reform.
- ADR-0153 — outbox pattern.
- ADR-0184 — storage tier layering.
- ADR-0186 — observability backplane layering.
- ADR-0192 — vector database canonical (Milvus).
- ADR-0193 — OLAP analytics warehouse (ClickHouse — provides the MV substrate this ADR defaults to).
- ADR-0194 — tenant-facing time-series (TimescaleDB extension).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.
