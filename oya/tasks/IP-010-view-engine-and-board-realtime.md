---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-010-view-engine-and-board-realtime
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, board-rerank-determinism, crdt-merge-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: view-engine BC — board DnD + collaborative description CRDT (Loro 1.x)

## Intent

Ship the `view-engine` BC end-to-end. Per ADR-TASKS-0004: deterministic
LexoRank-style re-rank for board drag-and-drop moves (no CRDT — pure
deterministic algorithm); Loro 1.x CRDT for collaborative description
editing only (aligned with workflow-studio ADR-WS-0001 CRDT scope).
Saved-view kinds per PRD: list / board / gantt / calendar / timeline /
table. Valkey-backed `ViewStateStore` for presence + cursor; per-tenant
key prefix; cluster-mode-safe.

Realtime path: WebSocket gateway on `view-engine-rest` (port 8443)
fans out reorder events + CRDT description deltas to subscribers in
the same project. Backpressure: bounded per-connection ring buffer
(1k events); slow consumers dropped with `429 Channel::Backpressure`.

## ChangeSet boundary

8 view-engine crates (kernel/domain/usecase/api/adapter/adapter-valkey/
rest/app) — kernel + domain already authored in IP-005; this IP fills
the remaining 6. Loro 1.x dependency added; Valkey adapter (RESP wire-compatible) implements
`ViewStateStore` against Valkey 8.1 (RESP wire-compatible).

## Crate Naming

`oya-tasks-view-engine-{usecase,api,adapter,adapter-valkey,rest,app}`
per ADR-0056 v4.1 + ADR-0105 Amendment 3 backend-qualification.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-view-engine-{usecase,api,adapter,adapter-valkey,rest,app}/src/lib.rs` | created/replaced | 6-crate completion |
| `microservices/tasks/src/oya-tasks-view-engine-domain/tests/rerank_determinism.rs` | created | property test |
| `microservices/tasks/src/oya-tasks-view-engine-usecase/tests/crdt_merge.rs` | created | Loro convergence |
| `microservices/tasks/catalog/oya-tasks-view-engine-*.yaml` | created | catalog entries |

## Acceptance Gates

```bash
cargo test -p oya-tasks-view-engine-domain rerank_determinism
cargo test -p oya-tasks-view-engine-usecase crdt_merge
buck2 build //:quality-lane-registry-authority-check # lane=board-rerank-determinism --microservice tasks
buck2 build //:quality-lane-registry-authority-check # lane=crdt-merge-correctness --microservice tasks
```

## Test Plan

- Reorder determinism: 1000-task project; 100 random reorder ops in
  random arrival order across 3 clients converge to the same final
  ranking (LexoRank-style closed-form proof).
- CRDT convergence: 3 clients concurrently edit a task description;
  Loro RGA merge yields the same final document across all 3 after
  bidirectional sync (no lost writes).
- WebSocket backpressure: slow consumer drops; subsequent reconnect
  receives `last_event_id`-based replay.

## Halt Conditions

- Re-rank non-determinism — refuse; fix at domain.
- CRDT divergence between clients — refuse; this is a P0 invariant.

## Next IP

[`IP-011-search-and-filter.md`](IP-011-search-and-filter.md)

## References

- ADR-TASKS-0004 (CRDT for description; rank-key for moves).
- ADR-WS-0001 workflow-studio CRDT scope alignment.
- Loro 1.x — `loro.dev`.
- LexoRank — `medium.com/whisperarts/lexorank-a-ranking-algorithm`.
