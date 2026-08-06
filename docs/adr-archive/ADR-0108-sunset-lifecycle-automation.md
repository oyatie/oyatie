---
id: ADR-0108
title: Sunset → deprecation → removal lifecycle automation schema (machine-readable)
status: Superseded
superseded_by: [ADR-709]
doc_status: published
owner: council-architecture
date: 2026-05-15
relates_to:
  - ADR-0037-public-api-stability-tiers-and-deprecation.md
  - ADR-0054-grit-scaffold-claim-pattern.md
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0083-rust-error-handling-tier-decision.md
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
  - ADR-0105-13-layer-enum-and-check-family-patterns.md
  - ADR-0107-tools-implicit-app-convention.md
  - ADR-0109-lifecycle-automation-framework.md
amends: []
supersedes: []
sunset_at: ~
sunset_milestone: ~
sunset_topic: adr-0108-self
---

# ADR-0108: Sunset → deprecation → removal lifecycle automation schema (machine-readable)

## Status
Accepted — 2026-05-15.

## Context

User directive 2026-05-15 — `sunset > deprecation > removal. dispatch.`
The `feedback_no_exceptions_canonical.md` doctrine reframes time-bounded
sunset clauses as canonical *because of* the sunset clause, not despite
it:

> "Time-bounded carve-outs with named sunsets … are canonical *because*
> of the sunset clause, not despite it. Wording must make the canonical
> pattern explicit (canonical sunset-bounded extension), not frame it as
> a temporary escape hatch."

To honor that framing, every sunset clause MUST be enforceable by an
automated fitness lane. Without a machine-readable schema:

1. Sunset prose ("scheduled to sunset 2026-06-01", "sunset_note: …") is
   indistinguishable from intent vs. commitment to the build.
2. The 30-day deprecation lag and 90-day removal lag implicit in
   ADR-0037 cannot be evaluated automatically.
3. ADR-0083 silent-failure-hunter cannot diff sunset reach across PRs.
4. The repo currently has 6 sunset prose mentions (3 ADRs + 3 specs)
   with zero machine-readable schema — every one of them is a stalled
   sunset whose state today is not observable by CI.

ADR-0037 ships the *vocabulary* (preview / stable / GA, 6 / 12 month
deprecation, per-deprecation telemetry) but pins it to runtime call
events (`DeprecationUsed` audit-chain event). It does NOT pin a
file-system-discoverable schema authors put on ADR frontmatter, spec
JSON, or `Cargo.toml`. This ADR fills that gap.

ADR-0109 ships a *generic lifecycle-automation framework* (kernel +
per-lifecycle configs). It is complementary: ADR-0108 pins the **schema
+ defaults** for the sunset → deprecation → removal lifecycle
specifically; ADR-0109 pins the framework that can host this and other
lifecycles.

## Decision

### Machine-readable sunset schema

Every sunset clause MUST be representable as a `SunsetClause` record
with the following fields. The schema is identical across all three
surfaces (ADR frontmatter YAML, spec JSON `_sunset` object,
`[package.metadata.oya.sunset]` Cargo manifest section):

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `sunset_at` | RFC3339 date `YYYY-MM-DD` | one of `sunset_at` OR `sunset_milestone` | — | Calendar-anchored sunset date. |
| `sunset_milestone` | canonical milestone id (e.g. `M01-P08-merge`) | one of `sunset_at` OR `sunset_milestone` | — | Milestone-anchored sunset gate. |
| `deprecation_at` | RFC3339 date `YYYY-MM-DD` | no | `sunset_at + 30 days` | Date at which the surface MUST carry a deprecation marker. |
| `removal_at` | RFC3339 date `YYYY-MM-DD` | no | `effective deprecation_at + 90 days` | Date at which the surface MUST be deleted from the repo. |
| `sunset_topic` | short slug | yes | — | Cross-surface correlation id (e.g. `tools-implicit-app-exception`). |

**Canonical sub-rule (defaulting).** The 30/90 day defaults are a
**canonical sub-rule** of the schema, not exceptions. Per
`feedback_no_exceptions_canonical.md` vocabulary registry:

> "exception → extension; carve-out → bounded-extension (with explicit
> sunset clause if time-bounded)."

A clause that supplies only `sunset_at` is canonical because the
`deprecation_at` and `removal_at` are derivable. Authors MAY override
the defaults; lane evaluation always uses the effective dates
(`explicit ?? default`).

### State machine

For each clause, the fitness lane evaluates one of five lifecycle
states given `(clause, now, reached_milestones)`:

| State | Trigger | Lane action |
|---|---|---|
| `PRE_SUNSET` | `now < sunset_at` AND `sunset_milestone` not reached | OK (silent). |
| `SUNSET_REACHED` | `now ≥ sunset_at` (or milestone reached) AND no deprecation marker | **Finding: should-be-deprecated.** |
| `DEPRECATED` | deprecation marker present, `now < effective removal_at` | OK (informational). |
| `REMOVAL_REACHED` | `now ≥ effective removal_at` AND clause/code still in repo | **Finding: must-be-removed.** |
| `MISSING_FIELDS` | clause has sunset prose but neither `sunset_at` nor `sunset_milestone` | **Finding: needs-schema-upgrade.** |

Findings carry `{ clause_location, state, expected_action, days_overdue }`.

**Determinism.** `now` is passed as a parameter to the kernel; the
kernel does NOT read the system clock. The discovery layer (dev-CLI)
captures `now = today UTC` at startup unless `--now YYYY-MM-DD`
overrides it. This makes the kernel deterministic per ADR-0083 Tier 1
and makes time-travel testing trivial.

### Three discovery surfaces

The dev-CLI walks three concrete surfaces:

1. **ADR frontmatter** — `docs/decisions/*.md` YAML frontmatter keys
   `sunset_at`, `sunset_milestone`, `deprecation_at`, `removal_at`,
   `sunset_topic`, OR a `sunset_note:` prose key OR body-level sunset
   prose (the prose case maps to `MISSING_FIELDS`).
2. **Spec JSON `_sunset` object** — `specs/*.json` files
   containing a top-level `"_sunset": { "sunset_at": …, "sunset_topic":
   …, … }` object.
3. **Cargo manifest** — `[package.metadata.oya.sunset]` section with
   the same scalar fields. Co-exists with the existing
   `[package.metadata.oya]` block (per `oya-shared-*-check-cli` pattern).

### Marker recognition

`has_deprecation_marker = true` when:

- ADR frontmatter `status: Deprecated` or `status: Superseded`.
- Spec `_sunset.status` = `"Deprecated"` / `"Superseded"`.
- Cargo manifest `[package.metadata.oya.sunset].status` = `"Deprecated"`.

Rust source-level `#[deprecated]` attribute recognition is a Wave-B
successor-IP (see Follow-ups §1).

### Lane shape

The lane is split across two crates per the
`oya-governance-<topic>-<layer>` canonical pattern (ADR-0105
§"Amendment 1" + ADR-0107 §"Amendment 2026-05-15"):

- **`crates/oya-governance-sunset-lifecycle-kernel`** — I/O-free
  pure kernel (port-in-kernel per ADR-0056 §"Layer semantics > kernel";
  Tier 1 library per ADR-0083 — no `.unwrap()` outside `#[cfg(test)]`).
  Exposes `Date`, `SunsetClause`, `LifecycleState`, `Violation`,
  `evaluate(clauses, now, reached_milestones)`,
  `effective_deprecation_at`, `effective_removal_at`.
- **`tools/oya-governance-sunset-lifecycle-app`** — composition-root
  binary (Tier 2 per ADR-0083; `-app` suffix per ADR-0107 §"Amendment
  2026-05-15"). Walks the three discovery surfaces, calls the kernel,
  prints findings, exits non-zero on any violation.

### Naming justifications (per `feedback_naming_justification`)

- `oya-governance-sunset-lifecycle-kernel`: v4 BNF
  `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:kernel>`;
  13-layer-enum suffix `kernel` (ADR-0105 Amendment 1); port-in-kernel
  per ADR-0056.
- `oya-governance-sunset-lifecycle-app`: v4 BNF
  `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:app>`;
  13-layer-enum suffix `app` (ADR-0107 §"Amendment 2026-05-15 —
  no-exception canonical naming"); composition-root binary per
  ADR-0056 §"Layer semantics > app".

### Live baseline (2026-05-15)

First run against the repo found **6 violations**, all `MISSING_FIELDS`:

| Surface | Path |
|---|---|
| ADR | `docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md` |
| ADR | `docs/decisions/ADR-0067-ops-oyatie-com-hyperscaler-operations-console.md` |
| ADR | `docs/decisions/ADR-0083-rust-error-handling-tier-decision.md` |
| Spec | `specs/markdown-retirement-policy.json` |
| Spec | `specs/multispectrum-review.json` |
| Spec | `specs/oyatie-doctrine.json` |

Wave-A authors WARN these in CI; Wave-B blocks NEW clauses without
schema; Wave-C drives the 6 to zero by schema-upgrade authoring (per the
plan file).

## Consequences

- **Sunset prose stops being intent-only.** Every clause is either in
  the schema (and therefore observable across `now`) or surfaced as
  `MISSING_FIELDS` until it is.
- **The 30/90 default is canonical.** Authors don't restate the lag;
  the schema's defaulting rule does. This eliminates the per-clause
  drift that ADR-0037's prose ("6 months minimum") leaves room for.
- **Removal becomes mechanical.** A clause that overshoots `removal_at`
  by a single day appears as `REMOVAL_REACHED` on the next lane run —
  no more "we'll get to it eventually" sunsets.
- **ADR-0037 stays primary for the runtime-event side** (per-tenant
  `DeprecationUsed` audit-chain emission). ADR-0108 covers the
  static-file side. They compose: the runtime event tells you which
  deprecated endpoints tenants still call; the static lane tells you
  which static surfaces should have been deprecated/removed by now.

## Alternatives Considered

1. **Extend ADR-0037 with an "Amendment 2026-05-15" section.** Rejected:
   ADR-0037 is a contract-tier ADR; folding a static-file schema into it
   confuses the runtime-event scope. The two ADRs have different audit
   surfaces (audit chain vs. file system) and different consumers
   (tenants vs. authors). They reference each other instead.
2. **Single sunset-at field, no deprecation_at/removal_at.** Rejected:
   the lifecycle has three states (sunset / deprecation / removal); a
   single field collapses two of them and loses the 30-day deprecation
   lead that ADR-0037 GA tier requires.
3. **Compute removal_at = sunset_at + 120 days flat.** Rejected: the
   30/90 split lets authors override deprecation_at without recomputing
   removal_at (and vice versa). Two anchors compose; one anchor doesn't.
4. **Walk the git history for retired clauses.** Rejected for now: the
   lane scans the working tree only. Git-history scanning is a
   Wave-D successor-IP.

## Drivers

- User directive 2026-05-15 — `sunset > deprecation > removal. dispatch.`
- `feedback_no_exceptions_canonical.md` §"What is NOT an escape hatch"
  — time-bounded sunset clauses are canonical because of the sunset
  date, not despite it.
- `decision-principles.json` DP-03 (Mechanical prevention over process)
  — prose-only sunset clauses are the failure mode this ADR mechanizes.
- `feedback_no_silent_regression.md` — sunset is the *legitimate*
  channel for breaking change; this ADR removes its silent-drift mode.

## Follow-ups

1. **Rust source `#[deprecated]` recognition.** Wave-B addition to the
   dev-CLI: walk `crates/*/src/**/*.rs` for `#[deprecated]` /
   `#[doc(hidden)]` and pair attribute metadata with discovered
   clauses. Currently `has_deprecation_marker` is sourced from frontmatter
   `status:` only.
2. **Schema-upgrade authoring sweep.** Drive the 6 baseline `MISSING_FIELDS`
   to zero by adding the schema fields to ADR-0037 / 0067 / 0083 and the
   3 specs. Each upgrade is its own atomic commit per ADR-0054.
3. **Git-history walk.** Wave-D successor-IP: scan `git log` for retired
   sunset clauses to ensure deletion happened on or after `removal_at`.
4. **Pattern-B status under ADR-0109.** Per ADR-0109 §"Sunset-lifecycle:
   dedicated kernel as canonical pattern (not exception)" (Amendment
   2026-05-15), the sunset-lifecycle stays on its dedicated kernel as
   the canonical Pattern-B shape for date-arithmetic lifecycles. The
   earlier "MAY migrate to ADR-0109 generic framework" successor-IP is
   superseded — there is no migration: sunset-lifecycle and the generic
   framework are both canonical at the kernel layer, selected by
   lifecycle-domain semantics (see ADR-0109's decision rule).

## References

- `feedback_no_exceptions_canonical.md` — canonical-sunset doctrine.
- ADR-0037 — runtime-side deprecation telemetry (pairs with this ADR).
- ADR-0109 — generic lifecycle-automation framework (complementary).
- ADR-0083 — Tier 1 library policy this kernel honors.
- ADR-0056 — port-in-kernel doctrine this kernel honors.
- `.omc/plans/milestones/M01-foundation/phases/P02-doc-automation-freshness/fitness-sunset-lifecycle-lane.md`
  — execution plan + ratchet waves.
