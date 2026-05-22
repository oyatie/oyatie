# IP-092 — Per-Tenant Vector Collection Bootstrap

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** backend (axis-foundry + council-tenancy)
**Authority ADRs:** ADR-0192 §"Multi-tenancy isolation" and §"Naming + isolation primitives", ADR-0155 per-tenant resource quotas, ADR-0145 inter-microservice communication reform, ADR-0038 DSR cascade, ADR-0136 foundry-as-single-microservice
**Depends on:** IP-091
**Status:** Planned
**Phase trace:** PHASE-02 §"Tenant lifecycle reconciliation" (addendum lines 26-34).

## Scope

Author a controller that listens for tenant lifecycle events from the tenancy µservice and reconciles per-tenant Milvus state. The controller:

1. Creates the per-µservice **database** (one-time, idempotent) — naming pattern `db_{microservice_id}` per ADR-0192 §"Naming + isolation primitives".
2. Creates per-tenant **collections** — naming pattern `tenant_{tenant_id}__{domain}` (e.g., `tenant_ten_acme__rag_corpus`).
3. Creates per-data-class **partitions** within each collection — `partition_public`, `partition_pii`, `partition_phi`, `partition_restricted`.
4. Applies per-tenant resource quotas per ADR-0155 — projected into Milvus `db.replica.num`, `quota.dml.upsertRate`, `quota.dql.searchRate`.
5. Drives the DSR cascade — on `tenant.offboarded`, drops all per-tenant collections + emits proof-of-erasure.

The controller is a long-running k8s-native deployment (single replica with leader-election); state is stored in Postgres at `oya_foundry.milvus_tenant_reconcile_state`. Idempotency is enforced on `(tenant_id, microservice_id)`.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `crates/oya-foundry-milvus-tenant-bootstrap-app/Cargo.toml` | create | 1-40 | Workspace member; tokio, sqlx, milvus-rs client |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/main.rs` | create | 1-120 | Composition root, signal handling, OTel init |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/reconciler.rs` | create | 1-220 | Event-driven reconcile loop |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/milvus_actions.rs` | create | 1-180 | Idempotent create_db / create_collection / drop_collection |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/quota_projector.rs` | create | 1-140 | ADR-0155 projection logic |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/proof_of_erasure.rs` | create | 1-100 | DSR cascade emitter |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/event_consumer.rs` | create | 1-120 | Pulsar consumer for `oya.tenancy.tenant.lifecycle` |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/src/state_store.rs` | create | 1-90 | Postgres-backed idempotency state |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/tests/integration/onboard_then_offboard.rs` | create | 1-150 | End-to-end happy path |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/tests/integration/idempotent_replay.rs` | create | 1-90 | Replay-safety |
| `crates/oya-foundry-milvus-tenant-bootstrap-app/tests/integration/dsr_cascade.rs` | create | 1-120 | Offboard drops + emits proof |
| `microservices/foundry/iac/kustomize/components/milvus-tenant-bootstrap/deployment.yaml` | create | 1-80 | k8s Deployment |
| `microservices/foundry/iac/kustomize/components/milvus-tenant-bootstrap/rbac.yaml` | create | 1-50 | ServiceAccount + Role |

## Per-tier QUOTA matrix (per ADR-0155 projection)

| Tier | qps/collection (`quota.dql.searchRate`) | upsertRate (rows/sec, `quota.dml.upsertRate`) | `db.replica.num` | Default partition count |
|---|---|---|---|---|
| Trial | 50 | 100 | 1 | 4 (public/pii/phi/restricted) |
| Starter | 500 | 1,000 | 2 | 4 |
| Growth | 5,000 | 10,000 | 2 | 4 + custom |
| Enterprise | 50,000 | 100,000 | 3 | 4 + custom + per-domain |

Custom quota outside this matrix requires capacity-team approval + audit-chain `tenant.quota.override` event.

## Acceptance criteria

- `tenant.onboarded` event → per-tenant collection created within 30s p99; all 4 data-class partitions present.
- `tenant.offboarded` event → all per-tenant collections dropped within 60s; proof-of-erasure event emitted.
- Per-tenant QUOTA matches the tenant's tier (validated via `kubectl exec milvus-proxy-0 -- milvus_cli show quota tenant_<id>`).
- Re-publishing `tenant.onboarded` is idempotent (no duplicate collection, no error).
- Cross-tenant collection access denied at the Milvus auth layer (validated by `cross_tenant_resolve_denied` test).
- `tenant.tier_changed` event triggers QUOTA re-application within 30s.
- Per-µservice database created exactly once across N replicas of the controller (leader-election enforced).
- Reconciler keeps running through Pulsar broker failover (no event loss verified by `tests/integration/broker_failover.rs`).

## Test plan

| Test | Verifies |
|---|---|
| `test_onboard_creates_collection_with_partitions` | happy path: 1 collection + 4 partitions per data class |
| `test_onboard_applies_tier_quota_growth` | Growth tier sets searchRate=5000 + replica=2 |
| `test_offboard_drops_all_collections` | DSR cascade |
| `test_offboard_emits_proof_of_erasure` | proof-of-erasure event hash matches per-tenant Merkle root |
| `test_idempotent_replay_no_duplicate` | duplicate `tenant.onboarded` → no Milvus mutation |
| `test_tier_change_growth_to_enterprise` | QUOTA upgrade applied within 30s |
| `test_cross_tenant_resolve_denied` | Tenant B cannot describe Tenant A's collection |
| `test_leader_election_single_db_create` | 3 replicas; only leader creates the per-µservice DB |
| `test_broker_failover_no_event_loss` | Pulsar broker restart mid-stream → no missed events |
| `test_offboard_then_re_onboard_clean` | a tenant can be offboarded and re-onboarded (new collection) |

## Evidence emission

- **Audit chain (ADR-0145):** every reconcile action emits `tenant.milvus.{onboarded,offboarded,quota_applied,collection_created,collection_dropped,proof_of_erasure}` to `oya.foundry.audit.milvus.tenant` Pulsar topic; sealed via Ed25519.
- **Metrics:** `foundry_milvus_tenant_reconcile_duration_seconds` (histogram), `foundry_milvus_tenant_reconcile_errors_total` (counter), `foundry_milvus_tenants_total{tier}` (gauge).
- **Proof-of-erasure pack:** on offboard, write `evidence/proof-of-erasure/milvus-tenant-<tenant-id>-<ts>.json` containing pre-drop vector count, post-drop attestation, audit-chain seal hash.
- **Dashboard:** `microservices/foundry/dashboards/milvus-tenants.json` (already present) shows per-tier tenant count + reconcile lag.

## Rollback procedure

1. **Per-tenant reconcile failure.** Reconciler captures error in Postgres `milvus_tenant_reconcile_state.last_error`; emits `tenant.milvus.reconcile_failed` audit event; controller does NOT loop on the same event indefinitely (max 3 retries with exponential backoff, then quarantine).
2. **Bad deploy (controller crash-loop).** `kubectl rollout undo deployment/foundry-milvus-tenant-bootstrap -n foundry`. Reconcile state persists in Postgres; restart resumes from last-acked Pulsar offset.
3. **Bad QUOTA push.** Audit-chain event records the pre-state QUOTA values; restore by re-applying the prior tier's QUOTA via CLI (`milvus_cli set quota`).
4. **Erroneous offboard (data loss prevention).** Collection drop is hard-delete; recovery is via the IP-096 backup restore. **No automatic undo path** — this is the load-bearing safety property of DSR. Offboard event must be confirmed by 2-person rule in tenancy µservice before it ever lands on the topic.

## Blocking deps

- IP-091 (cluster IaC) accepted.
- Tenancy µservice publishing `oya.tenancy.tenant.lifecycle` topic per its IP (cross-µservice contract).
- Postgres schema `oya_foundry.milvus_tenant_reconcile_state` provisioned (sqlx migration ships with this IP).

## Exit criteria

All test-plan rows green in CI; controller deployed in dev cell; 100 synthetic onboard/offboard cycles complete with 0 errors; DSR proof-of-erasure pack accepted by legal review; foundry-oncall has drilled the `milvus-tenant-quota.md` runbook.

## Event contract (consumed from tenancy µservice)

```protobuf
message TenantLifecycleEvent {
  enum Kind {
    UNSPECIFIED = 0;
    ONBOARDED = 1;
    OFFBOARDED = 2;
    TIER_CHANGED = 3;
    RESIDENCY_CHANGED = 4;
  }
  Kind kind                                  = 1;
  string tenant_id                           = 2;
  string microservice_id                     = 3;  // determines target Milvus database
  Tier old_tier                              = 4;
  Tier new_tier                              = 5;
  ResidencyClass residency                   = 6;
  google.protobuf.Timestamp event_ts         = 7;
  string causation_id                        = 8;  // 2-person approval trace
}
```

## Security posture

- **2-person rule.** `tenant.offboarded` events require dual approval at the tenancy µservice before reaching the bus; controller verifies the approval trace in `causation_id` before acting.
- **Per-tenant credentials.** Each tenant gets its own Milvus user; cross-tenant access is denied at the auth layer.
- **Cedar audit.** Cedar policy fragment per tenant generated by IP-097's residency engine; controller installs the fragment on collection creation.
- **Proof of erasure.** On offboard, the controller writes the Merkle root of all dropped vectors to the audit chain (sealed Ed25519); the proof-of-erasure pack contains pre-state vector count + post-state attestation.

## Observability mapping

| Signal | Metric / span | Dashboard panel |
|---|---|---|
| Reconcile latency | `foundry_milvus_tenant_reconcile_duration_seconds` | Milvus Tenants > Reconcile lag |
| Reconcile errors | `foundry_milvus_tenant_reconcile_errors_total` | Milvus Tenants > Errors |
| Active tenants per tier | `foundry_milvus_tenants_total{tier}` | Milvus Tenants > Distribution |
| Quota application count | `foundry_milvus_quota_applied_total{tier}` | Milvus Tenants > Quota churn |
| DSR cascade duration | `foundry_milvus_dsr_cascade_duration_seconds` | Milvus Tenants > DSR |

## Capacity sizing

| Resource | per replica | replicas |
|---|---|---|
| CPU request | 0.5 | 3 (leader + 2 warm) |
| CPU limit | 1 | — |
| Memory request | 1Gi | — |
| Memory limit | 2Gi | — |
| Postgres connection pool | 8 per replica | — |

## References

- ADR-0192 §"Multi-tenancy isolation" + §"Naming + isolation primitives".
- ADR-0155 — per-tenant resource quotas.
- ADR-0145 — audit-chain emission.
- ADR-0038 — DSR cascade.
- ADR-0136 — foundry-as-single-microservice.
- Runbook: `microservices/foundry/runbooks/milvus-tenant-quota.md`.
- Kernel crate: `oya-tenancy-kernel::B2bTenantTier`.

## Wave 15 counterpart anchor

- Counterparts: Snowflake Cortex Search, Databricks Vector Search, OpenAI vector stores, and Palantir AIP ontology retrieval.
- Gap closure: this IP closes Foundry retrieval/vector substrate for tenant-isolated agent grounding and eval replay.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
