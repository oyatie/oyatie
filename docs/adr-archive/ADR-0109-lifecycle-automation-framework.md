---
id: ADR-0109
title: Lifecycle-automation framework (generic kernel + per-lifecycle configs)
status: Superseded
superseded_by: [ADR-0709]
doc_status: published
owner: council-architecture
date: 2026-05-15
relates_to:
  - ADR-0037-public-api-stability-tiers-and-deprecation.md
  - ADR-0054-grit-scaffold-claim-pattern.md
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
  - ADR-0105-13-layer-enum-and-check-family-patterns.md
  - ADR-0107-tools-implicit-app-convention.md
amends: []
supersedes: []
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0109: Lifecycle-automation framework (generic kernel + per-lifecycle configs)

## Status
Accepted — 2026-05-15.

## Context

User directive 2026-05-15:

> "sunset > deprecation > removal. dispatch" + "for all lifecycle" + "where it makes sense" + "it should be automated"

Follow-up clarification:

> "automation cost is non-existent?"

The repo houses many artifacts with distinct lifecycles — ADR status, API stability tier, crate status, capability status, migration cutover, plan status, doc freshness, dependency status, feature flag status. Each has a clear state machine (e.g. `Proposed → Accepted → Superseded`, `Scaffolded → Live → Quiescent → Archived`, `Experimental → Stable → Deprecated → Removed`). Today, transitions are tracked manually in front-matter or implicit in `supersedes:`/`superseded_by:` cross-references, and drift (stale state, missed deadlines, ungrounded supersession claims) is detected — if at all — by human review.

Per memory feedback `lifecycle-automation-universal` (the canonical directive file), the bar for "automate this lifecycle?" collapses to one test: **does the artifact have a clear state machine with transitions that could be missed or late?** Population thresholds do not apply. Automation cost is effectively zero in this agentic codebase (scaffolding ≈ 0, maintenance ≈ 0 because config-driven, CI runtime ≈ 0, false positives ≈ 0 with deterministic kernels + wave ratchets).

Per memory feedback `no-exceptions-canonical`, the framework must declare ONE canonical shape that all lifecycle lanes parameterize, not N bespoke kernels each redefining transitions.

The parallel `oya-governance-sunset-lifecycle-kernel` (committed independently by a concurrent agent) is a one-off dedicated crate; this ADR establishes the FRAMEWORK that future lifecycles (ADR status, crate status, plan status, capability status, migration, dependency, feature-flag, API-stability, doc) consume.

## Decision

1. **One generic kernel.** `oya-governance-lifecycle-kernel` exposes the canonical `LifecycleConfig`, `LifecycledArtifact`, `Stage`, `Transition`, `Violation`, and `evaluate()` function. Every lifecycle lane is data — a JSON config under `specs/lifecycle-configs/`. Adding a new lifecycle is a config-file + thin dev-CLI commit, not a new kernel.

2. **Per-lifecycle dev-CLI wrappers.** Each lifecycle ships `tools/oya-governance-<lifecycle-name>-lifecycle-app/` — a thin binary that loads its config, discovers artifacts from the repo (via the kernel's source-spec abstraction), calls `evaluate()`, and reports/exits. The CLI is the IO ring (clean-architecture port-in-kernel: kernel is I/O-free; the app does the directory walk + front-matter parsing).

3. **Canonical lifecycle metadata schema.** Every lifecycled artifact MAY carry `_lifecycle.kind`, `_lifecycle.stage`, `_lifecycle.transitions[]`, `_lifecycle.deadline_at?`, `_lifecycle.milestone_anchor?`. Existing fields (`status:`, `supersedes:`, `superseded_by:`) remain canonical inputs; the framework reads them via per-config source specs and does NOT require a flag-day migration.

4. **State-machine algebra (kernel contract).** Given a `LifecycleConfig { name, stages: [Stage], transitions: [from → to (predicate)], sources: [SourceSpec], defaults }` and a slice of `LifecycledArtifact { location, kind, current_stage, observed_at, deadline_at, history }`, plus `now: NaiveDate` and `reached_milestones: &[String]`, `evaluate()` returns `Vec<Violation>` for:
   - `StageNotDeclared` — artifact in scope but no stage detected.
   - `UnknownStage` — declared stage absent from config's stage enum.
   - `OverdueTransition` — `deadline_at < now` and `current_stage` not terminal.
   - `MissingSupersession` — terminal stage reached but no supersedes/superseded_by edge.
   - `MilestoneOverdue` — milestone reached, transition not advanced.
   - `IllegalTransition` — `history` contains a `from → to` not in `transitions`.

5. **Wave ratchet (canonical per fitness-lane pattern).** Wave A = WARN baseline; Wave B = BLOCK on NEW violations; Wave C = BLOCK on all. Each per-lifecycle plan declares its wave.

6. **Concurrent sunset-lifecycle integration.** The dedicated `oya-governance-sunset-lifecycle-kernel` is canonical for the sunset-clause domain. The framework applies to OTHER lifecycles immediately; sunset-lifecycle remains on a dedicated kernel per the canonical pattern declared in §"Sunset-lifecycle: dedicated kernel as canonical pattern" below. Both shapes are canonical at the kernel layer; the framework's value is per-lifecycle DRY for the 9+ pure-stage-transition state machines.

## Sunset-lifecycle: dedicated kernel as canonical pattern (not exception)

Amendment 2026-05-15. Per `feedback_no_exceptions_canonical.md`
("predictable-naming kernel — 13-value enum + adopted patterns") and
`feedback_lifecycle_automation_universal.md`, this section codifies the
canonical pattern for lifecycle types whose semantics exceed pure
stage-transition.

### Two canonical lifecycle-kernel patterns

Lifecycle kernels in this codebase ship in one of two canonical shapes,
selected by the lifecycle's domain semantics:

**Pattern A — generic-kernel + config (this ADR's framework).** Applies
when the lifecycle is a *pure stage-transition state machine*: each
artifact has a current stage, transitions between stages are declarative,
and the only time-sensitive check is a flat `deadline_at` (single
calendar anchor per artifact, no defaulting, no multi-anchor arithmetic).
The 9 lifecycles enumerated in §"Initial lifecycle catalog (Wave-A
scaffold)" all fit Pattern A — adding a new Pattern-A lifecycle is a
3-file commit (config JSON, thin dev-CLI, plan file).

**Pattern B — dedicated kernel.** Applies when the lifecycle requires
*date-arithmetic with milestone-equivalence semantics*: multi-anchor
defaulting (e.g. 30/90-day lag chains), canonical sentinel-milestone
recognition with calendar-vs-milestone precedence rules, or
domain-specific finding categories that cannot be expressed as
StageNotDeclared / UnknownStage / OverdueTransition /
MissingSupersession / MilestoneOverdue / IllegalTransition. The
sunset-lifecycle (ADR-0108) is the load-bearing Pattern-B instance:
sunset → deprecation → removal is a three-anchor lifecycle with
defaulted lags, three distinct findings (SunsetReached / RemovalReached
/ MissingFields), and a `doctrine-not-time-bounded` canonical sentinel
with calendar-wins precedence.

### Decision rule (machine-readable)

When designing a new lifecycle automation lane, apply this gate:

```
if lifecycle has:
  - multi-anchor date defaulting (e.g. dep_at defaults to sunset_at + N days), OR
  - canonical sentinel-milestone recognition with precedence rules, OR
  - domain-specific finding categories outside the framework's six
    ViolationKinds, OR
  - multi-surface discovery markers with surface-specific schemas (e.g.
    YAML frontmatter + JSON `_sunset` object + Cargo
    `[package.metadata.oya.sunset]` triple)
then Pattern B (dedicated kernel)
else Pattern A (generic framework + config)
```

Both patterns are canonical. Authoring a Pattern-B kernel does NOT
violate the no-exceptions doctrine: it is a *canonical extension* for
lifecycles whose semantics exceed Pattern A's expressive range, parallel
to how ADR-0083 declares three canonical Tier patterns (Tier 1 library,
Tier 2 binary, Tier 3 test) rather than one rule with two exceptions.

### Why not extend the generic kernel to absorb sunset semantics

Absorbing date-arithmetic defaulting, sentinel recognition, three-surface
discovery, and the SunsetReached/RemovalReached/MissingFields finding
categories into `oya-governance-lifecycle-kernel` would:

1. Add ≈560 LOC of domain-specific schema + algorithm + parser code to
   the generic kernel, diluting its canonical posture for the 9
   Pattern-A lifecycles.
2. Force the 9 Pattern-A configs to declare empty `defaults: {}`,
   `sentinels: []`, `precedence: ~` fields they don't use, smearing
   sunset-specific concerns across every lifecycle config.
3. Couple the generic kernel to a `Date` type with proleptic-Gregorian
   arithmetic (`add_days`, `days_since`, day-number conversions) — a
   dependency that the 9 Pattern-A lifecycles don't need.
4. Re-introduce date-arithmetic discovery surfaces (Cargo manifest
   `[package.metadata.oya.sunset]` sections, top-level JSON `_sunset`
   objects, body-level sunset prose) into the framework's source-spec
   abstraction, which today supports only YAML frontmatter + Cargo
   metadata table scalars.

The dedicated-kernel pattern keeps each canonical shape clean: the
generic kernel stays a pure state-machine matcher; the sunset kernel
stays a date-arithmetic-aware classifier; both honor ADR-0083 Tier 1 /
ADR-0056 port-in-kernel / ADR-0105 13-layer-enum naming uniformly.

### Naming consequence

Pattern-B kernels follow the same v4 BNF as Pattern-A dev-CLIs:
`oya-governance-<topic>-lifecycle-kernel` (kernel layer) plus
`oya-governance-<topic>-lifecycle-app` (composition-root binary).
The naming layer-enum is unchanged — both patterns terminate in
`-kernel` / `-app` per ADR-0105 Amendment 1 / ADR-0107 Amendment
2026-05-15. The pattern selection is invisible in the crate name; it
shows up only in the kernel's internal schema (multi-anchor `Date`
helpers vs. flat stage-machine evaluation).

### Pattern-B registry

The repo currently has one Pattern-B lifecycle:

| Lifecycle | Kernel | Dev-CLI | Driving ADR | Domain semantics |
|---|---|---|---|---|
| sunset-lifecycle | `crates/oya-governance-sunset-lifecycle-kernel` | `tools/oya-governance-sunset-lifecycle-app` | ADR-0108 | 30/90-day lag defaulting; doctrine-not-time-bounded sentinel; three-surface discovery |

Future Pattern-B candidates (declare here when introduced): credential
rotation (multi-anchor expiry + reissue lag), incident postmortem
lifecycle (root-cause → mitigation-deadline → review-deadline → publish
deadline arithmetic), SLA-window breach lifecycle (continuous-time
windowing rather than discrete stages).

### Migration policy

The successor-IP listed under §Decision item 6 ("a successor-IP refactor MAY
convert sunset-lifecycle into a config-driven instance") is hereby
withdrawn. Sunset-lifecycle remains on its dedicated Pattern-B kernel
indefinitely. Removing this entry would be a silent regression of the
canonical pattern.

## Rationale — why automate ALL lifecycles, not just the populous ones

The original gate ("≥5 instances + drift cost > automation cost") was withdrawn per the 2026-05-15 clarification. In an agentic codebase:

- Scaffolding cost ≈ 0 (agents author the kernel/config/plan in minutes).
- Maintenance cost ≈ 0 (config-driven; schema evolves under the kernel, not 9 kernels).
- CI runtime ≈ 0 (each fitness lane is seconds; embarrassingly parallel).
- False positives ≈ 0 with deterministic kernels + wave ratchets (A=WARN until baseline-zero, then B=BLOCK).

A small-population lifecycle is actually a GOOD place to start — fewer findings to debug against fresh-kernel logic, and the kernel is in place for when population grows. The anti-pattern is gating automation on "≥N instances or manual is fine"; the canonical posture is automate-by-default when a state machine exists.

## Schema (canonical)

### `LifecycleConfig` (JSON)

```json
{
  "name": "adr-status",
  "version": 1,
  "stages": [
    { "id": "proposed", "terminal": false },
    { "id": "accepted", "terminal": false },
    { "id": "superseded", "terminal": true, "requires_supersession_edge": true },
    { "id": "archived", "terminal": true }
  ],
  "transitions": [
    { "from": "proposed", "to": "accepted" },
    { "from": "accepted", "to": "superseded" },
    { "from": "accepted", "to": "archived" },
    { "from": "superseded", "to": "archived" }
  ],
  "sources": [
    {
      "kind": "front_matter",
      "glob": "docs/decisions/ADR-*.md",
      "stage_field": "status",
      "stage_aliases": { "Accepted": "accepted", "Proposed": "proposed", "Superseded": "superseded" },
      "supersession_field": "superseded_by"
    }
  ],
  "defaults": {
    "wave": "A",
    "case_insensitive_stage_match": true
  }
}
```

### `LifecycledArtifact`

```rust
pub struct LifecycledArtifact {
    pub location: String,
    pub kind: String,           // == LifecycleConfig.name
    pub current_stage: Option<String>,
    pub observed_at: NaiveDate,
    pub deadline_at: Option<NaiveDate>,
    pub history: Vec<Transition>,
    pub supersession_target: Option<String>,
    pub milestone_anchor: Option<String>,
}
```

## Consequences

- Adding a new lifecycle is a 3-file commit: config JSON, thin dev-CLI, plan file. No new kernel.
- Schema evolution (e.g. new violation type) happens once in the framework kernel and propagates to every lane.
- The sunset-lifecycle crate stays canonical at its layer; the framework is the canonical shape for everything else.
- Future M-CC phase: convert sunset-lifecycle into a config-driven instance for full DRY (successor-IP commit, not blocking).

## Initial lifecycle catalog (Wave-A scaffold)

Per Phase-1 audit 2026-05-15:

| Lifecycle | Stages | Source | Population |
|---|---|---|---|
| `adr-status` | proposed → accepted → superseded → archived | `docs/decisions/ADR-*.md` front-matter | 85 ADRs (18 with status) |
| `crate-status` | scaffolded → live → quiescent → archived | `crates/*/Cargo.toml` + workspace membership + git activity | 283 crates |
| `plan-status` | scaffolded → in-progress → complete → archived | `.omc/plans/**/*.md` front-matter | 336 plan files |
| `api-stability-tier` | experimental → stable → deprecated → removed | `#[deprecated]` attrs + Cargo metadata table | 10 `#[deprecated]` sites |
| `capability-status` | proposed → granted → revoked → expired | capability spec files + capability-registry-domain | 4 capability crates |
| `migration-status` | pre-cutover → in-cutover → cleanup → done | cutover plan front-matter + milestone anchor | 9 cutover/migration files |
| `doc-status` | drafted → published → stale → archived | `docs/**/*.md` mtime + front-matter | broad (covered by doc-freshness; framework owns schema) |
| `dependency-status` | added → in-use → deprecated → removed | `Cargo.lock` + ADR supersession refs | Cargo.lock-driven |
| `feature-flag-status` | proposed → live → ramped → deprecated → removed | `#[cfg(feature` + feature manifest | 6 occurrences |

Each lane ships under M01-foundation/phases/P03-purpose-orphan-detection/ as `fitness-<name>-lifecycle-lane.md`.

## Naming justification (per `feedback_naming_justification`)

- Crate `oya-governance-lifecycle-kernel` — v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:lifecycle>-<layer:kernel>`; 13-layer-enum suffix `kernel` (I/O-free port + pure check functions per ADR-0056 "port-in-kernel").
- Dev-CLI `oya-governance-<lifecycle-name>-lifecycle-app` — v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:<lifecycle-name>-lifecycle>-<layer:app>`; canonical `app` suffix per ADR-0105 amendment 2026-05-15 / ADR-0107 amendment 2026-05-15.

## References

- Memory feedback `lifecycle-automation-universal` (canonical directive file 2026-05-15).
- Memory feedback `no-exceptions-canonical` (one canonical shape; no per-lifecycle exception kernels).
- ADR-0037 (public API stability tiers — informs `api-stability-tier` config).
- ADR-0054 (grit scaffold-claim — used to scaffold this framework atomically).
- ADR-0056 (clean architecture BNF — kernel is I/O-free).
- ADR-0105 (13-layer enum — `kernel`/`app` suffixes).
- ADR-0107 (tools/ canonical suffix binding — dev-CLIs end in `-app`).
