---
doc_class: Checklist
checklist_id: CHK-XAXIS
status: pending approval
purpose: |
  Cross-axis contract change cascade. Walked before any change that modifies a row in `docs/DESIGN.md §10` (cross-axis contract registry) or any file under `contracts/`. Extends existing `templates/checklists/cross-axis-contract-change.md` (preserved).
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

> A cross-axis contract is any API / event / schema row in `docs/DESIGN.md §10` or any file under `contracts/`. A silent cross-axis change breaks consumers. This checklist enforces explicit notification + consumer-axis approval.

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

- [ ] **X5** Notify every consumer-axis team via PR review request. *Lane:* `oya-governance-cross-axis-notify`. (Auto-emit to per-axis Slack channels via Foundry capability `pr.cross-axis.notify`.)
- [ ] **X6** Update `docs/DESIGN.md §10` registry row in the same PR. *Lane:* `oya-governance-design-contracts-mirror`.
- [ ] **X7** If breaking: author an ADR per `/templates/adr-template.md` documenting the migration path. *Lane:* `oya-governance-adr-shape`.

## Review and notification (blast-radius lenses per `docs/DESIGN.md §3.0.5.3`)

- [ ] **X8** One author-distinct reviewer agent approves the exact PR head; no human approval or reviewer quorum is required.
- [ ] **X9** **council-architecture** is notified for non-binding input on a new cross-axis contract.
- [ ] **X10** If data-class impact: **council-privacy** is notified for non-binding input.
- [ ] **X11** If regulatory impact: **ops-compliance** is notified for non-binding input.
- [ ] **X12** If security-class: **ops-security** is notified for non-binding input.

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

- Any consumer-axis team declines without an alternative path → halt; council-architecture mediates.
- Schema-migration rollback not validated → halt; route to schema-migration checklist.
- Audit-chain emission fails → halt; do not merge.

## Anti-patterns

- Marking a breaking change as `non-breaking-additive` to avoid the consumer-axis review — auto-fails on schema diff.
- Splitting a single breaking change across multiple PRs to fly under per-PR review — the cross-axis fitness lane spans PRs over a single contract over a 30-day window.
- Skipping `docs/DESIGN.md §10` update — `oya-governance-design-contracts-mirror` refuses the merge.
