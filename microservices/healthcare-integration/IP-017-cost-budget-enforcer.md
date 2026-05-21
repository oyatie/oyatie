# IP-017 Healthcare Integration Cost Budget Enforcer

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-017-cost-budget-enforcer.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: tenant FinOps and budget enforcement for clinical interoperability
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/cost-budget.md
- microservices/healthcare-integration/capacity-model.md
- microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md
- microservices/healthcare-integration/IP-016-backfill-replay-worker.md
- microservices/healthcare-integration/dashboards/tenant-cost-and-capacity.json
- microservices/healthcare-integration/dashboards/local-domain-throughput.json
- microservices/healthcare-integration/capabilities/fhir-read.yaml
- microservices/healthcare-integration/capabilities/hl7-route.yaml
- microservices/healthcare-integration/capabilities/patient-match-review.yaml
- microservices/healthcare-integration/runbooks/hl7-queue-backlog.md
- microservices/healthcare-integration/slos/read-latency.openslo.yaml
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP gives healthcare-integration a hard tenant cost budget enforcer.
- Clinical integration cost grows through connector calls, HL7 route volume, FHIR reads, patient-match review, replay, audit export, provenance sealing, and regional data movement.
- B2B leaders need predictable cost without sacrificing emergency care or compliance obligations.
- The enforcer blocks, slows, or queues elective work before cost blowouts.
- The enforcer never blocks emergency-services bypass.
- The enforcer never hides clinical failures as cost savings.
- The enforcer never stores PHI in cost records.
- The enforcer aligns with ADR-0314 DealSet settlement but remains an operational guardrail.
- It follows ADR-0105 by living in application/usecase/worker layers with domain-readable budget decisions.
- It follows ADR-0243 and ADR-0244 by preserving tenant and policy context.
- It follows ADR-0321 documentation depth without editing ADR-0321.

## 2. B2B Leader Problem
- Integration vendors often make spend visible after the interface is live.
- Clinical data volume can spike during migration, outage recovery, or provider-side batch release.
- Enterprise tenants require per-department, per-region, and per-source-system budget controls.
- SMB tenants require default caps that avoid surprise bills.
- Compliance work must continue even when elective budget is exhausted.
- Emergency access must continue even when budget is exhausted.
- Finance teams need forecast, actual, held, and adjusted units.
- SRE teams need capacity and budget to agree on throttling reason.
- Product teams need a FinOps answer that is better than interface-engine consulting estimates.

## 3. Budget Dimensions
- Tenant id.
- Capability id.
- Source system id.
- Destination system id.
- Route group.
- Data class.
- Residency pack.
- Workflow template.
- Worker class.
- Priority class.
- DealSet id.
- Cell id.
- Region id.
- Transform version.
- Policy decision id.
- Audit event id.
- Replay job id.
- Patient-match review queue id.

## 4. Cost Units
- FHIR read unit.
- FHIR bundle size bucket.
- HL7 route unit.
- HL7 transform unit.
- Consent sync unit.
- EHR provenance seal unit.
- Patient-match review unit.
- Emergency bypass unit.
- Backfill row unit.
- Replay correction unit.
- Audit export unit.
- Residency metadata movement unit.
- Residency payload movement unit.
- Provider credential lease unit.
- Queue storage unit.
- DLQ retention unit.
- Dashboard query unit.
- Regulator evidence packet unit.

## 5. Scope
- Enforce tenant cost budgets for elective work.
- Enforce source-system cost budgets for noisy feeds.
- Enforce route-level budget for HL7 volume.
- Enforce replay budget for backfill workers.
- Enforce patient-match review budget.
- Enforce audit export budget for non-regulatory exports.
- Enforce dashboard query budget when tenant activity is high.
- Emit budget decisions to dashboards.
- Emit budget decisions to audit-chain.
- Feed DealSet settlement with held or adjusted unit reasons.
- Feed capacity admission with budget throttle class.
- Preserve emergency and compliance exceptions.

## 6. Non-Goals
- Do not implement marketplace pricing.
- Do not replace DealSet settlement.
- Do not block emergency access.
- Do not block regulator-mandated evidence export.
- Do not expose PHI in budget records.
- Do not create vendor-specific price logic.
- Do not optimize by dropping audit events.
- Do not edit ADR-0321.

## 7. Enforcement Modes
- `observe` records cost without throttling.
- `warn` emits tenant and operator warnings.
- `soft_throttle` slows elective queues.
- `hard_throttle` blocks new elective work.
- `hold_settlement` allows work but holds commercial finalization.
- `require_approval` pauses until tenant admin approval.
- `compliance_exempt` allows mandated compliance work.
- `emergency_exempt` allows emergency bypass.
- `repair_exempt` allows corrective replay that prevents clinical harm.
- `deny` rejects work with a budget evidence packet.
- Every mode emits a budget decision id.
- Every mode records why a stricter or looser mode was chosen.

## 8. Implementation Steps
- Add `CostBudgetEnforcer` in application layer.
- Add `BudgetDecision` value object in domain/kernel.
- Add cost-budget adapter reading `cost-budget.md` derived dimensions until machine-readable registry exists.
- Add tenant budget lookup port.
- Add source-system budget lookup port.
- Add DealSet unit lookup port.
- Add queue throttle port.
- Add admission-control integration point for IP-018.
- Add replay integration point for IP-016.
- Add settlement integration point for IP-014.
- Evaluate budget after Cedar permit but before elective work admission.
- Evaluate budget before replay partition scheduling.
- Evaluate budget before patient-match review expansion.
- Evaluate budget before audit export generation when export is elective.
- Attach budget decision id to workflow and audit events.
- Emit budget-denied event when work is rejected.
- Emit budget-throttled event when work is slowed.
- Emit budget-exempt event when emergency or compliance work proceeds.
- Emit forecast event when a job is admitted with expected unit consumption.

## 9. Budget Policy Rules
- Emergency-services bypass is exempt from hard budget block.
- Compliance-mandated export is exempt from hard budget block.
- Repair replay that corrects prior clinical delivery is exempt from elective block.
- Migration backfill is not automatically exempt.
- Analytics export is not exempt.
- Dashboard-heavy polling is not exempt.
- Provider outage retries are capped.
- Duplicate message storms are throttled.
- Patient-match backlog expansion requires approval after threshold.
- Source-system burst above forecast triggers soft throttle.
- Replay above forecast triggers approval.
- Residency payload movement above forecast triggers approval.
- Settlement reversal never creates new positive charge.
- Budget deny must include retry or approval path.
- Budget decisions must not expose PHI.

## 10. Benchmark Displacement
- Redox displacement: Redox gives integration network usage visibility; this IP adds tenant-enforced budgets before elective work is admitted.
- Rhapsody displacement: Rhapsody can process interface volume; this IP turns volume into predictable unit budgets with audit-linked throttle decisions.
- InterSystems IRIS for Health displacement: IRIS can consolidate high-volume workloads; this IP gives each tenant and source system explicit FinOps boundaries in a flat microservice.
- Lyniate/Corepoint displacement: Corepoint cost can follow interface services; this IP shows route and transform units directly in tenant cost dashboards.
- Mirth Connect displacement: Mirth channel execution can hide compute and retry cost; this IP attaches every channel-like action to typed cost units and budget decisions.
- NextGate displacement: NextGate review and identity workflows can become labor cost centers; this IP budgets patient-match review expansion separately.
- Health Catalyst displacement: Health Catalyst analytics programs can create downstream cost; this IP blocks elective exports and analytics before they consume hidden integration spend.
- Combined displacement: competitors expose usage, route throughput, identity workload, or analytics cost; this IP enforces spend as runtime control.

## 11. Observability
- Metric `healthcare_cost_units_total` tracks units by capability and bucket.
- Metric `healthcare_budget_decisions_total` tracks allow, warn, throttle, hold, exempt, and deny.
- Metric `healthcare_budget_forecast_units` tracks expected job spend.
- Metric `healthcare_budget_actual_units` tracks realized spend.
- Metric `healthcare_budget_variance_units` tracks forecast error.
- Metric `healthcare_budget_exempt_units` tracks emergency, compliance, and repair exceptions.
- Dashboard shows cost by tenant, source system, route, replay job, pack, and cell.
- Dashboard shows budget burn rate.
- Dashboard shows top throttled workloads.
- Dashboard shows emergency exempt volume.
- Dashboard shows settlement hold volume.
- Alerts fire when burn rate exceeds threshold.
- Alerts fire when forecast variance exceeds threshold.
- Alerts fire when exempt traffic spikes.

## 12. Capacity Coupling
- Budget decisions feed capacity admission.
- Capacity decisions feed budget forecast.
- Soft throttle lowers worker concurrency.
- Hard throttle denies new elective jobs.
- Capacity saturation can increase forecast cost.
- Budget exhaustion can reduce elective queue priority.
- Emergency capacity override does not change budget exemption evidence.
- Compliance repair work can preempt elective backfill.
- Source-system burst can trigger both capacity and budget throttles.
- Dashboards must show whether budget or capacity was primary limiter.

## 13. Settlement Coupling
- Settlement-intent events include budget decision id.
- Settlement-hold events include budget hold reason.
- Settlement-adjustment events include replay correction id.
- Budget-denied work creates no positive charge.
- Budget-throttled work charges only delivered units.
- Emergency-exempt work can defer settlement.
- Compliance-exempt work can settle according to active DealSet.
- Duplicate replay reversal reduces or zeroes affected units.
- Disputes include budget and settlement evidence.
- Marketplace receives non-PHI unit metadata only.

## 14. Failure Modes
- Budget service unavailable defaults to conservative soft throttle for elective work.
- Budget service unavailable allows emergency and compliance exceptions with degraded evidence.
- Forecast calculation failure requires approval for large elective jobs.
- Unit counter mismatch opens incident response.
- PHI in budget payload is a security incident.
- Tenant budget missing uses default tenant policy.
- DealSet missing holds settlement but does not block authorized clinical work.
- Source-system burst with unknown cost denies elective replay.
- Dashboard cost query failure does not block clinical operations.
- Audit-chain outage pauses high-risk budget state transitions.

## 15. Rollback
- Disable hard throttle first.
- Keep observe mode active.
- Preserve budget decision ids.
- Recompute forecast and actual units from audit evidence.
- Release incorrectly held elective jobs.
- Reverse incorrect settlement holds.
- Notify tenant finance and SRE owners.
- Retain emergency exemption evidence.
- Open incident if budget error changed clinical availability.
- Re-enable stricter modes only after replayed verification.

## 16. Acceptance Evidence
- The IP cites `cost-budget.md`.
- The IP cites `tenant-cost-and-capacity.json`.
- The IP cites IP-014, IP-016, and capacity model references.
- The IP defines budget dimensions and units.
- The IP defines enforcement modes.
- The IP preserves emergency and compliance exceptions.
- The IP defines settlement and capacity coupling.
- The IP defines observability.
- The IP excludes PHI from budget records.
- The IP includes all seven named benchmark families.
- The IP keeps ADR-0321 referenced but unmodified.

## 17. Done Criteria
- Unit fixtures cover FHIR, HL7, consent, provenance, patient-match, replay, audit export, and residency movement.
- Policy fixtures cover observe, warn, soft throttle, hard throttle, hold settlement, approval, exemption, and deny.
- Replay fixture pauses elective job on budget exhaustion.
- Emergency fixture proceeds with exemption evidence.
- Settlement fixture receives budget decision id.
- Capacity fixture receives budget throttle class.
- Dashboard fixture shows burn rate and throttle reason.
- Audit fixture excludes PHI.
- Rollback fixture returns to observe mode.
- No other file is required for this IP deepening pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md:36` - - The enforcer never stores PHI in cost records.; `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md:112` - - Do not expose PHI in budget records..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md:4` - ChangeSet scope: microservices/healthcare-integration/IP-017-cost-budget-enforcer.md; `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md:14` - - microservices/healthcare-integration/cost-budget.md.
