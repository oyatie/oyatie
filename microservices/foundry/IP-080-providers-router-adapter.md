---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-005-router-adapter
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-foundry-providers-router-adapter

## Intent

Repository implementations for `ProviderConfigRepository` (Postgres) and `TokenBucket` (Valkey). Connects domain ports to Layer-A substrate.

## ChangeSet boundary

One new crate `microservices/foundry/src/crates/oya-foundry-providers-router-adapter/`. Depends on kernel.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — deps include `sqlx`, `redis`, `tokio` |
| `.../src/lib.rs` | create |
| `.../src/postgres_provider_config.rs` | create — `PostgresProviderConfigRepository` |
| `.../src/redis_token_bucket.rs` | create — `RedisTokenBucket` |
| `.../src/health_recording.rs` | create — emits provider-health metrics to Mimir |
| `.../src/migrations/` | create — sqlx migrations dir |

## Schema

Postgres schema for provider config:

```sql
CREATE TABLE tenant_provider_config (
    tenant_id TEXT NOT NULL,
    pack TEXT NOT NULL,
    capability_profile JSONB NOT NULL,
    per_vendor_credential_refs JSONB NOT NULL,    -- map vendor → openbao:// URI
    cost_ceilings_per_day_usd JSONB NOT NULL,
    forbidden_vendors TEXT[] NOT NULL,
    adapter_pins JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, pack)
);
CREATE INDEX idx_tenant_provider_config_pack ON tenant_provider_config(pack);
```

Valkey bucket key shape: `oyp:bucket:{tenant_id}:{vendor}` (token-count + last-refill-time hash).

## Test Plan

| Test | Verifies |
|---|---|
| `test_postgres_provider_config_roundtrip` | integration |
| `test_redis_token_bucket_take_refill` | integration |
| `test_postgres_handles_missing_tenant_returns_default` | error case |
| `test_redis_bucket_handles_clock_skew` | clock-skew resilience |
| `test_no_credential_in_postgres` | grep table contents for credential-shaped strings; 0 hits |

## Acceptance Gates

Standard + `integration-test` lane (against ephemeral Postgres + Valkey).

## Next IP

[`IP-006-adapter-anthropic-api.md`](IP-006-adapter-anthropic-api.md)
