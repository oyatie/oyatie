---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-008.1
title: Banned-primitives lane
status: active-implementation
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
blocker_ref: F-FORBIDDEN-PRIMITIVES-CI-GUARD
purpose: Ship the real `oya-governance-banned-primitives` detector, gate dispatch, quality-lane registration, workflow context, branch-protection row, source-corpus contract, sanitized command-log corpus contract, and evidence before product fanout relies on forbidden-primitive claims.
---

# M01-P17-IP-008.1 — Banned-primitives lane

## Scope

- Use `crates/oya-governance-banned-primitives-kernel` as the pure detector for fenced `agent-instructions` source blocks and typed command invocations.
- Use `tools/oya-governance-banned-primitives-app` as the direct runner.
- Expose the merge-blocking path through `cargo run -q -p oya-dev-cli -- gate validate banned-primitives --require-command-log-corpus --command-log-root registry/fitness-corpora/banned-primitives`.
- Register `oya-governance-banned-primitives` in `registry/quality/lanes.yaml`, `docs/standards/ci-lanes.md`, `.github/workflows/`, `.github/branch-protection.yaml`, and the `oya gate run-all` catalog.
- Advance `F-FORBIDDEN-PRIMITIVES-CI-GUARD` to source-corpus plus sanitized tracked command-log corpus coverage with local gate evidence showing the detector, workflow-context mirror, branch-protection context, quality-lane mirror, fenced source corpus, and redacted JSONL command corpus are green; hosted branch-protection deployment remains a GitOps verification gap.

## Acceptance

- `oya gate validate banned-primitives --require-command-log-corpus --command-log-root registry/fitness-corpora/banned-primitives` fails closed when `manual-branch`, `manual-rebase`, `manual-merge`, `manual-push`, hook bypass, force push, forge merge, external fetch, process kill, or home-directory mutation appears inside an agent instruction fence or sanitized command-log corpus.
- Sanctioned `oya-git` / `oya git <git-subcommand>` text does not self-trigger as direct `git`.
- Sanitized command-log records must be JSONL, must set `redacted=true`, and must not record direct `git` / `gh` surfaces.
- `oya gate validate quality-lanes`, `oya gate validate protection-context-match`, and `oya gate validate aspirational-enforcement` pass with the new lane.
- Evidence is emitted under `/evidence/fd001/` and `/evidence/multispectrum/`.

## Exit evidence

- `/evidence/fd001/cs-fitness-001-banned-primitives.json`
- `/evidence/multispectrum/cs-fitness-001-banned-primitives-1779160471.json`
- `/evidence/fd001/cs-fitness-001-command-log-corpus.json`
- `/evidence/multispectrum/cs-fitness-001-command-log-corpus-1779163756.json`
