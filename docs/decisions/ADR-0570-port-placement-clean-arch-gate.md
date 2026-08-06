---
id: ADR-0570
title: "Clean-arch port-placement gate (ports defined in core, not adapters)"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-22
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0132, ADR-0280, ADR-0245, ADR-0510, ADR-0515, ADR-0538, ADR-0540, ADR-0547]
amends: []
related: [ADR-0017, ADR-0512, ADR-0539, ADR-0540, ADR-0544, ADR-0547, ADR-0562]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0570: Clean-arch port-placement gate (ports defined in core, not adapters)

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — the gate is born-advisory
with a frozen baseline and can be tuned/retired by editing its policy DATA).**

## Context

The owned-stack ports/adapters doctrine (CLAUDE.md `owned_stack_policy`; ADR-0510 transient-adapter
framing; ADR-0547 kernel-purity) draws a clean-architecture seam: a **port** — the
storage/repository interface a domain depends on — is part of the cutover-stable surface and must be
DEFINED in a `core`/`ports`/`kernel` crate, while the throwaway `*-adapter` crate DEPENDS on core and
IMPLEMENTS the port. The litmus is *"would this interface change at owned-stack cutover?"*; a storage
port answers no, so it belongs in core.

**PR #116/#802** (`billing: relocate AccountingJournalStoragePort to core`) is the motivating defect:
billing's `AccountingJournalStoragePort` storage-port trait (with its port-definitional
`AccountingStoredRecord`/`AccountingStorageError` types) had been DEFINED in the in-memory adapter
crate, inverting the convention proven by tenancy's `TenantLifecycleStore` and SCIM's
`UserStore`/`GroupStore` (ports in core). It was caught only by reviewer eyes. No existing gate
catches "a port trait DEFINED in an `*/adapters/*` crate": face-direction and tier-acyclicity gates
check dependency EDGES; kernel-purity checks `*-kernel`/`*-core` transient-dep containment. Per the
founder doctrine ("impossible to ship anti-patterns through enforcement"; "construction > reaction";
friction = process failure, productize the class), the defect class must be made mechanically
unshippable, not fixed one instance at a time.

## Decision

Ship `cloud-ci-port-placement`, a born-blocking cloud-ci gate that flags a `pub trait <Name>` whose
name matches a storage/repository/port suffix heuristic and is DEFINED in a crate whose
repo-relative path contains a forbidden layer-dir segment (`adapters`).

- **HERMETIC pure-Rust predicate.** `collect_port_traits(root, policy)` enumerates workspace members
  (reusing `oya-workspace-members-kernel`), keeps those under a forbidden layer dir, and scans their
  `src/**/*.rs` for `pub trait` definitions with a conservative comment-aware line detector.
  `evaluate_keyed(policy, baseline, observed)` is pure and filesystem-free. No shell/net/clock/rand.
- **UNIVERSAL (policy-as-data).** The forbidden layer-dir segment set, the port-name suffix set
  (`StoragePort`/`Repository`/`Store`/`Repo`/`Port`), the per-(crate, trait) allowlist, and the
  member-count floor are DATA in `port-placement-policy.json`; the engine hardcodes no oyatie name.
  The suffix set was calibrated against the live corpus so behavioral adapter seams
  (`*Authorizer`/`*Backend`/`*Spawner`/`*Observer`/`*Provider`/`*Actuator`/`*Evaluator`/`*Sink`)
  carry no port suffix and never false-positive.
- **Born-advisory + enforce-no-regression.** After #116, billing is clean but the corpus still
  carries 5 pre-existing storage-port traits in adapter crates (tenant-rbac storage + workflow,
  intelligence session-store + secret-provider, kms-operator domain-repo). They are frozen in
  `port-placement-baseline.json`; the gate is advisory against them and a NEW port trait
  defined in an adapter (beyond the baseline) fails CLOSED. Relocating a baselined trait self-cleans
  (`PP-STALE-BASELINE`). The gate flips to fully blocking when the baseline reaches 0. Relocating the
  5 baselined ports is OUT OF SCOPE for this slice (each its own follow-up).
- **AUTOMATED (v1 flag-with-precise-remediation).** Each finding names the trait, the adapter crate
  that wrongly defines it, and the sibling core/ports crate it should move to. A full auto-MOVE
  codemod (relocate the trait + its port-definitional types into core, rewrite the adapter to
  depend-and-implement) is a non-trivial design act and a NOTED FOLLOW-UP, not this slice.

## Consequences

- The #116 ports-in-adapter seam-mis-draw class becomes mechanically unshippable for NEW code.
- The gate is registered in `.github/workflows/oya-ci-required.yml` and the firewall
  gate-registration meta-test, mirroring kernel-purity (ADR-0547).
- v1 is detect-only; the auto-move codemod and (optionally) emitting the baseline from the accounting
  producer are tracked follow-ups.

## Violation codes (the contract)

`PP-PORT-IN-ADAPTER` (new port trait in an adapter beyond baseline) · `PP-STALE-BASELINE` (relocated
baseline entry) · `PP-STALE-ALLOWLIST` (unused carve-out) · `PP-EMPTY-SCAN` (member floor not met,
fail-closed) · `PP-POLICY-GATE-ID-MISMATCH` · `PP-POLICY-MALFORMED`.

## Governed surfaces

This ADR OWNS and JUSTIFIES the gate crate; its verbatim tracked paths (the canonical, byte-stable
enumeration the born-accounting producer credits — the same set `register_crate` would emit) are:

```
ci/facade/port-placement/BUCK
ci/facade/port-placement/Cargo.toml
ci/facade/port-placement/OWNERS
ci/facade/port-placement/port-placement-baseline.json
ci/facade/port-placement/port-placement-policy.json
ci/facade/port-placement/src/lib.rs
ci/facade/port-placement/src/main.rs
ci/facade/port-placement/tests/port_placement.rs
```
