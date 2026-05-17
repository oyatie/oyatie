---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-003
title: Mistakes-ledger 5-control stack (preflight runbook + template + fitness lane + ICM hook + citation probe)
status: scaffolded
tier: M
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #3: mistakes-ledger lane does not exist (0/5 controls on disk)"
memory_ref: feedback_repeat_mistake_prevention
purpose: Ship the 5-control mistakes-ledger stack from `feedback_repeat_mistake_prevention.md` — currently 0/5 on disk despite being a memory directive. Closes Stage 10 of the pipeline-maturity matrix.
---

# M01-P17-IP-003 — Mistakes-ledger 5-control stack

## Scope

Memory directive `feedback_repeat_mistake_prevention.md` mandates 5 permanent controls on second-occurrence errors: (1) preflight runbook, (2) ledger row template, (3) fitness lane, (4) ICM context-store hook, (5) citation-probe check. Reality on disk today: zero of five. Today's session paid 3 commit cycles for 3 CI-infrastructure regressions (broken action SHA, missing nextest profile, missing shebang) — exactly the repeat-class signature this stack exists to prevent. This IP lands all five controls plus integration into `oya gate run-all` (from IP-002).

## Dependencies

- IP-002 (`oya gate run-all`) — the fitness lane must register into the gate aggregator.

## Acceptance

- `docs/runbooks/sanctioned-primitives/preflight.md` exists, lists every agent-callable command, references the ledger template, and is referenced from `docs/AGENTS.md`.
- `docs/templates/mistakes-ledger-row-template.md` exists with strict fields: `date`, `mistake-class`, `first-occurrence-evidence`, `second-occurrence-evidence`, `5-control-ids`, `prevention-evidence`.
- `crates/oya-foundry-fitness-mistakes-ledger-kernel` exists with `(ledger_rows, current_pr_diff, current_command_log) -> Vec<Violation>` signature.
- `tools/oya-foundry-fitness-mistakes-ledger-app` exists as the binary surface.
- ICM hook: a small `tools/oya-icm-mistakes-ledger-hook` watches for PR-close events and posts a ledger-row template if the PR had ≥2 CI cycles.
- Citation-probe: extends an existing citation lane to require `mistakes-ledger:<row-id>` citation on any commit that fixes a repeat-class failure.
- All 5 controls are registered in `registry/fitness-lane-registry.json` and visible from `oya gate run-all`.
- Backfill: today's 3 cascading CI-infrastructure regressions get retroactive ledger rows (commit SHA + class + control bindings).
- Evidence at `/evidence/pipeline-maturity-glue/ip-003-mistakes-ledger.json`.

## Symbols to grit-claim

- `docs/runbooks/sanctioned-primitives/preflight.md::*`
- `docs/templates/mistakes-ledger-row-template.md::*`
- `crates/oya-foundry-fitness-mistakes-ledger-kernel/src/lib.rs::evaluate`
- `tools/oya-foundry-fitness-mistakes-ledger-app/src/main.rs::main`
- `tools/oya-icm-mistakes-ledger-hook/src/main.rs::main`
- `registry/fitness-lane-registry.json::mistakes-ledger`
- `registry/mistakes-ledger.json::*` (new — the actual ledger data store)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-003-mistakes-ledger.json`
- `/evidence/pipeline-maturity-glue/ip-003-backfill-rows.json` (the 3 today-session retroactive rows)
