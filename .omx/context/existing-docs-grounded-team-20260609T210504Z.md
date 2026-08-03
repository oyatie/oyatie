# Existing-docs grounded team context

## Hard instruction from user
This is NOT a request for new feature/product invention. It is parallelization of existing plans and development. Check and use existing documentation before editing. No stubs. No demos. No MVP. No placeholders. No fake "v1" scope. Production-quality, production-ready, industry-leading, modern, research/ADR-backed development only. If a production-quality slice is too large or blocked, report the blocker and exact production acceptance criteria; do not create placeholder work.

## Existing product/plan documentation anchors
- Overall product PRD: `docs/PRD.md` — North Star, target users, scope, cohesion thesis, hard constraints.
- Product index: `docs/products/README.md` — per-product PRD reading order and update protocol.
- Cloud product: `docs/products/cloud/PRD.md` — Cloud Provider (AWS-class), stories CLOUD-HS-001..026, frontend/components sections, hyperscaler audit.
- ERP/business/enterprise product: `docs/products/erp-coverage/PRD.md` — SAP-parity composition, FI/CO/MM/SD/PP/QM/PM/HCM/PS/PLM/EHS/SRM/CRM/SCM/GTS/TM/EWM stories.
- Workplace/business cross-cutting layer: `docs/products/workplace-integration/PRD.md` — workplace integration, B2B flows, workflow studio, plugin store, compliance, NFRs.
- SaaS platform: `docs/teams/axis-saas/CHARTER.md` and `docs/runbooks/saas/*.md` — workflow engine/plugin runtime/marketplace operations.
- Existing user journeys: B2B `docs/user-journeys/j33..j42`, business/sidebusiness `j48..j54`, supply-chain/cross-tenant `j100..j125`, HR/audit/business `j132..j141`, executive/ERP migrations `j165..j180`.
- Existing implementation roadmap: `specs/masterplan.json` contains M03 Cloud + SaaS + Search + Workspace Preview, M03-P01 cloud foundations, M03-P04 SaaS platform preview, M03-P06 workspace 14 surfaces, M03-P08 cross-axis contracts, and M07 P06 Application B2B Live.
- Unified frontend/app shell: `oya/app-shell-frontend/` is existing SolidJS app shell superseding Leptos prototype per ADR-0372; package `@oyatie/app-shell-frontend`, scripts `build`, `typecheck`, `codegen:check`.
- Existing office/workplace surfaces: `oya/office/**`, `oya/workplace-integration/**`.

## Cloud direction anchors
- User direction: cloud/Kubernetes-native; no local CLI/dev-cli as merge authority.
- Current merge authority: `oya-ci-required` bridge; future Kubernetes-native controller/status authority.
- Buck2 is canonical direction; Cargo remains in current CI until atomically migrated into cloud-ci app/fan-in.
- SeaweedFS is current object substrate; owned DB/object-store later behind stable interfaces; MinIO is non-final/stale/benchmark/forbidden context; RustFS has no repo refs.
- IAM/KMS are foundational; IaC consumes frozen IAM/KMS interfaces.

## Required skills/process per worker
Use `using-agent-skills` to pick lane workflow. Apply `spec-driven-development` against existing docs (do not invent scope), `test-driven-development` for behavior changes, `incremental-implementation`, `security-and-hardening`, `code-review-and-quality`, `code-review`, `code-simplification`, `ai-slop-cleaner`, and `shipping-and-launch` evidence. Workers may create nested subagents/goals inside their owned worktree/slice only; do not mutate leader goal state.

## Cross-lane rules
- Workers are not alone in the codebase; do not revert peer edits.
- Stay inside owned paths. Shared specs/registry/HANDOFF/branch protection require leader handoff before writing unless lane explicitly owns that file.
- Prefer retiring stale authority/slop over adding new layers.
- No new dependencies.
- No external production/cloud mutation.
- No generated JSON add/modify surfaces unless policy requires and validation proves it.
- No blind `git add -A`.
- Final report: existing docs consulted, changed files, tests/static checks, security review, simplification/anti-slop review, shipping readiness, risks/blockers.

## N=6 lane ownership
Worker 1 — Cloud identity/security foundation:
- Own `cloud/cloud-iam/**`, `cloud/cloud-kms/**`, `cloud/cloud-secrets/**`, `infra/kms/**`.
- Work from existing Cloud PRD/M03-P01 foundations. Align stale CLI/Jenkins authority to cloud/K8s-native non-authority. Production-grade IAM/IDP/KMS semantics only; no key material/secrets.

Worker 2 — Cloud control plane/IaC/GitOps/CI authority:
- Own `cloud/cloud-iac/**`, `microservices/cloud-iac/**`, `cloud/cloud-ci/**`, `.github/workflows/oya-ci-required.yml`, `.github/branch-protection.yaml`, `infra/branch-protection/**`, `tools/hooks/**`, `scripts/hooks/**`, `docs/checklists/**`, `.codex/hooks.json`.
- Work from Cloud PRD, SaaS/Cloud masterplan, current CI docs. Make cloud CI/Kubernetes-native authority explicit; local hooks are shift-left only. Do not remove Cargo CI until atomic migration exists.

Worker 3 — Cloud runtime substrate:
- Own `cloud/cloud-data/**`, `cloud/cloud-storage/**`, `infra/seaweedfs/**`, `cloud/cloud-compute/**`, `cloud/cloud-k8s/**`, `cloud/cloud-kernel/**`, `cloud/cloud-os/**`, `cloud/cloud-capacity/**`, `cloud/cloud-cell/**`, `cloud/cloud-network*/**`.
- Work from Cloud PRD/M03-P01..P03/P06. Clarify SeaweedFS/current + owned substrate-later; Kubernetes-native runtime only; no stale MinIO final claims.

Worker 4 — Business/ERP/B2B/workplace domain:
- Own `docs/products/erp-coverage/**`, `docs/products/workplace-integration/**`, B2B/business/workplace user journeys, `oya/workplace-integration/**` only if needed.
- Use existing ERP and workplace PRDs; do not invent product name unless documenting evidence-backed taxonomy/options. Production-grade cross-service business process semantics.

Worker 5 — SaaS/workflow/marketplace/platform:
- Own `docs/teams/axis-saas/**`, `docs/runbooks/saas/**`, SaaS/workflow/plugin/marketplace docs/surfaces, relevant M03-P04/M03-P08 existing plan references.
- Ensure SaaS platform is an existing axis built atop cloud control plane, not local CLI flows. Production-grade plugin/runtime/marketplace safety only.

Worker 6 — Unified frontend shell / office UX / integration review:
- Own `oya/app-shell-frontend/**`, `oya/office/**`, frontend/app-shell docs; read-only integration review across other lanes unless leader/owner ACK.
- Work from app shell package/source and product PRDs. Make unified shell serve existing cloud/business/SaaS/workplace plans; production-grade accessibility/performance/typecheck/codegen evidence.
