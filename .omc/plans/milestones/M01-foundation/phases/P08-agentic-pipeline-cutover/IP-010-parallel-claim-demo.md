---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P08-IP-010
title: Parallel-claim demo runbook (P8)
status: complete
migration_status: cleanup
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Reproducible parallel-claim demo on pinned symbols.
---

# M01-P08-IP-010 — Parallel-claim demo runbook (P8)

## Acceptance Criteria

- **AC-001**: The parallel-claim demo runbook file exists at `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` and the companion script at `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh`.
  - test_id: `test -f docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md && test -f docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh`
  - verification_command: `test -f docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md`
- **AC-002**: The runbook script executes without errors against the pinned symbols listed in the Symbols-to-grit-claim section.
  - test_id: `bash docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh --dry-run`
  - verification_command: `bash docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh --dry-run`
  - status: pending-spec-author
- **AC-003**: Runbook freshness gate passes (runbook indexed and not stale).
  - test_id: `oya gate validate runbook-freshness`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate runbook-freshness`
- **AC-004**: Cohesion fitness lane passes for any crate touched by this IP.
  - test_id: `oya gate validate cohesion`
  - verification_command: `cargo run -p oya-dev-cli -- gate validate cohesion`

## Purpose
Reproducible parallel-claim demo on pinned symbols.

## Symbols-to-grit-claim
```
docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md::Procedure
docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh::Script
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-governance-cohesion -- <owning-crate-glob>
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
icm store -t context-oyatie -c 'M01-P08-IP-010 Parallel-claim demo runbook (P8) shipped; acceptance commands green' -i high -k 'M01-P08-IP-010,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: the runbook records actual session-less `grit claim` behavior instead of assuming broad parallel expansion; the demo uses the current filesystem path `crates/oya-cloud-billing-application/src/lib.rs` while documenting the legacy planning alias.
