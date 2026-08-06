# Tutorial — Review plan/apply/drift-remediation contracts without local authority

Goal: understand how cloud-iac plans, applies, detects drift, and remediates through the GitOps/control-plane contracts already committed in this service. This tutorial is a review drill, not a local production runbook.

Authority boundary:

- PR merge readiness is the cloud-ci/oya-ci `oya-ci-required` status.
- Runtime mutation is performed by cloud-iac workers and GitOps/Kubernetes controllers with policy and audit-chain handoffs.
- controller/API/GitOps evidence, loopback commands, and mock-provider output are not production-readiness evidence.

## Step 1 — locate the declaration inputs

Start from committed sources:

| Concern | Source |
|---|---|
| Module catalog | `iac/tofu/modules/catalog.json` |
| Module releases | `iac/tofu/modules/release-index.json` |
| OpenTofu module bodies | `iac/tofu/modules/*/main.tofu` |
| GitOps roots | `iac/iac/helm/**`, `iac/iac/kustomize/**` |
| API contract | `iac/contracts/openapi/cloud-iac.yaml` |
| Event contract | `iac/contracts/asyncapi/cloud-iac-events.yaml` |
| Worker policy | `iac/policy/ci-scope.cedar` |

## Step 2 — review the plan-preview contract

Read `iac/contracts/openapi/cloud-iac.yaml` and find the plan-preview operation and schemas. A valid plan-preview contract must carry:

- tenant/account/project scope;
- principal and policy context;
- desired module or manifest reference;
- deterministic diff/resource changes;
- idempotency and audit correlation fields;
- version carrier where public contract boundaries apply.

The plan-preview result is evidence for review; it is not authorization to mutate.

## Step 3 — review the apply authorization boundary

Read:

- `iac/policy/ci-scope.cedar`
- `iac/policy/iac-isolation.md`
- `iac/threat-model.md`

Confirm these invariants:

- validator identities may write drift/validation evidence but may not apply;
- applier identities mutate only declared apply scope;
- rollback identities revert only declared apply scope;
- registry writes are append/provenance oriented;
- raw secrets stay behind cloud-secrets/OpenBao references.

## Step 4 — review drift detection

Read:

- `iac/IP-GITOPS-005-drift-detection.md`
- `iac/runbooks/drift-remediation.md`
- `iac/contracts/asyncapi/cloud-iac-events.yaml`

A valid `DriftDetected` path includes a signed drift report, affected resource reference, severity, tenant/account/project scope, audit correlation, and recommended remediation action.

## Step 5 — review remediation and rollback

Read:

- `iac/runbooks/rollback-orchestration.md`
- `iac/runbooks/state-lock-break.md`
- `iac/runbooks/stuck-apply-recovery.md`

The remediation path must use a separate action from initial apply. Rollback must prove prior known-good state, scope containment, state-lock behavior, and audit-chain emission before durable mutation is considered complete.

## Step 6 — collect local shift-left evidence

Use local checks to catch syntax and graph issues before the PR waits on cloud-ci:

```bash
buck2 build //iac/...
buck2 test //cloud/cloud-ci/...
```

If local tools are unavailable, report that as a validation gap and rely on the PR `oya-ci-required` run. Do not replace the protected status with local output.

## What you should be able to verify

- How a committed module/catalog change flows into render/validate/apply contracts.
- Which worker identity owns plan evidence, mutation, rollback, registry, and drift evidence.
- Which audit-chain events prove state transitions.
- Which runbook owns stuck apply, rollback, and drift remediation.
- Why no local command or mock-provider transcript can claim production drift remediation.
