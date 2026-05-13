---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P05-eventing
status: Proposed
acceptance_lanes: []
entry_gate: |
  M01-P05 complete; oya-tenancy-kernel ships; Kafka KRaft cluster reachable
  in dev environment (docker-compose kafka kraft mode); cargo check exits 0.
exit_gate: |
  Outbox dispatcher worker publishes to Kafka KRaft; CloudEvents framing
  verified; per-tenant per-cell topic naming pattern enforced; dead-letter
  queue wired; k6 sub-second end-to-end event propagation test passes;
  grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite"
owner_team: council-architecture
---

# P05-eventing: Eventing substrate — outbox dispatcher, Kafka KRaft, CloudEvents framing, per-tenant per-cell partitioning

## Purpose

This phase delivers the complete Eventing substrate: the bridge between the Postgres outbox (write-side guarantee) and Kafka KRaft (fan-out mechanism). Per Bominal ADR-0116 (superseded by ADR-0174 selecting Apache Kafka KRaft GA 4.x over Redpanda due to BSL license incompatibility), every domain mutation writes transactionally to its `<bc>_outbox` table; the outbox dispatcher worker polls via `LISTEN/NOTIFY` and publishes to Kafka KRaft topics. Events use CloudEvents 1.0 framing. Topic naming follows `{tenant_context}.{microservice}.{event_type}.{version}` per the oyatie convention (adapted from Bominal). Per-tenant per-cell ACL isolation enforced at broker level. Sub-second end-to-end propagation is the performance target. Without Eventing no cross-product Workflow automation can function.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `eventing` | `outbox`, `topics`, `subscriptions` | `crates/oya-eventing-{outbox,topics,subscriptions}-{kernel,domain,application,adapter}/`, `crates/oya-eventing-worker/`, `crates/oya-eventing-rest/`, `crates/oya-eventing-app/` | 3×4 + 1 worker + 1 rest + 1 app = 16 crates |

Naming justification:

```
NAME: oya-eventing-outbox-kernel
JUSTIFICATION:
- microservice = eventing: the event-streaming substrate; Kafka KRaft outbox bridge
- bc-tokens = outbox: the Postgres-outbox BC; distinct from topics (Kafka topic
  management) and subscriptions (consumer group registration)
- layer = kernel: OutboxDispatchPort + OutboxRecord types; zero I/O
- exemptions claimed: none

NAME: oya-eventing-worker
JUSTIFICATION:
- microservice = eventing: same µservice
- bc-tokens = (none): single dispatcher worker binary; ADR-0056 BC-optionality
- layer = worker: long-running outbox poller + Kafka publisher; Tokio JoinSet
- exemptions claimed: none
```

### Out-of-scope

- Kafka cluster provisioning / Helm chart — owned by oya-cloud infra phase.
- Schema registry server — Confluent-compatible schema registry is a deployment
  concern; Protobuf schemas live in `contracts/`.
- MQTT IoT bridge — separate IoT substrate, not part of eventing substrate.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + OutboxDispatchPort + CloudEvents + Kafka producer adapter + dead-letter + load test | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P05-eventing
oya gate validate lean-a2 --phase P05-eventing
oya gate validate lean-a3 --phase P05-eventing
oya gate validate lean-a4 --phase P05-eventing
```

### Eventing integration gates

```bash
# CloudEvents framing round-trip
cargo nextest run -p oya-eventing-outbox-domain --test cloudevents_framing  # exit 0
# Topic naming convention enforced
cargo nextest run -p oya-eventing-topics-domain --test topic_naming_pattern  # exit 0
# Dead-letter queue wired
cargo nextest run -p oya-eventing-outbox-application --test dead_letter_routing  # exit 0
```

### Load test gate (sub-second propagation)

```bash
k6 run tests/load/smoke-eventing-outbox.js --env BASE_URL=http://localhost:8083
# Pass: p99 outbox-insert → Kafka-publish latency ≤1000ms; 0 message loss at 10k events/min
vegeta attack -rate=5000/s -duration=30s -targets=tests/load/eventing-targets.txt | vegeta report
# Pass: p99 ≤200ms on outbox write endpoint
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-eventing-outbox-kernel` | `kernel` | Yes — `OutboxDispatchPort`, `OutboxRecordStore` | N/A | No |
| `oya-eventing-topics-kernel` | `kernel` | Yes — `TopicAdminPort` | N/A | No |
| `oya-eventing-subscriptions-kernel` | `kernel` | Yes — `SubscriptionRegistry` | N/A | No |
| `oya-eventing-outbox-adapter` | `adapter` | N/A | Yes — `KafkaProducerAdapter`, `PgOutboxAdapter` | No |
| `oya-eventing-worker` | `worker` | N/A | No direct adapter import | No |
| `oya-eventing-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-eventing-outbox-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait OutboxRecordStore: Send + Sync + sealed::Sealed {
    /// Write outbox record transactionally inside caller's DB transaction.
    async fn enqueue(&self, tenant_id: TenantId, record: OutboxRecord)
        -> Result<OutboxId, EventingError>;
    /// Fetch unpublished records in insertion order for a given tenant+topic.
    async fn fetch_unpublished(&self, tenant_id: TenantId, topic: &Topic, limit: u32)
        -> Result<Vec<OutboxRecord>, EventingError>;
    /// Mark records as published after Kafka ack.
    async fn mark_published(&self, ids: &[OutboxId]) -> Result<(), EventingError>;
    /// Move to dead-letter after max_retries exceeded.
    async fn dead_letter(&self, id: OutboxId, reason: &str) -> Result<(), EventingError>;
}

#[async_trait::async_trait]
pub trait OutboxDispatchPort: Send + Sync + sealed::Sealed {
    /// Publish a CloudEvent-framed record to the Kafka topic.
    /// Topic naming: {tenant_context}.{microservice}.{event_type}.{version}
    async fn publish(&self, tenant_id: TenantId, record: &OutboxRecord)
        -> Result<KafkaOffset, EventingError>;
}

// oya-eventing-topics-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait TopicAdminPort: Send + Sync + sealed::Sealed {
    /// Create per-tenant per-cell Kafka topic with ACL isolation.
    async fn ensure_topic(&self, tenant_id: TenantId, microservice: &str,
        event_type: &str, version: u32) -> Result<TopicName, EventingError>;
    async fn delete_topic(&self, topic: &TopicName) -> Result<(), EventingError>;
    async fn list_topics(&self, tenant_id: TenantId) -> Result<Vec<TopicName>, EventingError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P05-eventing` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P05-eventing` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P05-eventing` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P05-eventing` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `outbox` | `eventing` | pending |
| `topics` | `eventing` | pending |
| `subscriptions` | `eventing` | pending |

---

## Grit Claim Symbols

```
crates/oya-eventing-outbox-kernel/src/ports.rs::OutboxDispatchPort
crates/oya-eventing-topics-kernel/src/ports.rs::TopicAdminPort
crates/oya-eventing-outbox-adapter/src/kafka.rs::KafkaProducerAdapter
crates/oya-eventing-worker/src/dispatcher.rs::OutboxDispatcherWorker
migrations/eventing/V001__eventing_init.sql::eventing_schema
contracts/eventing.proto::OutboxEventPublished
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P05-eventing started; scope: 16 crates (outbox/topics/subscriptions); Kafka KRaft Apache-2.0 (not Redpanda BSL); CloudEvents 1.0 framing; sub-second propagation SLA" \
  -i high \
  -k "M02,P05,phase-start,eventing"

icm store \
  -t context-oyatie \
  -c "Phase P05-eventing complete; outbox→Kafka pipeline verified; CloudEvents framing correct; topic naming enforced; p99≤200ms write; sub-second propagation; next: P06-secrets" \
  -i high \
  -k "M02,P05,phase-complete,eventing"
```

---

## References

- Bominal ADRs inherited: ADR-0116 (event streaming; Kafka KRaft over Redpanda per ADR-0174 supersession)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05
- unblocks: Wave-B Workflow engine (consumes Kafka topics), all product phases (publish via outbox)
