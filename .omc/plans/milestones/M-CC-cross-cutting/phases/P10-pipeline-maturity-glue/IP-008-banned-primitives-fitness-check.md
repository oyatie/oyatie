---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P10-IP-008
title: Banned-primitives fitness check (catches grit/git bypass; auto-flips when grit retires)
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #5: grit registry hangs + agents silently bypassing"
claude_md_ref: "CLAUDE.md::sanctioned_primitives + sunset_note"
purpose: Fail-fast fitness lane that BLOCKS any PR whose agent command-log contains `git`/`gh` invocations outside the sanctioned-primitives whitelist. Auto-flips its allow-list when grit/oya-tooling-agent-read sunset per the `CLAUDE.md::sunset_note`.
---

# M-CC-P10-IP-008 — Banned-primitives fitness check

## Scope

`CLAUDE.md` declares `grit` and `oya-tooling-agent-read` as the only sanctioned primitives, with a `sunset_note` that says both retire on Oya VCS + Foundry go-live. Today's session shows agents using plain `git` (`git mv`, `git commit`, `gh pr create`) — the contract is broken in reality, not just on paper. This IP closes the credibility hole:

- `crates/oya-foundry-fitness-banned-primitives-kernel` reads a per-PR command-log (sourced from CI agent-trace files or `.audit/agent-read.jsonl`) and emits Violations for any `git`/`gh` invocation outside an explicit allow-list.
- Allow-list is config-driven via `registry/sanctioned-primitives.json`, which mirrors `CLAUDE.md::sanctioned_primitives` and is itself CI-verified to match the source-of-truth.
- **Auto-flip on sunset**: when `CLAUDE.md::sunset_note` flips (Oya VCS + Foundry live), the registry's `grit` and `oya-tooling-agent-read` entries auto-deprecate and the lane fail-builds on continued use. This is the lifecycle-automation contract from `feedback_lifecycle_automation_universal.md`.
- Wave-1-baseline-zero: the lane is initially WARN-only to establish a clean baseline, then ratchets to BLOCK once the existing offenders are remediated (per the wave-ratcheted pattern in `feedback_lifecycle_automation_universal.md`).

## Dependencies

- IP-002 (`oya` CLI) — the lane registers into `oya gate run-all`.
- IP-003 (mistakes-ledger) — repeat offenders generate ledger rows.

## Acceptance

- `crates/oya-foundry-fitness-banned-primitives-kernel/src/lib.rs::evaluate` exists with `(command_log, allow_list, sunset_state) -> Vec<Violation>` signature.
- `tools/oya-foundry-fitness-banned-primitives-app` exists as binary surface.
- Lane registered in `registry/fitness-lane-registry.json` and visible to `oya gate run-all`.
- A test PR with a `git commit` step in its workflow is flagged WARN at baseline-zero, then BLOCKED after ratchet.
- A test of the sunset auto-flip: setting `CLAUDE.md::sunset_note: active` removes `grit` from the allow-list automatically; lane fails on `grit claim` invocations.
- Evidence at `/evidence/pipeline-maturity-glue/ip-008-banned-primitives.json`.

## Symbols to grit-claim

- `crates/oya-foundry-fitness-banned-primitives-kernel/Cargo.toml::package`
- `crates/oya-foundry-fitness-banned-primitives-kernel/src/lib.rs::evaluate`
- `crates/oya-foundry-fitness-banned-primitives-kernel/src/sunset_state.rs::SunsetState`
- `tools/oya-foundry-fitness-banned-primitives-app/src/main.rs::main`
- `registry/sanctioned-primitives.json::*` (new; mirrors `CLAUDE.md::sanctioned_primitives`)
- `registry/fitness-lane-registry.json::banned-primitives` (extend)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-008-banned-primitives.json`
- `/evidence/pipeline-maturity-glue/ip-008-sunset-auto-flip.json` (the sunset-state regression test)
