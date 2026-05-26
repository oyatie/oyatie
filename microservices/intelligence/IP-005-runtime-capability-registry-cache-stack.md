---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-005-capability-registry-cache-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-foundry-runtime-capability-registry-cache stack

## Intent

The full capability-registry-cache BC: kernel + usecase + api + adapter + adapter-postgres + worker + app. Implements `CapabilityResolver` port via in-process cache + Postgres mirror. Worker subscribes to `CapabilityRegistryUpdated` events from foundry-supervisor and hot-reloads cache entries. Pre-warms on registration to satisfy PRD OQ#4.

## ChangeSet boundary

7 new Rust crates. Postgres mirror table is initialised by IP-002; this IP populates + maintains.

## Concrete File Targets

For each layer crate under `microservices/intelligence/src/crates/oya-foundry-runtime-capability-registry-cache-<layer>/`:
- `Cargo.toml` + `src/lib.rs`
- kernel: `entities.rs` (CapabilityDescriptor, RegistryVersion, CacheEntry) + `ports.rs` (RegistryMirror, CacheStore, RegistryVersionClock) + `errors.rs`
- usecase: `read_through_use_case.rs` + `invalidate_use_case.rs`
- api: `requests.rs` + `responses.rs`
- adapter: `in_memory_cache_store.rs` (DashMap-backed; ≤10ms p99 lookup)
- adapter-postgres: `mirror_repo.rs` (sqlx 0.8; row-level signature validation per T-T-04 mitigation)
- worker: `hot_reload_worker.rs` (AsyncAPI subscriber for `CapabilityRegistryUpdated`)
- app: `main.rs` (composition root binary)
- catalog row per crate

## Crate Naming Justifications

All crates follow `oya-foundry-runtime-capability-registry-cache-<layer>` per ADR-0105 + ADR-0131. Domain elided per PRD §"runtime-pool" Amendment 4 rationale (mechanism, no arithmetic).

## Code Shape

```rust
// adapter-postgres/src/mirror_repo.rs
use oya_foundry_runtime_capability_registry_cache_kernel::*;
use sqlx::PgPool;

pub struct PostgresRegistryMirror { pool: PgPool, supervisor_pubkey: ed25519_dalek::VerifyingKey }

#[async_trait]
impl RegistryMirror for PostgresRegistryMirror {
    async fn load(&self, tenant_id: &str, capability_id: &str) -> Result<CapabilityDescriptor, MirrorError> {
        let row: CapabilityRow = sqlx::query_as(
            "SELECT capability_id, tenant_id, descriptor_yaml, version, signature
             FROM capability_mirror
             WHERE tenant_id = $1 AND capability_id = $2 AND active = true"
        )
        .bind(tenant_id).bind(capability_id).fetch_one(&self.pool).await?;

        // Validate signature per T-T-04 mitigation
        self.supervisor_pubkey.verify(row.descriptor_yaml.as_bytes(), &row.signature)
            .map_err(|_| MirrorError::SignatureInvalid)?;

        Ok(CapabilityDescriptor::from_yaml(&row.descriptor_yaml)?)
    }
}
```

```rust
// worker/src/hot_reload_worker.rs
pub struct HotReloadWorker<M, C> { mirror: M, cache: C, /* AsyncAPI subscriber */ }

impl<M: RegistryMirror, C: CacheStore> HotReloadWorker<M, C> {
    pub async fn run_forever(&self) -> Result<(), WorkerError> {
        loop {
            let event = self.subscribe_next().await?;
            match event.action {
                Action::Created | Action::Modified => {
                    let descriptor = self.mirror.load(&event.tenant_id, &event.capability_id).await?;
                    self.cache.put(&event.tenant_id, &event.capability_id, descriptor).await?;
                }
                Action::Deleted => {
                    self.cache.evict(&event.tenant_id, &event.capability_id).await?;
                }
            }
            // Emit cache_age metric
        }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-registry-cache-{kernel,usecase,api,adapter,adapter-postgres,worker,app}
cargo nextest run -p oya-foundry-runtime-capability-registry-cache-{kernel,domain,usecase}
cargo nextest run -p oya-foundry-runtime-capability-registry-cache-adapter-postgres --features testcontainers
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_cache_lookup_p99_under_10ms` | adapter in-memory lookup latency |
| `test_mirror_signature_invalid_refuses` | T-T-04 mitigation correctness |
| `test_hot_reload_picks_up_event` | worker subscribes + cache reflects within 30s |
| `test_cache_evict_on_deleted` | deleted descriptors removed from cache |
| `test_eu_ai_act_high_risk_pack_eu_only` | pack-eu high-risk → instantiable; non-eu high-risk → refused per data-residency.md |

## Halt Conditions

- Cache lookup p99 > 10ms — refactor.
- Mirror signature not validated on every load — refactor.
- Hot-reload latency > 30s — refactor.

## Next IP

[`IP-006-session-state-stack.md`](IP-006-session-state-stack.md)

## References

- ADR-0025; ADR-0105.
- PRD §"Bounded Contexts" capability-registry-cache.
- `threat-model.md` T-T-04 (descriptor signature validation).
- `policy/data-residency.md` (capability descriptor routing).

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
