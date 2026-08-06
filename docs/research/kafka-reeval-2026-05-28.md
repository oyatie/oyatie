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

Next available ADR number as of 2026-05-28: **ADR-0377** (ADR-0376 is the last filed).

Proposed: `docs/decisions/ADR-0377-migrate-event-backbone-kafka-to-pulsar.md`

> **Renumber note (2026-06-12, FRIC-1781390000):** the "next available" claim above was wrong
> when written — ADR-0377 had already been claimed on 2026-05-27 by
> `docs/decisions/ADR-0703-cas-cache-live-apex.md`. The amendment ADR proposed
> here was filed as a second ADR-0377 and has been renumbered to **ADR-0557**
> (`docs/decisions/ADR-0709-general-live-apex.md`) via the accounting-registry
> allocator (`--next-adr`). All ADR-0377 references in this document mean the kafka-to-pulsar
> decision now identified as ADR-0557.
