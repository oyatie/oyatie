---
purpose: Auto-backfilled purpose for IP-003-eng-excellence-merge-gate.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P07-IP-003
title: Engineering Excellence Council merge gate
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Council-Architecture signs every cross-axis-contract PR; merge-gate enforces signature.
---

# M-CC-P07-IP-003 — Engineering Excellence Council merge gate

## Purpose
Council-Architecture signs every cross-axis-contract PR; merge-gate enforces signature.

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-pr-merge-gate-kernel/src/lib.rs::CouncilSignature
crates/oya-foundry-fitness-pr-merge-gate-kernel/src/lib.rs::parse_council_signature
crates/oya-foundry-fitness-pr-merge-gate-kernel/src/lib.rs::evaluate
```
(Migrated 2026-05-14 from `scripts/hooks/guard-pr-merge-review.mjs` per user directive "no shellscript no mjs etc all rust" — original .mjs deleted; semantics preserved 1:1 in pure Rust.)
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged (except for IPs IN M-CC-P01 itself).

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M-CC-P07-IP-003 Engineering Excellence Council merge gate shipped; acceptance commands green' -i high -k 'M-CC-P07-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `APPROVED_VERDICTS` is a single `Set` — adding/removing accepted verdicts (e.g., `APPROVE_WITH_NITS`) is one literal change, not scattered conditionals.
- The block reader terminates explicitly at the next `## ` heading, so a downstream "Architect:" mention in a different section can't satisfy the gate by accident.
- Bulleted (`- Architect:`) and plain (`Architect:`) lines are both accepted via one regex — reviewers don't have to remember which style passes.
- Case-insensitive matching at header + verdict — gate doesn't second-guess casing conventions.
- Self-test (`--self-test`) ships in the same file as the implementation; CI runs the gate's own test suite before relying on it.
- Pure node18+ stdlib — no npm install, no supply-chain surface from a dep.
