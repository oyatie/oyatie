---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-007
title: Surface-all-failures CI workflow refactor (relax `needs:`, `if: always()`)
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_amendment_ref: "Amendment 2026-05-15 §C: Surface-all-failures CI"
purpose: Refactor `.github/workflows/pr-tests.yml` so one CI cycle surfaces ALL failures simultaneously, not one-at-a-time. Eliminates today's broken-SHA → missing-profile → missing-shebang cascade pattern.
---

# M01-P17-IP-007 — Surface-all-failures CI workflow refactor

## Scope

New constraint from 2026-05-15 amendment §C. PR #3's three-cycle cascade (broken action SHA → missing nextest profile → missing shebang) is the canonical anti-pattern. With `needs:` chains gating job execution and `fail-fast: true` defaults, each CI cycle exposes only one issue. A mature pipeline shows ALL issues in cycle 1 so the fix-loop agent (IP-005) and merge-queue (IP-006) consume one round-trip per fix wave, not per fix.

This IP refactors workflows to:

- Relax `needs:` chains: `cargo-nextest` no longer hard-needs `cargo-check`. Each gate runs independently and reports independently.
- Add `if: always()` on dependent jobs that previously short-circuited on upstream failure.
- Keep `continue-on-error: false` (the default) so each job still FAILS the workflow if it fails — but `if: always()` ensures every gate still RUNS.
- Set `strategy.fail-fast: false` on all matrix jobs (e.g. multi-toolchain matrix).
- Confirm `[profile.ci].fail-fast = false` in `.config/nextest.toml` (already set per audit Stage 6).
- Add a fitness lane `oya-foundry-fitness-workflow-surface-coverage-kernel` that BLOCKS any workflow file where a `needs:` chain would suppress another gate's signal.

## Dependencies

None at planning level — Wave-1 IP, can ship in parallel with IP-001. (Implementation agent is already executing this in parallel with the planning commit.)

## Acceptance

- `.github/workflows/pr-tests.yml` has no `needs:` chains gating fitness-class jobs against build-class jobs.
- All matrix jobs declare `strategy.fail-fast: false`.
- A test PR with deliberately seeded fmt-failure + clippy-failure + nextest-failure surfaces all three failures in cycle 1 (not three sequential cycles).
- New fitness lane `oya-foundry-fitness-workflow-surface-coverage-kernel` BLOCKS a deliberate regression PR that re-adds a suppressive `needs:` chain.
- Evidence at `/evidence/pipeline-maturity-glue/ip-007-surface-all-failures.json` includes a triple-failure CI run with all three failures reported in cycle 1.

## Symbols to grit-claim

- `.github/workflows/pr-tests.yml::*` (refactor `needs:` + add `if: always()`)
- `.config/nextest.toml::profile.ci` (verify `fail-fast = false`, no change expected)
- `crates/oya-foundry-fitness-workflow-surface-coverage-kernel/src/lib.rs::evaluate`
- `tools/oya-foundry-fitness-workflow-surface-coverage-app/src/main.rs::main`
- `registry/fitness-lane-registry.json::workflow-surface-coverage`

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-007-surface-all-failures.json`
- `/evidence/pipeline-maturity-glue/ip-007-triple-failure-cycle-1.json` (the deliberate triple-failure test PR run)

## Concurrent-execution note

Implementation agent is editing `.github/workflows/pr-tests.yml` in parallel with this plan commit. Plan file documents the contract; implementation lands the YAML. Commit ordering: this plan file may land before or after the workflow edits — both orderings are valid since this plan file is intent-only and the workflow edit is the realization.
