# G026 CI runtime face-birth design — 2026-08-02

State: **PLANNING_ONLY — FACE-BIRTH MAP, NOT A MOVE PLAN, NOT ACTIVATED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-CI-PRODUCT-DESTINATION-FACE-PROOF-2026-08-02.md`.  
No path move, Cargo/BUCK rewrite, registry edit, push, or activation occurred.

## Purpose

Prove a non-colliding destination leaf for each of the twelve live CI runtime crates under the closed capability `ci` face grammar **before** any executable move-plan JSON is authored.

## Ownership (already proven)

`specs/capability-registry.json` capability `ci`:

- charter: delivery fabric — cloud-ci gates, controller, Tide merge queue, webhook gateway
- absorbs: `ci`, `oya/ci-controller`, `oya/ci-tide`, `oya/ci-webhook-gateway`

Live `ci/` today has ports/path-resolver, adapters/path-resolver, and many `ci/facade/*` **gates**. It has **no** `ci/core/*` and no controller/tide/webhook-gateway leaves. Gate plan `specs/reorg/ci-move-plan.json` (46 rows) is disjoint from these twelve crates.

## Source inventory (12 crates)

| Service | Crate | Role today | Path deps |
|---|---|---|---|
| controller | `oya-ci-controller-kernel` | pure domain, no I/O | none |
| controller | `oya-ci-controller-github-adapter` | CommitStatusPoster HTTP | → kernel |
| controller | `oya-ci-controller-k8s-adapter` | Job spawn/watch | → kernel |
| controller | `oya-ci-controller-app` | kube-rs controller composition + bin | → kernel, k8s-adapter, github-adapter |
| tide | `oya-ci-tide-kernel` | pure merge-queue domain | none |
| tide | `oya-ci-tide-github-adapter` | ForgeClient HTTP | → kernel |
| tide | `oya-ci-tide-app` | poll loop composition + bin | → kernel, github-adapter |
| webhook | `oya-ci-webhook-gateway-kernel` | pure webhook domain | none |
| webhook | `oya-ci-webhook-gateway-github-adapter` | commit-status poster | → kernel |
| webhook | `oya-ci-webhook-gateway-ed25519-adapter` | signature verify | → kernel |
| webhook | `oya-ci-webhook-gateway-authz-cedar-adapter` | Cedar authz gate | → kernel |
| webhook | `oya-ci-webhook-gateway-app` | axum composition + bin | → kernel + 3 adapters |

All twelve currently build as libraries; apps also ship `src/bin/*` entrypoints. Port traits live **inside kernels** today (no separate ports crates). That is intentional for face birth: do not invent empty `ci/ports/{controller,tide,webhook-gateway}` packages until a real trait extraction PR needs them. Live `ci/ports/path-resolver` remains unrelated.

## Destination face map (proposed)

Closed faces only: `core | ports | adapters | facade`.  
Leaf names are de-branded and service-scoped. Proposed Cargo package names follow path=namespace (`ci-<face>-<leaf>` style already used by gate facades such as `ci-license-policy`).

| # | Source path | Source package | Destination leaf | Dest package (proposed) | Face |
|---:|---|---|---|---|---|
| 1 | `oya/ci-controller/crates/oya-ci-controller-kernel` | `oya-ci-controller-kernel` | `ci/core/controller` | `ci-core-controller` | core |
| 2 | `oya/ci-controller/crates/oya-ci-controller-github-adapter` | `oya-ci-controller-github-adapter` | `ci/adapters/controller-github` | `ci-adapters-controller-github` | adapters |
| 3 | `oya/ci-controller/crates/oya-ci-controller-k8s-adapter` | `oya-ci-controller-k8s-adapter` | `ci/adapters/controller-k8s` | `ci-adapters-controller-k8s` | adapters |
| 4 | `oya/ci-controller/crates/oya-ci-controller-app` | `oya-ci-controller-app` | `ci/facade/controller` | `ci-facade-controller` | facade |
| 5 | `oya/ci-tide/crates/oya-ci-tide-kernel` | `oya-ci-tide-kernel` | `ci/core/tide` | `ci-core-tide` | core |
| 6 | `oya/ci-tide/crates/oya-ci-tide-github-adapter` | `oya-ci-tide-github-adapter` | `ci/adapters/tide-github` | `ci-adapters-tide-github` | adapters |
| 7 | `oya/ci-tide/crates/oya-ci-tide-app` | `oya-ci-tide-app` | `ci/facade/tide` | `ci-facade-tide` | facade |
| 8 | `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-kernel` | `oya-ci-webhook-gateway-kernel` | `ci/core/webhook-gateway` | `ci-core-webhook-gateway` | core |
| 9 | `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-github-adapter` | `oya-ci-webhook-gateway-github-adapter` | `ci/adapters/webhook-gateway-github` | `ci-adapters-webhook-gateway-github` | adapters |
| 10 | `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-ed25519-adapter` | `oya-ci-webhook-gateway-ed25519-adapter` | `ci/adapters/webhook-gateway-ed25519` | `ci-adapters-webhook-gateway-ed25519` | adapters |
| 11 | `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-authz-cedar-adapter` | `oya-ci-webhook-gateway-authz-cedar-adapter` | `ci/adapters/webhook-gateway-authz-cedar` | `ci-adapters-webhook-gateway-authz-cedar` | adapters |
| 12 | `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-app` | `oya-ci-webhook-gateway-app` | `ci/facade/webhook-gateway` | `ci-facade-webhook-gateway` | facade |

### Collision check against live `ci/` on tip

Immutable probes show **absent** for every destination leaf above. Live facade gate names (`license-policy`, `slo-coverage`, `affected-target-set`, …) do not include `controller`, `tide`, or `webhook-gateway`. No dest path collides with `ci/adapters/path-resolver` or `ci/ports/path-resolver`.

### Why apps land in `facade`, not a fourth face

The closed grammar has no `app/` face under a capability. Composition roots that wire core+adapters and expose the binary/API are the capability facade. Gate packages already occupy `ci/facade/*` as **check products**; runtime facades are a second product class under the same face directory, distinguished by leaf name. Do not invent `ci/app/*`.

### Why ports are not born yet

Kernels already own the port traits (`CommitStatusPoster`, `JobSpawner`, `ForgeClient`, `SignatureVerifier`, `WebhookAuthzGate`). Extracting them to `ci/ports/*` is a later REFACTOR with importer proof, not a prerequisite for first placement. First birth keeps ports collocated in core (current shape).

## Dependency orientation after birth (must hold)

```
ci/facade/{controller,tide,webhook-gateway}
    → ci/adapters/*
    → ci/core/*
ci/adapters/* → ci/core/* only
ci/core/* → no ci/adapters, no ci/facade, no oya/*
```

Same sandwich as today; only paths/names change. Facade gate packages must not gain edges into runtime facades in the birth PR unless a measured importer already requires it (today: catalog/membership/lock only).

## Importer / projection rewrite surface (not executed here)

Outside the three service trees, package-name hits concentrate in:

- `Cargo.lock` (workspace)
- `ci/facade/module-membership/capability-membership-policy.json`
- `ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json` (controller/tide)
- `registry/catalog/oya-ci-webhook-gateway-*.yaml` (+ stores/dependency-rationales)
- `ci/facade/affected-target-set` (controller-app path literal class)
- ADRs / audit docs (historical; no forced rewrite in move PR)
- deploy/image/workflow surfaces under each `oya/ci-*` tree (co-move with service)

Any future move PR is incomplete unless same-commit updates cover: path deps, package renames, BUCK targets, workspace/reindeer membership, module-membership + crate-catalog projections (producer-regenerated, never hand-edited generated faces), registry catalog rows, and deployment manifests that reference old crate/image names.

## Birth procedure (when unblocked)

Activation only after: independent design APPROVE + preferred #1526 observed green + #1523 promoted green (admission chain). Serial, one service at a time is allowed; all twelve may share one plan JSON **after** leaves exist or are created in the same PR as the moves.

Ordered mechanics per service:

1. **Create empty destination packages** (Cargo+BUCK+OWNERS+module registration) matching the table — or use codemod move that births the leaf in one step. Prefer the owned reorg codemod over hand moves.
2. **Move sources** kernel → adapters → facade (leaf-first dependency order).
3. **Rewrite path deps and package names**; run cold Buck2 + workspace member coverage.
4. **Regenerate** controller-owned catalog/membership faces via producers; never commit hand-edited `*.generated.json`.
5. **Delete** emptied `oya/ci-<service>/` tree only when no remaining tracked cargo/deploy residue.
6. **Do not** extend `ci-move-plan.json` gate rows; author a **new** `specs/reorg/ci-runtime-move-plan.json` only after this design is approved and destinations are non-colliding on the then-current tip.

## Explicit non-goals

- No move-plan JSON in this document’s landing.
- No product `app/<vertical>` birth (separate G026 product track).
- No tools↔libs moves.
- No G025 license/SLO kernel absorb.
- No rename of gate facades.
- No live cluster deploy of controller/tide/webhook during birth.

## Independent review

Not obtained. Agent/Codex/ouroboros review transports remain failed/quota-fused. This is coordinator design evidence only — **not APPROVE**.

## Non-claims

- Not permission to push or merge a move.
- Not proof that proposed Cargo names are final (naming grammar review may adjust `ci-core-*` vs `ci-*` stem rules at implementation time; path leaves above are the stable placement claim).
- Not a claim that ports extraction is forbidden forever — only deferred.
