# CLOUD-QK-03 Plan/Spec/RED — privacy/data-governance dogfood harness evidence producer

Task: `t_ce4c0509`  
Generated: 2026-07-01T09:29:44Z  
Kit: `QK-03-privacy-data-governance`  
Claim ceiling: Plan/Spec/RED only. This artifact does not assert green runtime evidence, production-readiness, public SLA/SLO, tenant-workload readiness, hyperscaler maturity, certification, external SaaS fallback, GitHub Actions fallback, or public-cloud-provider runtime fallback.

## Source-bound authority

- Target kit definition: `specs/cloud-production-quality-kits-target.json:61-80`.
- Evidence backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:487-758`.
- Root-hub pointer for quality-kit target: `specs/root-hub-pointers.json:243-248`.
- Parent gap matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:35`.
- Parent de-dupe map: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:58-69`.
- Data-boundary/retention authority consumed, not redefined: `libs/oya-data-boundary-kernel/src/retention_policy.rs:177-218` and `libs/oya-data-boundary-kernel/fixtures/retention_qk03_dsr_evidence_contract.txt:27-44`.
- Privacy review source for DSR/residency triggers: `docs/standards/privacy-review.md:21-24` and `docs/standards/privacy-review.md:50-55`.

## Future harness contract

Command shape:

`python3 scripts/tests/qk_03_privacy_data_governance_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-03-privacy-data-governance/runs/<run_id>.json`

Evidence record path:

`evidence/cloud/quality-kits/qk-03-privacy-data-governance/runs/<run_id>.json`

The future run is dogfood-only. The record value for `dogfood_environment` must be exactly `oyatie-dogfood-cell`. The harness must not use external SaaS, GitHub Actions, or public-cloud-provider runtime fallback as the dogfood environment.

## Scenario binding

| Scenario id | Source scenario | Required evidence binding | Future receipt proof | Current status |
| --- | --- | --- | --- | --- |
| `QK-03-privacy-data-governance-S01` | personal-data inventory | `data_flow_map` | A run-produced data-flow map artifact under `evidence/cloud/quality-kits/qk-03-privacy-data-governance/` with data-class and tenant/cell boundary references. | RED/pending future dogfood receipt |
| `QK-03-privacy-data-governance-S02` | residency enforcement | `residency_enforcement_test` | A run-produced residency enforcement receipt proving pack/cell policy evaluation and cross-border-transfer disposition. | RED/pending future dogfood receipt |
| `QK-03-privacy-data-governance-S03` | retention expiry | `dsr_delete_export_proof` plus retention provenance | A run-produced retention-expiry receipt tied to the data-boundary `RetentionPolicy` window/action source and purge decision. | RED/pending future dogfood receipt |
| `QK-03-privacy-data-governance-S04` | deletion/erasure | `dsr_delete_export_proof` | A run-produced DSR delete/erasure receipt with purge-action selection and audit proof reference. | RED/pending future dogfood receipt |
| `QK-03-privacy-data-governance-S05` | export/portability | `dsr_delete_export_proof` | A run-produced DSR export/portability receipt with export artifact digest and audit proof reference. | RED/pending future dogfood receipt |
| `QK-03-privacy-data-governance-S06` | redaction in logs/telemetry | `telemetry_redaction_check` | A run-produced telemetry/log redaction receipt proving no raw PII in emitted samples. | RED/pending future dogfood receipt |

## Evidence record schema additions for this Plan/Spec/RED lane

The future receipt must include the backlog-required fields:

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

Digest-bearing fields are `source_commit`, `command`, `dogfood_environment`, and `artifact_digest`.

The future receipt must also include `privacy_data_governance_provenance` so the RED check can reject static text and missing DSR/residency/retention provenance. Required provenance keys:

- `data_flow_inventory_ref`
- `dsr_delete_receipt_ref`
- `dsr_export_receipt_ref`
- `residency_policy_ref`
- `residency_enforcement_receipt_ref`
- `retention_policy_ref`
- `retention_expiry_receipt_ref`
- `telemetry_redaction_receipt_ref`

`result_summary` must contain all kit output keys from the backlog row:

- `data_flow_map`
- `dsr_delete_export_proof`
- `residency_enforcement_test`
- `telemetry_redaction_check`

Each summary entry must carry `output_key`, `expected_value_or_threshold`, `observed_value`, `artifact_ref`, and `evaluation_status`.

## RED/fail-closed check

Created check:

`python3 scripts/tests/qk_03_privacy_data_governance_future_harness_check.py`

Fail-closed behavior specified by this lane:

1. Rejects a missing run receipt at the `--emit-evidence` path.
2. Rejects `dogfood_environment` values other than `oyatie-dogfood-cell`.
3. Rejects evidence paths outside `evidence/cloud/quality-kits/qk-03-privacy-data-governance/runs/`.
4. Rejects static source text as run evidence by refusing `artifact_ref` and receipt provenance references that point at static source roots such as `specs/`, `docs/`, `libs/`, `registry/catalog/`, or `templates/`.
5. Rejects missing required fields and missing digest fields.
6. Rejects missing DSR delete/export provenance, residency enforcement provenance, retention-expiry provenance, and telemetry redaction provenance.
7. Rejects forbidden fallback markers for external SaaS, GitHub Actions, or public-cloud-provider runtime fallback.
8. Rejects a fabricated `passed_after_future_runtime_evidence` status unless every required output has `evaluation_status=passed` with concrete evidence references.

The check is RED by construction today: invoking the future command with a concrete run id fails until a real dogfood run receipt exists.

## No-action boundaries

- TrustCenter portal/API/ingest: `t_157e833c` and children `t_3af64a26`, `t_3a144f8c`, `t_c9fba41f` own customer evidence portal, API, ingestion, and publishability policy surfaces. This QK-03 lane only defines the dogfood data-flow/DSR/residency/retention receipt contract and does not recreate TrustCenter UX/API work.
- Data-boundary authority: `t_134ad89c`, `t_d6f15090`, and the related `RETENTION-QK03-RED-001` fixture own taxonomy/retention authority seams. This lane consumes their retention and DSR source references; it does not redefine `DataClass`, `RetentionPolicy`, Cedar residency policy, purge executors, KMS shredding, or audit-chain proof emission.
- Security/compliance: `t_77bc98f3` owns security/compliance authority reconciliation. This lane remains a QK-03 receipt contract and does not claim certification, compliance approval, or a broad security gate result.
- SREOPS/OBS: `t_9d403ad1`, `t_cc1cb9cd`, `t_1004e37b`, `t_bfcbdde5`, and `t_cb12fdb1` own operations/observability/SLO/status evidence surfaces. This lane references telemetry redaction only as a privacy-output receipt and does not define measured SLO or operations-center behavior.
- Release/resilience: `t_62cf60fe`, `t_c127bb35`, and `t_a7a9ed48` own release, rollback, resilience, and DR sub-slices. This lane does not change rollout, canary, rollback, or release-governance semantics.

## Future implementation handoff

A future implementation lane should:

1. Implement the dogfood producer behind the exact command above without changing the dogfood environment semantics.
2. Emit one JSON receipt per concrete run id under the required `runs/` path.
3. Produce all four required output artifacts and reference them from `result_summary`.
4. Attach DSR delete/export, residency enforcement, retention-expiry, and telemetry redaction provenance to `privacy_data_governance_provenance`.
5. Compute `artifact_digest` after output artifacts and summary fields are finalized.
6. Keep `status=passed_after_future_runtime_evidence` unavailable until the future dogfood run has real artifacts, reviewer identity, source commit, and digest fields.

## Verification performed for this Plan/Spec/RED card

- Read the QK-03 target row, backlog row, root-hub pointer, parent matrix, and parent de-dupe map.
- Read nearby validators and source authorities before creating the check.
- Added the fail-closed RED check without editing generated JSON, root-hub pointers, compliance certification artifacts, provider/IaC state, or broad product/control-plane surfaces.
- Required validator command remains `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`.
- New check commands for closeout:
  - `python3 scripts/tests/qk_03_privacy_data_governance_future_harness_check.py --self-test`
  - `python3 scripts/tests/qk_03_privacy_data_governance_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-03-privacy-data-governance/runs/red-missing-receipt.json` (expected RED failure until the future run receipt exists)
