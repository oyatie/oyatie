---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-002-redis-and-postgres-baseline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [helm-install-smoke, postgres-rls-coverage, session-prefix-isolation, foundry-runtime-iac-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Valkey 8.1 (Redis wire-compat) + Postgres 16 LTS baseline

## Intent

Ship Helm charts for Valkey 8.1 (Redis wire-compat) OSS LTS cluster (6 shards × 1 primary + 1 replica) and Postgres 16 LTS primary + read-replica. Bind ACLs + RLS to tenant-isolation invariants TI-01..TI-05. Wire OpenBao SecretReferences for Valkey AUTH + Postgres credentials. Provision capability_mirror + session_mutation_log + invocation_lifecycle tables with RLS policies.

## ChangeSet boundary

All paths under `microservices/foundry/iac/helm/{redis,postgres}/` + `iac/postgres-schema/`. No Rust crate changes.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/redis/Chart.yaml` | create (Valkey 8.1 (Redis wire-compat) OSS LTS pin) |
| `iac/helm/redis/values.yaml` | create (cluster mode; 6 shards; TLS + AUTH; ACL declared inline) |
| `iac/helm/redis/expected-acl.txt` | create (canonical ACL for drift detection) |
| `iac/helm/postgres/Chart.yaml` | create (Postgres 16 LTS pin) |
| `iac/helm/postgres/values.yaml` | create (TDE; streaming replication; WAL archive to OCI object-storage) |
| `iac/postgres-schema/001-capability-mirror.sql` | create (table + RLS policy) |
| `iac/postgres-schema/002-session-mutation-log.sql` | create (table + RLS) |
| `iac/postgres-schema/003-invocation-lifecycle.sql` | create (table + RLS + indexes) |
| `iac/postgres-schema/004-row-level-security-policies.sql` | create (per-tenant RLS policies) |
| `iac/postgres-schema/005-audit-triggers.sql` | create (audit-chain seal triggers) |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/redis/
helm lint microservices/foundry/iac/helm/postgres/
psql --dry-run -f microservices/foundry/iac/postgres-schema/001-capability-mirror.sql
cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage --microservice foundry-runtime
cargo run -p oya-dev-cli -- gate validate session-prefix-isolation --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| Valkey ACL probe | `default` user disabled; per-tenant role refused on cross-prefix |
| Postgres RLS coverage | Every multi-tenant table has RLS policy + denies cross-tenant SELECT |
| OpenBao SecretReference materialisation | Pods receive Valkey AUTH + Postgres creds without raw values in environment |
| Streaming replication health | `pg_replication_lag_seconds < 30` for ≥5min |

## Halt Conditions

- Any table without RLS — refactor.
- Valkey `default` user enabled — refactor (security risk).
- Raw secrets in pod env — refactor.

## Next IP

[`IP-003-capability-executor-kernel.md`](IP-003-capability-executor-kernel.md)

## References

- `policy/runtime-isolation.md` TI-01..TI-05.
- Valkey 8.1 (Redis wire-compat) — `redis.io/docs/about/releases/7-4-0/`.
- Postgres 16 LTS RLS — `postgresql.org/docs/16/ddl-rowsecurity.html`.
