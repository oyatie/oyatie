---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-009-trending-topics-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: trending-topics BC (kernel → domain → usecase → adapter-postgres + adapter-redis + worker + sdk)

## Intent

Author the `trending-topics` BC: windowed compute over hashtag + mention
emissions; per-tenant + per-pack ranking; sybil-detector signal integration
from foundry-guardrails; tenant-admin pin/unpin override; EU DSA Art. 27
recommender-transparency for trending visibility.

## ChangeSet boundary

`trending-topics` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-trending-topics-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-trending-topics-domain/src/{trending_topic,trend_window,trend_rank,sybil_signal}.rs` | create |
| `src/crates/oya-social-trending-topics-usecase/src/{compute_window,rank,apply_sybil_filter}.rs` | create |
| `src/crates/oya-social-trending-topics-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-trending-topics-adapter-redis/src/cache.rs` | create |
| `src/crates/oya-social-trending-topics-worker/src/compute_loop.rs` | create — 5min windowed compute |
| `tests/trending_topics_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-trending-topics-kernel
cargo nextest run -p oya-social-trending-topics-domain
```

## Test Plan

- Windowed compute: synthetic hashtag emissions across 5min window → expected ranking.
- Per-author influence cap (sybil resistance): single author cannot inflate rank > N posts.
- foundry-guardrails sybil signal applied: artificial trend dropped in ranking.
- Tenant-admin pin/unpin override.
- Per-pack trending computed independently; cross-pack trending forbidden.

## Halt Conditions

- Trending poisoning detected (FM-17) → `runbooks/trending-topic-poisoning.md` activates.

## Next IP

[`IP-010-notifications-bc.md`](IP-010-notifications-bc.md)
