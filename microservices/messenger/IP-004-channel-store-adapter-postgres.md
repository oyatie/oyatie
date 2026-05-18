---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-004-channel-store-adapter-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, sqlx-migration-lint, oya-governance-shardability]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: channel-store adapter-postgres + migrations

## Intent

Implement `ChannelRepository` against Postgres 16 LTS. Author the sqlx
migrations enforcing per-tenant RLS, context_kind CHECK constraint, and
partition key `(tenant_id, channel_id)`.

## ChangeSet boundary

`-adapter-postgres` crate + `migrations/` directory.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-channel-store-adapter-postgres/src/lib.rs` | create |
| `src/crates/oya-messenger-channel-store-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-messenger-channel-store-adapter-postgres/migrations/0001_init.sql` | create |
| `src/crates/oya-messenger-channel-store-adapter-postgres/migrations/0002_rls.sql` | create |
| `src/crates/oya-messenger-channel-store-adapter-postgres/migrations/0003_holds.sql` | create |
| `tests/repository_e2e.rs` | create — testcontainers Postgres |

## Code Shape

```sql
-- migrations/0001_init.sql
CREATE TABLE messenger_channels (
    channel_id        bytea PRIMARY KEY,           -- ULID raw
    tenant_id         text  NOT NULL,
    context_kind      text  NOT NULL CHECK (context_kind IN ('Personal','Professional')),
    kind              text  NOT NULL CHECK (kind IN ('Public','Private','DM','GroupDM')),
    name              text  NOT NULL,
    topic             text,
    member_count      integer NOT NULL DEFAULT 0,
    retention_policy_id text,
    created_at        timestamptz NOT NULL DEFAULT now(),
    archived_at       timestamptz
) PARTITION BY HASH (tenant_id);
-- 32 partitions for tenant-id shardability
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-channel-store-adapter-postgres
sqlx migrate run --source migrations
cargo run -p oya-dev-cli -- gate validate shardability --microservice messenger
```

## Test Plan

- testcontainers-rs spinning Postgres 16; run all migrations; round-trip CRUD.
- RLS test: insert as tenant A; read as tenant B returns 0 rows.
- Context-mix test: insert Personal channel; assert constraint refuses Professional change.

## Halt Conditions

- Migrations not idempotent — fix; never edit history.
- RLS bypass detected — Sev-1; block release.

## Next IP

[`IP-005-message-stream-kernel-domain.md`](IP-005-message-stream-kernel-domain.md)

## References

- ADR-0008 Data Use Boundary; ADR-0028 Audit Chain.
- Postgres 16 LTS docs `postgresql.org/docs/16`.
