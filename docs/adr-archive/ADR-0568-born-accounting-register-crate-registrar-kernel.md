---
id: ADR-0568
title: "born-accounting register_crate: the pure registrar kernel (RegisterCrateRequest → RegistrationPlan)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-21
door: two-way
owner: cloud-ci-platform
supersedes: []
superseded_by: [ADR-709]
amends: []
depends_on: [ADR-0515, ADR-0548, ADR-0555, ADR-0562]
related: [ADR-0542, ADR-0017, ADR-0064, ADR-0131, ADR-0245, ADR-0538, ADR-0540, ADR-0552]
related_specs:
  - /specs/capability-registry.json
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0568: born-accounting register_crate — the pure registrar kernel

## Status

**Proposed - 2026-06-21 (authored for founder sign-off; door: two-way — additive, generates no
faces itself, and is removable by deleting the crate without unwinding any SSOT; the producer
remains the sole face generator).**

## Context

Every new-crate PR this session (#783 sqlx, #779/#780 gates) took roughly four CI round-trips
because born-accounting is a derived join over about six hand-authored SSOTs with NO single
entrypoint. Adding one crate means hand-editing OWNERS, the workspace member globs, the capability
registry, the owning ADR's governed-path justification, the catalog record, and the reachability
registry, then materializing and settling the generated faces — and missing any one of them turns
a gate RED only after a push. Founder doctrine is explicit: manual-twice means automate it, and
productize the pipeline (it is itself a product).

The fleet is already roughly 90% producer-driven: `oya-cloud-ci-accounting-registry-app` reads a
few hand-authored SSOTs and generates 12 of 14 gate faces (registry-drift is regenerate +
byte-diff). The friction is the absence of an orchestrator for the HAND inputs — and a recurring
defect class in one of them: the governed-path justification. The producer's
`resolve_justifications` (accounting-registry-app `main.rs:2899`) tokenizes each ADR body on
whitespace/quotes/brackets, trims `:`/`#`/`*` and a trailing `.`, and credits a tracked path only
when a token equals it EXACTLY. A brace-glob like `src/{lib,plan}.rs` therefore tokenizes to the
literal `src/{lib,plan}.rs`, which equals no tracked path, leaving every real source file
unjustified. That is task #66.

## Decision

Introduce **`libs/oya-crate-registrar-kernel`**: the PURE planner half of `register_crate`
(G011 pipeline-as-product, slice 1). It composes a [`RegisterCrateRequest`] with a
[`CurrentState`] snapshot of the born-accounting SSOTs and computes an ordered, typed
[`RegistrationPlan`] — the set of edits that make a new crate fully born-accounted. It is a
diff/upsert: re-planning against an already-registered snapshot yields an empty plan.

### D1 — R0 pack-shape (pure kernel; ADR-0548 D2)

No clock, no rand, no net, no shell, no filesystem in the verdict path: the kernel COMPUTES a
plan, it does NOT apply it. Apply/I/O is slice 3 (the registrar app). Inputs are the request plus
a caller-supplied snapshot of current SSOT state; the closed capability set is supplied as DATA so
the kernel stays repo-neutral. The output is a plan or a typed validation refusal. The `*-kernel`
name puts the crate inside the kernel-purity scan from birth.

### D2 — Typed plan as an ordered upsert diff

`RegistrationPlan` is an ordered list of typed `Edit`s: `OwnersWrite`, `WorkspaceMemberGlob`,
`CapabilityMapping`, `AdrGovernedPathAppend` (verbatim paths), `CatalogYaml`, `ReachabilityEntry`,
and the mandatory-last `FacesSettle`. Each edit is emitted ONLY when the snapshot shows the
corresponding SSOT does not already carry the registration, so registering twice is a no-op. A
purely no-op plan emits no `FacesSettle` (settle only when something changed). Automation APPLIES
human decisions, it never invents them (ADR-0548 D2): `capability`, `owning_adr`, `owner`, and the
catalog SLO are human-supplied request inputs; the kernel rejects a catalog request whose SLO is
empty rather than defaulting it.

### D3 — Fail-closed validators

Capability membership is validated against the provided closed set (an unknown capability — or any
capability against an empty set — is refused). The crate leaf must end with the suffix its role
requires (`-kernel`/`-domain`/`-api`/`-adapter`/`-app`), and the role maps to its BUCK rule
(`rust_library` for libraries, `rust_binary` for apps, plus `rust_test` when the crate has test
code — the target-parity contract). The owner must satisfy the OWNERS principal schema, and the
owning ADR id must be a real, allocated `ADR-` identifier (four-plus digits).

### D4 — Verbatim governed-path enumeration (closes #66)

The kernel enumerates the crate's governed paths LITERALLY — the conventional set (`Cargo.toml`,
`BUCK`, `OWNERS`, and `src/lib.rs` when present) plus any caller-supplied extras — and REFUSES any
governed path containing brace-glob syntax (`{`/`}`). Emitting one verbatim tracked path per line
is precisely the shape `resolve_justifications` matches, so the `AdrGovernedPathAppend` edit
produces a `## Governed surfaces` block every entry of which the producer credits. This kernel
dogfoods the fix: its own governed surfaces below are enumerated verbatim, brace-glob-free.

### D5 — Composition

`register_crate` is the reusable per-crate primitive the lane-verify shared primitive (ADR-0542
slice 4) invokes per new crate: lane-verify owns lane policy, the kernel owns per-crate mechanics,
no duplication. Slices 2-4 add the thin writers, the app I/O bridge that wires the plan to the
existing producer bridges (`--fix-owners`, `--fix-reachability`, `--next-adr`) and invokes
materialize, and the CRD-reconciler wrapper (ADR-0548 D3) — none of which this slice ships.

Precedent (proven patterns, Rust reimplementation): `cargo new` + workspace registration, Bazel
gazelle, Nx/Turbo create-package, Backstage scaffolder, OPA ConstraintTemplate — each a
scaffold-the-accounting primitive, here reimplemented Rust-native and pure.

## Governed surfaces

The following repo paths are governed by this ADR. The accounting gate validates that each is
justified (this ADR is the justification reference):

```
ci/facade/crate-registration/BUCK
ci/facade/crate-registration/Cargo.toml
ci/facade/crate-registration/OWNERS
ci/facade/crate-registration/src/lib.rs
ci/facade/crate-registration/src/tests.rs
libs/oya-crate-registrar-app/BUCK
libs/oya-crate-registrar-app/Cargo.toml
libs/oya-crate-registrar-app/OWNERS
libs/oya-crate-registrar-app/src/lib.rs
libs/oya-crate-registrar-app/src/tests.rs
libs/oya-crate-registrar-kernel/BUCK
libs/oya-crate-registrar-kernel/Cargo.toml
libs/oya-crate-registrar-kernel/OWNERS
libs/oya-crate-registrar-kernel/src/lib.rs
```

## Consequences

- One typed entrypoint for the new-crate born-accounting diff: the multi-round-trip friction class
  collapses to a single computed plan, applied by slice 3 and re-checked by lane-verify per push.
- The #66 unjustified-path class is closed at the planner: brace-globs are refused and every
  governed path is emitted verbatim, matching the producer's exact-token justification.
- The kernel is itself governed (the `*-kernel` naming puts it inside the kernel-purity scan) and
  cutover-stable: std-only, no transient deps (ADR-0510 indifferent).
- Idempotency is structural: the plan is a diff vs the snapshot, so applying it twice is a no-op
  and a partially-registered crate yields only the missing edits.
- Residual scope (explicitly deferred): the thin writers, the app I/O bridge and its self-validation
  over regenerated faces, and the CRD reconciler are slices 2-4; this ADR commissions only the
  pure planner.

## Alternatives considered

- **A single I/O app with no pure kernel.** Rejected: the plan computation is the part most worth
  unit-testing with RED/GREEN fixtures and the part the kernel-purity doctrine wants pure; folding
  it into an I/O binary would make the verdict path untestable without a filesystem.
- **Brace-glob governed paths (status quo).** Rejected: that is the #66 defect — the producer's
  exact-token matcher never credits a `{a,b}` token, so the paths stay unjustified.
- **Inventing default capabilities / SLOs.** Rejected per ADR-0548 D2: automation applies human
  decisions, it never invents them; an unknown capability and an empty SLO are refusals.

## References

- ADR-0515 (one canonical CI), ADR-0548 (pipeline-as-product paved road), ADR-0555 (unaccounted
  artifacts unmergeable — structural accounting), ADR-0562 (capability-first organization + closed
  capability registry).
- ADR-0542 (the lane-verify shared primitive that invokes this per-crate primitive).
- accounting-registry-app `main.rs:2899` `resolve_justifications` (the exact-token justification
  matcher this kernel's verbatim enumeration targets — task #66).
- Founder doctrine: manual-twice means automate it; productize the pipeline; proven patterns,
  Rust reimplementation; enforcement layering (structural impossibility over per-lane discipline).
