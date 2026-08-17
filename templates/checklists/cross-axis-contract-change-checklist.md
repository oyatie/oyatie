---
doc_class: Checklist
checklist_id: CHK-XAXIS
status: pending approval
purpose: |
  Cross-axis contract change cascade. Walked before any change that modifies a row in `docs/DESIGN.md §10` (cross-axis contract registry) or any file under `contracts/`. Extends existing `templates/checklists/cross-axis-contract-change.md` (preserved). One author-distinct reviewer agent covers the full profile union; no human or reviewer quorum is required.
lift_target: oyatie/templates/checklists/cross-axis-contract-change.md
extends: templates/checklists/cross-axis-contract-change.md
enforcing_fitness_lane: oya-governance-cross-axis-notify
owner_team: council-architecture
related:
  - docs/DESIGN.md
  - docs/adr-archive/ADR-0011-cross-microservice-contract-registry.md
  - /templates/pull-request-template.md
---

# Cross-Axis Contract Change Checklist

> A cross-axis contract is any API / event / schema row in `docs/DESIGN.md §10` or any file under `contracts/`. A silent cross-axis change breaks consumers. This checklist enforces explicit notification plus one eligible author-distinct independent-agent `APPROVE`.

## Pre-flight

- [ ] **X1** Identify which axes are **producers** of the contract (own the surface).
- [ ] **X2** Identify which axes are **consumers** of the contract (depend on the surface). *Lane:* `oya-governance-cross-axis-deps` (graph from `Cargo.toml [dependencies]` + `contracts/<surface>` references).
- [ ] **X3** Identify the change class:
  - **Breaking** (removed field, changed type, changed semantics).
  - **Non-breaking-additive** (new optional field, new event topic).
  - **Non-breaking-renaming** (rename with alias retained ≥1 wave).
  - **Non-breaking-clarification** (docs / examples only).
- [ ] **X4** Apply PR label `cross-axis-contract-change` AND sub-label `breaking | non-breaking-additive | non-breaking-renaming | non-breaking-clarification`. *Lane:* `oya-governance-cross-axis-notify`.

## Notification

- [ ] **X5** Notify every consumer-axis team for findings/context; notification is not an approval quorum. *Lane:* `oya-governance-cross-axis-notify`. (Auto-emit through the governed notification capability.)
- [ ] **X6** Update `docs/DESIGN.md §10` registry row in the same PR. *Lane:* `oya-governance-design-contracts-mirror`.
- [ ] **X7** If breaking: author an ADR per `/templates/adr-template.md` documenting the migration path. *Lane:* `oya-governance-adr-shape`.

## Approval (one reviewer agent per `docs/DESIGN.md §3.0.5.3`)

- [ ] **X8** One author-distinct independent reviewer agent issues `APPROVE` for the exact current head and applicable plan digest. Human approval and reviewer quorum are not required; any other or missing verdict blocks.
- [ ] **X9** The trusted eligibility policy covers the cross-axis + architecture profiles for the full consumer union.
- [ ] **X10** If data-class impact: the same reviewer is eligible for the privacy profile.
- [ ] **X11** If regulatory impact: the same reviewer is eligible for the compliance profile.
- [ ] **X12** If security-class: the same reviewer is eligible for the security profile. If one agent cannot cover the full profile union, halt instead of assembling a quorum.

## Implementation discipline

- [ ] **X13** Contract file (`contracts/<surface>.<openapi|proto|asyncapi>.yaml`) updated; lints clean. *Lane:* `oya-governance-contracts-lint`.
- [ ] **X14** Provider-neutral interface preserved: provider-specific code remains in `oya-<context>-adapter-<provider>-*` crates only. *Lane:* `oya-governance-provider-coupling`.
- [ ] **X15** `cargo-semver-checks` PASS on every consumer crate. *Command:* `cargo semver-checks check-release -p <crate>`.
- [ ] **X16** `cargo public-api --diff` reflects the change accurately on producer crate. *Command:* `cargo public-api --diff`.
- [ ] **X17** Schema migration (if state) ships up + down + dry-run + per-tenant + per-cell rollback. *Lane:* `oya-governance-schema-migration`.
- [ ] **X18** Feature-flag wired for breaking changes; canary rollout planned per `docs/RELEASE-MANAGEMENT.md`. *Lane:* `oya-governance-release-readiness`.

## Audit + comms

- [ ] **X19** Audit-chain `EVT-CROSS-AXIS-CONTRACT-CHANGED` emitted with producer-axis, consumer-axis list, change-class, old/new schema hashes. *Lane:* `oya-governance-audit-emission`.
- [ ] **X20** `docs/CHANGELOG.md` row appended.
- [ ] **X21** `docs/MISTAKES-LEDGER.md` row added if this change exists because a prior cross-axis change broke a consumer silently. *Lane:* `oya-governance-mistakes-ledger-cite`.

## Stop conditions

- Any unresolved consumer-axis finding or decline without an alternative path → halt; review cannot approve over an unresolved finding.
- Schema-migration rollback not validated → halt; route to schema-migration checklist.
- Audit-chain emission fails → halt; do not merge.

## Anti-patterns

- Marking a breaking change as `non-breaking-additive` to avoid the consumer-axis review — auto-fails on schema diff.
- Splitting a single breaking change across multiple PRs to fly under per-PR review — the cross-axis fitness lane spans PRs over a single contract over a 30-day window.
- Skipping `docs/DESIGN.md §10` update — `oya-governance-design-contracts-mirror` refuses the merge.
