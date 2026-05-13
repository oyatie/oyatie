---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P04
title: Eventing Backbone + Outbox + Object Graph
status: stub
purpose: Ship the exactly-once outbox + Kafka-class topic registry + Object Graph entity upsert with engine-enforced row-level isolation.
---

# M01-P04 — Eventing Backbone + Outbox + Object Graph

## Purpose
Per ADR-0046 (exactly-once via outbox + Kafka) and ADR-0006 (Object Graph row-level isolation). Every axis publishes/consumes through this backbone.

## Acceptance
- `eventing.outbox.publish` row green at `stable`; exactly-once semantics demonstrated.
- `object-graph.entity.upsert` + 5 property tiers (`vector`, `timeseries`, `geo`, `ciphertext`, `struct`) green per ADR-0006..0112.
- Topic registry tracked; per-axis topic naming convention published.
- Kafka-compatible (provider-agnostic per Directive 4) — Kafka, Redpanda, Pulsar all listed as supported adapters.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Outbox + topic registry kernel | stub | [`IP-001-outbox-topic-registry.md`](IP-001-outbox-topic-registry.md) |
| IP-002 | Object Graph entity upsert + 5 property tiers | stub | [`IP-002-object-graph-property-tiers.md`](IP-002-object-graph-property-tiers.md) |
| IP-003 | Kafka adapter + Redpanda adapter + Pulsar adapter (provider-agnostic) | stub | [`IP-003-eventing-adapters.md`](IP-003-eventing-adapters.md) |

## Estimated parallelism
3 agents; IP-001 + IP-002 disjoint; IP-003 fans out to 3 sub-agents per provider adapter.

## Symbols-touched
`crates/oya-platform-eventing-{kernel,domain,app,worker,adapter-{kafka,redpanda,pulsar}}-*`, `crates/oya-platform-object-graph-{kernel,domain,app,api,vector,timeseries,geo,ciphertext,struct}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P04 complete: outbox + topic registry + Object Graph 5 tiers; eventing provider-agnostic" -i critical -k "M01,P04,eventing,object-graph,complete"
```
