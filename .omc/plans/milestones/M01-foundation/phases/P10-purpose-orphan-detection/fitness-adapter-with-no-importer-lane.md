---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P10-IP-FITNESS-ADAPTER-WITH-NO-IMPORTER
title: Fitness lane — adapter-with-no-importer (ADR-0104 audit-#7 recurrence prevention)
status: scaffolded
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - crates/oya-governance-adapter-with-no-importer-kernel
  - tools/oya-governance-adapter-with-no-importer
adr_anchor: docs/decisions/ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
audit_anchor: 2026-05-15 audit finding #7 (14 placeholder-shell adapter crates)
naming_justification:
  oya-governance-adapter-with-no-importer-kernel: |
    v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:adapter-with-no-importer>-<layer:kernel>`;
    12-layer-enum suffix `kernel` (port-in-kernel, I/O-free check function per ADR-0056).
  oya-governance-adapter-with-no-importer: |
    v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:adapter-with-no-importer>`;
    dev-CLI surface (no layer suffix; binary tool wrapping the kernel for `oya gate validate`).
purpose: Detect any `*-adapter` crate in the workspace that has no `*-importer-*` consumer and fail the gate so audit-#7 cannot recur silently.
---

# M01-P10-IP-FITNESS-ADAPTER-WITH-NO-IMPORTER — Fitness lane: adapter-with-no-importer

## Purpose
Per ADR-0104 Consequences §4 ("PR template gains a new check: any new `*-adapter-*` crate must declare a consumer in the same PR. Mechanical-prevention candidate: extend …with an 'adapter-with-no-importer' check.") and Follow-up #4 ("Author `oya-governance-adapter-with-no-importer` lane as a mechanical-prevention for premature adapter creation"). The lane scans the workspace for adapter crates that have no matching importer and fails the gate when any is found. This makes the ADR-0104 ecosystem-expansion rule (no adapter without a consumer) mechanically-enforced rather than process-enforced (DP-03).

## Naming justification (per [[feedback_naming_justification]])
- `oya-governance-adapter-with-no-importer-kernel` — v4 BNF compliant: product=`foundry`, facet=`fitness`, topic=`adapter-with-no-importer`, layer=`kernel`. The kernel is I/O-free per ADR-0056 §"port-in-kernel" — runners do the directory walk + manifest parse, the kernel does the pure check.
- `oya-governance-adapter-with-no-importer` (dev-CLI) — v4 BNF compliant: product=`foundry`, facet=`fitness`, topic=`adapter-with-no-importer`, no layer suffix (binary tool surface). Mirrors the `oya-governance-portfolio-citation` pattern landed in ICM ip004.

## Symbols-to-grit-claim
```
crates/oya-governance-adapter-with-no-importer-kernel/src/lib.rs::check
crates/oya-governance-adapter-with-no-importer-kernel/src/lib.rs::Violation
crates/oya-governance-adapter-with-no-importer-kernel/src/lib.rs::AdapterImporterReport
tools/oya-governance-adapter-with-no-importer/src/main.rs::run
```
Scaffold-claim via ICM `scaffold-locks-oyatie` per ADR-0054 — window opened in this PR's scaffold.

## Agent-prerequisites
ADR-0104 read; ADR-0054 read; M01-P10 INDEX read; user memory `feedback_naming_justification.md` + `feedback_clean_architecture_requirements.md` honored.

## Algorithm (kernel)
1. Walk workspace; load each `[package].name`.
2. Mark crates whose final segment is `adapter`, or whose name matches `<base>-adapter-<single-token-non-layer-variant>` (e.g. `…-adapter-aws`, `…-adapter-inmemory`). Exclude port-declaring siblings (`…-adapter-kernel`, `…-adapter-domain`).
3. For each adapter crate, derive `<base>` and search the workspace for any crate matching `<base>-importer*`.
4. Emit a `Violation` for each adapter with no matching importer; emit hint citing ADR-0104.

## Acceptance-test-commands
```
cargo test -p oya-governance-adapter-with-no-importer-kernel
cargo test -p oya-governance-adapter-with-no-importer
cargo run -q -p oya-governance-adapter-with-no-importer -- --root crates --root tools
```

## Done-criteria
- Kernel + dev-CLI tests green.
- `cargo check --workspace` green.
- Lane runs against the live workspace and reports a non-empty violation set (audit-#7 evidence): the 29 baseline violations are the WARN ledger.
- ADR-0104 cited from kernel doc-comment header and from this plan's frontmatter.

## Ratchet plan (WARN → BLOCK)
- **Wave A (now, this PR):** lane runs but is non-blocking; CI captures violation count as the baseline.
- **Wave B (next PR cluster, M01-P10 follow-up):** lane blocks any NEW adapter crate without an importer (delta-gate against baseline) while allowing the existing 29.
- **Wave C (after audit-#7 cleanup):** lane is full BLOCKER; baseline drops to zero either via importer authoring (per ADR-0104 ecosystem-expansion rule: ship the consumer in the same PR) or via crate sunset per ADR-0056 + the no-silent-regression doctrine ([[feedback_no_silent_regression]]).

## Rollback-procedure
`grit done` is atomic per-symbol. The lane is purely additive — reverting the merge commit removes the two new crates from `[workspace.members]` and the dev-CLI binary; no other crate depends on either.

## ICM coordination
Scaffold-lock window OPEN/CLOSE pair logged in `scaffold-locks-oyatie` (per ADR-0054), tagged `oya-governance-adapter-with-no-importer,ADR-0104`.

## Next-IP-pointer
Wave B (delta-gate against baseline) lands as a follow-up IP in the same phase index.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: the audit-#7 "name-only, behavior-zero" failure mode is mechanically detected on every PR; no more silent adapter-shell creation.
