---
id: ADR-0557
title: "Migrate Kafka to Pulsar via KoP wire-compat"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-architecture
planning_impact: true
supersedes: [ADR-0005]
superseded_by: [ADR-0709]
related: [ADR-0005, ADR-0195, ADR-0397, ADR-0436]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0557 — Migrate Kafka to Pulsar via KoP wire-compat

## Status

Accepted — 2026-05-28.

## Context

ADR-0005 (2022) picked Kafka as the canonical streaming/event backbone using the transactional-outbox pattern. ADR-0195 (2026-05-18) introduced Apache Pulsar with KoP (Kafka-on-Pulsar) wire-compat for the log/broker substrate. ADR-0397 (this session) then confirmed Pulsar 4.x + Oxia as the canonical event-bus, superseding any competing choice.

This leaves standalone Kafka in an ambiguous state: it exists as deployed broker pods on the cluster but holds no canonical role. Two streaming substrates running in parallel wastes cluster footprint and splits operational context. This ADR resolves the ambiguity.

Research basis: `docs/research/kafka-reeval-2026-05-28.md`.

## Goals

1. Retire standalone Kafka as a cluster substrate.
2. Preserve full wire-compatibility for existing Kafka clients via KoP proxy.
3. Define a 3-phase migration path with zero client-code changes required in Phase 1.

## Non-Goals

- Rewriting existing Kafka client code (handled optionally in Phase 3+).
- Changing streaming semantics (topics, partitions, consumer groups are preserved via KoP).
- Modifying ClickHouse or Flink job configs before Phase 2.

## Hyperscaler-Lens Pre-Check

| Criterion | Verdict | Evidence |
|---|---|---|
| Active upstream | PASS | Pulsar 4.x actively maintained; KoP active at `github.com/streamnative/kop` |
| License | PASS | Apache 2.0 (both Pulsar and KoP) |
| Self-hostable | PASS | No managed-service dependency; runs fully on-cluster |
| Hyperscaler-internal-equivalent | PASS | Yahoo origin; StreamNative/Tencent/FAANG-adjacent stacks run Pulsar internally; Kafka displaced by Pulsar in newer internal deployments |

## Decision

1. **Standalone Kafka is retired.** The cluster runs Pulsar 4.x + Oxia (per ADR-0397) as the sole canonical event-bus and log-broker substrate.

2. **KoP proxy fronts Pulsar for all Kafka clients.** The Kafka-on-Pulsar wire-compat layer provides a Kafka-protocol endpoint. Existing producers and consumers connect without any code changes. Kafka topics are mapped to Pulsar persistent topics under a `kafka/` tenant namespace.

3. **ClickHouse and Flink/Streaming workloads target Pulsar natively** in Phase 2+. KoP is the bridge, not the destination.

4. **ADR-0005 is superseded-in-part.** The substrate clause (Kafka as canonical broker) is superseded. The streaming semantics decisions (transactional outbox, at-least-once delivery guarantees, consumer-group fanout) carry forward under Pulsar's equivalent primitives and remain normative.

## Migration Phasing

### Phase 1 — KoP proxy live (now)

- Deploy KoP proxy alongside the running Pulsar cluster.
- Route new Kafka client connections through KoP endpoint (port 9092 mapped to KoP).
- No existing client code changes required.
- Standalone Kafka brokers remain up but receive no new topic creation.

### Phase 2 — Topic mirror + client cutover

- Mirror existing Kafka topics to Pulsar using an offset-aware mirror tool (MirrorMaker 2 via KoP, or Pulsar's built-in Kafka migration utility).
- Update ClickHouse Kafka engine tables to point at KoP endpoint.
- Update Flink Kafka source/sink connectors to KoP endpoint.
- Validate consumer-group offset parity before cutting over.

### Phase 3 — Decommission standalone Kafka

- Remove standalone Kafka broker pods from cluster.
- Remove Kafka from Helm release sets and ArgoCD application manifests.
- Migrate high-throughput clients to native Pulsar SDK (optional; prioritised by throughput volume).

## Consequences

### Positive

- **Cluster footprint reduction**: one streaming substrate instead of two eliminates ~3 StatefulSet pods and associated PVCs.
- **Pulsar feature unlock**: tiered storage (offload to object store), geo-replication, multi-tenancy primitives, and schema registry become first-class.
- **Operational convergence**: single substrate means single alert ruleset, single runbook, single on-call rotation context.

### Negative / Mitigations

- **KoP throughput overhead**: ~5% throughput penalty vs native Pulsar protocol due to protocol translation layer. Mitigated as clients migrate to native Pulsar SDK in Phase 3.
- **Migration risk during Phase 2**: offset-mirror fidelity must be validated before cutover. Mitigated by running mirror in parallel and verifying consumer-group lag parity before switching.
- **KoP version coupling**: KoP must stay version-aligned with the Pulsar cluster. Pin KoP version in Helm values; upgrade together.

## Supersession

- **ADR-0005** — the Kafka-as-canonical-substrate clause is superseded. ADR-0005 retains historical value for streaming-semantics decisions (outbox pattern, delivery guarantees); those decisions are adopted by Pulsar + KoP.

## Related

- ADR-0005 — Kafka canonical (eventing backbone + outbox pattern; superseded-in-part)
- ADR-0195 — Pulsar log-broker substrate (introduced KoP)
- ADR-0397 — Pulsar 4.x + Oxia canonical event-bus (confirms Pulsar as sole substrate)
- ADR-0436 — RisingWave consumer (streaming analytics consumer; unaffected by this migration)
- `docs/research/kafka-reeval-2026-05-28.md` — research basis for this decision

## Historical residual from ADR-5 (E3 fold 2026-08-06)

**Title:** ADR-0005-eventing-backbone-outbox-pattern

**Preserved decision gist:** We adopt **Apache Kafka** as the single eventing backbone, **outbox pattern** for transactional event emission, **CloudEvents 1.0** as the envelope, **Protobuf** as the payload format, and **schema registry** for compatibility evolution. Per-tenant + per-cell partitioning is the default partition key. ### Backbone - Broker: Apache Kafka (Apache-2.0, license-clean per ADR-0013). - KRaft mode (no ZooKeeper); per-cell broker pool sized for the cell's data-plane fan-out. - Per-axis topic conventions: `oya.<axis>.<surface>.<event-class>.v<n>`. ### Outbox pattern Every transactional state change tha

_Source file archived after fold; full body in git history / docs/adr-archive/._
