# Kafka Re-Evaluation — 2026-05-28

## Current status

Apache Kafka is the canonical eventing backbone per **ADR-0005** (Accepted).
It covers: outbox pattern, CloudEvents 1.0 envelope, Protobuf payload, per-tenant/per-cell partitioning.
ADR-0005 explicitly rejected Apache Pulsar at the time ("Kafka's operability tooling is more mature").

## ADR-0195 supersedes for stream-processing lane

**ADR-0195** (Accepted 2026-05-18) establishes the stream-processing tier:

- **Log-broker substrate = Apache Pulsar 4.2.x** (Apache-2.0)
- ClickHouse `Kafka` engine connects via **Pulsar's Kafka-on-Pulsar (KoP) proxy** — no native Kafka broker required for the stream-processing path
- Apache Flink escalation path also connects via KoP or Pulsar native connector
- Kafka Streams explicitly **REJECTED** (JVM-only, clashes with Rust-first fleet)

ADR-0195 does not amend ADR-0005 — the two decisions currently coexist, creating a split:
- ADR-0005: "deploy Apache Kafka brokers"
- ADR-0195: "log-broker substrate is Pulsar; use KoP for Kafka-wire-compatible consumers"

## Assessment

ADR-0195's Pulsar adoption effectively makes a standalone Kafka broker cluster **redundant** for all stream-processing workloads (the dominant use case). Pulsar's KoP proxy provides full Kafka wire-protocol compatibility, so any consumer written against the Kafka API continues to work against Pulsar without code changes.

Standing up a separate Apache Kafka cluster alongside Pulsar would mean:
1. Two separate broker clusters to operate (doubles ops surface)
2. ADR-0195's ClickHouse Kafka Engine and Flink connectors already point at Pulsar/KoP
3. Hyperscaler-lens filter: redundant substrate with overlapping capability = not justified

## Recommendation: migrate ADR-0005 to Pulsar + KoP; retire standalone Kafka

**Action required: new ADR (ADR-0377 or next available) to amend ADR-0005.**

The amendment should:
1. Replace "Apache Kafka brokers" with "Apache Pulsar 4.2.x as the single log-broker substrate"
2. Declare Pulsar KoP proxy as the Kafka-wire-compatible access path (zero consumer code changes)
3. Retire the standalone Kafka broker deployment from all microservice manifests
4. Retain outbox pattern, CloudEvents 1.0 envelope, Protobuf payload, per-tenant/per-cell partitioning — these are broker-agnostic

No new ADR is needed for the doctrine check itself; this doc serves as the research basis for the amendment ADR.

## Open ADR numbers

Original proposed ADR number was **ADR-0377** (ADR-0376 was the last filed on 2026-05-28); AC-0.3 later renumbered the Kafka/Pulsar decision to **ADR-0520** because ADR-0377 is assigned to the Forgejo board projection decision.

Accepted: `docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md`
