---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-004-postgres-adapters-blinding-migration
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + ops-data
acceptance_lanes: [cargo-check, sqlx-prepare, oya-governance-blinding-column-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: Postgres adapters with blinding-column migration

## Intent

Author 4 Postgres adapters (post-thread, affinity-attestation, legal-process-disclosure, retention-policy) and the matching schema migrations. Critical: the `posts` table has a `blinded_commitment` column and explicitly NO `user_id` column. The LEAN lane `oya-check-blinding-column-isolation` enforces this at CI.

## ChangeSet

| Path | Action |
|---|---|
| `src/post-thread-adapter-postgres/*` | create |
| `src/affinity-attestation-adapter-postgres/*` | create |
| `src/legal-process-disclosure-adapter-postgres/*` | create |
| `src/retention-policy-adapter-postgres/*` | create |
| `migrations/20260517_initial.sql` | create |
| `migrations/20260517_blinding_columns.sql` | create |

## Schema overview

```sql
CREATE TABLE posts (
  post_id UUID PRIMARY KEY,
  blinded_commitment BYTEA NOT NULL,  -- NEVER user_id (I1)
  affinity_cluster_id TEXT NOT NULL,
  body_ciphertext BYTEA NOT NULL,
  posted_at TIMESTAMPTZ NOT NULL,
  retention_tier TEXT NOT NULL CHECK (retention_tier IN ('30d', '60d', '90d')),
  audit_chain_seal BYTEA NOT NULL,
  tenant_id TEXT NOT NULL
);
CREATE INDEX idx_posts_affinity ON posts(affinity_cluster_id, posted_at DESC);
CREATE INDEX idx_posts_retention ON posts(retention_tier, posted_at);
-- RLS per tenant_id; per-affinity scope further restricted via Cedar
ALTER TABLE posts ENABLE ROW LEVEL SECURITY;
```

## Acceptance

- `cargo check` passes
- `oya-check-blinding-column-isolation` verifies no `user_id` column exists in any anonymous-µservice migration
- `sqlx-prepare` exits 0
