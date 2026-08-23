---
doc_status: published
---

# Team: Platform — Eventing & Object Graph

## Mission
This team owns the eventing backbone (outbox pattern, Kafka topic contracts, event-schema registry) and the Object Graph property-tier system that underlies SaaS tenant data modeling. It exists to give every axis a reliable, schema-governed, partition-safe event bus and to ensure Object Graph property tiers are consistent with the Data Use Boundary. It does **not** own the audit chain (which has its own append path), the tenant kernel, or per-axis business-event logic.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (SaaS primary, consumed by all axes)
- **Surfaces:**
  - `platform-eventing-kernel` — `EventEnvelope`, `TopicName`, `PartitionKey`, `OutboxEntry`, `EventSchema`
  - `platform-eventing-app` — outbox relay, topic provisioning, schema registration
  - `platform-eventing-adapter-kafka` — Kafka producer/consumer adapters
  - `platform-eventing-api` — topic management REST/gRPC surface
  - `platform-object-graph-kernel` — `OgNode`, `OgEdge`, `PropertyTier`, `OgSchema` (ADR-0006..0112)
  - `platform-object-graph-domain` — property-tier lifecycle, schema evolution use-cases
- **Cross-axis contracts (DESIGN §10):**
  - `Eventing backbone (outbox + Kafka topic)` (owner) — topic shape changes require cross-axis review
  - `Object Graph property tier` (co-owner with `platform-privacy-dub`) — tier changes trigger Data Use Boundary check
- **Catalog records:** `crates/platform-eventing-*`, `crates/platform-object-graph-*`
- **Runbooks:** `runbooks/kafka-topic-provisioning.md`, `runbooks/outbox-relay-lag.md`, `runbooks/og-schema-rollback.md`
- **ADRs:** ADR-0006..0112 (Object Graph property tiers)

## In-scope work
- Outbox pattern implementation for all axes (transactional write + relay)
- Kafka topic contract ownership: schema, partition strategy, retention policy
- Event-schema registry: version control, compatibility enforcement (backward/forward)
- Topic provisioning as code: every new topic requires a schema record and CI gate
- Object Graph node/edge/property-tier lifecycle
- Property-tier classification (which tier → which `DataClass` in Data Use Boundary)
- OG schema evolution with backward-compatible migration policy
- Backpressure and admission control on Kafka consumers
- Partition key conventions for tenant isolation
- Fitness function `governance-eventing-topic` — rejects topic shape changes without schema registration

## Out-of-scope (anti-scope)
- Audit chain append (→ `platform-audit-evidence` — separate append path)
- Business-event definitions for each axis (each axis defines its events; this team owns the envelope)
- Search index ingestion (→ `axis-search`)
- Ads event ingestion for attribution (→ `axis-ads-analytics`)
- DSR cascade orchestration (→ `platform-privacy-dub`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | `TenantId` for partition keying | Per-schema-change |
| `platform-privacy-dub` | `DataClass` mapping for OG property tiers | ADR lifecycle |
| `ops-sre-reliability` | Kafka cluster SLOs, consumer-lag alerting runbooks | Quarterly |
| `axis-cloud` | Kafka broker hosting in cloud cells | Wave gate |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All 7 axes | Outbox relay, Kafka producer/consumer adapters, event envelopes | Per-release |
| `platform-audit-evidence` | Kafka topic for cross-axis audit fanout (analytics-plane reads) | Per-release |
| `axis-search` | OG property-tier schema for search-indexable classification | Monthly |
| `axis-ads-analytics` | Event stream for ad attribution (via privacy-gated topic) | Wave gate |
| `axis-foundry` | Event envelope for capability invocation telemetry | Per-release |

## Success metrics
- **Outbox relay lag p99:** < 500 ms (data-plane target)
- **Kafka consumer-lag alert response time:** < 15 min
- **Event-schema compatibility violations at merge:** 0 (fitness gate enforces)
- **OG property-tier coverage of all registered OG nodes:** 100%
- **Topic provisioning without schema record:** 0 (fitness gate)
- **Outbox relay availability:** ≥ 99.9%

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) for topic-shape contract disputes
- Privacy: privacy council for OG property-tier → data-class disputes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — consumer-lag review, schema-change queue
- Cross-team review: participates in monthly cross-axis contract audit

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Kafka consumer lag spikes cross-tenant due to shared partition | High | Per-tenant partition key convention; capacity planning with `ops-dr-capacity` |
| Event-schema backward-incompatible change breaks consumers | High | Schema registry compatibility gate; CI rejects incompatible schemas |
| OG property tier misclassified → PHI enters search index | Catastrophic | `governance-data-use-boundary` CI gate; co-owned with `platform-privacy-dub` |

## Sources scanned
DESIGN.md §10 (eventing backbone row, OG property tier row), PRD.md §5 (cohesion thesis OG reference), ADR-0006..0112, DOC-CATALOG.md §2.1.
