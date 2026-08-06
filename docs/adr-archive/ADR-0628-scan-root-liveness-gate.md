---
id: ADR-0628
title: "Scan-root liveness: a declared coverage root that no longer resolves is a gate blind spot, not clean coverage"
status: Superseded
planning_impact: false
deciders: council-architecture
date: 2026-07-28
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0562]
amends: []
related: [ADR-0515, ADR-0527, ADR-0551, ADR-0554, ADR-0562]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0628 — Scan-root liveness

## Status

Proposed (2026-07-28). Two-way door: the gate is additive, carries a frozen
shrink-only baseline of today's dead roots, and can be removed without unwinding any
other decision.

## Context

Gate policies declare the roots they scan. The cloud-ci fleet already enforces that a
declared root cannot be **removed** without the removal being the subject of a
reviewed change — the anti-narrowing ratchet, e.g.
`rust_first_automation_scan_scope_narrowing`.

Nothing enforced that a declared root still **resolves**. The two rules interact
badly:

1. an ADR-0562 capability move empties `oya/<service>/`
2. the gate's scan root now matches nothing — its coverage silently drops to zero
3. the gate still reports GREEN, because it found no violations in no files
4. the anti-narrowing ratchet now *blocks* deleting the dead root

The ratchet whose purpose is to prevent coverage shrinking ends up preserving the
evidence of coverage that already shrank. `rust-first-automation-policy.json`
documents this against itself: a root is "RETAINED because removing a root fires
rust_first_automation_scan_scope_narrowing".

Measured at time of writing: **186 declared coverage-bearing roots, 4 dead**, all four
in a single policy. With ~250 crates still to move out of the legacy `oya/` and
`cloud/` roots, every remaining move can silently blind a gate with nothing reporting
it.

## Decision

A declared **coverage-bearing** scan root MUST resolve to a real path, or to a glob
matching at least one path, or be declared FORWARD with a stated reason.

Three declaration classes are distinguished, because only one is a defect when dead:

| Class | Keys | Dead entry means |
|---|---|---|
| coverage-bearing | `roots`, `scan_roots`, `crate_root_globs`, `manifest_paths`, `store_manifest_paths` | the gate scans less than it claims — **the defect** |
| vocabulary | `exclude_prefixes`, `allowed_paths`, `forbidden_prefixes` | excludes or permits nothing — stale, not blinding; OUT of scope |
| forward | any of the above, explicitly declared | correct — the destination is declared ahead of the move |

Forward declarations are first-class DATA carrying a reason and what will create the
path. `module-membership` already declares `app`, `base` and `policy` this way so an
ADR-0562 move can land without a policy edit; a gate that flagged those would punish
exactly the practice this repo wants. A forward declaration whose path **lands** must
be retired in the same change, or it degrades into a permanent unaudited exemption
that would hide the path dying later.

Keys are the FULL JSON POINTER, not the leaf name: `roots` occurs at three distinct
nesting levels inside `rust-first-automation-policy.json` alone, and leaf-name keying
would collapse separate declarations so that baselining one silently tolerated
another.

A policy file declaring coverage-bearing roots MUST be registered with this gate, or
exempted with a reason. Without that completeness rule a newly-added gate escapes
liveness checking silently — the same blind spot this decision exists to close.

## Consequences

- A reorg move that empties a declared root now fails CI with the exact root named,
  and with all three legitimate responses stated (repoint / remove-as-subject /
  declare forward), including the anti-narrowing interaction so the reader does not
  attempt a removal and get blocked without explanation.
- The 4 dead roots observed today are frozen as shrink-only debt, not fixed here;
  removing them is a separate reviewed change per the ratchet's own rule.
- No autofix. A dead root has three valid resolutions and the gate cannot choose
  between them; auto-deleting would launder coverage loss as cleanup, which is the
  precise failure being detected. Recorded in
  `ci/facade/gate-self-conformance/gate-self-conformance-policy.json`.

## Governed paths

This decision governs, and justifies the existence of:
`ci/facade/scan-root-liveness/Cargo.toml`,
`ci/facade/scan-root-liveness/BUCK`,
`ci/facade/scan-root-liveness/OWNERS`,
`ci/facade/scan-root-liveness/scan-root-liveness-policy.json`,
`ci/facade/scan-root-liveness/src/lib.rs`,
`ci/facade/scan-root-liveness/tests/scan_root_liveness.rs`, and
`registry/catalog/ci-scan-root-liveness.yaml`.

## Alternatives considered

- **Require every declared path to exist.** Rejected: it flags forward declarations,
  punishing the practice of declaring an ADR-0562 destination before the move.
- **Let the anti-narrowing ratchet handle it.** Rejected: the ratchet governs removal,
  not resolution, and actively preserves dead roots once they appear.
- **Detect dead roots inside each gate.** Rejected: 51 gates would each need the same
  logic, and a gate whose corpus is empty is exactly the one least able to report it.
