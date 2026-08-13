---
doc_class: Runbook
template_id: TPL-RUNBOOK
runbook_id: RB-AGENTIC-GJC-DISPATCH
title: "Dispatch autonomous GJC work into oyatie without dirtying the hub"
status: active
severities_supported: [Sev-3, Sev-4]
owner_team: axis-foundry + council-architecture
last_verified: 2026-08-13
last_drilled: null
slo_topic: oya.foundry.pipeline.dispatch
audit_emission_topic: oya.ops.runbook.invoked
related_runbooks: [RB-SANCTIONED-PRIMITIVES-PREFLIGHT]
related_adrs: [ADR-0711, ADR-0700, ADR-0515]
diataxis_class: how-to
data_classes_touched: [INTERNAL_ONLY]
audience: INTERNAL
authority_chain_declaration: |
  docs/AGENTS.md sanctioned worktree sequence + ADR-0711 worktree-per-agent isolation +
  ADR-0700 / ADR-0515 oya-ci-required merge admission > this runbook.
doc_status: published
---

# Runbook RB-AGENTIC-GJC-DISPATCH: Dispatch autonomous GJC work into oyatie without dirtying the hub

## Trigger / symptom

Open this runbook when a GJC master must send implementation work into the oyatie hub without writing tracked files in the hub checkout.

- Autonomous master about to enqueue workers against oyatie.
- Hub `git status --porcelain` would otherwise grow from lane worktrees.
- Wrong-root enqueue (`INVALID_WORKDIR`) or unvalidated capacity raise.

Do not use this runbook for production incidents, merge-queue conflicts, or Swarm Delivery Law `integ/*` land. Those route to their own runbooks.

## SLO impact

- SLO affected: `oya.foundry.pipeline.dispatch`.
- This is planned dispatch work, not live mitigation.
- Hub dirt from unsanctioned worktrees is Sev-3 process failure; treat as stop-the-line for further enqueue.

## Purpose

Autonomous GJC masters MUST dispatch work into isolated lane worktrees under the hub's `/.worktrees/` tree so the hub checkout stays a read-only coordination surface. The root-anchored gitignore entry `/.worktrees/` keeps those checkouts out of hub porcelain. Nothing in this runbook authorizes editing tracked hub files, stashing, resetting, or removing worktrees.

## Pre-checks (5 minutes max)

- [ ] Hub is not the write root — `git -C <hub> status --porcelain` is recorded; do not clean, stash, or reset it.
- [ ] `origin/dev` is fetched — `git -C <hub> fetch origin dev` then `git -C <hub> rev-parse origin/dev`.
- [ ] Target lane path is under `<hub>/.worktrees/` and does not already exist.
- [ ] Target branch name is `agent/<id>` and is not already checked out.

If any pre-check fails, **STOP**. Do not enqueue Phase B. Do not raise capacity.

## Two-phase batch protocol

### Phase A — provision-only (capacity 1, hub cwd)

Run a single provision-only worker with working directory = hub cwd and capacity 1. That worker MUST only fetch + add the lane worktree. It MUST produce a lane manifest before any implementation enqueue:

- `hub_cwd`
- porcelain census before provision
- porcelain census after provision
- `origin/dev` SHA
- lane path
- lane HEAD SHA
- lane branch

Phase A MUST NOT edit tracked hub files. If adding the worktree changes hub porcelain, record that as evidence that `/.worktrees/` must be ignored; do not "fix" the hub.

### Phase B — implement in the lane workdir

After the manifest validates, the master enqueues autonomous tasks with an explicit per-lane `workdir` equal to the manifest lane path.

- Wrong-root enqueues (hub cwd, sibling checkouts, or any path not in the manifest) are rejected with `INVALID_WORKDIR`.
- Capacity raises above 1 are gated on a valid Phase A manifest. No manifest, no fan-out.

## Sanctioned lane provisioning

Use this form only (hub-relative; do not invent sibling paths):

```sh
git fetch origin dev
git worktree add <hub>/.worktrees/lane-<id> -b agent/<id> origin/dev
```

Exact example used by this mechanism PR:

```sh
git -C /Users/jasonlee/Developer/oyatie fetch origin dev
git -C /Users/jasonlee/Developer/oyatie worktree add /Users/jasonlee/Developer/oyatie/.worktrees/lane-m3-dispatch-runbook-20260812 -b agent/m3-dispatch-runbook-20260812 origin/dev
```

## Lane lifecycle

- One writer per lane. A finished agent id is not evidence the worktree is free.
- At terminal state the lane is **PARKED** in place. NEVER `git worktree remove`.
- NEVER `pkill`/`killall` git, and NEVER `rm` `index.lock` / `gc.pid`.
- Parked register fields: `path`, `branch`, `head`, `owner`, `reason`.

## Landing

- Open a normal protected PR to `dev` from the lane worktree (`gh pr create --base dev`).
- `oya-ci-required` MUST be green before merge.
- No automation merges. No auto-merge. Humans squash-merge after review.

## Rollback

If Phase A or B dirtied the hub or wrote outside the lane:

1. STOP further enqueue.
2. Leave the lane worktree parked.
3. Do not stash, reset, checkout, or clean the hub.
4. Capture hub porcelain, lane path, and tip SHA; escalate.

## Verification

- [ ] Lane path is `<hub>/.worktrees/lane-<id>` and HEAD is a descendant of the recorded `origin/dev`.
- [ ] Implementation commits exist only in the lane; hub tracked files unchanged.
- [ ] Hub porcelain count is unchanged, or the delta is only untracked paths that prove `/.worktrees/` was missing.
- [ ] Protected PR against `dev` exists; `oya-ci-required` is the merge authority.
- [ ] Lane remains parked; no `git worktree remove` ran.

## Post-incident updates

- [ ] Update this runbook if a new wrong-root or porcelain class appears.
- [ ] Add a `docs/MISTAKES-LEDGER.md` row if a mechanical prevention is identified.
- [ ] Bump `last_verified` and the `docs/RUNBOOKS-INDEX.md` row.

## Audit-chain emission

Emit `oya.ops.runbook.invoked` with: `runbook-id=RB-AGENTIC-GJC-DISPATCH`, invoker-id, timestamp, outcome (`resolved|escalated|unresolved`), hub SHA, lane path, lane HEAD, and whether `INVALID_WORKDIR` fired.

## Sources

- [`docs/AGENTS.md`](../../AGENTS.md) isolated-worktree sequence.
- [ADR-0711](../../decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md) worktree-per-agent isolation.
- [ADR-0700](../../decisions/ADR-0700-ci-admission-live-apex.md) / [ADR-0515](../../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md) `oya-ci-required`.
- [`templates/runbook-template-v2.md`](../../templates/runbook-template-v2.md).
