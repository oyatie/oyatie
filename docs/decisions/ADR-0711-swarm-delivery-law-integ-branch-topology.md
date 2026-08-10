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
related: [ADR-0111, ADR-0119, ADR-0131, ADR-0363, ADR-0366, ADR-0515, ADR-0541, ADR-0554, ADR-0562]
milestone: W0
deliverables:
  - id: ADR-0711-D1
    description: "Durable integ/<root> + integ/docs + integ/specs branch topology with machine-readable path envelopes."
    exit_criteria: "specs/integ-branch-envelopes.json exists; lists governed roots (os, ci, governance, workflow, cell, comms, data, iam, build, cloud, flags, libs, console, oya, marketplace, registry, tools) and planes (docs, specs); hub sole-owner list and adjunct claim rules are machine-readable; envelope self-ownership is integ/specs."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D2
    description: "Worktree-per-agent isolation plus worker git allowlist (no stash/reset) and server-side integ reset after land."
    exit_criteria: "PORTABLE-SWARM-CONTRACT.md carries Swarm Delivery Law; deliver.js Claim verifies envelope + merge-tree + hub exclusivity; Land upserts one PR per integ/<root> and documents server-side reset refspec; concurrent-safe exemptions for .beads/** and evidence/** are registered."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D3
    description: "Hyperscaler monorepo patterns + anti-patterns encoded as first-class Swarm Delivery Law (not agent-swarm lessons alone)."
    exit_criteria: "ADR-0711 and PORTABLE-SWARM-CONTRACT.md each have a dedicated Hyperscaler monorepo patterns section; specs/integ-branch-envelopes.json carries matching notes citing ADR-0119/0131/0515/0541/0562/0700/0701."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D4
    description: "Amendment A — docs governability epic gate, buck2 [check] daemon (no cargo revival), comment doctrine, generated-files doctrine."
    exit_criteria: "ADR-0711 Amendment A + PORTABLE-SWARM-CONTRACT Amendment A present; check-daemon invokes buck2 build //...[check] only under SWARM_ORCHESTRATOR=1 with zero cargo build/check/test/clippy invocations; docs-governance beads epic exists gated on integ/docs+integ/specs live."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D5
    description: "Amendment B — reorg-target debt freeze: no new path births under libs/, cloud/, oya/, infra/, toolchains/, tools/ except explicit reorg-move-out beads; tools/swarm/** one-shot bootstrap on integ/specs then shrink-only."
    exit_criteria: "ADR-0711 Amendment B + PORTABLE-SWARM-CONTRACT Amendment B present; specs/integ-branch-envelopes.json lists frozen_prefixes; deliver.js Claim rejects new births under frozen prefixes unless bead title/body marks reorg-move-out."
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

`os`, `ci`, `governance`, `workflow`, `cell`, `comms`, `data`, `iam`, `build`, `cloud`,
`flags`, `libs`, `console`, `oya`, `marketplace`, `registry`, `tools`.

Plus planes:

- `integ/docs` — envelope `docs/**`
- `integ/specs` — envelope `specs/**`

**Capability-root note (2026-08-10):** top-level dirs `cell/`, `comms/`, `data/`, and `iam/`
are capability roots under ADR-0701. They are now first-class governed integ envelopes
(`integ/cell`, `integ/comms`, `integ/data`, `integ/iam`) — ownership = path = integ scope
(D-9). Residual unit-PR content under those roots lands on the matching durable integ, not
as standalone trunk PRs.

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

### D-9 — Hyperscaler monorepo patterns (first-class)

These are **repo law**, not optional etiquette and not merely agent-swarm folklore. They bind
layout, CI economics, and integ envelopes together. Cite live apex ADRs where they exist;
historical ADRs are provenance only.

#### Required patterns

1. **Capability-first layout = ownership = integ scope.** `integ/<root>` envelopes align to
   top-level capability/meta roots. Path ownership and merge scope are the same fact
   (ADR-0701 / ADR-0562 lineage).
2. **Clean architecture per capability.** Envelopes follow capability boundaries
   (`core/` / `ports/` / `adapters/` / `facade/`), not arbitrary folder dumps
   (ADR-0701 / ADR-0131 lineage).
3. **Central `docs/` and `specs/` are exceptions for cross-cutting hubs only.** Capability-owned
   product artifacts colocate with the owner (e.g. `{oya,cloud}/<service>/specs/`). Prefer
   colocation for new product specs. Flat root `specs/` / `docs/` remain for hubs and
   cross-cutting governance only (ADR-0119 + ADR-0131 product-owned-spec colocation amendment;
   live layout apex ADR-0701). Do **not** type-dump product artifacts into central hubs.
4. **Trunk-based economics with scoped parallel lanes.** `integ/<root>` is the merge scope.
   Serialize only at trunk and on hub contention. No long-lived divergent feature branches —
   integ content resets to `dev` after every land (this ADR D-1 / D-4).
5. **Hermetic policy-as-data gates in one blocking CI context.** Merge authority is solely
   `oya-ci-required` (ADR-0700 restating ADR-0515). Checks consume declared SCM facts only —
   never ambient `git` probes inside hermetic evaluators.
6. **Selective / affected testing doctrine.** Worker lanes do not require a full monorepo
   rebuild. Binding CI uses the affected-set cone (ADR-0554 / ADR-0700 lineage). Workers read
   orchestrator `err.txt`; they never run `cargo` / `buck2` locally. The orchestrator check
   daemon (main checkout only) runs `buck2 build //...[check]` — cargo check is retired
   (founder directive 2026-05-29; Amendment A-2).
7. **Owner-colocated docs (g3doc / ADR-0541).** Co-change code + docs when editing a leaf.
   Central hubs (`docs/**` indexes, root `specs/**` hubs) are sole-owned by `integ/docs` or
   `integ/specs` (or hub-waivered) — not edited from every code PR.
8. **Concurrent-safe paths registry for true concurrent writers.** Only registered exemptions
   (`.beads/**`, per-lane `evidence/**`, …) may overlap; product code remains one-writer-per-path
   (`registry/vcs/concurrent-safe-paths.yaml`, ADR-0111 lineage).
9. **Small frequent lands onto durable integs; squash to trunk; server-side integ reset.**
   Prefer many small cherry-picks onto `integ/<root>` over mega-branches. After squash-merge,
   reset remote integ with the blessed refspec (D-4) — never local `git reset`.
10. **CODEOWNERS / path-envelope discipline — one writer queue per integ tip.** At most one open
    trunk PR per `integ/<root>`; Claim serializes writers onto that tip. Envelope + hub
    exclusivity are the mechanical CODEOWNERS equivalent for swarm assembly.

#### Forbidden anti-patterns

- **Type-based central dumping** of product artifacts into root `specs/` / `docs/` (dual-home
  debt; fights ADR-0119/0131/0701 colocation doctrine).
- **N unit PRs racing trunk** under slow CI (~29 min) — false parallelism; banned (D-1).
- **Shared working directory** for parallel agents — stash/reset/clean chaos substrate; banned
  (D-5 / D-6).
- **Long-lived divergent topic branches** with date suffixes as durable names — integ names are
  durable; content is not (D-4).
- **Editing hub files from every code PR** without sole-owner / in-diff waiver (D-2).
- **Agents running `cargo` / `buck2` / other slow commands in worker lanes** — orchestrator check
  daemon + affected CI only (D-5 / D-6; pattern 6). `cargo build|check|test|clippy|run|bench`
  is retired repo-wide (Amendment A-2); workers also never invoke `buck2`.
- **Inventing work / inventing lanes for empty verified space** — claim only labeled
  `implementable` beads; do not fabricate scope to keep agents busy.

#### Swarm operating lessons (carried, still binding)

Kept from Bun / Cursor swarm / Gaebal-gajae practice and already encoded in Claim/Land /
PORTABLE-SWARM-CONTRACT: one implementer + adversarial reviewers; planner ≠ implementer;
fix the process not the output; batch same-subsystem work into one lane; green CI is not
authorization — re-verify at the moment of action; never delete a git lock without checking the
owning process; automation stops at the edge of its authority; design cleanup/ownership for
replicated state before scaling parallelism.

### Amendment A (2026-08-10)

Binding amendment to the Swarm Delivery Law. Full portable mirror:
`.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md` § Amendment A.

#### A-1 — Docs governability track (beads epic)

N docs / JSONs / YAMLs / ADRs without findability produces stale, duplicate, and conflicting
decisions. Create a `docs-governance` beads epic, **gated on `integ/docs` + `integ/specs`
going live** (do not activate execution until both planes have landed trunk PRs). Scope:

1. Inventory + dedupe map against `docs/DOC-CATALOG.md` / registry projections.
2. Enforce `specs/markdown-retirement-policy.json` phases.
3. Admission rule for new docs: register owner + catalog row + supersede check (no orphans;
   extend the `adr-orphan-detect` pattern).
4. Findability via machine-readable indexes as entry points.
5. Contradiction detection via `docs/CONTRADICTION-LEDGER.md` discipline.
6. Growth budget on root `docs/` / `specs/` with colocation preferred (ADR-0131 / ADR-0701).

#### A-2 — Fast-feedback plane correction (no cargo revival)

`cargo check` is **retired** by founder directive 2026-05-29, codified in
`tools/hooks/no-cargo-enforcer.sh`. The hook names `buck2 build //...[check]` as the
type-check equivalent (`rustc --emit=metadata`).

- **Check daemon** (orchestrator, main checkout, `SWARM_ORCHESTRATOR=1` only): runs
  `buck2 build //...[check]` (or an explicit per-target list when configured), parses rustc
  diagnostics from buck2 stderr/stdout, groups by crate then file, writes `err.txt` at the
  main-checkout root and `.check/errors.json`. Single builder keeps the buck2 daemon +
  `buck-out` warm.
- **Worker lanes:** `tools/swarm/toolguard` continues to deny **both** `cargo` and `buck2`.
  Workers read `err.txt`; they never build.
- **sccache:** considered and **rejected**. It is cargo/`RUSTC_WRAPPER`-world; adopting it
  reintroduces a second build path + second cache layer (anti-cargo-culting / sprawl).
- **Cross-worktree warmth:** investigate buck2-native local dir-cache keys in `.buckconfig`,
  validated against facebook/buck2 upstream source before adoption — same key-verification
  practice already used for `[buck2]` daemon keys in `.buckconfig`. Track as a bead; **do not
  adopt unverified keys now**.

#### A-3 — Comment doctrine

- No paragraph comment blobs that narrate what the code or feature does.
- Code must be self-explanatory for maintenance.
- Comments only for non-obvious intent, trade-offs, or constraints the code cannot convey.
- Diff-only reviewers **reject narration comments**.

Encode in review checklists (PORTABLE-SWARM-CONTRACT + dual-critic).

#### A-4 — Generated-files doctrine (hyperscaler codegen)

Every generated artifact MUST carry:

1. `@generated` marker
2. generator id
3. source-of-truth (SSOT) pointer

Plus:

- generator inputs pinned in-repo
- one-command reproducible regeneration
- hermetic CI drift check (regen == committed)
- **no hand-edits** to generated files (extends existing `*.generated.json` ban)
- generator catalog row in the registry

Audit existing generated artifacts (search `@generated` / "do not edit" / auto-generated
JSON/YAML hubs) as a bead under the docs-governance epic; future generation must comply at
admission.


### Amendment B (2026-08-10) — reorg-target debt freeze

Binding amendment. Full portable mirror:
`.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md` § Amendment B.
Policy-as-data: `specs/integ-branch-envelopes.json` → `reorg_target_debt_freeze`.

#### B-1 — Frozen prefixes (no new path births)

Do **not** add new debt to ADR-0562 / ADR-0701 strangler / transitional homes.
Frozen prefixes (new `A` path births forbidden unless exempted below):

- `libs/`
- `cloud/`
- `oya/`
- `infra/`
- `toolchains/`
- `tools/`

**Allowed** under frozen prefixes: shrink, move-out, delete, and mechanical catalog fixes
for an in-flight move to a durable capability root.

**Forbidden:** birthing new crates, domains, features, or other new paths under those trees.
The `#1642` allow-new for `cloud-os` / `libs` was a **one-shot drain — never repeat**.

#### B-2 — `tools/swarm/**` one-shot exception

`tools/swarm/**` on `integ/specs` (`#1644`) is a **one-shot process-kit land** (Phase A
harness). After `#1644` merges, `tools/` is **shrink-only** — no further births under
`tools/` without an explicit `reorg-move-out` bead naming the durable home.

#### B-3 — Claim enforcement

`deliver.js` Claim MUST refuse any unit tip that adds new paths under frozen prefixes
unless the claimed bead title or body contains `reorg-move-out` (explicit move to a
durable capability root). Envelope membership alone does **not** authorize births under
frozen prefixes (including `integ/libs`, `integ/cloud`, `integ/oya`, `integ/tools`).

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
- **`docs/<root>` / `specs/<root>` mirror trees for product artifacts** — fights ADR-0119/0131/0701
  colocation doctrine; hubs stay central, product specs stay owner-colocated.
- **Full monorepo rebuild in every worker lane** — fights affected-set doctrine (ADR-0554) and
  no-slow-commands rule; check daemon + CI cone only.
- **Revive `cargo check` in the check daemon** — fights founder 2026-05-29 / no-cargo-enforcer;
  blessed path is `buck2 build //...[check]` (Amendment A-2).
- **sccache as a second cache layer** — second build path + second cache = sprawl; rejected
  (Amendment A-2).
- **Unverified buck2 dir-cache keys in `.buckconfig`** — adopt only after upstream source
  validation (mirror existing `[buck2]` key-verification practice); tracked as bead, not now.

- **Revive unrestricted births under frozen reorg targets** — fights ADR-0701 strangler; Amendment B freezes libs/cloud/oya/infra/toolchains/tools to shrink/move-out only.

## References

- Policy: `specs/integ-branch-envelopes.json`
- Portable rule text: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md`
- Harness: `.claude/workflows/deliver.js` (Claim + Land)
- Concurrent-safe registry: `registry/vcs/concurrent-safe-paths.yaml`
- Operating contract: `docs/AGENTS.md` (worktree-per-lane; `oya-ci-required`)
- Layout apex: ADR-0701 (capability-first; supersedes ADR-0562 / ADR-0131 as live law)
- CI admission apex: ADR-0700 (single `oya-ci-required`; ADR-0515 lineage)
- Specs topology provenance: ADR-0119 (flat cross-cutting `specs/` hub)
- Doc colocation / g3doc pattern: ADR-0541 (corpus liveness; owner-colocated docs)
- Affected testing: ADR-0554 / ADR-0700 lineage
