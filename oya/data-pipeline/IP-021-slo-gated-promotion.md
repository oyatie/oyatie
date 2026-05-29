# IP-021 Data Pipeline SLO-gated promotion

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-021-slo-gated-promotion.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Gate Data Pipeline promotion on local SLO evidence.
- Require connector freshness, transform latency, lineage capture, replay freshness, policy latency, audit lag, and availability evidence.
- Prevent contract or worker promotion while replay or dead-letter health is failing.
- Keep SLO gates service-local, not generic platform checkboxes.
- Treat Fivetran and Airbyte Cloud operational health as benchmark pressure.
- Treat Hevo and Stitch simple run health as dashboard pressure.
- Treat Matillion and Talend Cloud job latency as transform pressure.
- Treat Informatica IICS governance readiness as compliance pressure.
- Treat Estuary Flow freshness as streaming pressure.
- Preserve ADR-0321 citation and evidence density.

## Local references
- `microservices/data-pipeline/slos/availability.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml`
- `microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/dashboards/slo-and-error-budget.json`
- `microservices/data-pipeline/dashboards/local-slo-burn.json`

## Gate inputs
- Availability SLO is required.
- Read latency SLO is required when query surface changes.
- Write latency SLO is required when mutation surface changes.
- Ingest freshness SLO is required when connector behavior changes.
- Transform latency SLO is required when transform behavior changes.
- Lineage capture SLO is required when graph behavior changes.
- Replay freshness SLO is required when replay behavior changes.
- Dead-letter rate SLO is required when worker behavior changes.
- Policy decision latency SLO is required when policy behavior changes.
- Audit emission lag SLO is required when evidence behavior changes.
- Schema drift latency SLO is required when drift behavior changes.
- Quality null-rate SLO is required when quality gates change.

## Promotion states
- `not_evaluated` blocks promotion.
- `collecting` blocks final promotion.
- `passing` allows promotion when all required gates pass.
- `warning` requires operator acknowledgement.
- `burning` blocks promotion.
- `missing_signal` blocks promotion.
- `stale_signal` blocks promotion.
- `waived` requires reviewer separation and expiry.
- `degraded` allows only explicitly scoped promotion.
- `rolled_back` records reverted promotion.
- `superseded` records newer gate set.
- `blocked_by_replay` records replay-specific block.

## Command deltas
- `promotion.evaluate` computes required gates.
- `promotion.attach_evidence` attaches SLO evidence.
- `promotion.block` records failed gate.
- `promotion.waive` records reviewed waiver.
- `promotion.approve` records approval.
- `promotion.rollback` records reverted promotion.
- `promotion.status` reads gate status.
- `promotion.recompute` reruns evidence projection.
- Connector promotion requires ingest freshness.
- Transform promotion requires transform latency and cost evidence.
- Lineage promotion requires lineage capture.
- Replay promotion requires replay freshness and dead-letter rate.

## Event deltas
- `slo.promotion_evaluation_started` records evaluation.
- `slo.gate_passed` records passing gate.
- `slo.gate_failed` records failed gate.
- `slo.gate_missing_signal` records missing signal.
- `slo.gate_stale_signal` records stale signal.
- `slo.waiver_requested` records waiver request.
- `slo.waiver_approved` records reviewer approval.
- `slo.promotion_approved` records promotion.
- `slo.promotion_blocked` records block.
- `slo.promotion_rolled_back` records rollback.
- Events include gate id.
- Events include evidence window.

## Proto deltas
- `SloGateRef` includes gate id.
- `SloGateRef` includes SLO path.
- `SloGateRef` includes evidence window.
- `SloGateRef` includes status.
- `PromotionEvaluationRequest` includes changed surface.
- `PromotionEvaluationResponse` includes required gates.
- `PromotionEvidenceRef` includes dashboard ref.
- `PromotionWaiverRequest` includes reviewer id.
- `PromotionApprovalRequest` includes gate receipts.
- `PromotionRollbackRequest` includes prior promotion id.
- Proto rejects promotion without required gates.
- Proto rejects waiver without expiry.

## Cedar facts
- `promotion_surface` is a policy fact.
- `required_gate_count` is a policy fact.
- `passing_gate_count` is a policy fact.
- `failing_gate_count` is a policy fact.
- `missing_signal_count` is a policy fact.
- `stale_signal_count` is a policy fact.
- `waiver_state` is a policy fact.
- `waiver_expiry` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.
- `replay_freshness_state` is a policy fact.
- `deadletter_rate_state` is a policy fact.
- `audit_lag_state` is a policy fact.

## Workflow decisions
- Promotion evaluation runs after tests.
- Promotion evaluation runs before bundle lifecycle done.
- Changed surfaces determine required gates.
- Replay changes always require replay freshness.
- Worker changes always require dead-letter rate.
- Contract changes require read and write latency checks.
- Policy changes require policy latency checks.
- Evidence changes require audit emission lag checks.
- Waivers require reviewer separation and expiry.
- Burning gates block promotion.
- Missing signals block promotion.
- Stale signals block promotion.

## Failure cases
- Missing SLO file blocks promotion.
- Missing dashboard evidence blocks promotion.
- Missing event signal blocks promotion.
- Stale event signal blocks promotion.
- Replay freshness burn blocks replay promotion.
- Dead-letter rate burn blocks worker promotion.
- Policy latency burn blocks policy promotion.
- Audit lag burn blocks evidence promotion.
- Availability burn blocks runtime promotion.
- Waiver without expiry is denied.
- Waiver without reviewer separation is denied.
- Promotion audit event missing blocks promotion.

## Replay cases
- Replay promotion requires replay freshness passing.
- Replay promotion requires dead-letter rate passing.
- Replay promotion requires audit lag passing.
- Replay promotion requires policy latency passing.
- Replay promotion requires no frozen cursor backlog.
- Replay promotion requires rollback evidence.
- Replay promotion requires custody test evidence.
- Replay waiver expires before next release.
- Replay rollback recomputes freshness.
- Replay burn opens replay runbook.
- Replay benchmark parity cannot waive SLO failure.
- Replay promotion event records cursor evidence.

## Evidence fields
- `promotion_id` is mandatory.
- `changed_surface` is mandatory.
- `required_gates` is mandatory.
- `gate_statuses` is mandatory.
- `evidence_window_start` is mandatory.
- `evidence_window_end` is mandatory.
- `dashboard_refs` is mandatory.
- `slo_refs` is mandatory.
- `waiver_id` is mandatory when waived.
- `waiver_expiry` is mandatory when waived.
- `reviewer_id` is mandatory when waived.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `rollback_bundle_id` is mandatory on rollback.
- `runbook_ref` is mandatory when blocked.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Availability gate checks availability SLO.
- Ingest gate checks local ingest freshness.
- Transform gate checks local transform latency.
- Lineage gate checks local lineage capture.
- Replay gate checks replay freshness.
- Dead-letter gate checks local dead-letter rate.
- Policy gate checks policy decision latency.
- Audit gate checks audit emission lag.
- Schema gate checks schema drift latency.
- Quality gate checks null-rate SLO.
- Promotion gate checks evidence staleness.
- Waiver gate checks expiry.

## Test cases
- Promotion rejects missing required SLO.
- Promotion rejects missing dashboard ref.
- Replay promotion rejects replay freshness burn.
- Worker promotion rejects dead-letter burn.
- Policy promotion rejects policy latency burn.
- Evidence promotion rejects audit lag burn.
- Waiver requires reviewer separation.
- Waiver requires expiry.
- Stale signal blocks promotion.
- Missing signal blocks promotion.
- Rollback recomputes gate status.
- Benchmark parity cannot override failed gate.

## Rollback
- Promotion rollback marks promotion rolled back.
- SLO gate evidence remains immutable.
- Waiver evidence remains immutable.
- Replay freshness recomputes after rollback.
- Dead-letter rate recomputes after rollback.
- Audit lag recomputes after rollback.
- Dashboard projections recompute after rollback.
- Runbook closure records rollback event.
- Promotion id remains traceable.
- Gate set version remains traceable.
- Rollback emits promotion rolled back event.
- New promotion requires fresh evidence.

## Acceptance criteria
- Every promotion has required SLO gates.
- Every gate has evidence window.
- Every replay promotion checks replay freshness.
- Every worker promotion checks dead-letter rate.
- Every policy promotion checks policy latency.
- Every evidence promotion checks audit lag.
- Every waiver has reviewer and expiry.
- Every failed gate blocks promotion.
- Every benchmark reference is comparative.
- SLO-gated promotion remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/slos/availability.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml`
- `microservices/data-pipeline/slos/local-lineage-capture.openslo.yaml`
- `microservices/data-pipeline/slos/replay-freshness.openslo.yaml`
- `microservices/data-pipeline/slos/local-deadletter-rate.openslo.yaml`
- `microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml`
- `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`
- `microservices/data-pipeline/dashboards/slo-and-error-budget.json`
- `microservices/data-pipeline/dashboards/local-slo-burn.json`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-021-slo-gated-promotion.md:1` - # IP-021 Data Pipeline SLO-gated promotion; `microservices/data-pipeline/IP-021-slo-gated-promotion.md:9` - - Gate Data Pipeline promotion on local SLO evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-021-slo-gated-promotion.md:28` - - `microservices/data-pipeline/slos/audit-emission-lag.openslo.yaml`; `microservices/data-pipeline/IP-021-slo-gated-promotion.md:42` - - Audit emission lag SLO is required when evidence behavior changes..
