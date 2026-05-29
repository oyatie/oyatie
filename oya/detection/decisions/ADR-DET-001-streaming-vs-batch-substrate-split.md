---
adr_id: ADR-DET-001
scope: microservices/detection
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0307, ADR-0308, ADR-0309, ADR-0131]
doc_status: published
---

# ADR-DET-001 — Streaming-vs-batch substrate split: Flink for sub-second, Spark for retrospective

## Status
Accepted 2026-05-20 by axis-detection. Lasts until at least the next ADR-0307 amendment (planned ≥ 2027-Q3).

## Context

The detection µservice owns 8 detection families (payment-fraud, account-takeover, synthetic-identity, aml-sanctions, content-abuse, fake-reviews-engagement, insider-risk, policy-violation) across two operational shapes:

1. **Streaming detection** — score events as they arrive; mitigation within seconds of the triggering event. Examples: card-not-present transaction risk score; login risk score; new-account risk score.
2. **Batch / retrospective detection** — sweep a corpus retrospectively to find patterns invisible in a single-event view. Examples: fake-review ring detection over a 7-day post-history window; insider-risk weekly model run; aml suspicious-activity-report (SAR) candidate generation.

Both shapes need:

- The same feature store (so model training + serving use identical features).
- The same model registry (per ADR-0308 — model card, drift, fairness gates).
- The same rule engine (so rules apply consistently across modes).
- Distinct compute primitives — streaming requires low-latency state; batch requires checkpointable long-running compute.

We evaluated 5 substrate combinations at Wave-3-C:

1. Flink (streaming) + Spark (batch) — current proposal.
2. Flink + Flink-batch (Flink's unified streaming-batch mode).
3. Kafka Streams + Spark.
4. Pulsar Functions (streaming) + Spark.
5. Materialize (streaming) + Spark.

Evaluation criteria, in priority order:

1. Sub-second p99 for streaming detection (the safe-harbor edge for payment-fraud auto-decline).
2. Stateful streaming with exactly-once semantics (transaction-shaped detection rejects duplicate scores).
3. Batch checkpointing across 1-PiB-class graph traversals.
4. Native ML model serving primitives (ONNX / TorchScript inference).
5. Operational maturity (drift handling, schema evolution, lateness handling).
6. Federation with `analytics` µservice's ClickHouse (we read feature snapshots from ClickHouse; write SAR candidates back).

## Decision

Use **Apache Flink 1.21 LTS** for streaming detection + **Apache Spark 4.0 LTS** for batch detection. Specifically:

- Streaming kernel (IP-001) + worker (IP-002): Flink, stateful streaming, RocksDB state backend, exactly-once via Pulsar source + Pulsar sink.
- Batch kernel (IP-003) + worker (IP-004): Spark 4.0, Iceberg table format on SeaweedFS-S3, checkpointing every 10 min.
- Feature store (IP-005 + IP-006): shared, persisted in Pulsar log-compacted topics + ClickHouse aggregation MVs.
- Rules engine (IP-007 + IP-008): Cedar policy expressions extended with rule-of-N constructs (per ADR-0307 §"Cedar over OPA").

Reject (2), (3), (4), (5) for the reasons in §Consequences.

## Detailed rationale

### Why Flink over the alternatives for streaming

Flink's stateful streaming model maps cleanly to fraud detection: a transaction event arrives, joins against the cardholder's recent-30-day state, applies the rule + model, emits a score within < 200 ms p99 — this is the proven Flink envelope at our scale (5 k events/sec sustained per stream).

Flink's RocksDB state backend handles the 50-100 GiB-per-stream state that fraud detection requires (per-cardholder rolling features over 30 d windows). Kafka Streams scales similarly but its state-store rebalancing during task reassignment is operationally painful (we measured 8-minute rebalance windows on 50-GiB state at Wave-3-B; Flink does it in < 2 min via incremental checkpointing).

Pulsar Functions is good for stateless transformations but the stateful model is immature (Pulsar 4.0's Function state is K-V only; no windowed aggregates). We use Pulsar as the EVENT BUS, but Flink as the COMPUTE.

Materialize is impressive for streaming SQL but its model-serving primitives are nascent (no native ONNX runtime; would require sidecar). For the 70 % of detection families that need a model in the hot path, Materialize would add a serving sidecar that Flink avoids (Flink ONNX runtime via DJL + DJL-Onnx is native).

### Why Spark over Flink-batch for retrospective

Flink does have a unified streaming-batch mode (the "Flink BATCH execution mode" since 1.12). For workloads under ~ 1 TiB it's competitive with Spark. Above that, Spark's mature shuffle + adaptive query execution (AQE) + Iceberg integration win clearly. Our biggest retrospective workload — the insider-risk 90-d graph traversal — touches ~ 4 TiB of feature data; Spark with AQE + dynamic partition pruning runs the job in 38 min vs Flink-batch's 84 min at Wave-3-B.

Also, Spark has the richest Iceberg ACID semantics (merge-on-read + copy-on-write + tag-based snapshots) which the SAR-candidate workflow needs (legally we must preserve the data + features + scores that led to a SAR for 5 y under FinCEN guidance + 7 y under SEC 17a-4(f)).

### Why a feature store shared across both modes

If streaming and batch use different feature definitions, the model's training distribution (batch) drifts from its serving distribution (streaming) — the classic "training-serving skew" failure mode. We avoid it by:

- Authoring features in a Rust-defined `Feature` trait that compiles to both Flink (streaming) and Spark (batch) runtime.
- Persisting computed features in Pulsar log-compacted topics (real-time available) + Iceberg tables (batch-queryable).
- The model card (per ADR-0308) names features by `Feature` trait + version; both modes resolve to the same compiled feature.

### Why Cedar rules over OPA / Drools / Easy Rules

Per ADR-0307 §"Cedar over OPA". Cedar's static analyzability — the validator proves termination + bounded resource use at policy-definition time — matters for streaming detection where a runaway rule has measurable customer impact. OPA's Rego is Turing-complete (not bounded); Drools is JVM-heavy; Easy Rules is too simple for our rule-of-N + temporal-window constructs.

We extended Cedar with `rule-of-N (count, window, predicate)` and `temporal-aggregate (sum/avg, window, predicate)` constructs per ADR-DET-001-cedar-extensions.md — these are statically analysable (we can bound the state required for each window) and serialise via JSON / Protobuf cleanly.

## Consequences

Positive:

- Sub-200 ms p99 streaming detection envelope hit; safe-harbor edge for payment-fraud auto-decline cleared.
- Batch retrospective jobs run in target windows (insider-risk weekly: 38 min); fits in the 4-h overnight batch window.
- Shared feature store eliminates training-serving skew; model drift detection (per ADR-0308) becomes meaningful.
- Cedar rule engine integrates with the rest of the platform's authorization (one policy substrate; one audit emission).

Negative:

- Two compute substrates to operate (Flink + Spark). Each has its own deployment + monitoring + upgrade dance. Ops cost is ~ 0.6 FTE total (vs 0.4 FTE if we could pick one).
- Flink 1.21 LTS requires Java 17; Spark 4.0 LTS requires Java 21. Different JVM versions in the substrate fleet.
- Feature trait code-generation (Rust → Flink-Java + Spark-Scala) is a custom toolchain. CI lane `feature-trait-codegen-roundtrip` validates the round-trip; if it fails, both pipelines break.

## Compliance

- Per ADR-0308, every model running in the streaming or batch substrate has a model card; the model card identifies the substrate (`flink` or `spark`) so we can target hot-fix substrates without retraining models.
- Per ADR-0309, fairness audit reads from the same feature store as production scoring; the audit doesn't drift from production.
- Per ADR-0310, investigation case-management replays both streaming and batch scores via the sandbox replay kernel (IP-015 + IP-016).

## Migration triggers

Re-open this ADR if any of:

- Flink 1.21 LTS has > 3 unpatched CVEs of CVSS ≥ 7.0 unresolved for > 60 days.
- Spark 4.0 LTS announces a shuffle-engine revamp incompatible with our Iceberg integration.
- A new substrate (e.g., Apache Beam unified-with-better-state-backend, or RisingWave reaching maturity) hits the same streaming envelope at lower ops cost.
- A pack mandates a specific certification (e.g., FIPS 140-3 for state-backend encryption) that one substrate ships and the other doesn't.
