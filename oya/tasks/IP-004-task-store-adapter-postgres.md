---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-004-task-store-adapter-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, oya-governance-amendment-3-backend-qualified-adapter, encryption-at-rest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: task-store adapter — Postgres 16 LTS with RLS + tenant-DEK

## Intent

Implement `TaskRepository`, `TaskHistoryStore`, `LegalHoldStore`,
`RetentionPolicyResolver` port traits against Postgres 16 LTS. Per
ADR-0117 + Bominal ADR-0111: per-tenant row-level-security (RLS)
policies + tenant-DEK envelope encryption at row-write time. Per
ADR-0105 Amendment 3: the crate name is the backend-qualified
`oya-tasks-task-store-adapter-postgres`.

Schema authored as SQL migrations (`sqlx-cli`) including: `tasks`,
`task_comments`, `task_history`, `task_dependencies`, `legal_holds`,
`projects`, `custom_field_definitions`. Indexes on
`(tenant_id, project_id, status)` + `(tenant_id, assignee_id)` +
`(tenant_id, due_at)` + GIN on `tsvector(title || description)` for
direct-Postgres-search fallback path per AC-09. Partition strategy:
`tasks` partitioned by `(tenant_id, project_id_hash)` per PRD §
"Horizontal Scalability".

## ChangeSet boundary

1 crate + SQL migration directory + sqlx-cli config + adapter integration
test using `testcontainers-rs`. Encryption-at-rest test verifies AC-08.

## Crate Naming

`oya-tasks-task-store-adapter-postgres` per ADR-0105 Amendment 3 backend-
qualified adapter pattern.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-task-store-adapter-postgres/src/lib.rs` | replaced | port implementations |
| `microservices/tasks/src/oya-tasks-task-store-adapter-postgres/migrations/*.sql` | created | 7 tables + RLS policies + partitions |
| `microservices/tasks/src/oya-tasks-task-store-adapter-postgres/tests/*.rs` | created | testcontainers integration |
| `microservices/tasks/catalog/oya-tasks-task-store-adapter-postgres.yaml` | created | catalog entry |

## Acceptance Gates

```bash
cargo test -p oya-tasks-task-store-adapter-postgres
buck2 build //:quality-lane-registry-authority-check # lane=amendment-3-backend-qualified-adapter --crate oya-tasks-task-store-adapter-postgres
buck2 build //:quality-lane-registry-authority-check # lane=encryption-at-rest --microservice tasks
```

## Test Plan

- RLS smoke: cross-tenant read REFUSED at the database level (defence-
  in-depth even if Cedar layer bypassed).
- Tenant-DEK envelope encryption: rows on disk show ciphertext; key
  rotation rotates DEK without rewriting plaintext (only the wrapping
  KEK changes).
- Partition pruning: `EXPLAIN ANALYZE` shows single-partition scan for
  per-project queries.

## Halt Conditions

- RLS policy bypass detected — refuse to ship; this is a P0 invariant.
- Encryption-at-rest test reveals plaintext on disk — refuse.

## Next IP

[`IP-005-project-and-board-bc.md`](IP-005-project-and-board-bc.md)

## References

- ADR-0105 Amendment 3 (backend-qualified adapter); ADR-0117 (residency).
- Bominal ADR-0111 (tenant-DEK envelope encryption); Bominal ADR-0028 (Ed25519 + Merkle).
- Postgres 16 LTS — `www.postgresql.org/docs/16/`.
