---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-006-session-state-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, session-prefix-isolation, postgres-rls-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-foundry-runtime-session-state stack

## Intent

The full session-state BC: kernel + domain + usecase + api + adapter + adapter-redis + adapter-postgres + sdk + app. Implements `SessionStore` + `SessionLeaseManager` + `SessionMutationLog` ports. Hot path = Valkey 8.1 (Redis wire-compat) with per-tenant prefix; cold restore = Postgres replay. Per `runtime-isolation.md` TI-01..TI-05 invariants.

## ChangeSet boundary

9 new Rust crates. Valkey schema + Postgres schema initialised by IP-002.

## Concrete File Targets

Per layer crate at `microservices/intelligence/src/crates/oya-foundry-runtime-session-state-<layer>/`:
- kernel: entities (Session, SessionTurn, ScratchpadEntry, SessionLease) + ports + errors
- domain: scratchpad merge logic + conflict-resolution (per-turn HMAC validation)
- usecase: load/persist/extend-lease orchestrators + DSR cascade handler
- api: typed contracts
- adapter: default protocol-neutral wrapper (NOOP delegating to backend adapters via DI)
- adapter-redis: Valkey 8.1 (Redis wire-compat) client with TLS + AUTH + tenant-prefix enforcement
- adapter-postgres: Postgres 16 client for cold restore + session_mutation_log (audit)
- sdk: tenant-facing Rust client
- app: composition root

## Crate Naming Justifications

All crates follow `oya-foundry-runtime-session-state-<layer>` per ADR-0105 + ADR-0131. `-adapter-redis` + `-adapter-postgres` use canonical `*-adapter-<backend>` pattern per ADR-0105 Amendment 3.

## Code Shape

```rust
// adapter-redis/src/session_store.rs
use oya_foundry_runtime_session_state_kernel::*;

pub struct RedisSessionStore { client: deadpool_redis::Pool, deployment_salt: SecretString }

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn load(&self, tenant_id: &str, session_id: &str) -> Result<Option<Session>, StoreError> {
        let key = self.prefixed_key(tenant_id, session_id);
        // TI-01: tenant_id prefix MANDATORY; never reachable bypass
        let mut conn = self.client.get().await?;
        let bytes: Option<Vec<u8>> = conn.get(&key).await?;
        Ok(bytes.map(Session::from_bytes).transpose()?)
    }

    async fn persist(&self, tenant_id: &str, session: &Session) -> Result<(), StoreError> {
        let key = self.prefixed_key(tenant_id, &session.session_id);
        let bytes = session.to_bytes()?;
        // Per-turn HMAC for T-T-02 mitigation
        let hmac = session.compute_hmac(&self.signing_key())?;
        let mut conn = self.client.get().await?;
        conn.set_ex(&key, &bytes, session.ttl_seconds()).await?;
        conn.set_ex(format!("{key}:hmac"), &hmac, session.ttl_seconds()).await?;
        Ok(())
    }

    fn prefixed_key(&self, tenant_id: &str, session_id: &str) -> String {
        // TI-01 mandatory; LEAN lane greps for any non-prefixed call
        format!("{tenant_id}:session:{session_id}")
    }
}
```

```rust
// usecase/src/dsr_cascade_handler.rs
pub struct DsrCascadeHandler<S, L> { store: S, log: L }

impl<S: SessionStore, L: SessionMutationLog> DsrCascadeHandler<S, L> {
    pub async fn handle(&self, tenant_id: &str, subject_hash: &str) -> Result<DsrReport, DsrError> {
        // 1. Scan Valkey per-tenant prefix for sessions containing subject_hash
        // 2. Postgres session_mutation_log query for cold-tier sessions
        // 3. Soft-delete with 30d grace; mark status=soft_deleted
        // 4. Audit-chain emit `dsr_executed{tenant, subject_hash, removed_session_count}`
        // ...
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-session-state-{kernel,domain,usecase,api,adapter,adapter-redis,adapter-postgres,sdk,app}
cargo nextest run -p oya-foundry-runtime-session-state-{kernel,domain,usecase}
cargo nextest run -p oya-foundry-runtime-session-state-adapter-redis --features testcontainers
cargo nextest run -p oya-foundry-runtime-session-state-adapter-postgres --features testcontainers
cargo run -p oya-dev-cli -- gate validate session-prefix-isolation --microservice foundry-runtime
cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_session_hot_read_p99_under_10ms` | Valkey adapter (Redis wire-compat) latency |
| `test_session_cold_restore_p99_under_100ms` | Postgres adapter latency |
| `test_cross_tenant_read_empty` | TI-01 / TI-03: reading another tenant returns empty |
| `test_redis_acl_refuses_cross_prefix` | TI-02 |
| `test_postgres_rls_refuses_cross_tenant` | TI-03 |
| `test_session_hmac_tamper_detected` | T-T-02 mitigation |
| `test_dsr_cascade_soft_deletes_in_30d` | DPIA R-08 |
| `test_session_resume_after_redis_eviction` | cold-restore from Postgres replay |

## Halt Conditions

- Any Valkey op without tenant prefix — refactor (TI-01 violation).
- Any Postgres tenant-data query without RLS — refactor (TI-03 violation).
- Session HMAC validation skipped — refactor (T-T-02).

## Next IP

[`IP-007-invocation-orchestrator-stack.md`](IP-007-invocation-orchestrator-stack.md)

## References

- ADR-0025; ADR-0105.
- `policy/runtime-isolation.md` TI-01..TI-05.
- `threat-model.md` T-I-01, T-T-02.
- `policy/data-residency.md` DSR cascade.
- Valkey 8.1 (Redis wire-compat) docs.
- Postgres 16 LTS docs.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
