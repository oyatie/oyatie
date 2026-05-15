---
purpose: Auto-backfilled purpose for IP-010-parallel-claim-demo.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P01-IP-010
title: Parallel-claim demo runbook (P8)
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Reproducible parallel-claim demo on pinned symbols.
---

# M-CC-P01-IP-010 — Parallel-claim demo runbook (P8)

## Purpose
Reproducible parallel-claim demo on pinned symbols.

## Symbols-to-grit-claim
```
docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md::Procedure
docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh::Script
```
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
icm store -t context-oyatie -c 'M-CC-P01-IP-010 Parallel-claim demo runbook (P8) shipped; acceptance commands green' -i high -k 'M-CC-P01-IP-010,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: the runbook records actual session-less `grit claim` behavior instead of assuming broad parallel expansion; the demo uses the current filesystem path `crates/oya-cloud-billing-application/src/lib.rs` while documenting the legacy planning alias.
