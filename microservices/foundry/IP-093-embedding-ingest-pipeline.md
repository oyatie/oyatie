# IP-093 — Embedding Ingest Pipeline

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** backend (axis-foundry)
**Authority ADRs:** ADR-0192 §"Embedding pipeline integration", ADR-0153 outbox, ADR-0026 AI substrate, ADR-0030 streaming, ADR-0145 inter-microservice communication
**Depends on:** IP-091, IP-092, IP-094 (adapter crate)
**Status:** Planned
**Phase trace:** PHASE-02 §"Embedding-to-vector ingest path" (addendum lines 36-42).

## Scope

Author a Pulsar consumer that subscribes to the canonical `oya.foundry.embeddings.outbox` topic and upserts each emitted embedding into the per-tenant Milvus collection. The consumer is the load-bearing bridge between any embedding-producing µservice (RAG corpus ingest, semantic search index, conversational memory) and the Foundry Milvus substrate.

Key properties:

- **Per-record idempotency** by `source_id` — Milvus 2.6 native upsert is the underlying mechanism.
- **Per-pack model binding** — per ADR-0026, the embedding model is selected per pack (KR=KoSimCSE-large, EN=bge-large-v1.5, multilingual=bge-m3); the ingest pipeline binds to whichever model the source declares (carried on the outbox record).
- **Per-tenant backpressure** — when a tenant's `quota.dml.upsertRate` would be exceeded, the consumer pauses consumption until headroom returns.
- **Batching** — accumulates up to 1,000 vectors per (tenant, collection) per batch, or 5s elapsed, whichever first; emits as a single Milvus upsert.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `crates/oya-foundry-milvus-ingest-app/Cargo.toml` | create | 1-50 | Workspace member |
| `crates/oya-foundry-milvus-ingest-app/src/main.rs` | create | 1-120 | Composition root; OTel; signals; leader election |
| `crates/oya-foundry-milvus-ingest-app/src/consumer.rs` | create | 1-200 | Pulsar consumer; acks on batch flush |
| `crates/oya-foundry-milvus-ingest-app/src/batcher.rs` | create | 1-160 | Per-(tenant, collection) accumulator |
| `crates/oya-foundry-milvus-ingest-app/src/flusher.rs` | create | 1-180 | Calls into adapter; handles partial failures |
| `crates/oya-foundry-milvus-ingest-app/src/backpressure.rs` | create | 1-140 | Per-tenant rate-limit with token bucket |
| `crates/oya-foundry-milvus-ingest-app/src/quarantine.rs` | create | 1-100 | Dimension-mismatch / schema-drift quarantine |
| `crates/oya-foundry-milvus-ingest-app/src/metrics.rs` | create | 1-80 | Prometheus metric definitions |
| `crates/oya-foundry-milvus-ingest-app/tests/integration/happy_path.rs` | create | 1-140 | end-to-end emit → search |
| `crates/oya-foundry-milvus-ingest-app/tests/integration/idempotent_replay.rs` | create | 1-100 | duplicate `source_id` is no-op |
| `crates/oya-foundry-milvus-ingest-app/tests/integration/dimension_mismatch_quarantine.rs` | create | 1-90 | bad record is quarantined |
| `crates/oya-foundry-milvus-ingest-app/tests/integration/backpressure_throttle_resume.rs` | create | 1-120 | quota breach pauses + resumes |
| `crates/oya-foundry-milvus-ingest-app/tests/integration/milvus_outage_backoff.rs` | create | 1-100 | cluster offline → resumes when ready |
| `microservices/foundry/iac/kustomize/components/milvus-ingest/deployment.yaml` | create | 1-90 | 3-replica HPA |
| `microservices/foundry/iac/kustomize/components/milvus-ingest/hpa.yaml` | create | 1-30 | scale on Pulsar consumer lag |

## Outbox record schema (consumed by this pipeline)

```protobuf
message EmbeddingOutboxRecord {
  string source_id        = 1;  // idempotency key
  string tenant_id        = 2;
  string microservice_id  = 3;  // determines target database
  string domain           = 4;  // determines target collection
  string data_class       = 5;  // public|pii|phi|restricted -> partition
  string model_id         = 6;  // pinned per pack per ADR-0026
  uint32 dimension        = 7;  // MUST match collection schema
  repeated float vector   = 8;
  map<string,string> tags = 9;  // searchable scalars
  google.protobuf.Timestamp emitted_at = 10;
}
```

## Acceptance criteria

- Emission of an embedding event lands in the per-tenant Milvus collection within **60s p99** end-to-end (matches `milvus-ingest-lag.openslo.yaml`).
- Re-emission of the same `source_id` is a no-op (Milvus native upsert; verified by `idempotent_replay`).
- QUOTA-exceeded backpressure cleanly throttles consumption and resumes within 5s of headroom.
- Per-batch tracing span tagged `(tenant_id, collection, batch_size, latency_ms, model_id)` visible in Tempo.
- Dimension-mismatch records are quarantined to `oya.foundry.embeddings.quarantine` (not crashed); alert fires on quarantine count > 0.
- Milvus cluster outage: consumer backs off (exp), resumes when cluster recovers, **no data loss** (Pulsar consumer offset not advanced until batch ack).
- Schema drift surfaced via `KernelError::DimensionMismatch`; quarantines the batch + alerts within 5min.
- Per-tenant per-collection per-hour upsert count visible in the Milvus tenants dashboard.

## Test plan

| Test | Verifies |
|---|---|
| `test_emit_then_search_roundtrip` | end-to-end |
| `test_idempotent_source_id` | duplicate source_id = no-op |
| `test_dimension_mismatch_quarantines` | bad record routed to quarantine topic |
| `test_backpressure_throttle_resume` | upsertRate breach paused + resumed |
| `test_milvus_outage_no_data_loss` | cluster down 60s → all records eventually delivered |
| `test_batch_size_cap_1000` | 1500 records → 2 batches (1000 + 500) |
| `test_batch_time_cap_5s` | 200 records over 6s → single batch flushed at 5s mark |
| `test_per_pack_model_binding_kr` | KR-tagged record routed to KoSimCSE collection |
| `test_per_pack_model_binding_en` | EN-tagged record routed to bge-large collection |
| `test_pulsar_offset_not_advanced_on_failure` | flusher error → offset stays at last good batch |
| `test_metrics_recorded` | all 5 Prometheus metrics present on `/metrics` |
| `test_hpa_scales_on_consumer_lag` | lag > 10s adds a replica |

## Evidence emission

- **Audit chain (ADR-0145):** per-batch `embedding.batch.upserted` event with `{tenant_id, collection, batch_size, source_id_hash_root, latency_ms}` to `oya.foundry.audit.milvus.ingest`.
- **Metrics:** `foundry_milvus_ingest_batch_size`, `foundry_milvus_ingest_latency_seconds`, `foundry_milvus_ingest_quarantined_total`, `foundry_milvus_ingest_consumer_lag_seconds`, `foundry_milvus_ingest_throttled_seconds_total`.
- **Tracing:** per-batch span via OTel; child spans for `adapter.upsert`, `backpressure.acquire`, `quarantine.emit`.
- **Quarantine evidence:** `evidence/quarantine/milvus-ingest-<date>.jsonl` append-only.

## Failure modes

| Mode | Mitigation |
|---|---|
| Milvus cluster unavailable | Pulsar consumer backs off (exponential, max 60s); resumes when cluster recovers; no data loss (offset not advanced) |
| Schema drift (dimension mismatch) | Kernel `KernelError::DimensionMismatch` surfaced; quarantines the record + alerts; batch continues with remaining records |
| Per-collection quota burst | Backpressure (token bucket); no data loss; latency SLO may degrade |
| Pulsar broker failover | Consumer rebalances; in-flight batch is replayed (idempotent) |
| Bad model_id on outbox record | Quarantined; ops portal exposes the bad-record stream |
| Tenant offboarded mid-flight | Reconciler drops collection; in-flight batch fails on upsert; record dropped (acceptable per DSR) |

## Rollback procedure

1. **Bad deploy.** `kubectl rollout undo deployment/foundry-milvus-ingest -n foundry`. Pulsar offsets are external; restart resumes from last commit.
2. **Bad batch flushed.** Milvus upsert is logically destructive only on same `source_id`. To restore: replay from Pulsar (retention window 7 days) with a corrected ingest binary.
3. **Schema-drift cascade.** If a producer emits malformed records en masse, quarantine consumes all. Pause the producer at its outbox; drain quarantine; re-emit with fixed shape.
4. **Capacity blow-out.** HPA-cap exceeded → manually scale replicas via `kubectl scale` + page capacity-planning.

## Blocking deps

- IP-091, IP-092, IP-094 (the adapter crate this pipeline calls).
- Pulsar topic `oya.foundry.embeddings.outbox` exists per cross-µservice contract.
- Per-pack model binding registry deployed (ADR-0026 substrate).

## Exit criteria

All test-plan rows green; 7 consecutive days in dev cell with consumer lag < 60s p99; quarantine count = 0 over a 24h golden run; SLO `milvus-ingest-lag` budget unburned over the burn-in window.

## Capacity sizing baseline (medium cell)

| Resource | per replica | replicas |
|---|---|---|
| CPU request | 1 | 3 (HPA scales 3-12) |
| CPU limit | 2 | — |
| Memory request | 2Gi | — |
| Memory limit | 4Gi | — |
| Disk | ephemeral | — |

HPA triggers: Pulsar consumer lag > 10s OR CPU > 70%. Scale-down debounce 5min.

## Security posture

- **No prompt text in logs.** The ingest pipeline carries embeddings + metadata; **never** the underlying prompt that generated them. Privacy invariant verified by `test_no_prompt_in_logs`.
- **Per-tenant per-collection write scoping.** The adapter is initialised with a per-µservice writer credential (INSERT scoped to `db_{microservice_id}`); never has DELETE / DROP.
- **Quarantine isolation.** Quarantine topic `oya.foundry.embeddings.quarantine` is read-only for foundry ops; tenant systems cannot inspect.
- **Data-class partition enforcement.** Per ADR-0192 §"Naming + isolation primitives", every record routes to its `data_class` partition; misrouted records (e.g., PII record routed to `partition_public`) are quarantined.

## Observability mapping

| Signal | Metric / span | Alert |
|---|---|---|
| Ingest lag (end-to-end) | `foundry_milvus_ingest_consumer_lag_seconds` | `MilvusIngestLagHigh` (> 60s p99 over 5min) |
| Batch size distribution | `foundry_milvus_ingest_batch_size` histogram | — |
| Per-batch latency | `foundry_milvus_ingest_latency_seconds` histogram | — |
| Quarantine count | `foundry_milvus_ingest_quarantined_total` | `MilvusIngestQuarantineNonZero` |
| Throttled time | `foundry_milvus_ingest_throttled_seconds_total` | `MilvusIngestThrottleHigh` (> 30s/min sustained) |

## References

- ADR-0192 §"Embedding pipeline integration".
- ADR-0153 — outbox pattern.
- ADR-0026 — AI substrate (per-pack model selection).
- ADR-0030 — streaming.
- ADR-0145 — audit chain emission.
- OpenSLO: `microservices/foundry/slos/milvus-ingest-lag.openslo.yaml`.
- Kernel: `oya-shared-vector-store-kernel`.
- Adapter (built in IP-094): `oya-shared-vector-store-milvus-adapter`.
