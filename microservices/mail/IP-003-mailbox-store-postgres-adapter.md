---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-003-mailbox-store-postgres-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, shardability, oya-governance-per-microservice-layout, oya-governance-rls-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-mail-mailbox-store-adapter-postgres

## Intent

Implement `MailboxRepository` + `ThreadRepository` + `RetentionLedgerWriter` against Postgres 16 with per-tenant Row-Level Security (RLS) per ADR-0117 + ADR-0133. Sharding by `tenant_id` (Citus distributed table when single-Postgres approaches 80% capacity per PRD §"Horizontal Scalability").

## ChangeSet boundary

One Rust crate at `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/`. Plus the Postgres schema (DDL) under `microservices/mail/iac/postgres/migrations/`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/Cargo.toml` | create | `sqlx` (compile-time SQL check) + `tokio` + kernel dep |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/src/lib.rs` | create | impl exports |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/src/mailbox_repo.rs` | create | `PgMailboxRepository` impl |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/src/thread_repo.rs` | create | `PgThreadRepository` impl |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-postgres/src/retention_ledger.rs` | create | `PgRetentionLedgerWriter` impl |
| `microservices/mail/iac/postgres/migrations/001_init.sql` | create | initial schema (mailboxes, threads, mail_messages, folders, retention_ledger) with RLS |
| `microservices/mail/iac/postgres/migrations/002_indexes.sql` | create | tenant_id + mailbox_id + received_at composite indexes |
| `microservices/mail/iac/postgres/migrations/003_citus_distribute.sql` | create | distributed-table directives gated by env var |
| `microservices/mail/catalog/oya-mail-mailbox-store-adapter-postgres.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-mail-mailbox-store-adapter-postgres
JUSTIFICATION:
- microservice = mail
- bc-tokens = mailbox-store
- layer = adapter (backend-qualified per ADR-0105 Amendment 3 *-adapter-<backend>)
- backend = postgres
- exemptions claimed: none
```

## Code Shape

```sql
-- 001_init.sql
CREATE TABLE mailboxes (
  mailbox_id        UUID PRIMARY KEY,
  tenant_id         TEXT NULL,                  -- NULL for Personal
  context_kind      TEXT NOT NULL CHECK (context_kind IN ('Personal', 'Professional')),
  owner_ref         TEXT NOT NULL,
  region            TEXT NOT NULL,
  retention_policy_id UUID NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE mailboxes ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_scope ON mailboxes
  USING (
    (context_kind = 'Professional' AND tenant_id = current_setting('app.tenant_id', true))
    OR
    (context_kind = 'Personal' AND owner_ref = current_setting('app.user_id', true))
  );

CREATE TABLE mail_messages (
  message_id        TEXT PRIMARY KEY,
  mailbox_id        UUID NOT NULL REFERENCES mailboxes(mailbox_id),
  tenant_id         TEXT NULL,
  context_kind      TEXT NOT NULL,
  headers_blob_ref  TEXT NOT NULL,
  body_blob_ref     TEXT NOT NULL,
  retention_policy_id UUID NOT NULL,
  legal_hold_ids    UUID[] NOT NULL DEFAULT '{}',
  data_class        TEXT NOT NULL,
  received_at       TIMESTAMPTZ NOT NULL,
  soft_deleted_at   TIMESTAMPTZ NULL
);
CREATE INDEX idx_mail_msg_received_at ON mail_messages(mailbox_id, received_at DESC);
CREATE INDEX idx_mail_msg_legal_hold  ON mail_messages USING GIN (legal_hold_ids);
ALTER TABLE mail_messages ENABLE ROW LEVEL SECURITY;
-- ... same tenant_scope policy
```

```rust
// src/mailbox_repo.rs
pub struct PgMailboxRepository { pool: sqlx::PgPool }

#[async_trait]
impl MailboxRepository for PgMailboxRepository {
    async fn read(&self, id: MailboxId) -> Result<Mailbox, RepositoryError> {
        // Caller MUST `SET LOCAL app.tenant_id = ...` (Professional) or app.user_id (Personal)
        // before invoking; RLS otherwise refuses. The adapter does NOT bypass RLS.
        let row = sqlx::query!(
            r#"SELECT mailbox_id, tenant_id, context_kind, owner_ref, region, retention_policy_id
               FROM mailboxes WHERE mailbox_id = $1"#,
            id.0
        ).fetch_one(&self.pool).await?;
        Ok(Mailbox { /* ... */ })
    }
    // ... list_by_user, create
}
```

## Acceptance Gates

```bash
cargo check -p oya-mail-mailbox-store-adapter-postgres
cargo build -p oya-mail-mailbox-store-adapter-postgres
cargo clippy -p oya-mail-mailbox-store-adapter-postgres -- -D warnings
cargo nextest run -p oya-mail-mailbox-store-adapter-postgres --features integration-test
cargo run -p oya-dev-cli -- gate validate shardability --crate oya-mail-mailbox-store-adapter-postgres
cargo run -p oya-dev-cli -- gate validate rls-conformance --crate oya-mail-mailbox-store-adapter-postgres
```

## Test Plan

- Unit: per-query test against testcontainers Postgres.
- Integration: 2 tenants concurrently; RLS prevents cross-tenant read.
- Shardability: distributed-table path verified in citus container.
- Coverage 85%/75% (adapter class per PHASE-01).

## Halt Conditions

- RLS policy bypass found — refactor; the test must fail without `SET app.tenant_id`.
- sqlx compile-time query check fails — fix schema or query.
- Shardability lane finds non-tenant-keyed query — add tenant_id to WHERE.

## Next IP

[`IP-004-mailbox-store-s3-adapter.md`](IP-004-mailbox-store-s3-adapter.md)

## References

- ADR-0117 (data residency)
- ADR-0133 (cross-tenant pattern)
- Postgres RLS docs — `postgresql.org/docs/current/ddl-rowsecurity.html`
- Citus distributed tables — `docs.citusdata.com`
- sqlx — `github.com/launchbadge/sqlx`
