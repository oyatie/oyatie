---
id: ADR-0109
title: Lifecycle-automation framework (generic kernel + per-lifecycle configs)
status: Accepted
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

The parallel `oya-foundry-fitness-sunset-lifecycle-kernel` (committed independently by a concurrent agent) is a one-off dedicated crate; this ADR establishes the FRAMEWORK that future lifecycles (ADR status, crate status, plan status, capability status, migration, dependency, feature-flag, API-stability, doc) consume.

## Decision

1. **One generic kernel.** `oya-foundry-fitness-lifecycle-kernel` exposes the canonical `LifecycleConfig`, `LifecycledArtifact`, `Stage`, `Transition`, `Violation`, and `evaluate()` function. Every lifecycle lane is data — a JSON config under `specs/cross-cutting/lifecycle-configs/`. Adding a new lifecycle is a config-file + thin dev-CLI commit, not a new kernel.

2. **Per-lifecycle dev-CLI wrappers.** Each lifecycle ships `tools/oya-foundry-fitness-<lifecycle-name>-lifecycle-app/` — a thin binary that loads its config, discovers artifacts from the repo (via the kernel's source-spec abstraction), calls `evaluate()`, and reports/exits. The CLI is the IO ring (clean-architecture port-in-kernel: kernel is I/O-free; the app does the directory walk + front-matter parsing).

3. **Canonical lifecycle metadata schema.** Every lifecycled artifact MAY carry `_lifecycle.kind`, `_lifecycle.stage`, `_lifecycle.transitions[]`, `_lifecycle.deadline_at?`, `_lifecycle.milestone_anchor?`. Existing fields (`status:`, `supersedes:`, `superseded_by:`) remain canonical inputs; the framework reads them via per-config source specs and does NOT require a flag-day migration.

4. **State-machine algebra (kernel contract).** Given a `LifecycleConfig { name, stages: [Stage], transitions: [from → to (predicate)], sources: [SourceSpec], defaults }` and a slice of `LifecycledArtifact { location, kind, current_stage, observed_at, deadline_at, history }`, plus `now: NaiveDate` and `reached_milestones: &[String]`, `evaluate()` returns `Vec<Violation>` for:
   - `StageNotDeclared` — artifact in scope but no stage detected.
   - `UnknownStage` — declared stage absent from config's stage enum.
   - `OverdueTransition` — `deadline_at < now` and `current_stage` not terminal.
   - `MissingSupersession` — terminal stage reached but no supersedes/superseded_by edge.
   - `MilestoneOverdue` — milestone reached, transition not advanced.
   - `IllegalTransition` — `history` contains a `from → to` not in `transitions`.

5. **Wave ratchet (canonical per fitness-lane pattern).** Wave A = WARN baseline; Wave B = BLOCK on NEW violations; Wave C = BLOCK on all. Each per-lifecycle plan declares its wave.

6. **Concurrent sunset-lifecycle integration.** The dedicated `oya-foundry-fitness-sunset-lifecycle-kernel` is canonical for the sunset-clause domain. The framework applies to OTHER lifecycles immediately; a follow-up refactor MAY convert sunset-lifecycle into a config-driven instance once that crate lands. Both shapes are canonical at the kernel layer; the framework's value is per-lifecycle DRY for the 9+ other state machines.

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
- Future M-CC phase: convert sunset-lifecycle into a config-driven instance for full DRY (follow-up commit, not blocking).

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

Each lane ships under M-CC-cross-cutting/phases/P03-purpose-orphan-detection/ as `fitness-<name>-lifecycle-lane.md`.

## Naming justification (per `feedback_naming_justification`)

- Crate `oya-foundry-fitness-lifecycle-kernel` — v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:lifecycle>-<layer:kernel>`; 13-layer-enum suffix `kernel` (I/O-free port + pure check functions per ADR-0056 "port-in-kernel").
- Dev-CLI `oya-foundry-fitness-<lifecycle-name>-lifecycle-app` — v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:<lifecycle-name>-lifecycle>-<layer:app>`; canonical `app` suffix per ADR-0105 amendment 2026-05-15 / ADR-0107 amendment 2026-05-15.

## References

- Memory feedback `lifecycle-automation-universal` (canonical directive file 2026-05-15).
- Memory feedback `no-exceptions-canonical` (one canonical shape; no per-lifecycle exception kernels).
- ADR-0037 (public API stability tiers — informs `api-stability-tier` config).
- ADR-0054 (grit scaffold-claim — used to scaffold this framework atomically).
- ADR-0056 (clean architecture BNF — kernel is I/O-free).
- ADR-0105 (13-layer enum — `kernel`/`app` suffixes).
- ADR-0107 (tools/ canonical suffix binding — dev-CLIs end in `-app`).
