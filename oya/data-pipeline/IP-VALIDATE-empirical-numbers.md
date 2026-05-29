# IP-VALIDATE Data Pipeline empirical performance numbers validation

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.1, §3.8
Source artifact: microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md
Binding ADRs: ADR-0105, ADR-0130, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0248, ADR-0251, ADR-0252, ADR-0253, ADR-0321, ADR-0329, ADR-0330, ADR-0331
Memory anchors: `feedback_quality_performance_scalability_bar`, `feedback_no_silent_regression`, `feedback_verify_deliverables_not_just_line_count`

## Objective
- Validate that every empirical performance number declared in `microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md` is either (a) sourced directly from a present OpenSLO file, (b) sourced directly from ADR-MS-001 §Decision rows, (c) derived from the capacity-model.md design, or (d) flagged with a TODO-PROVE marker pointing at a deferred load test.
- Refuse silent regression: if any benchmark number is changed downward (less ambitious) without an accompanying ADR amendment, the validator flags it.
- Convert prose-numeric claims into machine-readable rows that CI can parse.
- Establish an audit trail: every number must trace back to a primary source.
- Provide a re-validation gate that runs on every PR touching performance-benchmark-numbers-2026-05-20.md, slos/*.openslo.yaml, ADR-MS-001, or capacity-model.md.

## Prerequisites
- Read `microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md` end-to-end.
- Read each OpenSLO file under `microservices/data-pipeline/slos/`.
- Read `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md` §Decision rows.
- Read `microservices/data-pipeline/capacity-model.md`.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.1 and §3.8.

## Empirical-number row schema
Each empirical number in `performance-benchmark-numbers-2026-05-20.md` must produce a row with:
- `claim_id`: stable identifier (e.g., `PERF-DP-CONN-001`).
- `claim_kind`: connector_sync_latency | schema_migration_turnaround | transformation_runtime | lineage_query_latency | monitoring_delivery_latency | backfill_replay_runtime | dead_letter_drain_latency | quality_gate_latency | audit_emission_lag | cedar_decision_latency.
- `metric_name`: e.g., `ingest_freshness_p95_lag_seconds`.
- `p50_target` | `p95_target` | `p99_target` | `availability_target` (numeric).
- `source_kind`: `openslo` | `adr_ms_001` | `capacity_model` | `derived` | `todo_prove`.
- `source_anchor`: file path + section/line reference.
- `regression_guard_baseline`: previous canonical value (if any).
- `provenance_hash`: hash of `source_kind + source_anchor + p50 + p95 + p99 + availability` for change detection.

## Validation rules
1. Every empirical number row must have a non-null `source_anchor`.
2. Every `openslo` source must resolve to an existing `microservices/data-pipeline/slos/*.openslo.yaml` file whose `target` field matches the row's availability_target.
3. Every `adr_ms_001` source must resolve to a §Decision row in ADR-MS-001.
4. Every `capacity_model` source must resolve to a section in capacity-model.md.
5. Every `derived` source must declare which primitives (openslo + adr_ms_001 + capacity_model) it derives from in a `derivation_chain` field.
6. Every `todo_prove` source must declare a `target_proof_milestone` (e.g., `Wave-15B-load-test`).
7. If `regression_guard_baseline` exists and the new value is worse than baseline by more than `regression_tolerance_pct` (default 5%), the validator fails the PR.
8. Numbers must comply with the hyperscaler-grade memory: connector sync latency p95 must not exceed Fivetran 5-minute preset benchmark for the `interactive` workload class; schema migration p95 must not exceed 60s; transform p95 must not exceed 30 minutes for the standard incremental case; lineage query p95 must not exceed 5s; audit emission p99 must not exceed the SLO declared in `slos/audit-emission-lag.openslo.yaml`.
9. Per-tenant_class no-regression: numbers for paid tenants must not be looser than demo_trial (demo_trial may have explicit quota caps but the per-row latency budget is the same).
10. Per pack-overlay: KR-PIPA + HIPAA + GDPR + PCI-DSS-L1-v4 may impose stricter latency ceilings on cross-cell movements; the validator allows pack-stricter, never pack-looser.

## Validation procedure
1. Parse `performance-benchmark-numbers-2026-05-20.md` extracting each numeric claim and its surrounding prose.
2. For each claim, attempt to attribute a `source_kind`:
   - If the claim references an SLO target (e.g., "0.995 availability"), match against `slos/*.openslo.yaml`.
   - If the claim cites ADR-MS-001, match against §Decision rows.
   - If the claim cites capacity-model.md, match against capacity sections.
   - Otherwise mark as `derived` (with derivation chain) or `todo_prove`.
3. Compute provenance_hash for each row.
4. Compare each row against `regression_guard_baseline` from the prior validated state (stored in `evidence/perf-validation-baseline.json`).
5. Emit a validation report `evidence/perf-validation-report-2026-05-21.json` with pass/fail per row.
6. On any fail, refuse PR merge and link the runbook `runbooks/perf-claim-regression.md`.

## Empirical claims to validate (initial pass)
- Ingest freshness p95 lag: must trace to `slos/local-ingest-freshness.openslo.yaml`.
- Schema drift detection latency p95: must trace to `slos/local-schema-drift-latency.openslo.yaml`.
- Lineage capture lag p95: must trace to `slos/local-lineage-capture.openslo.yaml`.
- Transform runtime p95 per incremental: must trace to `slos/local-transform-latency.openslo.yaml`.
- Quality null-rate gate latency p95: must trace to `slos/local-quality-null-rate.openslo.yaml`.
- Dead-letter rate ceiling: must trace to `slos/local-deadletter-rate.openslo.yaml`.
- Replay freshness p95: must trace to `slos/replay-freshness.openslo.yaml`.
- Read latency p95: must trace to `slos/read-latency.openslo.yaml`.
- Write latency p95: must trace to `slos/write-latency.openslo.yaml`.
- Availability: must trace to `slos/availability.openslo.yaml`.
- Audit emission lag p99: must trace to `slos/audit-emission-lag.openslo.yaml`.
- Cedar policy decision latency p99: must trace to `slos/policy-decision-latency.openslo.yaml`.
- Destination commit latency p95 (post-IP-031): must trace to forthcoming `slos/local-destination-commit-latency.openslo.yaml`.
- Schedule fire jitter p95 (post-IP-032): must trace to forthcoming `slos/local-schedule-fire-jitter.openslo.yaml`.
- Semantic metric read latency p95 (post-IP-033): must trace to forthcoming `slos/local-semantic-metric-read-latency.openslo.yaml`.
- Exposure impact notify lag p95 (post-IP-034): must trace to forthcoming `slos/local-exposure-impact-notify-lag.openslo.yaml`.
- Materialization refresh success rate (post-IP-035): must trace to forthcoming `slos/local-materialization-refresh-success-rate.openslo.yaml`.
- Package install latency p95 (post-IP-036): must trace to forthcoming `slos/local-package-install-latency.openslo.yaml`.
- CDK publish latency p95 (post-IP-037): must trace to forthcoming `slos/local-cdk-publish-latency.openslo.yaml`.

## Vendor comparison rules
- Every comparison to Fivetran / Airbyte / dbt Cloud must cite a public source page or community post.
- Oyatie numbers may be more ambitious (faster / higher availability) than vendor numbers; the validator does not require parity, only honesty about source.
- Where vendor numbers are not publicly available, the comparison row must declare `vendor_number_source = unavailable_public` and use `community_reported` only with a stable citation.

## Implementation steps
- Add `evidence/perf-validation-baseline.json` capturing the current 2026-05-21 state of every row.
- Add `evidence/perf-validation-report-2026-05-21.json` (output artifact).
- Add a CI check `oya-governance-perf-claim-attribution` to `.github/workflows/` that runs the validator on every PR touching the listed files.
- Add a Rust validator binary `bin/oya-perf-validator` (Rust-strict per the no-Python rule) reading the markdown + yaml + ADR sources.
- Add runbook `runbooks/perf-claim-regression.md`.
- Add a section to `coherence-audit-2026-05-20.md` successor wave (or remediation-notes) capturing that this validator gate is active.
- Add `oya.data.pipeline.perf_claim.validation_passed` and `.validation_failed` AsyncAPI events.

## Policy gates
- Cedar denies perf-claim publish if any row has `source_kind = todo_prove` and no `target_proof_milestone`.
- Cedar denies perf-claim publish if regression_guard_baseline fails by more than tolerance.
- Cedar denies perf-claim publish if vendor comparison cites an unstable source.
- Cedar denies perf-claim publish during audit-chain outage.

## Failure handling
- If validator fails to parse a numeric row, emit `oya.data.pipeline.perf_claim.parse_failed` and refuse PR.
- If SLO file referenced is missing, refuse PR and link `runbooks/perf-claim-regression.md`.
- If ADR-MS-001 row referenced is missing, refuse PR.
- If derivation chain is incomplete, refuse PR.

## Tests and evidence
- Unit test: validator parses the canonical markdown correctly.
- Unit test: regression_guard_baseline comparison correct across tolerance window.
- Contract test: SLO file path resolution.
- Contract test: ADR-MS-001 §Decision row resolution.
- Replay test: validator emits identical output for unchanged input (idempotency).
- Audit test: validation_passed event includes the provenance_hash.

## Rollback
- Roll back perf-claim publish via amendment (append-only baseline).
- Restore prior provenance_hash from baseline.
- Link rollback to `runbooks/perf-claim-regression.md`.

## Acceptance criteria
- Every numeric claim in performance-benchmark-numbers-2026-05-20.md attributed to a primary source or flagged with TODO-PROVE.
- `bin/oya-perf-validator` exists in Rust.
- CI lane `oya-governance-perf-claim-attribution` exists.
- Baseline JSON committed.
- Runbook exists.
- Regression-guard catches downward changes.

## Citation map
- `microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md` (subject).
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.1, §3.8.
- `microservices/data-pipeline/slos/` (12 OpenSLO files).
- `microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md`.
- `microservices/data-pipeline/capacity-model.md`.
- `ADR-0130` agentic SLO-gated promotion.
- `ADR-0321` documentation-rigor.
- `ADR-0329` substance bar canonical sequence.
- Memory: `feedback_quality_performance_scalability_bar`.
- Memory: `feedback_no_silent_regression`.
- Memory: `feedback_verify_deliverables_not_just_line_count`.

## Operator review prompts
- Reviewer asks whether every empirical number has a primary source.
- Reviewer asks whether regression-guard baseline is current.
- Reviewer asks whether vendor comparisons are honest about source availability.
- Reviewer asks whether tenant_class consistency is preserved (paid no worse than demo_trial per row).
- Reviewer asks whether pack-overlay strictness is preserved.
- Reviewer asks whether TODO-PROVE markers have a milestone.
- Reviewer signs the validation report with the audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md:12` - - Validate that every empirical performance number declared in `microservices/data-pipeline/performance-benchmark-numbers-2026-05-20.md` is either (a) sourced directly...; `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md:20` - - Read each OpenSLO file under `microservices/data-pipeline/slos/`..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md:28` - - `claim_kind`: connector_sync_latency | schema_migration_turnaround | transformation_runtime | lineage_query_latency | monitoring_delivery_latency | backfill_replay_r...; `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md:44` - 8. Numbers must comply with the hyperscaler-grade memory: connector sync latency p95 must not exceed Fivetran 5-minute preset benchmark for the `interactive` workload....
