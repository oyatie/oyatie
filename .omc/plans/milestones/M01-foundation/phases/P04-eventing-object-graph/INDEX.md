---
purpose: Ship the exactly-once outbox + Kafka-class topic registry + Ontology entity upsert with engine-enforced row-level isolation.
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P04
title: Eventing Backbone + Outbox + Object Graph
status: complete
purpose: Ship the exactly-once outbox + Kafka-class topic registry + Ontology entity upsert with engine-enforced row-level isolation.
phase_evidence_refs:
  - /evidence/foundation/m01-p04-ip-001-outbox-topic-registry.json
  - /evidence/foundation/m01-p04-ip-002-object-graph-property-tiers.json
  - /evidence/foundation/m01-foundation-acceptance-audit-2026-05-14.json
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
| IP-001 | Outbox + topic registry kernel | probe-green / acceptance-blocked | [`IP-001-outbox-topic-registry.md`](IP-001-outbox-topic-registry.md) |
| IP-002 | Object Graph entity upsert + 5 property tiers | probe-green / acceptance-blocked | [`IP-002-object-graph-property-tiers.md`](IP-002-object-graph-property-tiers.md) |
| IP-003 | Kafka adapter + Redpanda adapter + Pulsar adapter (provider-agnostic) | stub | [`IP-003-eventing-adapters.md`](IP-003-eventing-adapters.md) |

## Estimated parallelism
Controlled fanout only: IP-001 and IP-002 are disjoint, and both now have scoped
probe-green evidence while repository-wide `scripts/check.sh` now passes the restored helper-script preflight
and is blocked later by stale connect-domain imports exposed by cargo check outside this scoped ChangeSet; IP-003 follows after IP-001/IP-002 contract
surfaces are accepted or the shared repository-check blocker is waived/fixed.

## Symbols-touched
Live BNF v4.1 foundation anchors: `crates/oya-eventing-domain`, `crates/oya-eventing-application` (SUPERSEDED: stub orphan deleted per ADR-0106 §Consequences + audit #6 — canonical `-app` scaffold pending; reference retained for plan-integrity continuity), `crates/oya-eventing-file-adapter`, and `crates/oya-ontology-domain`. Provider-specific eventing adapters remain later fanout scope; M01 must not invent stale `oya-platform-*` crates to satisfy this plan.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P04 in-flight / acceptance-blocked: IP-001 and IP-002 have scoped probe-green evidence; IP-003 remains stub; scripts/check.sh now reaches cargo check, whose stale connect-domain imports prevent phase completion" -i high -k "M01,P04,eventing,object-graph,acceptance-blocked"
```
