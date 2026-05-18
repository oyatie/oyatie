---
doc_class: Runbook
title: Spec rollback — deprecate or retire a workflow spec version
microservice: workflow-engine
severity: "Sev-2 (operational rollback) / Sev-3 (downgrade attempt detection)"
status: Accepted
owner_team: axis-workflow + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/workflow-engine/failure-modes.md (FM-06 spec downgrade attempt)
  - microservices/workflow-engine/policy/spec-integrity.md
  - microservices/workflow-engine/incident-response.md
doc_status: published
---

# Runbook: Spec rollback (deprecate or retire a workflow spec version)

## Trigger

ONE of:

1. **Spec defect found in production**: in-flight runs against version N are causing tenant impact; need to deprecate N + migrate forward to N-1 (safer prior version) or N+1 (newly-fixed).
2. **Spec downgrade attempt detected**: LEAN lane refuses promotion of a SHA referencing deprecated/retired spec versions; investigate intent.
3. **Compliance-driven retirement**: regulatory issue with spec content requires retroactive scope reduction.

## Severity

- Auto-detected via gate: Sev-3.
- Operator-driven rollback after tenant impact: Sev-2.
- Compliance-driven retroactive retirement: Sev-1 if data-subject-impact.

## Pre-checks

1. Identify the affected spec: `(tenant_id, spec_id, version_sha)`.
2. Identify in-flight runs against the version: `cargo run -p oya-dev-cli -- workflow-engine list-runs --tenant <hash> --spec-id <id> --version-sha <sha> --status active`.
3. Identify the prior version + the proposed forward version (if any).
4. Verify two-person-rule signatures (mandatory for production-tier rollback).

## Recovery Path A — Deprecate the spec (in-flight runs continue against this version)

Use when: in-flight runs are correct on this version; only new runs should be forbidden.

| Step | Action |
|---|---|
| 1 | Deprecate via CLI: `cargo run -p oya-dev-cli -- workflow-engine deprecate-spec --tenant <hash> --spec-id <id> --version-sha <sha> --reason "<rfc>"`. |
| 2 | Verify new run-starts refuse this version: try a synthetic run-start; should return 409 Conflict with reason. |
| 3 | In-flight runs continue against this version (immutability per `policy/spec-integrity.md`); audit-chain notes the deprecation. |
| 4 | Notify tenant. |

## Recovery Path B — Retire the spec + bulk-cancel in-flight runs

Use when: spec defect causes in-flight runs to misbehave; tenant agrees to cancel + restart with corrected version.

| Step | Action |
|---|---|
| 1 | Retire: `cargo run -p oya-dev-cli -- workflow-engine retire-spec --tenant <hash> --spec-id <id> --version-sha <sha> --reason "<rfc>" --two-person-signature <s1> <s2>`. |
| 2 | Bulk-cancel in-flight: `cargo run -p oya-dev-cli -- workflow-engine bulk-cancel-runs --tenant <hash> --spec-id <id> --version-sha <sha> --reason "<rfc>" --two-person-signature <s1> <s2>`. |
| 3 | Verify in-flight count → 0. |
| 4 | Tenant publishes new spec version + restarts affected runs against the new version. |
| 5 | Audit-chain seals + tenant communication. |

## Recovery Path C — Spec downgrade attempt detected

`oya-governance-workflow-spec-signature-verification` lane refused promotion. Investigate:

| Step | Action |
|---|---|
| 1 | Identify what spec versions the SHA being promoted references. |
| 2 | If accidental (versions deprecated since SHA was built): rebuild the SHA against current published versions. |
| 3 | If intentional (rare; debugging an old version): file ADR justifying the bypass; require 2-person rule + audit. |
| 4 | If neither: investigate possible spec-versioning bug or supply-chain compromise. |

## Recovery Path D — Promote a new spec version forward (after rollback)

After Path A or B, tenant typically publishes a fixed version:

| Step | Action |
|---|---|
| 1 | Tenant submits new spec version through Studio or SDK; engine validates schema + signature. |
| 2 | Engine assigns new `version_sha`. |
| 3 | Tenant tests new version in tenant_scope=sandbox or trial. |
| 4 | Tenant promotes new version for production runs. |
| 5 | If tenant wants to retroactively migrate paused in-flight runs (rare): explicit `oya vcs migrate-workflow-version` with 2-person rule. |

## Verification

- Spec lifecycle table shows the version's `lifecycle_to` = `deprecated` or `retired` with reason + signer.
- In-flight runs against the version are: continue (deprecated path), or cancelled (retired path).
- Audit-chain seal log includes the lifecycle transition.
- New spec version (if applicable) is published + signature-verified.

## Post-incident updates

- Postmortem within 5 business days.
- Action: identify why the spec defect wasn't caught at PR review or in tenant_scope=sandbox.
- Action: tenant onboarding doc updates if a common pattern.
- Action: extend LEAN check for the failure pattern.

## References

- `microservices/workflow-engine/failure-modes.md` FM-06.
- `microservices/workflow-engine/policy/spec-integrity.md`.
- `microservices/workflow-engine/incident-response.md`.
- `microservices/workflow-engine/PRD.md` FR-01, FR-02.
