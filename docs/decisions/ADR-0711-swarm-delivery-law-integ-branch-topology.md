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
    description: "Amendment B — REORG NOW ternary layout map: every root/meaningful subdir is reorg_now|keep_forever|delete_permanently; freeze prefixes block NEW births only while moves execute; libs/cloud/oya/infra/toolchains/tools are NOT keep_forever."
    exit_criteria: "ADR-0711 Amendment B + PORTABLE-SWARM-CONTRACT Amendment B present; envelopes reorg_debt_freeze rows carry action/destination/shape/rationale/redesign/judgment_status; evaluation_gate forbids git-mv-only; deliver.js Claim rejects births under freeze prefixes and rejects path changes without judgment_status=done."
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


### Amendment B (2026-08-10) — REORG NOW (ternary layout map)

Binding amendment. **Reorg happens NOW** — classification is a move map, not a parking lot.
Portable mirror: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md` § Amendment B.
Policy-as-data: `specs/integ-branch-envelopes.json` → `reorg_debt_freeze`.

#### B-0 — Greenfield question (placement law)

For every root and meaningful subdir ask:

> If we were greenfield and wanted an ideal hyperscaler monorepo clean-architecture shape,
> where would this belong and in what shape?

That answer **IS** placement law. Apply recursively (e.g. `docs/programs` vs `docs/decisions`).

#### B-1 — Ternary actions only

| Action | Meaning |
|---|---|
| `reorg_now` | Move to greenfield destination in this programme; delete vacated path; update registries in same land |
| `keep_forever` | Already the ideal hyperscaler clean-arch home |
| `delete_permanently` | No durable value; remove |

**No fourth state.** Banned language: strangler_freeze-as-destination, gradual, eventually,
6 months, deprecate-in-place, dual-home forever.

`libs/`, `cloud/`, `oya/`, `infra/`, `toolchains/`, `tools/` are **NOT** `keep_forever` —
they are `reorg_now` (or `delete_permanently` for empty residue).


#### B-1a — Evaluation gate (mandatory before any path change)

**Reorg is NOT a simple move.** `git mv` / rename-only waves are **forbidden**.

Before every ternary action that changes paths, evaluate the unit (root or meaningful subtree)
by **reading the code/docs** (do not trust folder names) and record short answers on the
classification row (`rationale`, `redesign`, `judgment_status`):

1. **What does it actually do?** Behavior, callers, contracts.
2. **How is it written?** Clean-arch, dual-home, generated, layering, narration comments.
3. **What is it trying to achieve?** North-star aligned?
4. **Should it exist at all** in north-star system design?
5. **System-design fit** — antipatterns / unwanted behavior?
6. **Improve how?** `refactor` vs `rewrite` vs `delete` — mechanical move is insufficient when
   shape/API/ownership is wrong. Disposition:
   `keep-as-is` | `refactor-then-place` | `rewrite-into-durable-shape` | `delete-permanently`.
7. **Where/why belong?** Greenfield ideal hyperscaler clean-arch shape.

Only when `judgment_status=done` may a unit land as `reorg_now` (redesign+land at destination)
or `delete_permanently`. Prefer rewrite/refactor into clean-architecture capability shape over
preserving accidental structure. **No dual-home. No move-now-fix-later.**

Row fields (policy-as-data): `action`, `destination`, `shape`, `rationale`, `redesign`
(`none|refactor|rewrite|delete`), `judgment_status` (`pending|done`).


#### B-2 — Freeze prefixes = no NEW births while moves execute

Prefixes in `reorg_debt_freeze.prefixes` block **new path births** only while `reorg_now` /
`delete_permanently` executes. They are not a durable home. `#1642` allow-new for cloud-os/libs
was a one-shot drain — **never repeat**. `tools/swarm/**` on `#1644` is a one-shot process-kit
land; schedule `reorg_now` → `.grok/` immediately after merge.

#### B-3 — Classification table (compact)

Full machine-readable rows: `specs/integ-branch-envelopes.json#reorg_debt_freeze.rows`.

| current path | action | greenfield destination | shape |
|---|---|---|---|
| `docs/` | keep_forever | docs/ | PLANE hub: operating contract, indexes, live ADRs, standards, archive only |
| `docs/ADR-INDEX.md` | keep_forever | docs/ADR-INDEX.md | hub index |
| `docs/AGENTS.md` | keep_forever | docs/AGENTS.md | operating-contract apex |
| `docs/CHANGELOG.md` | keep_forever | docs/CHANGELOG.md | hub changelog |
| `docs/DOC-CATALOG.md` | keep_forever | docs/DOC-CATALOG.md | hub catalog (until machine index replaces) |
| `docs/adr-archive/` | keep_forever | docs/adr-archive/ | historical ADR archive (read-only growth via supersession) |
| `docs/architecture/` | reorg_now | docs/decisions/ + capability ARCH.md | keep only cross-cutting; rest colocate |
| `docs/audit/` | reorg_now | audit/ | capability-owned audit docs (~233 files) |
| `docs/ci/` | reorg_now | ci/ | capability-owned CI docs |
| `docs/decisions/` | keep_forever | docs/decisions/ | live ADR apex files only |
| `docs/foundry/` | reorg_now | intelligence/ or retire | foundry brand residue |
| `docs/harness/` | delete_permanently | (none) | retired harness docs (ADR-0709 lineage) |
| `docs/ideas/` | delete_permanently | (none) | harvest any keeper into bead then delete |
| `docs/implementation-plans/` | reorg_now | <capability>/IPs/ + beads | IPs colocate; tracking in beads |
| `docs/localization-packs/` | reorg_now | compliance/packs/ or packs→compliance | jurisdiction packs with compliance capability |
| `docs/plans/` | reorg_now | specs/masterplan.json + beads | plan authority is masterplan v2; retire prose dumps |
| `docs/prds/` | reorg_now | <capability>/PRD.md or app/<product>/ | product PRDs colocate |
| `docs/products/` | reorg_now | <capability>/ or app/<product>/ | owner-colocated product docs (g3doc) |
| `docs/programs/` | reorg_now | governance/corpus/programs/ or owning capability | program dossiers leave central docs |
| `docs/raw/` | delete_permanently | (none) | throwaway drafts; never authoritative |
| `docs/regional-packs/` | reorg_now | compliance/packs/ | same as localization |
| `docs/runbooks/` | reorg_now | <capability>/runbooks/ | owner-colocated runbooks; keep RUNBOOKS-INDEX hub |
| `docs/standards/` | keep_forever | docs/standards/ | cross-cutting engineering standards hub |
| `docs/templates/` | reorg_now | templates/ | merge unique templates into root templates/; delete dual-home |
| `specs/` | keep_forever | specs/ | PLANE hub: cross-cutting machine-readable authority only |
| `specs/audit-event-class-registry.json` | reorg_now | audit/audit-event-class-registry.json | dual-home with audit/; single SSOT at audit/ |
| `specs/capabilities/` | reorg_now | <capability>/ or keep only cross-cutting schemas in specs/ | split: cross-cutting schemas KEEP; per-capability MOVE |
| `specs/capability-registry.json` | keep_forever | specs/capability-registry.json | closed registry (eventual governance/ copy is same hub move later if desired; ke |
| `specs/catalog/` | reorg_now | registry/catalog/ | crate/catalog authority is registry/ |
| `specs/design-system/` | reorg_now | console/ or app/shell/ | UI design system with console/app |
| `specs/fixtures/` | keep_forever | specs/fixtures/ | cross-cutting gate fixtures only |
| `specs/integ-branch-envelopes.json` | keep_forever | specs/integ-branch-envelopes.json | Swarm Delivery Law policy-as-data |
| `specs/ip/` | reorg_now | <capability>/IPs/ | implementation plans colocate |
| `specs/k8s-port/` | reorg_now | k8s/ or build/port-engine/ | port programme artifacts with owners |
| `specs/lifecycle-configs/` | delete_permanently | (none) | absorbed residuals; do not rebirth |
| `specs/markdown-retirement-policy.json` | keep_forever | specs/markdown-retirement-policy.json | docs lifecycle policy |
| `specs/masterplan.json` | keep_forever | specs/masterplan.json | live plan authority |
| `specs/microservices/` | reorg_now | <capability>/manifest + contracts | type-dump of product specs; colocate or delete superseded JSON |
| `specs/openslo/` | reorg_now | <capability>/slos/*.openslo.yaml | SLO authoring colocated per service doctrine |
| `specs/policy/` | reorg_now | policy/ (new capability root) + iam leftovers | policy capability home |
| `specs/products/` | reorg_now | app/<product>/ or capability facade | product composition specs |
| `specs/proto/` | reorg_now | contracts/proto/ or capability contracts/ | API contracts hub |
| `specs/regions/` | reorg_now | compliance/packs/ | regional regime data with compliance |
| `specs/regulatory-regimes/` | reorg_now | compliance/ | compliance-owned |
| `specs/reorg/` | keep_forever | specs/reorg/ | executable move-plan hub (singleton live plan) |
| `specs/root-hub-pointers.json` | keep_forever | specs/root-hub-pointers.json | entry hub |
| `plan/` | reorg_now | governance/corpus/plan/ + beads | no top-level plan dump in greenfield |
| `plan/fabric-loop/` | reorg_now | governance/corpus/fabric-loop/ | governance graph substrate |
| `templates/` | keep_forever | templates/ | PLANE: single template hub (authority chain) |
| `templates/checklists/` | keep_forever | templates/checklists/ | cross-cutting checklists |
| `governance/` | keep_forever | governance/ | META: checks + corpus off runtime ladder |
| `governance/check/` | keep_forever | governance/check/ | hermetic gate engines + policy-as-data |
| `governance/corpus/` | keep_forever | governance/corpus/ | living monorepo governance graph |
| `flags/` | keep_forever | flags/{core,ports,adapters,facade}/ | registered capability; finish clean-arch faces |
| `flags/policy/` | keep_forever | flags/policy/ or policy/ if PDP-shared | keep unless pure PDP extract |
| `audit/` | keep_forever | audit/{core,ports,adapters,facade}/ | registered capability; already faced |
| `audit/audit-event-class-registry.json` | keep_forever | audit/audit-event-class-registry.json | SSOT after specs dual deleted |
| `libs/` | reorg_now | base/ (≥3 caps) + owning capability core/ | ADR-0701 base/ + faces; rule-of-two ends |
| `cloud/` | reorg_now | os/, kernel/, <capability>/, build/ | strangler source → durable meta/capability homes |
| `oya/` | reorg_now | <capability>/ + app/<product>/ | product dump → capability/app homes |
| `infra/` | reorg_now | build/, ci/, .grok/, iac/ | split by concern; no infra junk-drawer |
| `toolchains/` | reorg_now | build/toolchains/ | build meta owns toolchains |
| `tools/` | reorg_now | .grok/ + ci/facade/ + delete remainder | process kit leaves tools/; tools/swarm one-shot then vacate |
| `packs/` | reorg_now | compliance/packs/ | jurisdiction localization+sovereignty under compliance |
| `scripts/` | reorg_now | ci/ + build/ + delete obsolete py checks | no top-level scripts in greenfield |
| `tasks/` | reorg_now | beads (implementable) + <capability>/IPs/ | harvest then vacate top-level tasks/ |
| `.agents/` | delete_permanently | (none) | untracked; use runtime agents not repo vendor |
| `.beads/` | keep_forever | .beads/ | PROCESS_KIT: issue DB (usually local) |
| `.cargo/` | keep_forever | .cargo/ | PROCESS_KIT: cargo config |
| `.claude/` | keep_forever | .claude/ | PROCESS_KIT: deliver.js + settings |
| `.codex/` | keep_forever | .codex/ | PROCESS_KIT: project skills overlay |
| `.config/` | keep_forever | .config/ | PROCESS_KIT: tool config |
| `.cursor/` | keep_forever | .cursor/ | PROCESS_KIT: editor agents/rules (thin) |
| `.github/` | keep_forever | .github/ | PROCESS_KIT: interim GHA runner surface (ADR-0700) until owned-runner cutover |
| `.gjc/` | delete_permanently | (none) | retired; untracked local only |
| `.grok/` | keep_forever | .grok/ | PROCESS_KIT: mm-delivery / swarm process (thin; not product) |
| `.omc/` | delete_permanently | (none) | retired OMC harness residue (ADR-0709); remove tracked files |
| `.omx/` | delete_permanently | (none) | retired; untracked local only |
| `benchmarks/` | reorg_now | governance/check/benchmark/ + capability perf evidence | no orphan benchmarks root |
| `billing/` | keep_forever | billing/{core,ports,adapters,facade}/ | registered capability |
| `buck-out/` | delete_permanently | (none) | build output; gitignored |
| `build/` | keep_forever | build/ | META: buck prelude, toolchains, generators |
| `cell/` | keep_forever | cell/{core,ports,adapters,facade}/ | registered capability |
| `ci/` | keep_forever | ci/{core,ports,adapters,facade}/ | registered capability |
| `comms/` | keep_forever | comms/{core,ports,adapters,facade}/ | registered capability |
| `compliance/` | keep_forever | compliance/{core,ports,adapters,facade}/ | registered capability; absorb packs/ |
| `compute/` | keep_forever | compute/{core,ports,adapters,facade}/ | registered capability |
| `console/` | keep_forever | console/{core,ports,adapters,facade}/ | registered capability |
| `contracts/` | keep_forever | contracts/ | PLANE: cross-µservice OpenAPI/Protobuf/AsyncAPI SSOT |
| `data/` | keep_forever | data/{core,ports,adapters,facade}/ | registered capability |
| `evidence/` | keep_forever | evidence/ | PLANE: append-only evidence (concurrent-safe) |
| `gateway/` | keep_forever | gateway/{core,ports,adapters,facade}/ | registered capability |
| `iac/` | keep_forever | iac/{core,ports,adapters,facade}/ | registered capability |
| `iam/` | keep_forever | iam/{core,ports,adapters,facade}/ | registered capability |
| `intelligence/` | keep_forever | intelligence/{core,ports,adapters,facade}/ | registered capability; absorb oya/intelligence |
| `k8s/` | keep_forever | k8s/{core,ports,adapters,facade}/ | registered capability |
| `kernel/` | keep_forever | kernel/{core,harness}/ | META rung-0 |
| `marketplace/` | keep_forever | marketplace/{core,ports,adapters,facade}/ | registered capability |
| `messaging/` | keep_forever | messaging/{core,ports,adapters,facade}/ | registered capability |
| `network/` | keep_forever | network/{core,ports,adapters,facade}/ | registered capability |
| `observability/` | keep_forever | observability/{core,ports,adapters,facade}/ | registered capability |
| `os/` | keep_forever | os/{core,harness}/ | META rung-1 |
| `registry/` | keep_forever | registry/ | PLANE: catalogs, concurrent-safe, fixuptasks |
| `secrets/` | keep_forever | secrets/{core,ports,adapters,facade}/ | registered capability |
| `storage/` | keep_forever | storage/{core,ports,adapters,facade}/ | registered capability |
| `target/` | delete_permanently | (none) | cargo output; gitignored |
| `tenancy/` | keep_forever | tenancy/{core,ports,adapters,facade}/ | registered capability |
| `third-party/` | keep_forever | third-party/ | META: reindeer vendored cell; never hand-edit product logic |
| `workflow/` | keep_forever | workflow/{core,ports,adapters,facade}/ | registered capability |
| `wt-1616/` | delete_permanently | (none) | accidental nested worktree in main checkout; remove |

#### B-4 — Claim enforcement + destination integ preference

`deliver.js` Claim MUST:

1. Refuse **new path births** under freeze/vacated prefixes unless the bead marks
   `reorg-move-out` naming `destination`, **and**
2. Refuse **reorg path changes** (deletes/renames under classified units) unless the unit row
   has `judgment_status=done` with `rationale` + `redesign` filled; PR body must paste the
   7-point judgment.

Prefer landing redesigns on the **destination** `integ/<root>`. Envelope membership of a freeze
source does **not** authorize births or blind moves there.

First wave = evaluated decisions with evidence (`judgments_done` / `first_wave`), not path shuffles.
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

- **Treat libs/cloud/oya/infra/toolchains/tools as keep_forever or gradual freeze** — banned; Amendment B ternary requires reorg_now/delete_permanently NOW; freeze prefixes only block new births during the move.

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
