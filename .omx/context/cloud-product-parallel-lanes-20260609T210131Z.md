# Context: cloud + product parallel lanes

## Task statement
Run durable OMX Team execution for HANDOFF.md backlog with cloud/Kubernetes-native authority, all cloud surfaces, B2B/ERP/enterprise/business/SaaS product surfaces, and the unified frontend shell.

## Desired outcome
Make small, verified, reviewable progress across independent owned lanes. Workers may create nested subagents/goals inside their own lane/worktree when it improves throughput, but must keep ownership boundaries and must not mutate the leader's active goal state.

## Evidence and direction
- Current branch has committed `1e970563a cleanup: retire tracked local durable authority`.
- User direction: no CLI authority; stay cloud/Kubernetes native. Local CLI/dev-cli/oya-dev-cli are retired as merge authority.
- Merge authority is cloud CI `oya-ci-required` bridge and future Kubernetes-native controller/status authority. Local hooks may shift-left but must not claim merge authority.
- Buck2 is canonical build direction; Cargo remains present in current CI until gates migrate into cloud-ci app/fan-in. Do not remove Cargo lanes without atomic migration.
- SeaweedFS is current object-storage substrate; RustFS has no repo refs; MinIO refs are non-final/benchmark/forbidden/stale context. ADR-0520/0521 move later to owned DB + object store behind stable interfaces.
- IAM/KMS are foundational; IaC consumes frozen IAM/KMS interfaces. Do not concurrently edit shared contracts/specs from multiple lanes.
- App/product surfaces found: `oya/app-shell-frontend`, `oya/office`, `oya/workplace-integration`, `docs/products/erp-coverage`, `docs/runbooks/saas`, `docs/teams/axis-saas`, B2B user journeys, specs masterplan SaaS/workspace/application-B2B entries.

## Skill process each worker must apply
- using-agent-skills: choose applicable skills before work.
- spec-driven-development: state lane spec/acceptance in first worker update.
- test-driven-development: behavior changes require RED/GREEN/REFACTOR; docs/static changes require grep/schema/static validation.
- incremental-implementation: thin slices only; no broad rewrites.
- security-and-hardening: default-deny, least privilege, no secrets/key material, no fail-open.
- code-review-and-quality + code-review: self-review and peer-impact/risk report.
- code-simplification + ai-slop-cleaner: delete stale authority/slop before adding abstractions.
- shipping-and-launch: final evidence, rollback note, remaining risk.

## Lane ownership
Worker 1 — Cloud identity/IDP/IAM:
- Owns `cloud/cloud-iam/**` and IAM/IDP product docs only.
- Align stale CLI/Jenkins authority to cloud/Kubernetes-native authority.
- Do not edit shared specs/registry/HANDOFF without leader handoff.

Worker 2 — Cloud KMS/secrets/security substrate:
- Owns `cloud/cloud-kms/**`, `infra/kms/**`, and KMS/security notes.
- Keep OpenBao/KMS Kubernetes-native; no local CLI authority; no key material/secrets in evidence.
- Coordinate if touching `cloud/cloud-secrets/**`.

Worker 3 — Cloud IaC/control plane/GitOps:
- Owns `cloud/cloud-iac/**`, `microservices/cloud-iac/**` if present, IaC module catalogs, and Argo/Kubernetes-native actuation docs.
- Consume frozen IAM/KMS interfaces; do not redefine them.
- Prefer declarative controller/operator/OpenTofu/Argo contracts over CLI instructions.

Worker 4 — Cloud data/storage/object substrate:
- Owns `cloud/cloud-data/**`, `cloud/cloud-storage/**`, `infra/seaweedfs/**`, object-store direction docs.
- Make SeaweedFS/current + owned-object-store-later story precise; remove/mark stale MinIO final-canonical claims.
- No RustFS claim unless repo evidence added by leader.

Worker 5 — Cloud compute/K8s/kernel/OS/capacity/network:
- Owns `cloud/cloud-compute/**`, `cloud/cloud-k8s/**`, `cloud/cloud-kernel/**`, `cloud/cloud-os/**`, `cloud/cloud-capacity/**`, `cloud/cloud-cell/**`, `cloud/cloud-network*` if present, and related Kubernetes-native surfaces.
- Focus on Kubernetes-native declarative control plane and non-CLI operations.

Worker 6 — Cloud CI/fabric/toolchain/hooks:
- Owns `cloud/cloud-ci/**`, `.github/workflows/oya-ci-required.yml`, `.github/branch-protection.yaml`, `infra/branch-protection/**`, `tools/hooks/**`, `scripts/hooks/**`, `docs/checklists/**`, `.codex/hooks.json`.
- Ensure dev-cli/local CLI retirement and hooks behavior align: cloud CI is merge authority; hooks are shift-left only. If changing required CI composition, update branch protection/gates atomically.

Worker 7 — B2B/ERP/enterprise/business domain:
- Owns `docs/products/erp-coverage/**`, B2B/business user journeys, enterprise/accounting/hr/tenant-rbac docs/registry rows where explicitly scoped, and product naming decision notes.
- Pick and document canonical label if possible (e.g. business platform / enterprise SaaS suite), or report options with evidence.

Worker 8 — SaaS/workflow/marketplace/product platform:
- Owns `docs/runbooks/saas/**`, `docs/teams/axis-saas/**`, SaaS platform specs/docs, workflow/plugin/marketplace surfaces under `cloud/` or `oya/` only with no overlap.
- Ensure SaaS platform is built atop cloud control plane, not separate CLI flows.

Worker 9 — Unified frontend shell / app shell / office/workplace UX:
- Owns `oya/app-shell-frontend/**`, `oya/office/**`, `oya/workplace-integration/**`, UX shell manifests/docs.
- Align unified frontend shell for business/SaaS/cloud surfaces; avoid backend contract edits unless leader-approved.

Worker 10 — Integration, code review, security, simplification, shipping:
- Owns no product implementation paths initially. Owns team-level integration review, conflict watch, security/hardening checklist, anti-slop/code-simplification pass, and shipping evidence.
- May inspect all diffs read-only. If fixes are needed, ask leader or coordinate with owner; do not directly edit another worker's owned files without ACK.

## Cross-lane rules
- Workers are not alone in the codebase; do not revert others.
- Stay in owned path set. Shared files require leader handoff.
- Prefer deletion/marking stale over adding new layers.
- No dependencies.
- No production/cloud mutation.
- No blind `git add -A`.
- Final worker report must include changed files, tests/static checks, risks, and whether it is ready for leader integration.

## Stop condition
All lanes terminal: verified complete, no-op with evidence, or blocked with exact scope/file blocker. Leader then reviews, runs repo-level verification, commits safe slices, and keeps broader HANDOFF goal active.
