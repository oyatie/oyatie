# IP-008: revocation-adapter-pulsar — high-priority partitioned fan-out

- Bounded context: revocation
- Layers: adapter (Pulsar specialization), app
- Crates:
  - `oya-consent-graph-revocation-adapter`
  - `oya-consent-graph-revocation-adapter-pulsar`
  - `oya-consent-graph-revocation-app`
- Acceptance status: ga
- Authority: ADR-0214 §2.4, ADR-0078 (Pulsar substrate), ADR-0064 (canonical-base neutrality —
  Pulsar is the only message broker across all packs).
- Depends on: `oya-consent-graph-revocation-{kernel, domain, usecase}`, `pulsar-rs = "6"`.

## 1. Goal

Implement the Pulsar publisher + subscriber for the real-time revocation propagation pipeline. This is
the single most latency-critical path in the µservice (per ADR-0214 §2.4 SLO: p99 ≤1s, p100 ≤3s).

## 2. Topic design

```
oya.consent-graph.revocation.v1
├── partitioned by (grantor_tenant_id, grantee_tenant_id) hash
├── 64 partitions per cluster (sized for 10K rev/s peak)
├── retention: 7 days
├── compaction: read_compacted=true on subscribe (most-recent-per-agreement-id wins)
├── messaging priority: high (Pulsar priority lane)
└── persistence: synced replication across 3 brokers; ack=quorum
```

Per-region instance: each Pulsar cluster has its own topic; cross-region propagation is via Pulsar
georeplication (mirror topology, not full mesh — see multi-region.md for details).

## 3. Publisher

```rust
pub struct RevocationPulsarPublisher {
    client: pulsar::Pulsar<TokioExecutor>,
    producer: pulsar::Producer<RevocationEventBytes, TokioExecutor>,
}

impl RevocationPublisher for RevocationPulsarPublisher {
    async fn publish(&self, event: &RevocationEvent) -> Result<PublishReceipt, PublishError> {
        let key = format!("{}:{}", event.grantor, event.grantee);
        let payload = serde_json::to_vec(event)?;
        let msg = MessageBuilder::new()
            .key(key)
            .properties([
                ("priority", "high"),
                ("revocation_id", event.revocation_id.to_string()),
                ("agreement_id", event.agreement_id.to_string()),
                ("deadline_unix_ms", event.deadline_at.unix_ms().to_string()),
            ])
            .payload(payload);
        let receipt = self.producer.send(msg).await?;
        Ok(PublishReceipt { sequence_id: receipt.sequence_id, broker_ack_ms: receipt.broker_ack_ms })
    }
}
```

Latency budget for publish-ack: ≤50ms p99.

## 4. Subscribers

Each downstream µservice that holds revocation-sensitive state runs a Pulsar subscriber on this topic.
**Subscription model**: shared subscription per (subscriber service, region); messages distributed
across pods. Each pod processes events in parallel.

Subscriber list (canonical, registered in `partner-directory`):
| Subscriber service | Region scope | Action on event |
|--------------------|--------------|-----------------|
| `consent-graph::enforcement-app` | all regions | `PolicyCache::invalidate(agreement_id)` |
| `consent-graph::projection-gateway-app` | all regions | destroy projection topic |
| `consent-graph::audit-bridge-app` | all regions | emit revocation-propagated audit event |
| `ontology::cross-tenant-projection-subscriber` | all regions | tombstone projection rows |
| `analytics::cross-tenant-feed-subscriber` | all regions | close grantee-side stream |
| `observability::cross-tenant-metric-stream-subscriber` | all regions | close metric stream |

A subscriber's `ConfirmReceipt` write to `consent_graph_revocation_receipts` is what closes the loop.

## 5. Cross-region propagation

Each region has its own Pulsar cluster. The revocation topic is geo-replicated via Pulsar's built-in
mirror feature:

```
us-east-1 cluster  ──georep──►  eu-west-1 cluster
                  ──georep──►  ap-northeast-2 cluster
                  ──georep──►  ap-southeast-1 cluster
                  ...
```

Georep lag SLO: p99 ≤500ms (within the overall 1s end-to-end revocation propagation budget).
This is enforced by `pulsar_geo_replication_lag_seconds` Prometheus metric + SLO
`pulsar-georep-lag-revocation-topic`.

## 6. Failure modes

| Failure | Detection | Response |
|---------|-----------|----------|
| Pulsar publish fails | publisher error | retry 3x with exp-backoff (50ms / 100ms / 200ms); on final fail, write to dead-letter table; alert fires |
| Subscriber pod crashes | subscription lag metric | Pulsar redelivers to other pod in shared subscription |
| Georeplication paused | `pulsar_geo_replication_paused` metric | alert + automated remediation via Pulsar admin API; deny-by-default in dest region until restored |
| Subscriber receipt slow | `revocation_propagation_seconds` p99 burn | page on 0.5s warning, alert on 1s breach |
| Duplicate delivery | message-id dedup | subscriber tracks last-seen seq per (rev_id, subscriber) → idempotent invalidation |

## 7. Backpressure

Pulsar subscriber is configured with:
- `messagePrefetch = 100`
- `maxConcurrentMessages = 50` per pod
- `negativeAckRedeliveryDelay = 100ms`

Under burst (1M revocations/s — DSAR cascade scenario), the system absorbs via:
- 64 partitions → 1.5M msg/s topic capacity.
- 50 concurrent × N pods per subscriber → linear scale.
- HPA scales pods on `pulsar_subscriber_consumer_unacked` > 1K.

## 8. Tests

- `publish_roundtrip` — publish + subscriber receives identical event.
- `publish_latency_p99_under_50ms` — 10K publishes, p99 ≤50ms.
- `priority_high_jumps_normal_queue` — interleaved normal + high → high arrives first.
- `subscriber_idempotent_on_duplicate` — replay same event → second invalidation is no-op.
- `cross_region_georep_lag_under_500ms` — measured via two-region integration suite.
- `dead_letter_on_permanent_publish_failure` — broker down 30s → event in DLQ.
- `shared_subscription_load_balance` — 3 pods, 100 events → ~33 each, no duplicates.

## 9. App composition

`revocation-app` wires:
- gRPC server (port 9444) for `OriginateRevocation` + `ConfirmReceipt`
- OutboxDrainer task (drains Postgres outbox → Pulsar)
- DeadlineReconciler task (every 100ms)
- Pulsar producer connection
- Health probes (`/readyz` requires Pulsar producer connected + Postgres connected)

The CacheInvalidator subscriber lives in `enforcement-app`'s sibling task pool (not in `revocation-app`)
to keep the invalidation hop in-process with the cache.

## 10. Multi-region active-active

This µservice runs active-active across regions. Each region originates its own revocations + propagates
locally; georeplication handles cross-region. The grantor-region is the authoritative source for any
given agreement (per ADR-SVC-CG-004), so a revocation initiated in grantee-region first writes to
grantor-region's database (cross-region Postgres write), then publishes to grantor-region's Pulsar
topic. Cross-region Postgres write SLO p99 ≤200ms (within the 100ms-publish + 200ms-georep + 500ms-fanout
≈ 1s total revocation budget).

## 11. Verification

- `cargo build` + `cargo test` clean.
- Integration test against Pulsar dev cluster: 1K rev/s sustained for 60s, p99 ≤500ms publish + fan-out.
- Chaos: kill 1 of 3 Pulsar brokers → publishes continue (quorum ack).
- Chaos: pause georep 30s → resume → catches up within 60s; alert fired during pause; no consent-graph
  data loss.

## 12. Risk

- **R**: Pulsar broker outage in grantor region during DSAR cascade.
  **M**: 3-broker quorum; dead-letter table for ultra-rare full-cluster outage; runbook
  `revocation-incident.md` covers manual replay from DLQ.
- **R**: Cross-region network partition.
  **M**: Per-region grantor-as-authority means a partition means *new* grantor-region revocations
  can't reach grantee region — by ADR-0214 §2.3 failure-mode-deny, grantee-region enforcement
  fails closed (no false-permits during partition).
- **R**: Pulsar version skew during upgrade.
  **M**: Topic schema versioned (v1 today); subscriber accepts v1+ forward-compat.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: Snowflake Secure Data Share, Databricks Delta Sharing, and AWS Data Exchange revocation paths are refresh/subscription oriented, while OneTrust and TrustArc are consent-workflow oriented. This IP is Oyatie-specific because revocation is a high-priority Pulsar fan-out path with grantor-region authority, cross-region failure-deny behavior, and sub-second enforcement invalidation.
