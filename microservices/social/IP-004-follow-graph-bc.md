---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-004-follow-graph-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, sqlx-migration-lint, oya-governance-shardability, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: follow-graph BC (kernel → domain → usecase → adapter-postgres + worker + sdk)

## Intent

Implement the directed follow-graph with adjacency-list storage per ADR-SOC-0002.
Cover follow / unfollow / block / mute edges; mutual-follow = friend derivation;
audit-chain seal per edge mutation; periodic graph-drift detector.

## ChangeSet boundary

`follow-graph` BC end-to-end: kernel + domain + usecase + api + adapter-postgres + worker + sdk crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-follow-graph-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-follow-graph-domain/src/{follow_edge,block_edge,mute_edge,friend_derivation}.rs` | create |
| `src/crates/oya-social-follow-graph-usecase/src/{follow,unfollow,block,mute,list_followers,list_following}.rs` | create |
| `src/crates/oya-social-follow-graph-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-follow-graph-adapter-postgres/migrations/0001_init.sql` | create — adjacency-list + indexes |
| `src/crates/oya-social-follow-graph-worker/src/audit_chain_emitter.rs` | create |
| `src/crates/oya-social-follow-graph-worker/src/drift_detector.rs` | create |
| `tests/follow_graph_e2e.rs` | create — testcontainers Postgres |

## Code Shape

```sql
-- migrations/0001_init.sql
CREATE TABLE social_follow_edges (
    follower_ref      text NOT NULL,
    followee_ref      text NOT NULL,
    tenant_id         text NOT NULL,
    established_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, follower_ref, followee_ref)
) PARTITION BY HASH (tenant_id);

CREATE INDEX social_follow_edges_followee_idx
  ON social_follow_edges (tenant_id, followee_ref);  -- reverse lookup

CREATE TABLE social_block_edges (
    blocker_ref       text NOT NULL,
    blocked_ref       text NOT NULL,
    tenant_id         text NOT NULL,
    blocked_at        timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, blocker_ref, blocked_ref)
) PARTITION BY HASH (tenant_id);

CREATE TABLE social_mute_edges (
    muter_ref         text NOT NULL,
    muted_ref         text NOT NULL,
    tenant_id         text NOT NULL,
    muted_at          timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, muter_ref, muted_ref)
) PARTITION BY HASH (tenant_id);

ALTER TABLE social_follow_edges ENABLE ROW LEVEL SECURITY;
ALTER TABLE social_block_edges ENABLE ROW LEVEL SECURITY;
ALTER TABLE social_mute_edges ENABLE ROW LEVEL SECURITY;
```

## Acceptance Gates

```bash
cargo nextest run -p oya-social-follow-graph-kernel
cargo nextest run -p oya-social-follow-graph-domain
cargo nextest run -p oya-social-follow-graph-adapter-postgres
cargo run -p oya-dev-cli -- gate validate shardability --microservice social
```

## Test Plan

- Unit tests: friend-derivation (mutual-follow = friend).
- Integration: follow + unfollow roundtrip; reverse lookup p99 ≤ 50ms.
- Drift detector: synthetic divergence (Postgres ≠ audit-chain replay) flagged.
- Mass-follow rate limit (FM-05 prevention): 100 follows/hr enforced.

## Halt Conditions

- Adjacency-list partition strategy regression — fix; re-shard.
- Audit-chain seal missing on edge mutation — block.

## Next IP

[`IP-005-post-composition-bc.md`](IP-005-post-composition-bc.md)

## References

- ADR-SOC-0002 (follow-graph storage rationale).
- `microservices/social/runbooks/follow-graph-corruption.md`.
- `microservices/social/threat-model.md` T-T-03.
