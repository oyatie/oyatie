---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-009-thread-tree-and-mention-router
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, ontology-integration-smoke]
---

# IP-009: thread-tree + mention-router BCs

## Intent

Thread reply chains with parent-child traversal + participant tracking;
mention parser + Ontology-backed identity resolution for Person/Team/Channel;
fanout to notification + WebSocket; ingest action-cards from mail µservice.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-thread-tree-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}/...` | create |
| `src/crates/oya-messenger-mention-router-{kernel,domain,usecase,api,adapter,worker,sdk,app}/...` | create |
| `tests/mention_resolve_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-thread-tree-kernel
cargo nextest run -p oya-messenger-mention-router-kernel
cargo nextest run --test mention_resolve_e2e
```

## Test Plan

- Thread tree: 3-deep nesting; parent-only traversal; participant set correct.
- Mention resolve: `@alice` resolves via Ontology Person lookup; emits MentionEmitted ≤ 250ms p99.
- Mail action-card ingest: action-card → mention-router → post into target channel.

## Next IP

[`IP-010-read-receipt-tracker.md`](IP-010-read-receipt-tracker.md)
