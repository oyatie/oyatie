# Parallel-Development Plan — owned stack + product infra + products + reorg

**Author:** codex `gpt-5.6-sol` @ `model_reasoning_effort=ultra` (read-only), 2026-07-10.
**Basis:** committed `origin/dev` @ `1e4347639ff8b87e74c3bb76d947fc18dbc282ef` via `git show`/`git grep`/`git ls-tree` (active checkout had 1,591 dirty entries → treated as unsafe planning evidence).
**Raw output:** `scratchpad/codex-parallel-dev-plan-output.txt` (final message lines ~25696-26095).
**Status:** DELIVERED. Cross-review/verify (opus) IN FLIGHT — the 8 hostile corrections below are Sol-Ultra claims to be VERIFIED against ground truth before they update doctrine/memory.

## Verdict: PARALLEL-WITH-LOCKS (high confidence)

The safe unit is a **write-disjoint target slice behind frozen ports** — NOT a whole capability dir or an overlapping Buck `rdeps()` closure.

- **10–12 stable-capability lanes can author concurrently NOW.**
- **One writer at a time per shared hub / public port.**
- **One active reorg-move PR at a time** (move *prep* parallel; *landing* serial).
- **Implementation in pre-move locations is prohibited.**
- **CI admission is serial at integration, not during authoring.**
- Lower-layer **internal** change → does not block consumers. Lower-layer **public-interface** change → serializes through a contract PR + downstream fan-out.

## Hostile corrections to my premises (VERIFY before trusting)

1. **Reorg is NOT ~90% done.** ~**307** `[package]` manifests remain in pre-move dirs: intelligence 124, CI 12, **OS 41, kernel 20, app products 110** (925 total). My "only intelligence+ci left" was wrong.
2. **Asterinas canonical-kernel is NOT an accepted decision.** ADR-0611:34 says the pivot is a *separate founder decision*; `cloud/cloud-kernel/` still holds a 7-member workspace. (Contradicts my `asterinas-canonical-pivot` memory.)
3. **kernel→os→k8s→… is a runtime/cutover order, NOT the Buck build DAG.** Real graph has cross-cutting edges: **k8s depends on IAM**, compute↔network feedback, storage consumes compute+network.
4. **Only `oya-data-boundary-kernel` is a real build hub** (128 BUCK consumers / 332 refs). PDP=7, canonical-json=1, corpus core=1, messaging=3 → these are **semantic/governance locks, not build hubs.** (Contradicts "canonical-json/corpus/PDP all high-fan-in".)
5. **The merge queue does NOT serialize live.** `merge_group` wired but ADR-0554 says inert until enablement; Tide `dry_run=true`; no Tide deploy manifests. Operating model = **many parallel authors, one full-state admission candidate at a time.**
6. **Policy is inside IAM, not a separate capability** (`iam/core/policy-cedar-domain`, `iam/ports/policy-cedar-api`, `iam/adapters/pdp-cedar`). One writer across the IAM/PDP seam.
7. **affected-set fail-closed is ALREADY landed** — "being hardened to fail-safe" is *stale framing*; the real remaining risk is **graph VISIBILITY/completeness** (ADR-0562 records blindness to new `intelligence/*` edges). → my `ci-affected-set-soundness` work is correct but should be framed as **completeness hardening**, not fail-open→fail-closed.
8. **Console/app is NOT pure greenfield:** `console/` 9 pkgs, `oya/application/` 8 pre-move shell pkgs. Only the living-graph UI + new spatial/multi-platform surfaces + `app/ops-console` are greenfield.

## Per-capability classification (drives the schedule)

**impl-now (stable location):** k8s (18), compute (8), storage (8), network (8), messaging (3), gateway (10), **IAM (68, #1287 no longer a blocker)**, IAM/policy seam, IAC (5), observability (5), data/runtime-ontology (23), governance/corpus seed (5), workflow (48, minus intelligence-integration), console (9), billing (17).

**reorg-first-then-impl:** **intelligence** (124 under `oya/intelligence`), **CI** (12: ci-controller 4 / ci-tide 3 / ci-webhook-gateway 5), **application** (`oya/application`→`app/application`), **existing product verticals** (110 across 33 `oya/*` dirs — move each to `app/<product>/` first), **kernel** (20, gated on the founder canonical-kernel decision), **OS** (41, after kernel contract).

**greenfield (create at destination `app/`):** full living-management-graph UI, healthcare/health-diagnostics/ops-console/spatial/multi-platform surfaces.

## Coordination locks (one-writer, interface-freeze-then-fan-out)

`libs/oya-data-boundary-kernel` (128) · `network/core/residency` (~83) · old OS/kernel labels (~82) · IAM/PDP seam · shared HTTP kernels (router/middleware) · `data/ports/ontology-api` · `governance/corpus/core` signature schema · messaging substrate vs workflow event-bus · canonical-json + CI gates (governance authority) · Cargo.lock + root workspace + capability-registry + move-plan (reorg-coordinator sole writer).

Protocol for every public port: `contract PR → green full graph → freeze version → parallel consumer fan-out`.

## Reorg conveyor (serial landing order)

1. Finish **intelligence** sub-batches (124, recompute each batch after predecessor lands).
2. Move **CI 12** → then live queue/controller work at `ci/`.
3. Move **`oya/application` → `app/application`** (add app workspace glob once).
4. Move **app products** one product/cohesive-batch per PR (payments/office/community get own batches).
5. Resolve **kernel authority** → move kernel → move OS (skip without idling conveyor if authority not ready).

Prep in parallel (isolated worktrees, no competing writes to Cargo.lock/root-toml/registry/move-plan/baselines): intelligence rename maps, CI move dry-run, app workspace mapping, product mappings, kernel collision inventory, OS label rewrite inventory.

## Top risks

Incorrect reorg-completion assumption (critical) · dual kernel authority (critical) · incomplete machine DAG / Tier-2 forward-declared (high) · old-label coupling into stable destinations (high) · data-boundary hub blast radius (high) · projected-state-queue overclaim (high) · 110-package app-move surface (high) · Proposed public contracts ADR-0035/0562/0611/0617 (med-high) · ontology-vs-corpus category error (med-high) · planning off the dirty checkout (operationally high).

## Ordered first moves

1. Stand up **one reorg coordinator + hub-lock ledger** (sole writer of move-plan/Cargo.lock/registry/catalogs; records the locks above).
2. Start the **intelligence move conveyor** now; no more impl in `oya/intelligence`.
3. Kick off **stable internal lanes in parallel today** (the impl-now list) with the exclusions enforced.
4. Open the **kernel-authority lane** (Asterinas replaces vs coexists) without blocking others; freeze kernel/OS port.
5. After intelligence → move CI → move application.
6. Release product lanes only on **post-move** paths.
7. Turn queue deployment into a **measured** throughput project after the CI move.
