# Context: production cloud + product teams

## Hard quality bar
No stubs. No demos. No MVP claims. No placeholders. No fake "v1 scope" or toy scope. No false green. Every worker must either deliver production-quality, production-ready, industry-leading, modern, research/ADR-backed changes with evidence, or explicitly report that a production-quality slice is blocked/no-op with exact reasons and next production acceptance criteria. Do not paper over gaps.

## User direction
- Stay cloud/Kubernetes native.
- Do not make local CLI/dev CLI/oya-dev-cli the authority.
- Buck2 is the canonical direction, but current Cargo CI lanes remain until atomically migrated into cloud-ci apps.
- Work in parallel where dependency seams allow.
- Include B2B/ERP/enterprise/business/SaaS product area and unified frontend/app shell along with all cloud.

## Current base evidence
- `1e970563a cleanup: retire tracked local durable authority` is committed on the current branch.
- `.omc/.omx` tracked authority retired; live authority is specs/registry/evidence/templates.
- Merge authority is current `oya-ci-required` cloud CI bridge, future Kubernetes-native controller/status authority.
- SeaweedFS is current object substrate; owned DB/object-store later behind stable interfaces; MinIO is not final-canonical; RustFS has no repo refs.
- IAM and KMS are foundational; IaC consumes frozen IAM/KMS interfaces.

## Skill process required in each lane
Use these practices explicitly: using-agent-skills, spec-driven-development, test-driven-development when behavior changes, incremental-implementation, code-review-and-quality, code-review, security-and-hardening, code-simplification, ai-slop-cleaner, shipping-and-launch. Workers may create nested subagents/goals inside owned slices/worktrees if useful, but must not mutate the leader's active goal state.

## Shared rules
- Workers are not alone in the codebase; never revert peer edits.
- Stay in owned files. Shared specs/registry/HANDOFF/branch protection require leader handoff before writing unless lane explicitly owns them.
- Prefer deletion/retirement of stale authority over adding layers.
- No new dependencies.
- No production/cloud mutation.
- No generated JSON add/modify surfaces unless materialization policy requires it and validation proves it.
- No blind `git add -A`.
- Final report: changed files, tests/static checks, security review, simplification pass, shipping readiness, risks/blockers.

## Team A — cloud substrate lanes
1. Cloud IAM/IDP: `cloud/cloud-iam/**`; align identity/IDP docs/contracts with cloud/K8s-native authority and production-quality IDP/federation semantics.
2. Cloud KMS/secrets/security substrate: `cloud/cloud-kms/**`, `infra/kms/**`, coordinate `cloud/cloud-secrets/**`; no secrets/key material; production-grade KMS/OpenBao/HSM story.
3. Cloud IaC/control-plane/GitOps: `cloud/cloud-iac/**`, `microservices/cloud-iac/**`; declarative OpenTofu/operator/Argo/K8s-native control plane, consuming frozen IAM/KMS.
4. Cloud data/storage/object substrate: `cloud/cloud-data/**`, `cloud/cloud-storage/**`, `infra/seaweedfs/**`; SeaweedFS now + owned DB/object-store later; retire stale MinIO final claims.
5. Cloud compute/K8s/kernel/OS/network/capacity: `cloud/cloud-compute/**`, `cloud/cloud-k8s/**`, `cloud/cloud-kernel/**`, `cloud/cloud-os/**`, `cloud/cloud-capacity/**`, `cloud/cloud-cell/**`, `cloud/cloud-network*`.

## Team B — product/fabric lanes
1. Cloud CI/fabric/toolchain/hooks: `cloud/cloud-ci/**`, `.github/workflows/oya-ci-required.yml`, `.github/branch-protection.yaml`, `infra/branch-protection/**`, `tools/hooks/**`, `scripts/hooks/**`, `docs/checklists/**`, `.codex/hooks.json`; enforce cloud CI authority and retired CLI semantics.
2. B2B/ERP/enterprise/business domain: `docs/products/erp-coverage/**`, B2B/business user journeys, enterprise/accounting/hr/tenant-rbac surfaces where scoped; define canonical naming if evidence supports it.
3. SaaS/workflow/marketplace product platform: `docs/runbooks/saas/**`, `docs/teams/axis-saas/**`, SaaS/workflow/plugin/marketplace docs/specs; ensure product platform is built atop cloud control plane.
4. Unified frontend shell/app shell/office/workplace UX: `oya/app-shell-frontend/**`, `oya/office/**`, `oya/workplace-integration/**`, UX shell manifests/docs; production-grade unified shell for cloud+business+SaaS.
5. Integration/review/security/simplification/shipping: read-only across both teams by default; conflict watch, security/hardening, anti-slop, code review, shipping evidence; edit only with leader/owner ACK.
