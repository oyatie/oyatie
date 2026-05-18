---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-007-reactions-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: reactions BC (kernel → domain → usecase → adapter-postgres + adapter-redis + worker + sdk)

## Intent

Author the `reactions` BC: inline reactions (bounded emoji set) with
conflict-free counter, Redis-buffered + Postgres flush, per-user idempotency
(one reaction per user per post per emoji).

## ChangeSet boundary

`reactions` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-reactions-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-reactions-domain/src/{reaction,tally,user_reaction_record}.rs` | create |
| `src/crates/oya-social-reactions-usecase/src/{add,remove,tally}.rs` | create |
| `src/crates/oya-social-reactions-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-reactions-adapter-postgres/migrations/0001_init.sql` | create |
| `src/crates/oya-social-reactions-adapter-redis/src/counter.rs` | create — Redis HINCRBY-based counter |
| `src/crates/oya-social-reactions-worker/src/flush_postgres.rs` | create — periodic Redis → Postgres flush |
| `tests/reactions_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-reactions-kernel
cargo nextest run -p oya-social-reactions-domain
cargo nextest run -p oya-social-reactions-adapter-redis
```

## Test Plan

- Per-user-per-post idempotency: add same reaction twice → single record.
- Redis ↔ Postgres reconciliation: synthetic divergence flagged.
- Reaction count consistency after restart.
- React p99 ≤ 50ms.

## Halt Conditions

- Counter consistency drift > 0 — escalate; rebuild from authoritative Postgres.

## Next IP

[`IP-008-mentions-and-hashtags-bc.md`](IP-008-mentions-and-hashtags-bc.md)
