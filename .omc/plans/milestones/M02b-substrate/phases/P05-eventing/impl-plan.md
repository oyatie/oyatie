---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P05-eventing
impl_plan_id: IP-P05-eventing-substrate
status: pending
owner: council-architecture
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Delivers the complete Eventing substrate: 16 crates across 3 BCs (outbox, topics, subscriptions), Postgres outbox table per µservice convention, LISTEN/NOTIFY poller, Kafka KRaft publisher adapter (Apache-2.0 `rdkafka` crate)."
execution_variant: merge-into-existing-crates
execution_variant_decided_at: 2026-05-17
execution_variant_decided_by: user-directive-option-2
execution_variant_note: "User chose merge-variant 2026-05-17 — the 16-crate FROM-SCRATCH scaffold below is preserved as reference; deltas land incrementally into existing oya-eventing-{domain,file-adapter} crates. Tracking: F-M02B-PLAN-LIVE-CRATE-RECONCILIATION. First delta landed: CloudEvent + CloudEventError added to oya-eventing-domain::cloud_event module (2026-05-17), encoding the CloudEvents 1.0 spec required-attribute set + tenant-id classification. Remaining deltas (sealed Outbox port traits, Postgres outbox DDL with LISTEN/NOTIFY, Kafka KRaft adapter, topic registry BC, subscription BC, Protobuf event schemas) tracked under the reconciliation FixupTask as separate slices."
---
# IP-P05-eventing-substrate: Scaffold 16 eventing crates with outbox dispatcher, Kafka KRaft adapter, CloudEvents framing

## Intent

Delivers the complete Eventing substrate: 16 crates across 3 BCs (outbox, topics, subscriptions), Postgres outbox table per µservice convention, LISTEN/NOTIFY poller, Kafka KRaft publisher adapter (Apache-2.0 `rdkafka` crate), CloudEvents 1.0 framing, dead-letter queue, per-tenant per-cell topic ACL, sub-second propagation load test.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-eventing-outbox-kernel/Cargo.toml` | create | OutboxRecordStore + OutboxDispatchPort port traits |
| `crates/oya-eventing-outbox-kernel/src/types.rs` | create | OutboxId, OutboxRecord, Topic, KafkaOffset, CloudEvent |
| `crates/oya-eventing-outbox-kernel/src/ports.rs` | create | OutboxRecordStore + OutboxDispatchPort sealed traits |
| `crates/oya-eventing-topics-kernel/Cargo.toml` | create | TopicAdminPort trait |
| `crates/oya-eventing-topics-kernel/src/ports.rs` | create | TopicAdminPort sealed trait; TopicName type |
| `crates/oya-eventing-subscriptions-kernel/Cargo.toml` | create | SubscriptionRegistry trait |
| `crates/oya-eventing-outbox-domain/src/outbox.rs` | create | OutboxRecord invariants; CloudEvents framing logic |
| `crates/oya-eventing-outbox-domain/src/cloudevents.rs` | create | CloudEvent 1.0 struct; serialize/deserialize; topic naming convention |
| `crates/oya-eventing-outbox-application/src/dispatch.rs` | create | DispatchUseCase: fetch unpublished → publish → mark published; dead-letter on max retry |
| `crates/oya-eventing-topics-application/src/ensure_topic.rs` | create | EnsureTopicUseCase: idempotent topic creation |
| `crates/oya-eventing-outbox-adapter/src/postgres.rs` | create | PgOutboxAdapter: sqlx queries against <bc>_outbox tables |
| `crates/oya-eventing-outbox-adapter/src/kafka.rs` | create | KafkaProducerAdapter: rdkafka producer; CloudEvents serialization; ACK confirmation |
| `crates/oya-eventing-topics-adapter/src/kafka_admin.rs` | create | KafkaAdminAdapter: rdkafka AdminClient; create/delete topics with ACL |
| `crates/oya-eventing-subscriptions-adapter/src/kafka_consumer.rs` | create | KafkaConsumerAdapter: rdkafka consumer group; at-least-once delivery |
| `crates/oya-eventing-worker/src/dispatcher.rs` | create | OutboxDispatcherWorker: LISTEN/NOTIFY + polling fallback; Tokio JoinSet per tenant |
| `crates/oya-eventing-worker/src/dead_letter.rs` | create | DeadLetterWorker: retry exhausted outbox rows → dead-letter topic |
| `crates/oya-eventing-rest/src/routes.rs` | create | GET /eventing/v1/topics, POST /eventing/v1/topics, GET /eventing/v1/subscriptions |
| `crates/oya-eventing-app/src/main.rs` | create | composition root |
| `migrations/eventing/V001__eventing_init.sql` | create | full DDL |
| `contracts/eventing/eventing.proto` | create | Protobuf schema |
| `policy/eventing/eventing.cedar` | create | Cedar policy |
| `tests/load/smoke-eventing-outbox.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 16 eventing crates |

---

## Crate Naming

```
NAME: oya-eventing-outbox-kernel
JUSTIFICATION:
- microservice = eventing: event-streaming substrate; Kafka KRaft outbox bridge
- bc-tokens = outbox: Postgres outbox BC; distinct from topics and subscriptions
- layer = kernel: OutboxRecordStore + OutboxDispatchPort port traits; zero I/O
- exemptions claimed: none
```

---

## Code Shape

### `migrations/eventing/V001__eventing_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS eventing;

-- Topic registry (per-tenant per-cell; naming: {tenant_ctx}.{µservice}.{event_type}.{version})
CREATE TABLE eventing.topics (
    topic_id     uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    uuid    NOT NULL,
    topic_name   text    NOT NULL,
    microservice text    NOT NULL,
    event_type   text    NOT NULL,
    version      int     NOT NULL DEFAULT 1,
    partitions   int     NOT NULL DEFAULT 3,
    replication  int     NOT NULL DEFAULT 1,
    created_at   timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE eventing.topics ENABLE ROW LEVEL SECURITY;
ALTER TABLE eventing.topics FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON eventing.topics
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_topics_name
    ON eventing.topics (tenant_id, topic_name);

-- Subscription registry (consumer group → topic mapping)
CREATE TABLE eventing.subscriptions (
    subscription_id  uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid    NOT NULL,
    topic_name       text    NOT NULL,
    consumer_group   text    NOT NULL,
    handler_microservice text NOT NULL,
    enabled          bool    NOT NULL DEFAULT true,
    created_at       timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE eventing.subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE eventing.subscriptions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON eventing.subscriptions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_subscriptions_unique
    ON eventing.subscriptions (tenant_id, topic_name, consumer_group);

-- Dead-letter queue
CREATE TABLE eventing.dead_letter (
    dead_letter_id   uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid    NOT NULL,
    original_outbox_id uuid  NOT NULL,
    topic            text    NOT NULL,
    payload          jsonb   NOT NULL,
    failure_reason   text    NOT NULL,
    retry_count      int     NOT NULL DEFAULT 0,
    created_at       timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE eventing.dead_letter ENABLE ROW LEVEL SECURITY;
ALTER TABLE eventing.dead_letter FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON eventing.dead_letter
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
```

### `crates/oya-eventing-outbox-domain/src/cloudevents.rs`

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// CloudEvents 1.0 spec envelope (https://cloudevents.io)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    pub specversion: &'static str,   // always "1.0"
    pub id: Uuid,                    // unique per event; idempotency key
    pub source: String,              // "//oyatie/{microservice}"
    #[serde(rename = "type")]
    pub event_type: String,          // "{tenant_ctx}.{µservice}.{event_type}.v{version}"
    pub datacontenttype: &'static str, // always "application/json"
    pub time: DateTime<Utc>,
    pub data: serde_json::Value,
    // oyatie extensions
    pub oyatie_tenant_id: uuid::Uuid,
    pub oyatie_cell_id: String,
}

impl CloudEvent {
    /// Construct CloudEvent and enforce topic naming convention:
    /// `{tenant_context}.{microservice}.{event_type}.v{version}`
    pub fn new(
        tenant_id: uuid::Uuid,
        cell_id: impl Into<String>,
        microservice: &str,
        event_type: &str,
        version: u32,
        payload: serde_json::Value,
    ) -> Self {
        let topic = format!("t.{tenant_id}.{microservice}.{event_type}.v{version}");
        Self {
            specversion: "1.0",
            id: Uuid::new_v4(),
            source: format!("//oyatie/{microservice}"),
            event_type: topic,
            datacontenttype: "application/json",
            time: Utc::now(),
            data: payload,
            oyatie_tenant_id: tenant_id,
            oyatie_cell_id: cell_id.into(),
        }
    }
}
```

### `crates/oya-eventing-outbox-adapter/src/kafka.rs`

```rust
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use oya_eventing_outbox_kernel::ports::{OutboxDispatchPort, KafkaOffset};
use oya_eventing_outbox_kernel::types::{OutboxRecord, TenantId};

pub struct KafkaProducerAdapter {
    producer: FutureProducer,
}

impl KafkaProducerAdapter {
    pub fn new(brokers: &str) -> anyhow::Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("enable.idempotence", "true")   // exactly-once at producer level
            .set("acks", "all")                   // wait for all ISR replicas
            .create()?;
        Ok(Self { producer })
    }
}

#[async_trait::async_trait]
impl OutboxDispatchPort for KafkaProducerAdapter {
    async fn publish(&self, tenant_id: TenantId, record: &OutboxRecord)
        -> Result<KafkaOffset, oya_eventing_outbox_kernel::EventingError>
    {
        let payload = serde_json::to_vec(&record.payload)
            .map_err(|e| oya_eventing_outbox_kernel::EventingError::Serialization(e.to_string()))?;
        let delivery = self.producer
            .send(
                FutureRecord::to(&record.topic)
                    .key(&record.key)
                    .payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| oya_eventing_outbox_kernel::EventingError::Kafka(e.to_string()))?;
        Ok(KafkaOffset { partition: delivery.0, offset: delivery.1 })
    }
}
```

### `contracts/eventing/eventing.proto`

```proto
syntax = "proto3";
package oyatie.eventing.v1;

message OutboxEventPublished {
    string tenant_id    = 1;
    string outbox_id    = 2;
    string topic        = 3;
    string key          = 4;
    int64  kafka_offset = 5;
    int32  partition    = 6;
    int64  timestamp_ms = 7;
}

message DeadLettered {
    string tenant_id          = 1;
    string original_outbox_id = 2;
    string topic              = 3;
    string failure_reason     = 4;
    int32  retry_count        = 5;
    int64  timestamp_ms       = 6;
}
```

### `tests/load/smoke-eventing-outbox.js`

```javascript
import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export const options = {
  vus: 100, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8083';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

export default function () {
  // Simulate outbox write (domain mutation endpoint that internally enqueues outbox)
  const res = http.post(`${BASE_URL}/eventing/v1/test/publish`, JSON.stringify({
    topic: `t.${TENANT_ID}.test.load_test.v1`,
    key: uuidv4(),
    payload: { test: true },
  }), {
    headers: { 'Content-Type': 'application/json', 'X-Tenant-Id': TENANT_ID },
  });
  check(res, { 'publish 202': (r) => r.status === 202 });
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-eventing-outbox-kernel --all-features    # exit 0
cargo check -p oya-eventing-outbox-adapter --all-features   # exit 0
cargo clippy --workspace --all-features -- -D warnings       # exit 0
cargo nextest run --workspace --all-features                 # exit 0
psql $DATABASE_URL -f migrations/eventing/V001__eventing_init.sql  # exit 0
# CloudEvents framing
cargo nextest run -p oya-eventing-outbox-domain --test cloudevents_framing  # exit 0
# Topic naming convention
cargo nextest run -p oya-eventing-topics-domain --test topic_naming_pattern  # exit 0
# Dead-letter routing
cargo nextest run -p oya-eventing-outbox-application --test dead_letter_routing  # exit 0
# Load test
k6 run tests/load/smoke-eventing-outbox.js --env BASE_URL=http://localhost:8083
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_cloud_event_topic_naming` | `t.{tenant_id}.{µservice}.{event}.v{N}` format enforced |
| `test_cloud_event_serialize_deserialize` | CloudEvent 1.0 round-trip |
| `test_outbox_record_dead_letter_after_max_retry` | After 3 retries → dead-letter |
| `test_kafka_producer_exactly_once_config` | `enable.idempotence=true`, `acks=all` set |
| `test_topic_name_per_tenant_acl` | Topic name embeds tenant_id for ACL enforcement |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_outbox_to_kafka_round_trip` | Insert outbox row → worker dispatches → Kafka topic has message |
| `integration_sub_second_propagation` | outbox insert → Kafka publish latency <1000ms measured |
| `integration_dead_letter_after_kafka_failure` | Simulate Kafka failure → after retries → dead-letter row inserted |

---

## Clean Architecture Compliance

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-eventing-outbox-kernel` | `kernel` | nothing project-internal | all layers |
| `oya-eventing-outbox-domain` | `domain` | `outbox-kernel` | `adapter`, presentation |
| `oya-eventing-outbox-application` | `application` | `outbox-domain`, `outbox-kernel` | `adapter`, presentation |
| `oya-eventing-outbox-adapter` | `adapter` | `outbox-application`, `outbox-kernel` | presentation |
| `oya-eventing-worker` | `worker` | `*-application`, `*-kernel` | direct adapter |
| `oya-eventing-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-eventing-outbox.js --env BASE_URL=http://localhost:8083
# Pass: p99 ≤200ms; 0 errors at 100 VUs

# Sub-second propagation measurement
cargo nextest run -p oya-eventing-outbox-application --test sub_second_propagation -- --nocapture
# Pass: "outbox→kafka latency: Xms" where X < 1000
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P05-eventing: 16 crates + outbox + Kafka KRaft + CloudEvents" \
  --ttl 7200 \
  crates/oya-eventing-outbox-kernel/src/ports.rs::OutboxDispatchPort \
  crates/oya-eventing-outbox-adapter/src/kafka.rs::KafkaProducerAdapter \
  crates/oya-eventing-worker/src/dispatcher.rs::OutboxDispatcherWorker \
  migrations/eventing/V001__eventing_init.sql::eventing_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P05-eventing merged; 16 crates; Kafka KRaft Apache-2.0 (rdkafka); CloudEvents 1.0; dead-letter queue; sub-second propagation tested; next: P06-secrets/impl-plan" \
  -i high \
  -k "M02,P05,IP-P05,eventing"
```

---

## Next IP Pointer

`phases/P06-secrets/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Bominal ADR-0116 (superseded by ADR-0174: Kafka KRaft Apache-2.0 over Redpanda BSL)
