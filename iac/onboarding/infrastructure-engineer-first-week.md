# Infrastructure Engineer — First Week on `cloud-iac`

Audience: an infra/platform engineer with OpenTofu, Kubernetes, GitOps, policy, and audit-chain experience joining the `oya-cloud-iac-*` lane.

Goal: by Friday EOD you can trace a cloud-iac change from declarative source through render/validate/apply contracts, review the drift-remediation loop, and submit a production-grade PR that waits on the cloud-ci/oya-ci `oya-ci-required` authority.

## Day 1 — read before touching

- `docs/products/cloud/PRD.md` — Cloud Provider substrate, managed Kubernetes, GitOps, audit-chain, IAM/KMS, and control-plane/data-plane expectations.
- `docs/adr-archive/ADR-0218-tenant-granular-control-surface.md` — binding definition for declarative infrastructure.
- `docs/decisions/ADR-0709-general-live-apex.md — cloud-scm/self-modification substrate; cloud-iac is a control-plane mutator and must preserve auditability.
- `docs/adr-archive/ADR-0250-build-ahead-of-certification-doctrine.md` — modules must be compliance-shaped from first authoring.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — `oya-ci-required` is the single current blocking CI authority; `oya-ci-required`/controller evidence is shift-left evidence only.
- `iac/PRD.md`, `iac/ARCHITECTURE.md`, and `iac/threat-model.md` — service-local product, architecture, trust boundaries, and failure-mode anchors.

Work in a plain-git branch/worktree. Do not use retired `plain git` or `GitOps change-bundle` wrappers.

## Day 2 — trace the real declarative surfaces

Review the current cloud-iac sources of truth:

| Surface | Path |
|---|---|
| OpenTofu module registry | `iac/tofu/modules/` |
| Module release/catalog records | `iac/tofu/modules/catalog.json`, `release-index.json`, `provenance.json` |
| GitOps composition roots | `iac/iac/helm/**`, `iac/iac/kustomize/**` |
| Policy boundaries | `iac/policy/*.cedar`, `iac/policy/*.md` |
| Public contracts | `iac/contracts/openapi/cloud-iac.yaml`, `contracts/asyncapi/cloud-iac-events.yaml`, `contracts/proto/cloud-iac.proto` |
| Runbooks | `iac/runbooks/` |

Verification orientation:

```bash
buck2 build //iac/...
buck2 test //cloud/cloud-ci/...
```

These are local confidence checks. The blocking decision remains the PR's `oya-ci-required` status.

## Day 3 — author or tighten one production module contract

Choose a real module under `iac/tofu/modules/<module>/`. Keep the change inside the existing module directory unless the PR explicitly adds a new reviewed module catalog entry.

Required module contract:

- `main.tofu` declares typed inputs, outputs, provider pins, compliance-pack tags, tenant/account/project fields, and state/backend expectations.
- `README.md` explains apply order, consumed secret references, emitted outputs, and rollback/drift behavior.
- No raw secrets, provider credentials, or live account identifiers appear in source or docs.
- State, KMS, OpenBao, namespace, and AppProject bootstrap remain split across the existing module concerns unless an accepted ADR changes the boundary.

Do not create throwaway module sets, trial tenants, or local-only module roots. If a loopback fixture is needed for a test, it must be clearly named as test data and must not claim production readiness.

## Day 4 — submit through the GitOps/CI authority path

Before opening the PR, confirm the change is reviewable:

```bash
git diff -- iac cloud/cloud-ci .github/workflows/oya-ci-required.yml infra/branch-protection tools/hooks scripts/hooks docs/checklists .codex/hooks.json
buck2 build //iac/...
```

Open the PR against `dev`. The PR is ready only when:

- `oya-ci-required` is green on the candidate commit.
- The PR body cites the Cloud PRD, cloud-iac PRD/architecture/threat model, and any ADRs that define the touched contract.
- Reviewer-agent verdict is recorded by the lead/review lane.
- Any local command output is labeled "local shift-left evidence", never "merge authority".

## Day 5 — drift and rollback review drill

Walk the drift-remediation contract without mutating a live cluster:

- `iac/IP-GITOPS-005-drift-detection.md`
- `iac/runbooks/drift-remediation.md`
- `iac/runbooks/rollback-orchestration.md`
- `iac/contracts/asyncapi/cloud-iac-events.yaml`
- `iac/policy/ci-scope.cedar`

You should be able to explain:

1. Which worker identity may write a drift report.
2. Why validation and mutation are separate.
3. Which audit event is emitted before durable mutation is considered complete.
4. How rollback stays inside declared apply scope.
5. Why a controller/API/GitOps evidence cannot prove production drift remediation.

## What "done with week 1" means

- [ ] You can trace render, validate, apply, rollback, registry, drift, policy, audit, and SLO surfaces from committed files.
- [ ] You can explain why `oya-ci-required` is the only blocking CI authority for PRs.
- [ ] You submitted one scoped PR or review note against a real `iac/**` artifact.
- [ ] You ran relevant Buck2/local checks and labeled them as shift-left evidence.
- [ ] You identified any exact blocker instead of creating temporary IaC, local-only command flows, or false production-readiness claims.

## Rookie traps

1. **Treating `oya-ci-required`/controller evidence as authority.** `oya-ci-required`/controller evidence is the blocking authority; retired local verifier output is not accepted.
2. **Using retired VCS wrappers.** `plain git` and `GitOps change-bundle` are blocked; use plain git.
3. **Hand-rolling unmanaged IaC roots.** New roots need catalog, provenance, policy, runbook, and CI evidence.
4. **Mixing tenant classes or packs.** Tenant/account/project, jurisdiction, and compliance-pack fields must be explicit.
5. **Skipping secret indirection.** Use OpenBao/SecretReference patterns; never commit literal credentials.
