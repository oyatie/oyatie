---
kanban_task: t_6874a462
source_parent: t_cf995f91
title: QK-02 Shuffle-Shard Isolation Dogfood Harness Evidence Producer Plan/Spec/RED
status: red-check-ready-runtime-evidence-pending
generated_at_utc: 2026-07-01T09:25:50Z
claim_ceiling: target-backlog-schema-and-red-check-only; no measured tenant-isolation result or green production_100 claim
---

# QK-02 Shuffle-Shard Isolation Dogfood Harness Evidence Producer Plan/Spec/RED

## 0. Claim boundary

This artifact is the Plan/Spec/RED handoff for Kanban task `t_6874a462`. It specifies the missing QK-02 dogfood harness/evidence producer and adds a fail-closed RED check. It does not implement the shuffle-shard runtime simulator, does not emit a dogfood run receipt, and does not claim green tenant-isolation, production readiness, public SLA/SLO, tenant workload readiness, hyperscaler maturity, or an external SaaS/public-cloud fallback.

The only claim this artifact makes now is: QK-02 has a source-cited future harness contract, scenario-to-output map, evidence record path, non-claim boundary, and a RED check that fails until a real dogfood receipt with tenant/shard assignment inputs exists.

## 1. Source authority read for this card

- Target definition: `specs/cloud-production-quality-kits-target.json:44-60` defines `QK-02-shuffle-shard-isolation`, source `AWS Builders' Library (shuffle sharding)`, harness `shuffle-shard assignment simulator + correlated-blast-radius analyzer`, scenarios `single-tenant fault`, `noisy-neighbor resource hog`, and `poison-pill request`, controls `shuffle-shard assignment policy per resource`, `blast-radius bound per shard set`, and `noisy-neighbor throttling`, evidence `correlated-impact probability matrix; measured blast-radius containment; noisy-neighbor isolation drill`, and gate `production_100_bar.security_exit (tenant isolation, shared+dedicated)`.
- Backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:304-486` keeps QK-02 `status=pending_implementation`, `runtime_status=not_implemented`, and `evidence_status=evidence_required`; it requires fields `kit_id`, `scenario_id`, `run_id`, `dogfood_environment`, `command`, `status`, `artifact_digest`, `reviewer`, `created_at`, `source_commit`, `evidence_window`, and `result_summary`.
- Official source evidence: `specs/cloud-production-quality-kit-evidence-backlog.json:50-53` binds objective domain `shuffle_sharding_cell_isolation` to `https://aws.amazon.com/builders-library/workload-isolation-using-shuffle-sharding/`.
- Parent matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:34` records QK-02 as target/backlog only and missing implemented harness, dated dogfood run receipt, artifact digest/source commit/reviewer, measured outputs, and green production_100 evidence.
- De-dupe source: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:45-56` records this child, command, future evidence path, fields, digest fields, outputs, and non-claim language.
- Validator baseline: `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py` passed before and during this card.

## 2. Future harness command and evidence path

Exact future command to preserve:

`python3 scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/runs/<run_id>.json`

Future evidence path:

`evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/runs/<run_id>.json`

Dogfood environment record value:

`oyatie-dogfood-cell`

Forbidden fallbacks:

- no external SaaS runner;
- no GitHub Actions runner as runtime fallback;
- no public-cloud-provider runtime fallback;
- no static source text, Markdown plan, backlog row, or target spec may stand in for a run receipt.

## 3. Scenario-to-evidence binding

| Scenario ID | Source scenario | Required future input | Required output families | RED/non-claim posture |
| --- | --- | --- | --- | --- |
| `QK-02-shuffle-shard-isolation-S01` | `single-tenant fault` | A dogfood-cell tenant/shard assignment input set that names the target tenant, its assigned shard/cell set, neighbor tenants, assignment policy version, and input digest. | `correlated_impact_probability_matrix`, `blast_radius_bound` | Current artifact defines the required run shape only. No containment measurement or isolation result exists until a real receipt is emitted. |
| `QK-02-shuffle-shard-isolation-S02` | `noisy-neighbor resource hog` | A dogfood-cell assignment input set plus noisy-neighbor workload profile tied to tenant and shard identifiers. | `noisy_neighbor_isolation_drill`, `blast_radius_bound`, `correlated_impact_probability_matrix` | Current artifact does not prove throttling or resource isolation. Future evidence must include observed values and artifact refs from the dogfood run. |
| `QK-02-shuffle-shard-isolation-S03` | `poison-pill request` | A dogfood-cell assignment input set plus poison-pill request fixture tied to the affected tenant/shard set and blast-radius evaluation window. | `correlated_impact_probability_matrix`, `blast_radius_bound`, `noisy_neighbor_isolation_drill` where applicable | Current artifact does not prove poison-pill containment. Static source text or fabricated status is rejected. |

A future single run receipt may cover all three scenarios if it includes a `scenario_results` map for every scenario ID above. If future implementers choose one receipt per scenario, each receipt must still preserve the command/evidence path family and must link to a rollup receipt before QK-02 can feed `production_100_bar.security_exit`.

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

QK-02 output keys that must be present in `result_summary` and/or `scenario_results` before any positive status can be considered by a future implementation:

- `correlated_impact_probability_matrix`
- `blast_radius_bound`
- `noisy_neighbor_isolation_drill`

Additional QK-02 RED requirement: the receipt must include tenant/shard assignment inputs. The current RED check accepts these either as a top-level `tenant_shard_assignment_inputs` list or as `result_summary.tenant_shard_assignment_inputs`. Each input item must include a source artifact reference and digest, not static source prose.

## 5. RED/fail-closed check added by this card

Added check path:

`scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py`

RED behavior:

1. Rejects the wrong dogfood environment; only `oyatie-dogfood-cell` is valid for this card.
2. Rejects `--emit-evidence` paths outside `evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/runs/`.
3. Rejects missing run receipts.
4. Rejects static source text or source-only artifacts as evidence, including `specs/`, `plan/`, and `docs/` references in receipt artifact refs.
5. Rejects missing required evidence fields and missing digest fields.
6. Rejects missing tenant/shard assignment inputs.
7. Rejects missing QK-02 output keys.
8. Rejects placeholder receipt paths, non-concrete run IDs, non-hex source commits, non-`sha256:<64 lowercase hex>` artifact digests, non-UTC `created_at`, and missing `evidence_window` bounds.
9. Rejects evidence artifact references outside `evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/`, including absolute repo paths back to static source text.
10. Rejects forbidden fallback markers (`external_saas_runner`, `github_actions_runner`, `public_cloud_provider_runtime`) anywhere in the receipt.
11. Rejects `status=passed_after_future_runtime_evidence` as fabricated while this RED-only checker is in place. A future Build card must replace/extend the checker with actual dogfood runtime verification before any positive status can pass.

Intentional RED command for this Plan/Spec card:

`python3 scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/runs/red-check-missing-receipt.json`

Expected result now: non-zero exit with a missing dogfood run receipt message. That failure is the correct Plan/Spec/RED proof because no runtime dogfood receipt exists yet.

## 6. No-action overlap rationale

- Managed-k8s (`t_e14c021c` and children): no action here. Those cards cover managed-k8s lifecycle/control-plane/SLO source-authority gaps; they do not own a shuffle-shard assignment simulator, correlated-blast-radius analyzer, or QK-02 dogfood run receipt.
- Security/compliance (`t_77bc98f3`): no action here. That lane preserves security/regulatory authority gates; it does not produce the measured tenant-isolation drill for QK-02.
- SREOPS (`t_9d403ad1`, `t_cc1cb9cd`): no action here. Those cards cover SRE operations contract and review, not a quality-kit harness receipt.
- TrustCenter (`t_157e833c`, `t_3af64a26`, `t_3a144f8c`, `t_c9fba41f`, `t_e615a913`): no action here. Those surfaces may publish or ingest final evidence later; they do not implement QK-02 runtime evidence.
- OBS/Release/Resilience/DR (`t_1004e37b`, `t_bfcbdde5`, `t_cb12fdb1`, `t_62cf60fe`, `t_c127bb35`, `t_a7a9ed48`): no action here. Those lanes own observability/release/resilience/DR prerequisites and sub-slices, not the shuffle-shard isolation harness.

## 7. Future Build card boundaries

Allowed future implementation path class:

- `scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py`
- `evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/`
- tenant/cell/sharding manifests only when the Build card explicitly owns them and serializes with competing lanes

Forbidden in this Plan/Spec/RED card:

- generated JSON edits;
- root-hub pointer edits;
- provider/IaC state edits;
- broad product/control-plane surface edits;
- public-cloud or external-SaaS fallback implementation;
- green runtime evidence, production readiness, public SLA/SLO, tenant workload readiness, or hyperscaler maturity claims.

## 8. Verification commands for this card

Run and record:

1. `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`
   - Expected now: pass for the existing backlog/schema.
2. `python3 scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-02-shuffle-shard-isolation/runs/red-check-missing-receipt.json`
   - Expected now: fail closed because the dogfood run receipt is absent.
3. A fabricated receipt check may be run with a temporary JSON file under the QK-02 runs directory; expected now: fail closed on static source text, missing tenant/shard assignment inputs, and fabricated `passed_after_future_runtime_evidence`.
4. `python3 scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py --self-test`
   - Expected now: pass the validator self-test suite, which exercises static-source rejection, external fallback rejection, missing assignment/output/scenario rejection, placeholder digest rejection, and fabricated positive-status rejection without requiring a real dogfood receipt.

Closeout condition for `t_6874a462`: this artifact plus the RED check exist, the backlog validator still passes, the QK-02 check fails closed for absent/fabricated receipts, and the Kanban closeout explicitly states that the future evidence path remains pending until a real dogfood run receipt exists.
