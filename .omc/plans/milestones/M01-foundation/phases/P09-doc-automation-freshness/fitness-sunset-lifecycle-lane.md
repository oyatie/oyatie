---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P09-IP-FITNESS-SUNSET-LIFECYCLE
title: Fitness lane — sunset-lifecycle (ADR-0108 sunset → deprecation → removal automation)
status: scaffolded
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - crates/oya-governance-sunset-lifecycle-kernel
  - tools/oya-governance-sunset-lifecycle-app
adr_anchor: docs/decisions/ADR-0108-sunset-lifecycle-automation.md
related_adrs:
  - docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md
  - docs/decisions/ADR-0083-rust-error-handling-tier-decision.md
  - docs/decisions/ADR-0109-lifecycle-automation-framework.md
naming_justification:
  oya-governance-sunset-lifecycle-kernel: |
    v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:kernel>`;
    13-layer-enum suffix `kernel` (ADR-0105 Amendment 1); port-in-kernel, I/O-free
    check function per ADR-0056; Tier 1 library per ADR-0083 (no `.unwrap()`
    outside `#[cfg(test)]`, kernel-local `Date` type — zero non-std deps).
  oya-governance-sunset-lifecycle-app: |
    v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:app>`;
    13-layer-enum suffix `app` (ADR-0107 §"Amendment 2026-05-15 — no-exception
    canonical naming"); composition-root binary tool surface; Tier 2 per ADR-0083.
purpose: Detect every sunset clause in the repo (ADR frontmatter, spec JSON, Cargo manifest) and emit findings based on its lifecycle state so sunset → deprecation → removal is automated, not intent-only.
---

# M01-P09-IP-FITNESS-SUNSET-LIFECYCLE — Fitness lane: sunset-lifecycle

## Purpose

User directive (2026-05-15): `sunset > deprecation > removal. dispatch.`
Per `feedback_no_exceptions_canonical.md`, time-bounded sunset clauses
are canonical *because of* the sunset clause, not despite it. This lane
makes that framing enforceable by:

1. **Discovering** every sunset clause across three surfaces:
   - ADR YAML frontmatter (`docs/decisions/*.md`)
   - spec JSON `_sunset` objects (`specs/*.json`)
   - `[package.metadata.oya.sunset]` Cargo manifest sections
2. **Classifying** each clause into a `LifecycleState` (PRE_SUNSET,
   SUNSET_REACHED, DEPRECATED, REMOVAL_REACHED, MISSING_FIELDS) given
   `(clause, now, reached_milestones)`.
3. **Emitting** findings for the three failure states; healthy states
   are silent.

The lane operationalizes ADR-0108. The 30-day deprecation lag and
90-day removal lag are **canonical sub-rules**, not exceptions, per the
no-exceptions doctrine.

## Naming justification (per `feedback_naming_justification`)

- `oya-governance-sunset-lifecycle-kernel` — v4 BNF compliant:
  product=`foundry`, facet=`fitness`, topic=`sunset-lifecycle`,
  layer=`kernel`. 13-layer-enum suffix `kernel` per ADR-0105 §"Amendment
  1". I/O-free port-in-kernel per ADR-0056 §"Layer semantics > kernel".
  Zero non-std deps (kernel-local `Date` type) — honors ADR-0083 Tier 1.
- `oya-governance-sunset-lifecycle-app` — v4 BNF compliant:
  product=`foundry`, facet=`fitness`, topic=`sunset-lifecycle`,
  layer=`app`. 13-layer-enum suffix `app` per ADR-0107 §"Amendment
  2026-05-15 — no-exception canonical naming" (every `tools/` crate
  MUST end in a canonical layer suffix; binaries take `-app`).
  Composition-root binary, walks file system, calls kernel.

## Symbols-to-grit-claim

```
crates/oya-governance-sunset-lifecycle-kernel/src/lib.rs::evaluate
crates/oya-governance-sunset-lifecycle-kernel/src/lib.rs::SunsetClause
crates/oya-governance-sunset-lifecycle-kernel/src/lib.rs::LifecycleState
crates/oya-governance-sunset-lifecycle-kernel/src/lib.rs::Violation
tools/oya-governance-sunset-lifecycle-app/src/main.rs::main
tools/oya-governance-sunset-lifecycle-app/src/main.rs::discover_adr
tools/oya-governance-sunset-lifecycle-app/src/main.rs::discover_specs
tools/oya-governance-sunset-lifecycle-app/src/main.rs::discover_cargo_metadata
```

Scaffold-claim via ICM `scaffold-locks-oyatie` per ADR-0054 — window
opened in this PR's scaffold with agent=`ad063e4f`.

## Agent-prerequisites

ADR-0108 read; `feedback_no_exceptions_canonical.md` read; ADR-0054
read; ADR-0083 Tier 1 honored; ADR-0105/0107 amended canonical naming
honored; M01-P09 INDEX read.

## Algorithm (kernel)

1. For each `SunsetClause`, resolve the effective `deprecation_at` and
   `removal_at` via the 30 / 90 day canonical defaults when absent.
2. Determine `sunset_reached`:
   - `sunset_at` present and `now ≥ sunset_at`, OR
   - `sunset_milestone` present and listed in `reached_milestones`.
3. If `effective removal_at ≤ now` → `RemovalReached`
   (days_overdue = now − removal_at).
4. Else if sunset reached:
   - has deprecation marker → `Deprecated` (silent).
   - else → `SunsetReached`
     (days_overdue = max(0, now − effective deprecation_at)).
5. Else if neither date nor milestone present → `MissingFields`.
6. Else → `PreSunset` (silent).

## Acceptance-test-commands

```
PATH="/Users/jasonlee/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo test -p oya-governance-sunset-lifecycle-kernel
PATH="/Users/jasonlee/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo test -p oya-governance-sunset-lifecycle-app
PATH="/Users/jasonlee/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo run -q -p oya-governance-sunset-lifecycle-app -- --now 2026-05-15
```

## Done-criteria

- Kernel: 11 unit tests green (state machine × 5 states + defaulting +
  date arithmetic + leap-year + milestone precedence + missing fields +
  empty input).
- Dev-CLI: 7 unit tests green (3 discovery surfaces + frontmatter
  fallback + prose-only body discovery + 2 options-parser tests).
- `cargo check --workspace` green.
- Lane runs against the live workspace and reports the WARN baseline:
  6 violations, all MISSING_FIELDS (3 ADRs + 3 specs).
- ADR-0108 cited from kernel doc-comment header and from this plan's
  frontmatter; ICM scaffold-lock open/close pair logged.

## Live baseline (2026-05-15)

```
sunset-lifecycle FAIL: clauses_scanned=6 violations=6
  MISSING_FIELDS (needs-schema-upgrade): 6
  - docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md
  - docs/decisions/ADR-0067-ops-oyatie-com-hyperscaler-operations-console.md
  - docs/decisions/ADR-0083-rust-error-handling-tier-decision.md
  - specs/markdown-retirement-policy.json#prose
  - specs/multispectrum-review.json#prose
  - specs/oyatie-doctrine.json#prose
```

## Ratchet plan (WARN → BLOCK)

- **Wave A (now, this PR) — WARN.** Lane runs but is non-blocking; CI
  captures the 6-finding baseline. Authors receive the schema-upgrade
  signal but no merge gate.
- **Wave B (day 7, next PR cluster) — BLOCK new clauses.** Any NEW
  sunset prose introduced after the baseline date that lacks
  schema fields hard-fails the gate. The existing 6 are grandfathered
  via a baseline-allowlist file (path tbd in Wave B).
- **Wave C (day 0 from day-N) — full BLOCK.** Lane is full BLOCKER;
  baseline must be zero (every ADR-0037, ADR-0067, ADR-0083, and the 3
  specs upgrade to the schema). Per the no-exceptions doctrine this is
  a hard wall.

This matches the `fitness-adapter-with-no-importer-lane` ratchet
structure (`.omc/plans/milestones/M01-foundation/phases/P03-purpose-orphan-detection/fitness-adapter-with-no-importer-lane.md`).

## Schema reference (for schema-upgrade authors)

Add to ADR frontmatter:

```yaml
sunset_at: "2026-09-01"
deprecation_at: "2026-10-01"   # optional; default = sunset_at + 30
removal_at: "2027-01-01"       # optional; default = deprecation_at + 90
sunset_topic: "<short-slug>"
```

Add to spec JSON:

```json
{
  "_sunset": {
    "sunset_at": "2026-09-01",
    "sunset_topic": "<short-slug>",
    "status": "Deprecated"
  }
}
```

Add to `Cargo.toml`:

```toml
[package.metadata.oya.sunset]
sunset_at = "2026-09-01"
sunset_topic = "<short-slug>"
status = "Deprecated"
```

For milestone-anchored sunsets, replace `sunset_at` with
`sunset_milestone: "M01-P08-merge"` (canonical milestone id).

## Rollback-procedure

`grit done` is atomic per-symbol. The lane is purely additive —
reverting the merge commit removes the two new crates from
`[workspace.members]`, the new ADR-0108, and this plan file; no other
crate depends on either.

## ICM coordination

Scaffold-lock window OPEN/CLOSE pair logged in `scaffold-locks-oyatie`
(per ADR-0054), tagged `oya-foundry-fitness-sunset-lifecycle,ADR-0108`.

## Next-IP-pointer

Wave B (delta-gate against the 6-clause baseline) lands as a follow-up
IP in this phase index, paired with the first schema-upgrade commit
on ADR-0037 / ADR-0067 / ADR-0083.

## Decision-log (Linus good-taste row)

Special cases eliminated by this IP: the
"sunset-prose-but-no-CI-observable-state" failure mode is mechanically
detected on every PR; sunset → deprecation → removal becomes a
machine-evaluated lifecycle rather than three disjoint author intents.
