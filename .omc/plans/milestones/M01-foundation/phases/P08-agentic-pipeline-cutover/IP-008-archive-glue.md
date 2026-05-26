---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P08-IP-008
title: Retired archive orchestration glue + archive-orphan lane (P6)
status: retired
migration_status: retired
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Historical record for the one-time archive-orphan lane; ADR-0118 removes the archive payload and retired lane crates after ADR-0116 promoted M01-P18 as the canonical VCS substrate.
---

# M01-P08-IP-008 — Retired archive orchestration glue + archive-orphan lane (P6)

## Purpose
Record IP-008 as retired: the one-time archive payload and executable archive-orphan lane are removed, while historical traceability stays in ADR-0118 and evidence metadata.

## Retired-symbols
```
bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/::RetiredArchiveDir
crates/oya-foundry-fitness-archive-orphan-kernel::RetiredKernel
tools/oya-foundry-fitness-archive-orphan-app::RetiredApp
```
Naming justification: `RetiredArchiveDir`, `RetiredKernel`, and `RetiredApp` are ledger-only names that preserve IP-008 traceability without keeping executable workspace members.

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

## Retirement-validation-commands
```
git diff --name-status origin/dev...HEAD
~/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo metadata --no-deps --format-version 1
~/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p oya-governance-authoritative-tracked-kernel
node scripts/validate-adr-shape.mjs docs/decisions/ADR-0118-retire-archive-orphan-fitness-lane.md
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
Revert the retirement PR with plain git if downstream history proves the one-time archive payload is still required. Do not recreate grit/rtk/icm/vox coordination flows; ADR-0116 keeps M01-P18 as the canonical substrate.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Retirement-note
The old archive-orphan executable lane is removed rather than renamed because its invariant was one-time cutover hygiene. Continuing to carry a runner would duplicate M01-P18 admission/projected-merge-state checks after ADR-0116.

## Decision-log (Linus good-taste row)
Special cases eliminated by this retirement: a one-time archive validator no longer masquerades as reusable VCS infrastructure after M01-P18 became canonical.
