// NOTE: extension of ontology projection adapter; owned jointly by axis-ontology + axis-consent-graph.

# IP-CT-002: Tenant-aware Pulsar ACL adapter (ontology extension)

- Microservice: ontology (extension)
- Bounded context: cross-tenant-projection
- Layer: adapter
- Crate: `oya-ontology-cross-tenant-projection-adapter-pulsar`
- Acceptance status: ga
- Authority: ADR-0214 §2.5, ADR-SVC-CG-004, IP-CT-001.
- Depends on: IP-CT-001, `pulsar-rs = "6"`, `oya-consent-graph-projection-gateway-sdk` (for topic
  mint coordination — ontology does NOT mint topics; consent-graph does).

## 1. Goal

Implement `CrossTenantProjectionEmitter` against grantor-region Pulsar cluster with tenant-aware JWT
authentication. **Topics are minted by consent-graph (not ontology)**; ontology only **emits** to
them. This keeps the topic-mint authority in one place (consent-graph) while ontology drives the
data flow.

## 2. Scope

In:
- Pulsar producer connection management to grantor-region cluster.
- Per-topic producer caching (one producer per active topic).
- Tenant-aware JWT minting via consent-graph SDK (ontology's emission service identity).
- Backpressure + retry.

Out:
- Topic minting (consent-graph IP-010 owns this).
- Topic ACL management (consent-graph IP-010 owns this).
- Scope narrowing (IP-CT-003).

## 3. Producer per topic

For each `CrossTenantProjectionTarget` that ontology emits to:
1. On first emit: open Pulsar producer with grantor-region cluster URL.
2. Authenticate via JWT minted from ontology's SPIFFE identity + consent-graph's projection-token
   service (subject = `oya-tenant-grantor`, audience = topic-name).
3. Cache producer for 1h or until topic destroyed.
4. On topic destroy notification (Pulsar admin client subscribes to admin events): close producer.

```rust
pub struct CrossTenantPulsarEmitter {
    client_per_region: HashMap<Region, pulsar::Pulsar<TokioExecutor>>,
    producer_cache: DashMap<TopicName, Arc<pulsar::Producer<...>>>,
    consent_graph_sdk: ConsentGraphProjectionGatewayClient,
}

#[async_trait]
impl CrossTenantProjectionEmitter for CrossTenantPulsarEmitter {
    async fn emit(&self, target: &CrossTenantProjectionTarget, event: &ProjectionEvent)
        -> Result<(), EmitError>
    {
        let producer = self.get_or_create_producer(target).await?;
        let msg = MessageBuilder::new()
            .key(event.entity_id.as_deref().unwrap_or("aggregate"))
            .properties([
                ("agreement_id", target.agreement_id.to_string()),
                ("schema_version", event.schema_version.to_string()),
                ("mode", format!("{:?}", target.mode)),
            ])
            .payload(serde_json::to_vec(event)?);
        producer.send(msg).await?;
        Ok(())
    }
}
```

## 4. JWT minting

Ontology's emission service holds a SPIFFE workload identity. To emit to a cross-tenant topic in
grantor's Pulsar cluster, ontology needs a JWT issued by *grantor's* OpenBao-signed key.

Flow:
1. Ontology requests projection-emit-token from consent-graph projection-gateway-app via SDK.
2. consent-graph mints JWT bound to (topic, ontology-spiffe-id, agreement-id).
3. Ontology uses this JWT in Pulsar authentication.

Tokens cached for 1h with 80% TTL refresh.

## 5. Backpressure

- Per-producer queue depth ≤1000 messages (Pulsar default).
- On `BatchOverflow`: caller (`projection-gateway-worker`) backs off; metric
  `ontology_cross_tenant_emit_backpressure_total` increments.
- On producer disconnect: 3× retry exp-backoff; permanent failure → dead-letter table; alert fires.

## 6. Retry policy

- Network/transient: 3× exp-backoff (50ms / 200ms / 1s).
- Topic-not-found (race with topic destroy): no-op + audit.
- ACL denied: hard fail + alert (means consent-graph token mismatch — investigate immediately).
- Quota exceeded: backoff + HPA signal.

## 7. Cross-region routing

Ontology emission service may be in *any* region. Cross-tenant emit may require connecting to
grantor's region's Pulsar cluster (different from ontology's local region). This is allowed:
- Pulsar supports cross-region clients via TLS.
- mTLS terminated by Pulsar broker.
- JWT auth on top of mTLS.

Latency cost: cross-region RTT (~80ms us-east↔us-west, ~150ms us↔eu).

## 8. Tests

- `producer_cache_hit_path` — second emit to same topic reuses producer.
- `producer_cache_eviction_on_destroy_event` — topic destroy → producer closed within 5s.
- `jwt_token_refresh_at_80_percent_ttl` — token cached + refreshed proactively.
- `backpressure_returns_error_not_block` — queue full → caller-handled error.
- `cross_region_emit_latency_under_300ms` — us-east emitter to us-west cluster.
- `topic_not_found_emits_audit` — race: topic destroyed mid-emit → audit + retry no-op.

## 9. Dependencies

- IP-CT-001 (kernel)
- `pulsar-rs = "6"`
- `oya-consent-graph-projection-gateway-sdk`
- `dashmap`, `tokio`, `serde`

## 10. Verification

- `cargo build` + `cargo test`.
- Integration test against dev Pulsar cluster: emit 10K events to cross-tenant topic; verify
  delivered + producer cached.
- Chaos: kill broker mid-emit → reconnect within 5s; events redelivered.

## 11. Risk

- **R**: JWT minting overhead on every first-emit to new topic.
  **M**: Token cached 1h; eager mint on agreement-accept notification (consent-graph emits projection-mint
  event; ontology subscribes + pre-mints token).
- **R**: Cross-region Pulsar connection pool exhaustion.
  **M**: Per-region connection cap (100 default); HPA on connection count.
- **R**: Token revocation lag.
  **M**: Revocation Pulsar subscriber invalidates cached token within 1s; per IP-008.

## 12. Cross-references

- IP-CT-001 kernel types
- microservices/consent-graph/IP-010 (topic mint authority)
- microservices/consent-graph/IP-008 (revocation fan-out)
