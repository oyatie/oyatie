---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-008
title: Banned-primitives fitness check
status: reconciled-to-M01-P17-IP-008.1
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
blocker_ref: F-FORBIDDEN-PRIMITIVES-CI-GUARD
successor_plan: ../P-fitness-chained-enforcement/IP-001-banned-primitives-lane.md
purpose: Preserve the P17 audit lineage while the active implementation contract lives in M01-P17-IP-008.1.
---

# M01-P17-IP-008 — Banned-primitives fitness check

## Current contract

The active lane is `M01-P17-IP-008.1`. This P17 record is the historical audit entry for the same blocker and must not keep stale API or registry names alive.

Current implementation shape:

- kernel crate: `crates/oya-governance-banned-primitives-kernel`
- direct runner: `tools/oya-governance-banned-primitives-app`
- gate runner: `cargo run -q -p oya-dev-cli -- gate validate banned-primitives --require-command-log-corpus --command-log-root registry/governance-corpora/banned-primitives`
- workflow context: `oya-governance-banned-primitives`
- quality lane: `registry/quality/lanes.yaml::oya-governance-banned-primitives`
- branch-protection row: `.github/branch-protection.yaml::branches.dev.required_status_checks`

## Acceptance

- `check_documented_genuine_need(...)` remains the kernel enforcement function.
- `scan_agent_instruction_file(...)` parses fenced `agent-instructions` source blocks and emits typed primitive usage records.
- `scan_command_invocation(...)` parses sanitized command-log records and emits typed primitive usage records.
- `oya gate validate banned-primitives --require-command-log-corpus --command-log-root registry/governance-corpora/banned-primitives` is part of `oya gate run-all`.
- `quality-lanes`, `protection-context-match`, `aspirational-enforcement`, and `planning-closure` remain green after the lane is registered.

## Exit evidence

- `/evidence/fd001/cs-fitness-001-banned-primitives.json`
- `/evidence/multispectrum/cs-fitness-001-banned-primitives-1779160471.json`
- `/evidence/fd001/cs-fitness-001-command-log-corpus.json`
- `/evidence/multispectrum/cs-fitness-001-command-log-corpus-1779163756.json`
