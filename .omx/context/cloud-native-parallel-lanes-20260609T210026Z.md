# Context: cloud-native parallel lanes

## Task statement
Run a durable OMX team with 5 workers to progress HANDOFF.md backlog toward cloud/Kubernetes-native authority, no local CLI merge authority, and safe dependency-aware parallelization.

## Desired outcome
Five owned lanes produce small, verified, reviewable changes or precise blockers:
1. IAM/IDP lane under cloud/cloud-iam/**
2. KMS lane under cloud/cloud-kms/** plus infra/kms/** if needed
3. IaC lane under cloud/cloud-iac/** and microservices/cloud-iac/** only when consuming frozen IAM/KMS interfaces
4. Storage lane under cloud/cloud-storage/** plus infra/seaweedfs/** for SeaweedFS/current-object-storage direction; no MinIO/RustFS final-canonical claims
5. Hooks/policy lane under tools/hooks/**, scripts/hooks/**, docs/checklists/**, .codex/hooks.json, and branch-protection/CI authority docs when needed

## Known evidence
- Commit 1e970563a retired tracked .omc/.omx authority and aligned planning/hyperscaler gates to cloud-native oya-ci-required bridge, future Kubernetes-native controller authority.
- HANDOFF.md says current stack direction is owned cloud-native fabric; W5 moves to bespoke distributed-SQL + object-store; W0 has hermetic Buck2 + firewall/cloud-ci stores.
- SeaweedFS is current canonical object-storage substrate per ADR-0196 and infra/seaweedfs; ADR-0520/0521 transition to owned object store later. RustFS has no repo refs. MinIO refs are stale/benchmark/forbidden or non-final context.
- IAM/KMS/IaC are not fully independent: IAM and KMS provide primitives; IaC consumes frozen interfaces.
- Buck2 is canonical long-term build surface; Cargo remains in current CI gates and Rust verification until migrated into cloud-ci apps. Cargo should not be removed from required CI without atomic gate migration.
- Dev CLI/local oya/oya-dev-cli are retired as merge authority; local hook uses are shift-left only and should be replaced or clearly marked non-authoritative where possible.

## Constraints
- Use OMX team runtime; do not replace with native subagent fanout.
- Workers may create nested subagents/goals only inside their owned slices/worktrees and must not mutate leader goal state.
- Keep changes small, incremental, and test/spec-driven.
- No new dependencies.
- No external production/cloud mutation.
- No generated JSON add/modify surfaces unless required and verified by materialization policy.
- Do not alter shared contracts/specs from multiple lanes concurrently. If a lane needs shared specs/registry/HANDOFF edits, report to leader and stop before editing.

## Skill process to apply
- using-agent-skills: choose applicable workflow per slice.
- spec-driven-development: write/confirm spec/acceptance before code.
- test-driven-development: RED/GREEN/REFACTOR for behavior changes; docs-only uses grep/static checks.
- incremental-implementation: thin vertical slices, working state after each slice.
- security-and-hardening: default-deny, no secret/key material, no fail-open, least privilege.
- code-review-and-quality + code-review: self-review diff, list risks and evidence.
- code-simplification + ai-slop-cleaner: remove stale authority/slop rather than add layers.
- shipping-and-launch: report launch/merge readiness evidence, rollback/remaining risks.

## Stop condition
Team reaches terminal state with each lane either verified complete with evidence or blocked with exact file/scope blocker; leader reviews and integrates/commits only after checks pass.
