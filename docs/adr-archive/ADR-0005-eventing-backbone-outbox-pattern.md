---
id: ADR-0005
status: Superseded
doc_status: published
superseded_by: [ADR-0557]
amended_by: [ADR-0350]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0005: Eventing backbone on Apache Kafka with outbox pattern, CloudEvents envelope, and per-tenant per-cell partitioning

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** ADR-0557 (supersedes-in-part: the Kafka-as-canonical-substrate clause; streaming-semantics decisions carry forward under Pulsar + KoP)
> **Owner:** `foundry` (eventing kernel) + `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0009, ADR-0011, ADR-0013

---

## Context

The cohesion thesis (ADR-0001), the audit chain (ADR-0003), and the plane-separation invariants (ADR-0004) together require a single eventing backbone that every axis writes to and reads from. Each cross-microservice contract row in DESIGN §10 — Tenant updates, IAM mutations, capability invocations, audit-chain emissions, billing events, DSR cascades, search-index lifecycle, ad-targeting decisions — flows through events. Without a single backbone, each axis selects its own broker, each integration becomes bespoke, the outbox-pattern guarantees diverge, and the audit chain cannot reliably correlate cross-microservice flows.

License posture (ADR-0013) constrains the choice. Confluent Community License, Redpanda BSL, and AWS-FSL are all `requires-review` or `forbidden` for product code; the only strictly Apache-2 broker that meets the scale + ecosystem bar is Apache Kafka itself. The PRD §3.1 commitment to optimization-built-in (`outbox + Kafka` per the toolchain manifest) and the prevention-doctrine commitment to single-source-of-truth integration patterns both push toward one canonical backbone.

---

## Decision

We adopt **Apache Kafka** as the single eventing backbone, **outbox pattern** for transactional event emission, **CloudEvents 1.0** as the envelope, **Protobuf** as the payload format, and **schema registry** for compatibility evolution. Per-tenant + per-cell partitioning is the default partition key.

### Backbone

- Broker: Apache Kafka (Apache-2.0, license-clean per ADR-0013).
- KRaft mode (no ZooKeeper); per-cell broker pool sized for the cell's data-plane fan-out.
- Per-axis topic conventions: `oya.<axis>.<surface>.<event-class>.v<n>`.

### Outbox pattern

Every transactional state change that emits an event writes the event into an `outbox` table inside the *same* DB transaction as the state change. A separate poller (or per-microservice CDC connector) ships outbox rows to Kafka. This guarantees at-least-once delivery without requiring a distributed transaction across the DB and the broker.

```rust
// crates/oya-eventing-kernel
pub struct OutboxRow {
    pub event_id: EventId,                     // ULID, monotonic per partition
    pub tenant_shard: TenantId,
    pub aggregate_id: AggregateId,
    pub envelope: CloudEventsEnvelope,
    pub payload: Vec<u8>,                       // Protobuf-encoded
    pub schema_id: SchemaId,                    // schema registry pointer
    pub status: OutboxStatus,                   // Pending | Shipped | Acked | Failed
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub partition_key: PartitionKey,            // (tenant_shard, cell_id)
}

pub struct CloudEventsEnvelope {
    pub specversion: String,                   // "1.0"
    pub id: String,
    pub source: String,                        // crate + surface
    pub type_: String,                         // event class
    pub subject: Option<String>,               // tenant + aggregate
    pub time: chrono::DateTime<chrono::Utc>,
    pub datacontenttype: String,               // "application/protobuf"
    pub dataschema: String,                    // schema-registry URL
    pub extensions: BTreeMap<String, String>,  // e.g. data_classes_touched, regulatory_packs_consumed
}
```

### Per-tenant + per-cell partitioning

Partition key = `(tenant_shard, cell_id)`. This:

1. Preserves per-tenant ordering (audit-chain replay works).
2. Confines each cell's blast radius — a noisy cell does not starve other cells (ADR-0009).
3. Aligns with audit-chain shards (ADR-0003) one-to-one.

Per-axis fan-out adds a secondary topic with an axis-specific partitioner (e.g. ads auction events partition by campaign for hot-key spreading).

### CloudEvents + Protobuf + schema registry

- CloudEvents 1.0 envelope is mandatory for cross-microservice topics; per-microservice-internal topics may use raw Protobuf if they declare so in catalog.
- Protobuf payload schemas live under `contracts/eventschemas/<axis>/<event>.proto`.
- Schema registry is in-house (`crates/oya-eventing-schema-registry-*`); license posture forbids Confluent Schema Registry for production.
- Compatibility rule: backward + transitive (consumers MUST tolerate older payloads).

### Per-event audit emission

Every event that crosses an axis boundary, touches a regulated data class (per ADR-0008), or invokes a capability (per ADR-0007) carries the `data_classes_touched` and `regulatory_packs_consumed` CloudEvents extensions. The audit-chain emitter (ADR-0003) consumes these directly; missing extensions on a cross-microservice topic are a CI failure.

### Topic governance

- Topic creation goes through a catalog PR (`registry/topics/<topic>.yaml`) — not via the broker admin API.
- Topic schema changes go through `oya-governance-event-schema` validator (forward + backward + transitive compat).
- Per-microservice-team owns its own topics; cross-microservice topics are co-owned per `RACI-OWNERSHIP.md`.

### Boundary

- Applies to: every event that crosses an axis boundary, every audit emission, every cross-cell coordination message.
- Does not apply to: in-process channels (tokio mpsc), in-cell synchronous RPC, build-time messaging.

---

## Consequences

### Positive

- One broker, one schema registry, one envelope — drift across axes becomes mechanically impossible.
- Outbox pattern makes "lost event after DB commit" impossible without losing the DB itself.
- Per-tenant + per-cell partitioning aligns naturally with cell architecture (ADR-0009) and audit-chain shards (ADR-0003).
- Apache-2 license posture clean (ADR-0013).
- CloudEvents envelope makes external consumer (regulator, auditor, third-party) integration straightforward.

### Negative

- Apache Kafka operability is non-trivial (per-cell broker pool sizing, partition rebalancing, KRaft cluster ops). Mitigation: managed Kafka via cloud microservice; per-cell isolation reduces blast radius.
- Outbox poller adds 100–500 ms of typical latency between DB commit and broker visibility; cross-microservice sagas account for this.
- Protobuf-only forces non-Rust consumers to compile generated bindings; mitigation: SDK gen ships per ADR-0011 cross-microservice contract registry.

### Operational

- On-call: per-cell broker SLO + per-topic consumer-lag SLO; alerts on `EVT-OUTBOX-POLLER-LAG > 30s` and `EVT-CONSUMER-LAG > regulator-bound`.
- Runbooks: `runbooks/outbox-poller-recovery.md`, `runbooks/topic-schema-rollback.md`, `runbooks/per-cell-broker-failover.md`.
- CI: `oya-governance-event-schema` (compat), `oya-governance-eventing-cohesion` (every cross-microservice event must declare audit extensions).
- DR: per-region broker mirror + cross-region MirrorMaker2 for residency-class `cross_region_replicated`.

---

## Alternatives considered

### Alternative A — Redpanda

- **Pros:** lower operational footprint; same Kafka API.
- **Cons:** BSL licensing post-2023; per-cluster license fees; ADR-0013 forbids BSL in product code.
- **Rejected because:** license posture.

### Alternative B — NATS JetStream

- **Pros:** Apache-2; smaller footprint; Rust-friendly.
- **Cons:** weaker exactly-once / outbox-pattern story than Kafka; smaller external ecosystem (KR-vendor + EU-vendor familiarity is Kafka-shaped).
- **Rejected because:** the cohesion + audit-chain integration relies on partition ordering guarantees Kafka has battle-tested at our target scale.

### Alternative C — Kafka + Confluent Schema Registry

- **Pros:** mature schema-registry product.
- **Cons:** Confluent Community License is `requires-review`; cleaner to ship in-house registry.
- **Rejected because:** ADR-0013 + sovereignty.

### Alternative D — Apache Pulsar

- **Pros:** Apache-2; per-tenant native multi-tenancy.
- **Cons:** smaller community; per-cell broker pool stories less mature; Bookkeeper operability adds a moving part.
- **Rejected because:** Kafka's operability tooling is more mature in the relevant ecosystems.

---

## Open questions

1. **Q1.** Per-axis topic naming convention vs per-domain — `oya.cloud.iam.role-published.v1` or `oya.iam.role-published.v1`? Default: per-microservice prefix (cohesion). → ADR-0011.
2. **Q2.** Per-cell broker pool vs per-region broker pool — does each cell get its own brokers, or do cells share a per-region pool with cell-keyed partitions? Default: per-region pool with cell-keyed partitions; per-cell brokers only for sovereign-isolation tenants. → ADR-0009.
3. **Q3.** Cross-region replication for `cross_region_replicated` residency: MirrorMaker2 vs in-house Rust replicator? Default: MirrorMaker2 initially; in-house when scale demands. → owner: `cloud`.
4. **Q4.** High-frequency data-plane events (ad serving, search query): are they on this backbone or on a dedicated low-latency path? Default: dedicated per-microservice low-latency adapter feeds an aggregate emission to this backbone. → ads-axis ADR + search-axis ADR.

---

## References

- `docs/DESIGN.md` §10 (cross-microservice contract `Eventing backbone (outbox + Kafka topic)`)
- `docs/TOOLCHAIN.md` §3 (Kafka per ADR-0050 — Apache 2.0; CloudEvents + Protobuf + schema registry)
- `docs/PRIVACY-PROGRAM.md` §2.2.4 layer 3 (singleton source services for ads/analytics topics)
- ADR-0001 (cohesion), ADR-0003 (audit chain — consumes events), ADR-0004 (plane separation — events as cross-plane mechanism), ADR-0009 (cell architecture — partition key), ADR-0011 (cross-microservice contract registry — schemas), ADR-0013 (license policy — Apache-2 selection rationale)
- CloudEvents 1.0 spec (https://cloudevents.io/), Apache Kafka KIP catalog
