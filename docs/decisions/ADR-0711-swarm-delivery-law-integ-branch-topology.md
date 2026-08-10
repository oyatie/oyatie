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
    exit_criteria: "specs/integ-branch-envelopes.json exists; governed roots/planes/hubs are enumerated ONLY in that JSON (#roots, #planes, #hubs.paths) — prose cites JSON pointers and MUST NOT re-list; hub sole-owner list and adjunct claim rules are machine-readable; envelope self-ownership is integ/specs."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D2
    description: "Worktree-per-agent isolation plus worker git allowlist (no stash/reset) and server-side integ reset after land."
    exit_criteria: "PORTABLE-SWARM-CONTRACT.md carries Swarm Delivery Law; deliver.js Claim verifies envelope + merge-tree + hub exclusivity; Land upserts one PR per integ/<root> and documents server-side reset refspec; concurrent-safe exemptions match specs/integ-branch-envelopes.json#concurrent_safe_exemptions.paths (narrowed per-lane evidence — not whole evidence/**)."
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
  - id: ADR-0711-D6
    description: "Amendment B Pattern-First + full 16-lens battery — establish specs/naming-taxonomy.json before renames; taxonomy REPLACES indefensible brand/ADR naming (does not encode it); judgments require lenses_applied=all-16 + challenges[] when keeping/replacing existing patterns; dual-emit merge-gate-context until founder protection flip."
    exit_criteria: "specs/naming-taxonomy.json with overturned_patterns; ADR-0711 B-1b/B-1c + PORTABLE mirror; envelopes naming.judgment_template + judgment_files.dir (naming_sweep lives under governance/check/integ-envelope/judgments/, not inlined); no mass rename without taxonomy instance."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D7
    description: "Amendment C — 137-entry archive distillation synthesized as clustered operating-patterns catalog (KEEP/BAN), not 137 paraphrases; machine-readable specs/agentic-operating-patterns.json; distill notes that said keep name oya-ci-required are OVERRULED (forever name merge-admission-required)."
    exit_criteria: "ADR-0711 Amendment C + PORTABLE-SWARM-CONTRACT Amendment C present; specs/agentic-operating-patterns.json carries KEEP/BAN clusters + oyatie_apply tags; explicit OVERRULE of oya-ci-required-as-forever-name."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D8
    description: "Amendment D — Anti-drift documentation doctrine (INV-DOC-1…9); enumerate ONLY in envelopes JSON; docs_touched/docs_action packet; same-wave colocation; versioned anti_drift_doctrine_version; merge_windows policy-as-data."
    exit_criteria: "ADR-0711 Amendment D + PORTABLE Amendment D present; envelopes #anti_drift + #merge_windows; deliver.js Claim requires docs_touched/docs_action; tools/swarm/self-check.sh drift-greps prose root enumerations."
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

One durable branch `integ/<root>` exists per governed top-level root, and planes exist for
cross-cutting hubs / process-meta. **Enumerate ONLY in machine law:**

- Roots: `specs/integ-branch-envelopes.json#roots`
- Planes: `specs/integ-branch-envelopes.json#planes`
- Hubs: `specs/integ-branch-envelopes.json#hubs.paths`

Prose (this ADR, PORTABLE, AGENTS, plans) MUST cite those JSON pointers and MUST NOT re-list
root/plane/hub/freeze sets (Amendment D / INV-DOC-2). Dual-truth between prose lists and JSON
is a defect — fix same wave; JSON wins after challenge.

**Capability-root note:** every closed capability-registry top-level dir that appears under
`#roots` is a first-class governed integ envelope — ownership = path = integ scope (D-9).
Residual unit-PR content under those roots lands on the matching durable integ, not as
standalone trunk PRs. Forward-declared roots (e.g. `base/`, `app/`) remain routable even when
the directory is still vacant.

Branch list and path envelopes are policy-as-data in `specs/integ-branch-envelopes.json`.
Changes reach `dev` only via a PR from `integ/*` (exception: `hotfix/*`, post-hoc review). At most
one open PR per integ. Unit work (`impl/*`, lane branches) never opens trunk PRs.

### D-2 — Envelope containment + hub sole-owner + adjunct claims

A PR from `integ/R` may touch only:

1. paths inside envelope(R), and
2. explicitly claimed adjunct leaves, and
3. waivered hub files.

Hub files are sole-owner per wave (one integ carries a given hub edit). Path set SSOT:
`specs/integ-branch-envelopes.json#hubs.paths` (do not re-list here).

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
`merge`, `branch -D/-f`, `update-ref`, `reflog expire`, `gc`, bare `push --force`,
`commit --no-verify` / `-n`, `push --no-verify`.

Destructive operations exist only inside versioned, reviewed scripts that the integrator role runs
(restack, server-side reset, worktree remove). Integrator uses cherry-pick (commit-producing,
atomic) — still no stash/reset in its vocabulary.

Enforcement shims (`tools/swarm/git-shim`, `tools/swarm/toolguard`, `tools/swarm/check-daemon`)
are Phase A companions; this ADR is the law they enforce.

### D-7 — Special files and concurrent-safe exemptions

- Citation census pins are re-derived on the integ tip (oyatie-o90), never git-merged as authority.
- `Cargo.lock` lands with the integ that changed workspace membership.
- Concurrent-safe exemptions are recorded in `registry/vcs/concurrent-safe-paths.yaml` and MUST
  match `specs/integ-branch-envelopes.json#concurrent_safe_exemptions.paths` (narrowed per-lane /
  CI evidence prefixes — **not** the whole `evidence/**` tree).

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

Kept from Bun / Cursor swarm / Amendment C archive practice and already encoded in Claim/Land /
PORTABLE-SWARM-CONTRACT: one implementer + adversarial reviewers; planner ≠ implementer;
fix the process not the output; batch same-subsystem work into one lane; green CI is not
authorization — re-verify at the moment of action; never delete a git lock without checking the
owning process; automation stops at the edge of its authority; design cleanup/ownership for
replicated state before scaling parallelism.

Full clustered KEEP/BAN from the 137-entry Amendment C archive distillation: **Amendment C**.

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

Legacy strangler prefixes are enumerated **only** in
`specs/integ-branch-envelopes.json#reorg_debt_freeze` — they are **NOT** `keep_forever`
(`reorg_now`, or `delete_permanently` for empty residue). Do not re-list those paths here
(INV-DOC-2).


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

Judgments that decide renames MUST also record `name_now` → `name_forever` (topic-/role-shaped,
hyperscaler-plain). Rename is part of `reorg_now` **or** a `keep_forever` quality fix when the
path stays but the name is wrong. **Forbidden:** keep misleading names for history/brand.

#### B-1b — Pattern-First Law (naming taxonomy)

**Establish the pattern before mass renames.** Grammar, semantics, taxonomy, structure, and
architecture MUST follow an established, maintainable, mechanically classifiable pattern.
If none exists, author it first — then apply renames/reorgs as **instances**. No one-off
bespoke names.

Policy-as-data: [`specs/naming-taxonomy.json`](../../specs/naming-taxonomy.json).

Extends (does not invent a parallel cosmology):

- Capability-first roots + C/P/A/F faces (ADR-0701 / ADR-0562 lineage)
- Flat cross-cutting `specs/` / `docs/` hubs (ADR-0119) with owner colocation (ADR-0131 / ADR-0541)
- Role segments from crate-naming / BNF standards — **greenfield SUPERSEDES** brand-first
  `required_prefix = "oya-"` (ADR-0017 era) as durable law

**Grammar (binding highlights):**

| Rule | Meaning |
|---|---|
| Role-first tokens | Name states behavior (`baseline-ratchet`, `merge-admission-required`) |
| Forbidden leading brand | `oya-` / `oya_` / `cloud-` / `cloud_` as leading durable prefixes |
| Forbidden ADR path segments | `ADR-NNNN` must not be required to navigate; indexes/frontmatter may list numbers |
| Self-explanatory path test | Path+filename alone convey purpose |
| Brand ≠ keep_forever | Brand prefix alone never justifies durable home |

**Kinds** (see taxonomy for full set): `capability-root`, `plane-hub`, `meta-root`,
`check-crate`, `ci-facade`, `ci-job`, `merge-gate-context`, `policy-face`,
`service-colocation`, `decision-record`, `process-kit`, `fixture-corpus`, `shared-lib-debt`.

**Mechanical classification:** given path/kind → integ envelope + name shape
(`naming-taxonomy.json#classification_rules`). `naming_sweep[]` rows on envelopes MUST cite
`kind` + `grammar_compliant` + `name_forever`.

**Canonical instances (not one-offs):**

| name_now | kind | name_forever |
|---|---|---|
| `oya-ci-required` | merge-gate-context | `merge-admission-required` (dual-emit until founder protection flip) |
| `freshness (…, ADR-0539)` | ci-job | `generated-artifact-freshness (lock + faces)` |
| `cloud-ci-firewall (…) ` | ci-job | `admission-baseline-ratchet (+ gate-registration)` |
| `docs/decisions/ADR-NNNN-*.md` | decision-record | `docs/decisions/<topic>.md` (batched; not blind mass-rename) |

**Execution order:** taxonomy lands on `integ/specs` → sweep rows cite kinds → renames execute
as taxonomy instances on owning `integ/<root>`. Agents MUST NOT change GitHub branch protection;
dual-emit legacy+forever merge contexts in-repo until founder flips protection in one line.

#### B-1c — Full 16-lens battery (new AND existing patterns)

Authority pack: [`.grok/harness/lenses.v1.json`](../../.grok/harness/lenses.v1.json).
**Never a subset.** Every judgment that keeps, replaces, or deletes an existing pattern/decision/
architecture/design MUST run the full battery and record:

| Field | Requirement |
|---|---|
| `lenses_applied` | exactly `all-16` (ids below) |
| `challenges[]` | at least one challenge when the unit is an *existing* pattern being kept or replaced |

Lens ids (stable): `cartesian_doubt`, `essentialism_yagni`, `chestertons_fence`, `contrarian`,
`socratic`, `pragmatism`, `red_team`, `systems_thinking`, `operability_day2`, `opportunity_cost`,
`blast_radius_cell`, `constant_work`, `shared_nothing`, `finops`, `telemetry_first`, `zero_trust`.

**Challenge posture:** If an existing pattern/decision/architecture is an anti-pattern, do **not**
silently ignore or follow it. Challenge it, research north-star (authoritative sources + code
evidence), suggest the fix. Defensibility bar: if not defensible under the full battery →
**delete or reshape**. Chesterton's fence still applies: state why the fence existed, then
replace it if indefensible.

**Taxonomy must REPLACE indefensible naming practice — not encode it.** Prior ADR/CI naming that
fails the battery (`oya-ci-required` brand, ADR numbers in job titles, leading `oya-`/`cloud-`
prefixes, `firewall` metaphor) is recorded in `specs/naming-taxonomy.json#overturned_patterns`
with fence rationale + replacement. `naming_sweep[]` / judgment rows cite those replacements.

Patterns this amendment recommends overturning (mechanism may stay; brand/shape must not):

| Existing practice | Fence (why it existed) | Overturn to |
|---|---|---|
| `oya-ci-required` as forever context name | Single required context (ADR-0515/0700) + brand cohesion | Keep **single** merge context; rename to `merge-admission-required` (dual-emit until founder protection flip) |
| ADR numbers in CI job `name:` | Operator shortcut to governing ADR | Role-first job titles; ADR cites live in comments/docs only |
| Leading `oya-` / `cloud-` on crates/gates/bins | ADR-0017 workspace uniqueness / AWS-style prefix | Role-first grammar; brand prefix = debt |
| `cloud-ci-firewall` brand | Phase-0 go-live metaphor | `admission-baseline-ratchet` (+ gate-registration) |
| `required_prefix = "oya-"` as greenfield law | Historical crate BNF | Superseded for greenfield by `naming-taxonomy.json` |
| `docs/decisions/ADR-NNNN-*.md` path shape | Numbered decision catalogs | Topic-shaped filenames; numbers in index/frontmatter (batched renames) |

#### B-2 — Freeze prefixes = no NEW births while moves execute

Prefixes in `reorg_debt_freeze.prefixes` block **new path births** only while `reorg_now` /
`delete_permanently` executes. They are not a durable home. `#1642` allow-new for cloud-os/libs
was a one-shot drain — **never repeat**. `tools/swarm/**` on `#1644` is a one-shot process-kit
land; schedule `reorg_now` → `.grok/` immediately after merge.

#### B-3 — Classification table (compact)

Full machine-readable rows: `specs/integ-branch-envelopes.json#reorg_debt_freeze.rows`
(and freeze birth prefixes at `#reorg_debt_freeze.prefixes` /
`#reorg_debt_freeze.no_new_births_while_reorg_prefixes`).

**INV-DOC-2:** This ADR MUST NOT re-list freeze/layout path rows. Cite the JSON pointers
above only — dual-truth prose tables are a defect.

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

### Amendment C (2026-08-10) — operating-patterns catalog

Binding amendment. Distilled from **all 137** archive entries (14 batch distillers,
000–136) into the operating-patterns catalog. Encode **clusters**, not 137 paraphrases.
Portable mirror: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md` § Amendment C.
Policy-as-data: `specs/agentic-operating-patterns.json`.

These clusters reinforce (do not replace) D-1…D-9, Amendment A, and Amendment B.

#### C-1 — KEEP (clustered)

| Cluster | Rule |
|---|---|
| **Replicated-state budget** | Design cleanup/ownership of replicated state **before** scaling parallelism; size by replicated mass (worktrees, caches, indexes), not lane count. |
| **Observation ≠ authority** | Dense observation does not grant intervene/APPROVE/own rights; automation stops at the edge of its authority. |
| **Verified-empty / honest halt** | Verified-empty, honest undecided, and precise halt are **completed ops** — record them; do not invent work to fill silence. |
| **Evidence-before-done / receipt WAL** | Evidence chain > narrative: HEAD-keyed receipts; multi-signal done (CI + review + mergeState + live surface); **new HEAD → new evidence**; invalid receipts quarantine, never durable memory. |
| **Fail-closed blockers** | Missing inputs/identity → fail closed; blockers named by gate with next action; silence only when proven idle. |
| **Subsystem batch + role separation** | Batch same-subsystem work into one lane; one implementer + adversarial reviewers; planner ≠ implementer; two pedals — accelerate on owner-hook, brake on verified empty. |
| **Git-lock pid check** | On `index.lock` / shared-object contention: check owning pid, retry idempotent — **never** blind-delete the lock. |
| **Green ≠ go** | Green is admission to the *next* gate, not departure; same-author APPROVE banned; conversation/authority lines are brakes; re-verify at the moment of action. |
| **Live smoke** | Public/live smoke after deploy; build ≠ publish; wrapper noise ≠ artifact fail; post-merge observation until live surface is actually green. |
| **Sharp labels / pattern-first** | Sharp labels as action switches; defaults verified by next dogfood use; forever names follow `specs/naming-taxonomy.json`. |

#### C-2 — BAN (clustered)

| Cluster | Ban |
|---|---|
| **Invented work** | Inventing work into verified empty; activity theater; filler to fill quotas; observing = intervening. |
| **Dual-truth** | Dual-home / dual-truth; carrying yesterday’s green as today’s proof; sticky prior `merge-admission` green across new HEAD. |
| **Second build path / shared WD** | Cargo/second-build-path revival; slow commands in lanes; shared WD stash/reset chaos; PID kill without identity. |
| **Brand / opaque / ADR-in-title names** | Leading `oya-*` / `cloud-*` / opaque / ADR-in-job-title durable names (Amendment B Pattern-First). |
| **Silent blocked success** | Silent success on blocked inputs; rubber-stamp thread resolve; merge ego after green; scoreboards from observation density. |

#### C-3 — OVERRULE: forever merge-gate name

Any distill note that said **keep the name `oya-ci-required`** is **OVERRULED**.

- **Forever name:** `merge-admission-required` (`specs/naming-taxonomy.json` overturned pattern
  `OP-merge-gate-brand`).
- Live GitHub protection may still pin the legacy string until dual-emit + founder flip
  (Amendment B / Phase C) — that is a cutover alias, **not** forever grammar.
- Brand-prefix bans and ADR-in-title bans stand. Distill corpus is evidence for operating
  patterns, not authority to freeze indefensible names.

### Amendment D (2026-08-10) — Anti-drift documentation doctrine

Binding amendment. Machine law + packet + same-wave colocation so docs cannot drift after change.
Portable mirror: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md` § Amendment D.
Policy-as-data: `specs/integ-branch-envelopes.json#anti_drift` (`anti_drift_doctrine_version`).

#### INV-DOC-1…9 (RFC 2119)

1. **INV-DOC-1 (packet):** Every material change MUST declare `docs_touched[]` + `docs_action`
   (`update|add|delete|n/a`) in Claim/Land/Fix-observation/commit trail; `n/a` REQUIRES
   `docs_action_why`. Missing packet ⇒ incomplete Claim.
2. **INV-DOC-2 (single enumeration SSOT):** Prose MUST NOT re-list roots/planes/freeze prefixes/hub
   path sets; MUST cite JSON pointers (`#roots`, `#planes`, `#hubs.paths`,
   `#reorg_debt_freeze.prefixes`, …). Dual-truth = defect; fix same wave; JSON wins after challenge.
3. **INV-DOC-3 (same-wave colocation):** Load-bearing doc updates MUST land on the owning integ
   same wave as the change. “Docs follow-up later” BANNED when docs are required for correct
   application.
4. **INV-DOC-4 (derived regen):** Generated artifacts MUST regenerate in the same change; hand-edit
   of generated output MUST NOT (Amendment A-4).
5. **INV-DOC-5 (cross-plane):** Specs/registry first when schema binds gates → docs hubs/indexes
   next same window when load-bearing → adjunct/waiver for cross-plane; MUST NOT fight sole-owner
   hubs; parked docs MUST cite specs tip SHA depended on.
6. **INV-DOC-6 (stale):** Unverified tips/SHAs MUST refresh or mark unverified. Declared ≠ verified.
7. **INV-DOC-7 (Limitations):** Doctrine docs MUST keep a Limitations section.
8. **INV-DOC-8 (evolve doctrine):** Amend only via challenge → OVERRULE receipt → edit
   ADR/PORTABLE/envelopes → bump `anti_drift_doctrine_version`. MUST NOT silently diverge plan
   from in-repo law.
9. **INV-DOC-9 (survival surfaces):** Canonical doctrine that must survive MUST live in the
   surfaces agents actually load every session — repo-root operating contracts (`AGENTS.md`,
   `CLAUDE.md`; `README.md` pointer), owning canonical docs (ADR / envelopes / PORTABLE), and
   the programme SSOT. Root files carry the **short binding form + why + JSON pointers**
   (INV-DOC-2 still bans duplicated enumerations). Doctrine that exists only in a plan file is
   **not survived**; doctrine only in chat is dead. Machine list:
   `specs/integ-branch-envelopes.json#anti_drift.invariants`.

#### Merge windows (policy-as-data)

Hot-set ≤4 and restack-once/window are encoded in
`specs/integ-branch-envelopes.json#merge_windows` — not plan-only dual-truth.

#### Limitations

Mechanical Claim packet parse + Claim↔diff bind (`docs_touched`/`paths` ↔
`git diff --name-only`) are live in `deliver.js` + `claim_packet.py`;
`claim-push.sh` refuses dirty porcelain. Does not rewrite DOC-CATALOG corpus; does not
authorize mass ADR renames. Drift-grep: `tools/swarm/self-check.sh`. Root-file content land of
INV-DOC-9 short form is owned by `integ/ci` (`planes.process_meta`) — route ≠ content.

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
- **Mass-rename without a naming taxonomy** — banned; Pattern-First Law (B-1b) requires `specs/naming-taxonomy.json` kinds + grammar before renames; one-off bespoke names are debt.
- **Keep `oya-` / `cloud-` leading brand prefixes because ADR-0017 / history said so** — banned for greenfield; brand prefix is not keep_forever; role-first forever names + dual-emit/alias for protection cutovers.
- **Keep forever name `oya-ci-required` because Amendment C distill notes said so** — OVERRULED (Amendment C-3); forever name is `merge-admission-required`; distill is operating-pattern evidence, not naming authority.

## References

- Naming taxonomy: `specs/naming-taxonomy.json`
- operating-patterns catalog: `specs/agentic-operating-patterns.json`
- Policy: `specs/integ-branch-envelopes.json`
- Portable rule text: `.grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md`
- Harness: `.claude/workflows/deliver.js` (Claim + Land)
- Concurrent-safe registry: `registry/vcs/concurrent-safe-paths.yaml`
- Operating contract: `docs/AGENTS.md` (worktree-per-lane; sole required merge-admission context)
- Layout apex: ADR-0701 (capability-first; supersedes ADR-0562 / ADR-0131 as live law)
- CI admission apex: ADR-0700 (single required merge-admission context; ADR-0515 lineage; forever name `merge-admission-required`)
- Specs topology provenance: ADR-0119 (flat cross-cutting `specs/` hub)
- Doc colocation / g3doc pattern: ADR-0541 (corpus liveness; owner-colocated docs)
- Affected testing: ADR-0554 / ADR-0700 lineage
