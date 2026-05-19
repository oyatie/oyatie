---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P-FITNESS-IP-001
title: Banned-primitives lane
status: active-implementation
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
blocker_ref: F-FORBIDDEN-PRIMITIVES-CI-GUARD
purpose: Ship the real `oya-foundry-fitness-banned-primitives` detector, gate dispatch, quality-lane registration, workflow context, branch-protection row, source-corpus contract, and evidence before product fanout relies on forbidden-primitive claims.
---

# M01-P-FITNESS-IP-001 — Banned-primitives lane

## Scope

- Use `crates/oya-foundry-fitness-banned-primitives-kernel` as the pure detector for fenced `agent-instructions` source blocks.
- Use `tools/oya-foundry-fitness-banned-primitives-app` as the direct runner.
- Expose the merge-blocking path through `cargo run -q -p oya-dev-cli -- gate validate banned-primitives`.
- Register `oya-foundry-fitness-banned-primitives` in `registry/quality/lanes.yaml`, `docs/standards/ci-lanes.md`, `.github/workflows/`, `.github/branch-protection.yaml`, and the `oya gate run-all` catalog.
- Advance `F-FORBIDDEN-PRIMITIVES-CI-GUARD` only to source-corpus coverage with local gate evidence showing the detector, workflow-context mirror, branch-protection context, quality-lane mirror, and fenced source corpus are green; full closure still requires session/tool-call command-log corpus enforcement.

## Acceptance

- `oya gate validate banned-primitives` fails closed when `manual-branch`, `manual-rebase`, `manual-merge`, `manual-push`, hook bypass, force push, forge merge, external fetch, process kill, or home-directory mutation appears inside an agent instruction fence.
- Sanctioned `oya-git` / `oya git <git-subcommand>` text does not self-trigger as direct `git`.
- `oya gate validate quality-lanes`, `oya gate validate protection-context-match`, and `oya gate validate aspirational-enforcement` pass with the new lane.
- Evidence is emitted under `/evidence/fd001/` and `/evidence/multispectrum/`.

## Exit evidence

- `/evidence/fd001/cs-fitness-001-banned-primitives.json`
- `/evidence/multispectrum/cs-fitness-001-banned-primitives-1779160471.json`
