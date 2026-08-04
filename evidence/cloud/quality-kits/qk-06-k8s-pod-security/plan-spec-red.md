---
kanban_task: t_48572795
source_parent: t_cf995f91
title: QK-06 Kubernetes Pod Security Dogfood Harness Evidence Producer Plan/Spec/RED
status: red-check-ready-runtime-evidence-pending
generated_at_utc: 2026-07-01T13:20:49Z
claim_ceiling: target-backlog-schema-and-red-check-only; no measured Kubernetes admission-policy result or green production_100 claim
---

# QK-06 Kubernetes Pod Security Dogfood Harness Evidence Producer Plan/Spec/RED

## 0. Claim boundary

This artifact is the Plan/Spec/RED handoff for Kanban task `t_48572795`. It specifies the missing QK-06 dogfood harness/evidence producer and adds a fail-closed RED check. It does not implement Kubernetes, Kubewarden, Kyverno, admission-controller runtime tests, Kata runtime proof, or a live dogfood run receipt. It does not claim green runtime evidence, production readiness, public SLA/SLO, tenant workload readiness, hyperscaler maturity, Kubernetes live-readiness, or an external SaaS/public-cloud fallback.

The only current claim is: QK-06 has a source-cited future harness contract, scenario-to-output map, dogfood evidence record path, de-duped scope boundary, and a RED check that fails until a real dogfood run receipt with admission-policy receipts and privileged-exception provenance exists.

## 1. Source authority read for this card

- Target definition: `specs/cloud-production-quality-kits-target.json:116-132` defines `QK-06-k8s-pod-security`, source `Kubernetes Pod Security Standards`, harness `admission policy test set (Kubewarden, default; Kyverno adapter)`, scenarios `restricted enforced`, `baseline fallback`, and `privileged exception path`, controls `PSS restricted by default`, `privileged workloads require owner + expiry + mitigation`, and `Kata runtime for tenant-untrusted (ADR-0338)`, evidence `admission-policy test results; privileged-exception register with expiries`, and gate `production_100_bar.security_exit`.
- Evidence backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:1294-1476` keeps QK-06 `status=pending_implementation`, `runtime_status=not_implemented`, and `evidence_status=evidence_required`; it requires fields `kit_id`, `scenario_id`, `run_id`, `dogfood_environment`, `command`, `status`, `artifact_digest`, `reviewer`, `created_at`, `source_commit`, `evidence_window`, and `result_summary`.
- Parent matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:38` records QK-06 as target/backlog only and missing implemented harness, dated dogfood run receipt, artifact digest/source commit/reviewer, measured outputs, and green production_100 evidence.
- Parent de-dupe map: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:97-108` records this child, command, future evidence path, fields, digest fields, outputs, and non-claim language.
- Validator baseline: `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` is the standing schema/backlog gate for the QK-06 row.

## 2. Future harness command and evidence path

Exact future command to preserve:

`python3 scripts/tests/qk_06_k8s_pod_security_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-06-k8s-pod-security/runs/<run_id>.json`

Future evidence path:

`evidence/cloud/quality-kits/qk-06-k8s-pod-security/runs/<run_id>.json`

Dogfood environment record value:

`oyatie-dogfood-cell`

Forbidden fallbacks and substitutes:

- no external SaaS runner;
- no GitHub Actions runner as runtime fallback;
- no public-cloud-provider runtime fallback;
- no static source text, Markdown plan, backlog row, cloud/IaC file, or target spec may stand in for a run receipt;
- no green `passed_after_future_runtime_evidence` claim while this RED-only checker is in place.

## 3. Scenario-to-evidence binding

| Scenario ID | Source scenario | Required future admission proof | Required output families | RED/non-claim posture |
| --- | --- | --- | --- | --- |
| `QK-06-k8s-pod-security-S01` | `restricted enforced` | A dogfood-cell admission-policy decision receipt proving the restricted PSS profile rejects disallowed workload attributes and records the policy engine (`Kubewarden` default, `Kyverno` adapter allowed). | `admission_policy_test_results`, `restricted_profile_receipt` | Current artifact defines the required run shape only. No restricted-profile enforcement result exists until a real dogfood receipt is emitted. |
| `QK-06-k8s-pod-security-S02` | `baseline fallback` | A dogfood-cell admission-policy decision receipt proving baseline fallback behavior is explicit, bounded, and does not weaken the restricted-by-default control without a recorded reason. | `admission_policy_test_results`, `restricted_profile_receipt` | Current artifact does not prove baseline fallback behavior. Future evidence must include observed values and artifact refs from the dogfood run. |
| `QK-06-k8s-pod-security-S03` | `privileged exception path` | A dogfood-cell privileged workload exception receipt with owner, expiry, mitigation, workload reference, exception receipt reference, and register entry. | `admission_policy_test_results`, `privileged_exception_register` | Current artifact does not approve privileged workloads. Missing owner/expiry/mitigation or fabricated green status is rejected. |

A future single run receipt may cover all three scenarios if it includes `admission_policy_test_receipts` entries for every scenario ID above. If future implementers choose one receipt per scenario, each receipt must still preserve the command/evidence path family and must link to a rollup receipt before QK-06 can feed `production_100_bar.security_exit`.

## 4. Evidence record contract

The future JSON receipt must include all backlog-required fields:

- `kit_id`
- `scenario_id`
- `run_id`
- `dogfood_environment`
- `command`
- `status`
- `artifact_digest`
- `reviewer`
- `created_at`
- `source_commit`
- `evidence_window`
- `result_summary`

Digest fields that must be present and non-placeholder:

- `source_commit`
- `command`
- `dogfood_environment`
- `artifact_digest`

QK-06 output keys that must be present in `result_summary` before any positive status can be considered by a future implementation:

- `admission_policy_test_results`
- `restricted_profile_receipt`
- `privileged_exception_register`

Additional QK-06 RED requirements:

- `pod_security_admission_provenance` must identify the policy engine and reference run-produced evidence for `admission_policy_test_results_ref`, `restricted_profile_receipt_ref`, `privileged_exception_register_ref`, and `kata_runtime_control_ref`.
- `admission_policy_test_receipts` must be a non-empty list covering all three QK-06 scenario IDs. Each entry must include `scenario_id`, `policy_engine`, `pss_profile`, `namespace_ref`, `workload_manifest_ref`, `decision_receipt_ref`, and `evaluation_status`.
- `admission_policy_test_receipts` must preserve the source scenario-to-profile binding: `S01` uses `restricted`, `S02` uses `baseline`, and `S03` uses `privileged-exception`.
- `privileged_exception_register` must be a non-empty list. Each entry must include `exception_id`, `owner`, `expiry`, `mitigation`, `workload_ref`, and `exception_receipt_ref`.
- Every receipt/artifact reference above must point under `evidence/cloud/quality-kits/qk-06-k8s-pod-security/`, not at static source text.

## 5. RED/fail-closed check added by this card

Added check path:

`python3 scripts/tests/qk_06_k8s_pod_security_future_harness_check.py`

RED behavior:

1. Rejects the wrong dogfood environment; only `oyatie-dogfood-cell` is valid for this card.
2. Rejects `--emit-evidence` paths outside `evidence/cloud/quality-kits/qk-06-k8s-pod-security/runs/`.
3. Rejects missing run receipts.
4. Rejects static source text or source-only artifacts as evidence, including `specs/`, `docs/`, `libs/`, `registry/`, `templates/`, `cloud/`, `infra/`, and `plan/` references in receipt artifact refs.
5. Rejects missing required evidence fields and missing digest fields.
6. Rejects missing admission-policy test receipts or scenario coverage.
7. Rejects mismatched scenario-to-PSS-profile receipts (`S01=restricted`, `S02=baseline`, `S03=privileged-exception`).
8. Rejects missing privileged-exception provenance, register entries, owner, expiry, mitigation, workload evidence, or exception receipt.
9. Rejects forbidden fallback markers for external SaaS, GitHub Actions, or public-cloud-provider runtime fallback.
10. Rejects `status=passed_after_future_runtime_evidence` as fabricated while this RED-only checker is in place. A future Build card must replace/extend the checker with actual dogfood runtime verification before any positive status can pass.

Intentional RED command for this Plan/Spec card:

`python3 scripts/tests/qk_06_k8s_pod_security_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-06-k8s-pod-security/runs/red-missing-receipt.json`

Expected result now: non-zero exit with a missing dogfood run receipt message. That failure is the correct Plan/Spec/RED proof because no runtime dogfood receipt exists yet.

## 6. No-action overlap rationale

- Managed-k8s decomposition (`t_e14c021c` and children `t_ec4285e5`, `t_9423867d`, `t_6a39f845`): no action here. Those cards own managed-k8s lifecycle/control-plane/SLO Plan/Spec slices and explicitly avoided broad duplicates; they do not own a QK-06 Pod Security Standards evidence harness.
- Security/compliance reconciliation (`t_77bc98f3`): no action here. That lane preserves security/regulatory gates and created no duplicate child for this measured admission-policy test set.
- Trust/root/KMS/quota lanes: no action here unless future implementation needs them as inputs. This lane does not redefine trust root material, KMS, tenant quota, or broad Kubernetes runtime readiness.
- TrustCenter/portal evidence surfaces: no action here. Those surfaces may eventually publish final receipts; this card only defines the dogfood producer contract and RED guard.

## 7. Future implementation handoff

A future Build card should:

1. Implement the dogfood producer behind the exact command above without changing dogfood environment semantics.
2. Run in `oyatie-dogfood-cell` only, using Kubewarden as the default policy engine and Kyverno only as an adapter path if explicitly wired.
3. Emit one JSON receipt per concrete run id under the required `runs/` path.
4. Produce all three required output artifacts and reference them from `result_summary`.
5. Attach admission decision receipts for all three scenarios plus a privileged-exception register with owner, expiry, and mitigation.
6. Compute `artifact_digest` after output artifacts and summary fields are finalized.
7. Keep `status=passed_after_future_runtime_evidence` unavailable until the future dogfood run has real artifacts, reviewer identity, source commit, digest fields, and a real runtime verifier replacing this RED-only guard.

## 8. Verification commands for this card

Run and record:

1. `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`
   - Expected now: pass for the existing backlog/schema.
2. `python3 scripts/tests/qk_06_k8s_pod_security_future_harness_check.py --self-test`
   - Expected now: pass the validator's fail-closed mutation tests.
3. `python3 scripts/tests/qk_06_k8s_pod_security_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-06-k8s-pod-security/runs/red-missing-receipt.json`
   - Expected now: fail closed because the dogfood run receipt is absent.

Closeout condition for `t_48572795`: this artifact plus the RED check exist, the backlog validator still passes, the QK-06 check self-tests pass, the future command fails closed for an absent receipt, and the Kanban closeout explicitly states that the future evidence path remains pending until a real dogfood run receipt exists.
