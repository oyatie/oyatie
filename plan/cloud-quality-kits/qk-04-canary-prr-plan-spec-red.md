---
kanban_task: t_6baaef13
source_parent: t_cf995f91
title: QK-04 Canary/PRR and DR Dogfood Harness Evidence Producer Plan/Spec/RED
status: red-check-ready-runtime-evidence-pending
generated_at_utc: 2026-07-01T13:20:46Z
claim_ceiling: target-backlog-schema-and-red-check-only; no measured canary, PRR, rollback, DR, green runtime, or production_100 claim
---

# QK-04 Canary/PRR and DR Dogfood Harness Evidence Producer Plan/Spec/RED

## 0. Claim boundary

This artifact is the Plan/Spec/RED handoff for Kanban task `t_6baaef13`. It specifies the missing QK-04 dogfood harness/evidence producer and adds a fail-closed RED check. It does not implement the one-cell canary orchestrator, automated metric evaluator, PRR template runtime, rollback drill, backup/restore drill, RTO/RPO validation, cell-failover drill, or dependency-failure recovery drill. It does not emit a dogfood run receipt and does not claim green runtime evidence, production readiness, public SLA/SLO, tenant workload readiness, hyperscaler maturity, live DR readiness, or an external SaaS/public-cloud fallback.

The only claim this artifact makes now is: QK-04 has a source-cited future harness contract, scenario-to-output map for the four canary/PRR scenarios plus DR01..DR04, evidence record path, non-claim boundary, and a RED check that fails until a real dogfood receipt with all required canary/PRR/rollback/DR receipts exists.

## 1. Source authority read for this card

- Target definition: `specs/cloud-production-quality-kits-target.json:81-98` defines `QK-04-canary-prr`, source `Google SRE (canary, production-readiness review)`, harness `one-cell canary orchestrator + automated metric evaluator + PRR template`, scenarios `canary to 1 cell`, `bake-time SLO observation`, `regression detection`, and `auto-rollback on breach`, controls `design-time PRR checklist`, `automated canary eval against SLO`, and `rollback triggers (ArgoCD, ADR-0349)`, evidence `canary eval reports; PRR sign-offs; rollback drill evidence`, and gate `production_100_bar.ci_cd_exit`.
- Backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:759-1110` keeps QK-04 `status=pending_implementation`, `runtime_status=not_implemented`, and `evidence_status=evidence_required`; it requires fields `kit_id`, `scenario_id`, `run_id`, `dogfood_environment`, `command`, `status`, `artifact_digest`, `reviewer`, `created_at`, `source_commit`, `evidence_window`, and `result_summary`.
- DR extension: `specs/cloud-production-quality-kit-evidence-backlog.json:1086-1108` binds the DR sub-slice to the official failover/DR source and requires `QK-04-canary-prr-DR01` through `QK-04-canary-prr-DR04` plus `backup_restore_drill_receipt`, `rto_rpo_restore_drill_receipt`, `cell_failover_drill_receipt`, and `dependency_failure_recovery_receipt`.
- Parent matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:36` records QK-04 as target/backlog only and missing implemented harness, dated dogfood run receipt, artifact digest/source commit/reviewer, measured outputs, and green production_100 evidence.
- De-dupe source: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:71-82` records this child, command, future evidence path, fields, digest fields, outputs, and non-claim language.
- Validator baseline: `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` passed before and during this card.

## 2. Future harness command and evidence path

Exact future command to preserve:

`python3 scripts/tests/qk_04_canary_prr_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-04-canary-prr/runs/<run_id>.json`

Future evidence path:

`evidence/cloud/quality-kits/qk-04-canary-prr/runs/<run_id>.json`

Dogfood environment record value:

`oyatie-dogfood-cell`

Forbidden fallbacks:

- no external SaaS runner;
- no GitHub Actions runner as runtime fallback;
- no public-cloud-provider runtime fallback;
- no static source text, Markdown plan, backlog row, target spec, PRR template, OpenSLO target, runbook, or control-plane spec may stand in for a dogfood run receipt.

## 3. Scenario-to-evidence binding

| Scenario ID | Source scenario | Required future input | Required output families | RED/non-claim posture |
| --- | --- | --- | --- | --- |
| `QK-04-canary-prr-S01` | `canary to 1 cell` | Dogfood-cell deployment candidate, selected canary cell, baseline cell, workload window, traffic-split policy, and source commit digest. | `canary_eval_report`, `prr_signoff` | Current artifact defines the required receipt shape only. No one-cell canary has run and no canary result exists until a real receipt is emitted. |
| `QK-04-canary-prr-S02` | `bake-time SLO observation` | Dogfood-cell bake window, OpenSLO/SLI query refs, evaluator thresholds, and raw metrics/traces artifact refs. | `canary_eval_report`, `prr_signoff` | Current artifact does not prove SLO observation. Future evidence must include dogfood artifact refs and observed values, not static OpenSLO text. |
| `QK-04-canary-prr-S03` | `regression detection` | Dogfood-cell baseline/canary comparison, regression detector config, alert/decision artifact refs, and source commit digest. | `canary_eval_report`, `rollback_drill_receipt` | Current artifact does not prove regression detection. Static source text or fabricated status is rejected. |
| `QK-04-canary-prr-S04` | `auto-rollback on breach` | Dogfood-cell breach stimulus, rollback trigger decision, deployment rollback receipt, and post-rollback observation window. | `canary_eval_report`, `rollback_drill_receipt` | Current artifact does not prove automatic rollback. Release cards may own progressive-delivery implementation, but QK-04 still needs its own run receipt. |
| `QK-04-canary-prr-DR01` | `backup and restore recovery rehearsal` | Dogfood-cell backup artifact, restore target, restore-window measurement, digest, and rollback/fallback linkage. | `backup_restore_drill_receipt`, `rto_rpo_restore_drill_receipt`, `rollback_drill_receipt` | Current artifact does not prove restore or RTO/RPO. Future evidence must include run evidence under the QK-04 evidence root. |
| `QK-04-canary-prr-DR02` | `cell or region failover recovery rehearsal` | Affected cell, fallback cell, failover trigger, traffic-routing receipt, and recovery observation window. | `cell_failover_drill_receipt`, `rollback_drill_receipt` | Current artifact does not prove cell/region failover. Resilience/DR cards own control-loop sub-slices; this card owns only QK-04 receipt contract. |
| `QK-04-canary-prr-DR03` | `RTO/RPO recovery objective validation` | Declared RTO/RPO objective, measured recovery point/time fields, backup/restore artifacts, and evaluator decision. | `rto_rpo_restore_drill_receipt`, `backup_restore_drill_receipt` | Current artifact does not prove RTO/RPO. Future receipt must carry measured objective fields and digest-bound artifacts. |
| `QK-04-canary-prr-DR04` | `dependency failure recovery rehearsal` | Dependency failure stimulus, degraded-mode artifact refs, restoration proof, recovery window, and rollback/fallback link. | `dependency_failure_recovery_receipt`, `rollback_drill_receipt` | Current artifact does not prove dependency-failure recovery. Static runbooks or resilience specs are not acceptable evidence. |

A future single run receipt may cover all eight scenarios if it includes a `scenario_results` map for every scenario ID above. If future implementers choose one receipt per scenario, each receipt must still preserve the command/evidence path family and must link to a rollup receipt before QK-04 can feed `production_100_bar.ci_cd_exit`.

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

QK-04 output keys that must be present in `result_summary` before any future implementation can consider a positive status:

- `canary_eval_report`
- `prr_signoff`
- `rollback_drill_receipt`
- `backup_restore_drill_receipt`
- `rto_rpo_restore_drill_receipt`
- `cell_failover_drill_receipt`
- `dependency_failure_recovery_receipt`

Additional QK-04 RED requirement: the receipt must include a `scenario_results` map for all eight scenario IDs, and each scenario entry must include dogfood evidence `artifact_refs` plus output keys for that scenario. Static source refs under `specs/`, `docs/`, `plan/`, `libs/`, `registry/`, or `templates/` are rejected as evidence refs.

## 5. RED/fail-closed check added by this card

Added check path:

`scripts/tests/qk_04_canary_prr_future_harness_check.py`

RED behavior:

1. Rejects the wrong dogfood environment; only `oyatie-dogfood-cell` is valid for this card.
2. Rejects `--emit-evidence` paths outside `evidence/cloud/quality-kits/qk-04-canary-prr/runs/`.
3. Rejects missing run receipts.
4. Rejects static source text or source-only artifacts as evidence, including `specs/`, `docs/`, `plan/`, `libs/`, `registry/`, and `templates/` references in receipt artifact refs.
5. Rejects missing required evidence fields and missing digest fields.
6. Rejects missing canary/PRR/rollback receipts: `canary_eval_report`, `prr_signoff`, and `rollback_drill_receipt`.
7. Rejects missing DR receipts: `backup_restore_drill_receipt`, `rto_rpo_restore_drill_receipt`, `cell_failover_drill_receipt`, and `dependency_failure_recovery_receipt`.
8. Rejects missing scenario coverage for `QK-04-canary-prr-S01`..`S04` and `QK-04-canary-prr-DR01`..`DR04`.
9. Rejects `status=passed_after_future_runtime_evidence` as fabricated while this RED-only checker is in place. A future Build card must replace/extend the checker with actual dogfood runtime verification before any positive status can pass.

Intentional RED command for this Plan/Spec card:

`python3 scripts/tests/qk_04_canary_prr_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-04-canary-prr/runs/red-check-missing-receipt.json`

Expected result now: non-zero exit with a missing dogfood run receipt message. That failure is the correct Plan/Spec/RED proof because no runtime dogfood receipt exists yet.

Self-test command for the fail-closed validator logic:

`python3 scripts/tests/qk_04_canary_prr_future_harness_check.py --self-test`

Expected result now: pass, proving the checker rejects static source text, missing canary/PRR/rollback/DR receipts, missing digest fields, missing scenario evidence refs, forbidden fallbacks, and fabricated `passed_after_future_runtime_evidence` status.

## 6. No-action overlap rationale

- SREOPS (`t_9d403ad1`, `t_cc1cb9cd`): no action here. Those cards cover operations-center product/spec contracts only; they do not produce live on-call/status/SLO or QK-04 canary/PRR/DR receipts.
- Release/rollback/feature-flag (`t_62cf60fe`, `t_cb12fdb1`): no action here. Those lanes own progressive-delivery and OpenSLO rollback sub-slices. QK-04 consumes their eventual release/rollback primitives but must still emit its own digest-bound dogfood run receipt before any quality-kit evidence claim.
- Resilience/DR (`t_c127bb35`, `t_a7a9ed48`): no action here. Those lanes own chaos/status/DR schema/control-loop sub-slices. QK-04 owns only the quality-kit evidence producer wrapper and fail-closed run-receipt contract for the DR01..DR04 outputs.
- OBS (`t_1004e37b`, `t_bfcbdde5`): no action here. Those lanes own observability/OpenSLO evidence substrate work. QK-04 consumes future metric/query evidence but does not duplicate the observability product/control-plane implementation.
- TrustCenter (`t_157e833c`, `t_3af64a26`, `t_3a144f8c`, `t_c9fba41f`, `t_e615a913`): no action here. TrustCenter may later ingest/publish final evidence; it does not produce the QK-04 dogfood harness receipt.

## 7. Future Build card boundaries

Allowed future implementation path class:

- `scripts/tests/qk_04_canary_prr_future_harness_check.py`
- `evidence/cloud/quality-kits/qk-04-canary-prr/`
- release/rollback, OpenSLO, resilience/DR, and cell/failover harness roots only when the Build card explicitly owns them and serializes with competing lanes

Forbidden in this Plan/Spec/RED card:

- generated JSON edits;
- root-hub pointer edits;
- provider/IaC/Kubernetes state edits;
- broad SRE/status/release/resilience/control-plane surface edits;
- public-cloud or external-SaaS fallback implementation;
- green runtime evidence, production readiness, public SLA/SLO, tenant workload readiness, hyperscaler maturity, live DR readiness, or external SaaS/public cloud fallback claims.

## 8. Verification commands for this card

Run and record:

1. `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`
   - Expected now: pass for the existing backlog/schema.
2. `python3 scripts/tests/qk_04_canary_prr_future_harness_check.py --self-test`
   - Expected now: pass for fail-closed validator mutation coverage.
3. `python3 scripts/tests/qk_04_canary_prr_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-04-canary-prr/runs/red-check-missing-receipt.json`
   - Expected now: fail closed because the dogfood run receipt is absent.

Closeout condition for `t_6baaef13`: this artifact plus the RED check exist, the backlog validator still passes, the QK-04 check passes self-tests and fails closed for absent/fabricated receipts, and the Kanban closeout explicitly states that the future evidence path remains pending until a real dogfood run receipt exists.
