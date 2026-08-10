---
doc_status: published
id: ADR-0711
title: "Swarm Delivery Law: integration branch topology and command discipline"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0700, ADR-0701]
amended_by: []
depends_on: [ADR-0700, ADR-0701]
related: [ADR-0111, ADR-0363, ADR-0366, ADR-0515]
milestone: W0
deliverables:
  - id: ADR-0711-D1
    description: "Durable integ/<root> + integ/docs + integ/specs branch topology with machine-readable path envelopes."
    exit_criteria: "specs/integ-branch-envelopes.json exists; lists governed roots (os, ci, governance, workflow, build, cloud, flags, libs, console, oya, marketplace, registry) and planes (docs, specs); hub sole-owner list and adjunct claim rules are machine-readable; envelope self-ownership is integ/specs."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D2
    description: "Worktree-per-agent isolation plus worker git allowlist (no stash/reset) and server-side integ reset after land."
    exit_criteria: "PORTABLE-SWARM-CONTRACT.md carries Swarm Delivery Law; deliver.js Claim verifies envelope + merge-tree + hub exclusivity; Land upserts one PR per integ/<root> and documents server-side reset refspec; concurrent-safe exemptions for .beads/** and evidence/** are registered."
    verified_by: "oya-ci-required"
---
# ADR-0711: Swarm Delivery Law — integration branch topology and command discipline

## Status

**Proposed.** Phase A of the Swarm Delivery Law rollout (advisory doctrine + policy-as-data +
harness). Phase B lands a hermetic CI envelope check under `oya-ci-required`. Phase C (branch
protection restricting `dev` PRs to `integ/*` + `hotfix/*`) is founder-paired and deliberately
out of this ADR's acceptance criteria.

## Context

Parallel agent delivery on this monorepo repeatedly hit the same failure classes:

1. **False parallelism** — many unit PRs racing trunk, each paying ~29 min CI and decaying in the
   merge queue.
2. **Shared working directories** — stash/pop/reset chaos when two agents share one index/HEAD.
3. **Hub contention** — registry and index hubs edited from everywhere without a sole owner.
4. **Divergent long-lived branches** — date-stamped one-shot integ names that tooling cannot reuse.

Industry practice converged on scoped parallel merge lanes and worktree-per-agent isolation. This
repo already requires one isolated worktree per lane (`docs/AGENTS.md`) and a single protected
context `oya-ci-required` (ADR-0700 / ADR-0515). What was missing is a **durable, envelope-scoped
integration topology** plus a **mechanical command discipline** so workers cannot recreate the
stash/reset substrate even by accident.

This ADR does not replace ADR-0700 merge admission or ADR-0701 capability layout. It amends how
agent lanes assemble onto trunk: unit work never opens a trunk PR; domain integ branches do.

## Decision

### D-1 — Durable integ branches are the only trunk admission surface

One durable branch `integ/<root>` exists per governed top-level root:

`os`, `ci`, `governance`, `workflow`, `build`, `cloud`, `flags`, `libs`, `console`, `oya`,
`marketplace`, `registry`.

Plus planes:

- `integ/docs` — envelope `docs/**`
- `integ/specs` — envelope `specs/**`

Branch list and path envelopes are policy-as-data in `specs/integ-branch-envelopes.json`.
Changes reach `dev` only via a PR from `integ/*` (exception: `hotfix/*`, post-hoc review). At most
one open PR per integ. Unit work (`impl/*`, lane branches) never opens trunk PRs.

### D-2 — Envelope containment + hub sole-owner + adjunct claims

A PR from `integ/R` may touch only:

1. paths inside envelope(R), and
2. explicitly claimed adjunct leaves, and
3. waivered hub files.

Hub files are sole-owner per wave (one integ carries a given hub edit):

- `specs/masterplan.json`
- `specs/capability-registry.json`
- `specs/root-hub-pointers.json`
- `docs/ADR-INDEX.md`
- `docs/DOC-CATALOG.md`
- `docs/CHANGELOG.md`
- `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (and other equality-pinned
  `*-policy.json` census pins)
- `Cargo.lock`

A code integ carries a hub edit only with an in-diff waiver row (branch + hub + reason) under
`governance/check/integ-envelope/waivers/` so atomic co-changes stay possible and auditable.

### D-3 — Claim before commit (check-before-push)

Before pushing to `integ/R`, the integrator MUST:

1. `git fetch`
2. verify the unit diff ⊆ envelope(R) (+ claimed adjuncts + waivered hubs)
3. run read-only `git merge-tree` against the integ tip as a conflict pre-flight
4. verify hub exclusivity against open PRs
5. admit by cherry-pick
6. re-verify at the moment of push — stale green is not authorization

`--force-with-lease` is allowed only inside blessed restack/reset scripts, never as ad-hoc worker
vocabulary.

### D-4 — Server-side integ reset after squash-merge

After squash-merge of `integ/R` → `dev`, reset the remote integ **server-side** with a push
refspec — no local `git reset` anywhere:

```bash
git push --force-with-lease origin origin/dev:refs/heads/integ/R
```

The branch name persists; the next wave reuses it. Divergence never exceeds one wave.

### D-5 — Worktree topology

| Role | Path | Branch | Lifetime |
|---|---|---|---|
| Orchestrator + check daemon | main checkout | `dev` / tools | durable |
| Integration station | `.worktrees/integ-<root>` | `integ/<root>` | durable while root is active |
| Worker lane | `.worktrees/lane-<bead>` | `impl/<bead>` | ephemeral; created from `origin/dev`, removed after assembly |

Workers never edit the main checkout. Lanes are created explicitly from `origin/dev` (never from
ambient HEAD). Replicated-state budget: lanes never build, so they never grow `target/`.

### D-6 — Worker git command discipline

Structural fix first: worktree-per-agent removes the shared-index substrate. Then allowlist:

**Allowed for workers:** read-only git (`status`, `diff`, `log`, `show`, `fetch`, `merge-base`,
`merge-tree`, `rev-parse`); `git add <explicit paths>` (no `.` / `-A`); `git commit` immediately
after; `git push` via blessed script. One logical change = one commit of specifically named files.

**Denied for workers:** `stash`, `reset` (all forms), `clean`, `restore`, `checkout`, `rebase`,
`merge`, `branch -D/-f`, `update-ref`, `reflog expire`, `gc`, bare `push --force`.

Destructive operations exist only inside versioned, reviewed scripts that the integrator role runs
(restack, server-side reset, worktree remove). Integrator uses cherry-pick (commit-producing,
atomic) — still no stash/reset in its vocabulary.

Enforcement shims (`tools/swarm/git-shim`, `tools/swarm/toolguard`, `tools/swarm/check-daemon`)
are Phase A companions; this ADR is the law they enforce.

### D-7 — Special files and concurrent-safe exemptions

- Citation census pins are re-derived on the integ tip (oyatie-o90), never git-merged as authority.
- `Cargo.lock` lands with the integ that changed workspace membership.
- Concurrent-safe exemptions (`.beads/**`, per-lane `evidence/**`) are recorded in
  `registry/vcs/concurrent-safe-paths.yaml` and referenced from the envelope spec.

### D-8 — Self-reference

`specs/integ-branch-envelopes.json` is owned by `integ/specs` and founder-reviewed.

## Consequences

### Positive

- Open trunk PRs become a readable map of domains in flight.
- Cross-lane file clobbering is structurally prevented (worktree + envelope).
- CI cost is paid once per domain wave instead of once per unit.
- Stash/reset chaos has no substrate and a mechanical deny list.

### Negative / deferred

- Branch protection (Phase C) is founder-paired; until then admission is advisory + harness.
- Hermetic CI envelope check (Phase B) must land before the law is blocking.
- Hub waivers add a small process tax; that tax is cheaper than silent hub races.

### Rollout

| Phase | What lands | Blocking? |
|---|---|---|
| A (this ADR) | ADR + envelope JSON + PORTABLE-SWARM-CONTRACT + deliver.js Claim/Land + shims | advisory |
| B | `governance/check/integ-envelope/` under `oya-ci-required` | blocking |
| C | restrict `dev` PRs to `integ/*` + `hotfix/*` | founder-paired |

## Alternatives considered

- **GitHub native merge queue with scopes** — no native scope support; settings changes founder-paired;
  envelopes give scoped queueing without it.
- **Date-stamped / topic integ branches** — disposable names defeat reuse and tooling.
- **Unit PRs to `dev`** — false parallelism; banned.
- **Shared working directory + etiquette** — already failed; structural isolation required.
- **Rewriting deliver.js** — standing constraint: extend, do not rewrite.

## References

- Policy: `specs/integ-branch-envelopes.json`
- Portable rule text: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md`
- Harness: `.claude/workflows/deliver.js` (Claim + Land)
- Concurrent-safe registry: `registry/vcs/concurrent-safe-paths.yaml`
- Operating contract: `docs/AGENTS.md` (worktree-per-lane; `oya-ci-required`)
