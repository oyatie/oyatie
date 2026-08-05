# CLOUD-QK-05 Plan/Spec/RED — FOCUS cost-export dogfood harness evidence producer

Task: `t_d0211b9f`  
Generated: 2026-07-01T13:21:04Z  
Kit: `QK-05-focus-cost-export`  
Claim ceiling: Plan/Spec/RED only. This artifact does not assert green runtime evidence, production-readiness, public SLA/SLO, tenant-workload readiness, hyperscaler maturity, real invoice/billing readiness, external SaaS fallback, GitHub Actions fallback, or public-cloud-provider runtime fallback.

## Source-bound authority

- Target kit definition: `specs/cloud-production-quality-kits-target.json:99-115`.
- Evidence backlog/schema row: `specs/cloud-production-quality-kit-evidence-backlog.json:1111-1293`.
- Root-hub pointer for quality-kit target: `specs/root-hub-pointers.json:250-255`.
- Parent gap matrix: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_0ca79b25/cloud-quality-slo-gap-matrix.md:37`.
- Parent de-dupe map: `/Users/jasonlee/.hermes/kanban/boards/oyatie/workspaces/t_cf995f91/cloud-quality-kit-dedupe-plan-spec-red-map.md:84-95` and no-action boundary at lines 123-128.
- FinOps authority consumed, not redefined: `docs/decisions/ADR-0199-per-tenant-cost-attribution-finops-substrate.md:105-118` and `docs/standards/finops-cost-attribution-canonical.md:124-134` bind OpenCost/FOCUS 1.3 normalization and the target export bucket shape.

## Future harness contract

Command shape:

`python3 scripts/tests/qk_05_focus_cost_export_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-05-focus-cost-export/runs/<run_id>.json`

Evidence record path:

`evidence/cloud/quality-kits/qk-05-focus-cost-export/runs/<run_id>.json`

The future run is dogfood-only. The record value for `dogfood_environment` must be exactly `oyatie-dogfood-cell`. The harness must not use external SaaS, GitHub Actions, or public-cloud-provider runtime fallback as the dogfood environment.

## Scenario binding

| Scenario id | Source scenario | Required evidence binding | Future receipt proof | Current status |
| --- | --- | --- | --- | --- |
| `QK-05-focus-cost-export-S01` | per-tenant cost attribution | `focus_schema_validation` plus `cost_attribution_reconciliation` | A run-produced FOCUS 1.3 export fixture and attribution reconciliation receipt under `evidence/cloud/quality-kits/qk-05-focus-cost-export/` with tenant/cost-center/workload/regulatory-pack dimensions. | RED/pending future dogfood receipt |
| `QK-05-focus-cost-export-S02` | allocation by tag/dimension | `cost_attribution_reconciliation` | A run-produced tag/dimension allocation receipt showing allocation columns and normalized FinOps label/tag dimensions tied to the emitted FOCUS export. | RED/pending future dogfood receipt |
| `QK-05-focus-cost-export-S03` | invoice reconciliation | `invoice_reconciliation` plus `focus_schema_validation` | A run-produced invoice-reconciliation receipt that ties the FOCUS export totals to invoice/source-bill evidence and records the reconciliation disposition. | RED/pending future dogfood receipt |

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

The future receipt must also include `focus_cost_export_provenance` so the RED check can reject static text, missing FOCUS export fixture/provenance, missing attribution reconciliation evidence, and missing invoice-reconciliation evidence. Required provenance keys:

- `focus_schema_version`
- `focus_schema_source_url`
- `focus_export_fixture_ref`
- `focus_schema_validation_receipt_ref`
- `cost_allocation_input_ref`
- `cost_attribution_reconciliation_receipt_ref`
- `tag_dimension_allocation_receipt_ref`
- `invoice_source_ref`
- `invoice_reconciliation_receipt_ref`

`focus_schema_version` must be `1.3`; `focus_schema_source_url` must bind `https://focus.finops.org/focus-specification/v1-3/`. All fixture, attribution, tag/dimension, invoice source, and reconciliation references must be run evidence under `evidence/cloud/quality-kits/qk-05-focus-cost-export/`, not static source text from `specs/`, `docs/`, `libs/`, `registry/`, or templates.

`result_summary` must contain all kit output keys from the backlog row:

- `focus_schema_validation`
- `cost_attribution_reconciliation`
- `invoice_reconciliation`

Each summary entry must carry `output_key`, `expected_value_or_threshold`, `observed_value`, `artifact_ref`, and `evaluation_status`.

## RED/fail-closed check

Created check:

`python3 scripts/tests/qk_05_focus_cost_export_future_harness_check.py`

Fail-closed behavior specified by this lane:

1. Rejects a missing run receipt at the `--emit-evidence` path.
2. Rejects `dogfood_environment` values other than `oyatie-dogfood-cell`.
3. Rejects evidence paths outside `evidence/cloud/quality-kits/qk-05-focus-cost-export/runs/`.
4. Rejects static source text as run evidence by refusing `artifact_ref` and `focus_cost_export_provenance` evidence references that point at static source roots such as `specs/`, `docs/`, `libs/`, `registry/`, or `templates/`.
5. Rejects missing required fields and missing digest fields.
6. Rejects missing FOCUS export fixture/provenance, missing cost-attribution reconciliation evidence, missing tag/dimension allocation evidence, and missing invoice-reconciliation evidence.
7. Rejects forbidden fallback markers for external SaaS, GitHub Actions, or public-cloud-provider runtime fallback.
8. Rejects `status=passed_after_future_runtime_evidence` as fabricated while this RED-only check is in place. A future Build card must replace/extend the checker with actual dogfood runtime verification before any positive status can pass.

The check is RED by construction today: invoking the future command with a concrete run id fails until a real dogfood run receipt exists.

## No-action boundaries

- FINOPS-UX product/fixture surface: `t_c8e52838` and child `t_9574c74d` own Cost Explorer, Budget, and Anomaly UX/fixtures. This QK-05 lane only defines the FOCUS export plus reconciliation receipt contract and does not add Cost Explorer UI, budget UI, anomaly UI, or customer-facing billing presentation.
- Cloud resource FinOps/resource-contract planning: `t_bf9346d9` and child `t_45c830e6` own cloud-finops resource-contract Plan/Spec work. This lane consumes the canonical label/tag and FOCUS requirements; it does not create a broad cloud-finops implementation duplicate.
- Billing substrate and invoices: this lane does not implement price books, tax calculations, invoice ledgers, invoice publication, customer chargeback reports, or external billing integrations.
- TrustCenter: TrustCenter may later ingest or publish evidence, but this card produces the dogfood harness receipt contract only.
- Provider/IaC/live billing data: this lane does not touch public-cloud-provider state, provider credentials, OpenTofu state, live invoices, or broad product/control-plane surfaces.

## Future implementation handoff

A future implementation lane should:

1. Implement the dogfood producer behind the exact command above without changing the dogfood environment semantics.
2. Emit one JSON receipt per concrete run id under the required `runs/` path.
3. Produce a FOCUS 1.3 export fixture and schema-validation receipt under the QK-05 evidence root.
4. Produce tenant/tag/dimension allocation inputs and cost-attribution reconciliation receipts tied to the FOCUS export.
5. Produce invoice source evidence and invoice-reconciliation receipts tied to the FOCUS export totals.
6. Compute `artifact_digest` after output artifacts and summary fields are finalized.
7. Keep `status=passed_after_future_runtime_evidence` unavailable until the future dogfood run has real artifacts, reviewer identity, source commit, digest fields, and a real runtime verifier replacing this RED-only guard.

## Verification performed for this Plan/Spec/RED card

- Read the QK-05 target row, backlog row, root-hub pointer, parent matrix, parent de-dupe map, ADR-0199, and the FinOps canonical standard.
- Read nearby validators and existing QK-02/QK-03 RED checks before creating the QK-05 check.
- Added the fail-closed RED check without editing generated JSON, root-hub pointers, provider/IaC state, live billing data, invoice runtime, tax/price-book logic, or broad product/control-plane surfaces.
- Required validator command remains `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`.
- New check commands for closeout:
  - `python3 scripts/tests/qk_05_focus_cost_export_future_harness_check.py --self-test`
  - `python3 scripts/tests/qk_05_focus_cost_export_future_harness_check.py --dogfood-environment oyatie-dogfood-cell --emit-evidence evidence/cloud/quality-kits/qk-05-focus-cost-export/runs/red-missing-receipt.json` (expected RED failure until the future run receipt exists)
