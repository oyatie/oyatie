# IP-018 Healthcare Integration Capacity Admission Control

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-018-capacity-admission-control.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: tenant, cell, route, and worker admission control
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/capacity-model.md
- microservices/healthcare-integration/cost-budget.md
- microservices/healthcare-integration/multi-region.md
- microservices/healthcare-integration/failure-modes.md
- microservices/healthcare-integration/IP-016-backfill-replay-worker.md
- microservices/healthcare-integration/IP-017-cost-budget-enforcer.md
- microservices/healthcare-integration/dashboards/tenant-cost-and-capacity.json
- microservices/healthcare-integration/dashboards/local-domain-throughput.json
- microservices/healthcare-integration/runbooks/hl7-queue-backlog.md
- microservices/healthcare-integration/slos/read-latency.openslo.yaml
- microservices/healthcare-integration/slos/write-latency.openslo.yaml
- microservices/healthcare-integration/slos/replay-freshness.openslo.yaml
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP defines capacity admission control for healthcare-integration.
- Admission control prevents one tenant, source system, route group, replay job, or provider outage from consuming the whole clinical interoperability service.
- It protects interactive FHIR reads.
- It protects urgent HL7 routes.
- It protects emergency-services bypass.
- It protects audit-chain evidence.
- It throttles elective replay.
- It cooperates with cost-budget enforcement.
- It respects residency pack overlays and cell-local capacity.
- It gives B2B leaders a scale story that beats middleware queues and manual interface tuning.
- It follows ADR-0105 by keeping admission decisions in application/usecase and worker orchestration boundaries.
- It follows ADR-0243 and ADR-0244 by preserving tenant and policy dimensions in every capacity decision.
- It follows ADR-0253-amendment by treating HTTP/3 and transport degradation as capacity signals.
- It follows ADR-0321 documentation depth without editing ADR-0321.

## 2. B2B Leader Problem
- Clinical integration workloads are bursty.
- EHR batch exports can arrive all at once.
- HL7 destinations can ACK slowly.
- Patient-match review can bottleneck replay.
- Audit-chain latency can block safe completion.
- Regional failover can reduce available payload-processing cells.
- Enterprise tenants need guaranteed slices for urgent workflows.
- SMB tenants need defaults that keep them from being starved by larger tenants.
- SRE teams need an admission decision that is explainable and reversible.
- Product leaders need operational scalability that does not depend on per-connector manual tuning.

## 3. Admission Dimensions
- Tenant id.
- Cell id.
- Region id.
- Jurisdiction code.
- Capability id.
- Source system id.
- Destination system id.
- Route group.
- Data class.
- Residency overlay id.
- Priority class.
- Workflow id.
- Replay job id.
- Patient-match queue id.
- Audit-chain lag.
- Provider credential pool.
- Transport protocol state.
- Budget decision id.

## 4. Priority Classes
- `emergency` for emergency-services bypass and urgent minimum necessary reads.
- `clinical_interactive` for clinician-facing reads and small writes.
- `clinical_route` for operational HL7 and FHIR exchange routes.
- `compliance` for mandated evidence, audit, and regulator exports.
- `repair` for corrective replay and provenance repair.
- `migration` for planned backfill.
- `elective` for tenant-requested non-urgent imports, analytics exports, and dashboard-heavy jobs.
- `background` for maintenance, compaction, and low-risk reconciliation.
- Emergency can preempt elective.
- Compliance can preempt migration.
- Repair can preempt elective.
- No priority can bypass tenant scope, policy, residency, or audit evidence.

## 5. Scope
- Define admission tokens for interactive operations.
- Define queue slots for HL7 routes.
- Define worker slots for replay.
- Define patient-match review slots.
- Define audit-chain backpressure behavior.
- Define provider credential pool limits.
- Define per-tenant fairness.
- Define per-cell fairness.
- Define source-system burst controls.
- Define destination ACK backlog controls.
- Define budget-aware throttling.
- Define dashboard evidence.
- Define runbook paths.

## 6. Non-Goals
- Do not replace Kubernetes scheduling.
- Do not replace HPA or PDB configuration.
- Do not create cross-service shared queues.
- Do not bypass Cedar policy.
- Do not bypass residency overlays.
- Do not create vendor-specific capacity paths.
- Do not block emergency access because elective queues are saturated.
- Do not edit ADR-0321.

## 7. Admission Decisions
- `admit` lets work begin immediately.
- `queue` places work behind same-class and fairness limits.
- `defer` returns a retryable response with reason and horizon.
- `throttle` reduces concurrency for an admitted worker.
- `preempt` pauses lower-priority work.
- `shed` rejects non-critical elective work.
- `pause_source` stops a noisy source system.
- `pause_destination` stops a slow destination route.
- `pause_replay_partition` freezes a replay partition at watermark.
- `emergency_reserve` spends reserved capacity.
- `degraded_admit` permits work with degraded evidence constraints.
- `deny` rejects work because safe capacity is unavailable.
- Every decision emits a capacity decision id.

## 8. Implementation Steps
- Add `CapacityAdmissionController` in application layer.
- Add `CapacityDecision` value object in domain/kernel.
- Add `AdmissionToken` for interactive work.
- Add `RouteSlot` for HL7 and FHIR exchange routes.
- Add `ReplayWorkerSlot` for backfill and repair.
- Add `ReviewSlot` for patient-match review.
- Add `AuditBackpressureSignal` from audit-chain latency.
- Add `CredentialPoolSignal` from provider credential adapters.
- Add `TransportPressureSignal` from HTTP/3 degradation and retry behavior.
- Add `BudgetThrottleSignal` from IP-017.
- Evaluate admission after policy and residency precheck for clinical work.
- Evaluate admission before provider credentials are leased.
- Evaluate admission before replay partition scheduling.
- Evaluate admission before patient-match review expansion.
- Emit capacity-admitted event.
- Emit capacity-queued event.
- Emit capacity-throttled event.
- Emit capacity-preempted event.
- Emit capacity-shed event.
- Emit capacity-denied event.

## 9. Fairness Rules
- Each tenant receives a baseline interactive reserve.
- Each tenant receives a bounded route reserve.
- Each tenant receives a bounded replay reserve.
- Emergency reserve is cell-local.
- Compliance reserve is pack-aware.
- Source-system burst cannot consume unrelated route groups.
- Destination ACK backlog cannot block unrelated destinations.
- Patient-match backlog cannot block clean identity rows.
- Audit-chain lag can slow terminal writes.
- Credential pool exhaustion can queue provider calls.
- Residency local-only cells cannot borrow payload slots from forbidden cells.
- Budget-throttled elective work loses priority before capacity-saturated clinical work.
- Large tenants can buy higher DealSet capacity but cannot starve baseline tenants.
- Reserved capacity must be observable.

## 10. Benchmark Displacement
- Redox displacement: Redox network operations can absorb integration bursts; this IP exposes tenant, cell, route, replay, and audit admission decisions directly.
- Rhapsody displacement: Rhapsody queues and throttles interface routes; this IP adds tenant fairness, residency-aware cell capacity, and emergency reserves.
- InterSystems IRIS for Health displacement: IRIS can scale data platform workloads; this IP keeps workload admission microservice-local, policy-aware, and DealSet-visible.
- Lyniate/Corepoint displacement: Corepoint operations often rely on interface-level tuning; this IP uses typed admission decisions and dashboards instead of manual route tuning.
- Mirth Connect displacement: Mirth channels can be throttled by channel configuration; this IP centralizes admission without arbitrary channel scripts.
- NextGate displacement: NextGate matching workload can saturate identity review; this IP isolates patient-match review slots from route and read capacity.
- Health Catalyst displacement: Health Catalyst analytics workloads can consume shared data platform resources; this IP deprioritizes elective analytics/export work under clinical pressure.
- Combined displacement: competitors queue, scale, or tune pieces; this IP admits work through clinical priority, tenant fairness, residency, budget, and audit backpressure together.

## 11. Emergency and Compliance Reserves
- Emergency reserve is reserved per cell.
- Emergency reserve cannot be spent by elective work.
- Emergency reserve still requires policy permit.
- Emergency reserve still requires residency decision.
- Emergency reserve emits reserve spend evidence.
- Compliance reserve is pack-aware.
- Compliance reserve supports audit exports and regulator evidence.
- Compliance reserve can preempt migration.
- Compliance reserve cannot be used for dashboard polling.
- Reserve exhaustion triggers degraded emergency behavior before denial.
- Degraded emergency behavior returns minimum necessary data.
- Degraded compliance behavior queues export with explicit horizon.

## 12. Replay Coupling
- Replay jobs request worker slots by tenant, source system, route group, and data class.
- Replay partitions carry watermarks.
- Capacity pause freezes watermark advancement.
- Capacity resume continues from last committed watermark.
- Replay DLQ growth reduces partition concurrency.
- Patient-match backlog reduces ambiguous-row concurrency.
- Audit lag reduces terminal write concurrency.
- Provider outage pauses affected destination partitions.
- Budget throttle reduces elective replay priority.
- Emergency repair replay can preempt migration replay.
- Replay freshness SLO tracks admission delay.
- Replay evidence records admission decisions.

## 13. Cost Coupling
- Budget warn does not change capacity by itself.
- Budget soft throttle lowers elective admission weight.
- Budget hard throttle causes elective shed.
- Budget approval pending queues work.
- Emergency budget exemption keeps emergency priority.
- Compliance budget exemption keeps compliance priority.
- Capacity saturation increases forecast variance.
- Capacity queue time is included in cost forecast evidence.
- DealSet higher capacity tier can increase route and replay limits.
- Higher capacity tier cannot bypass tenant baseline fairness for others.
- Settlement records include capacity decision id for chargeable work.

## 14. Observability
- Metric `healthcare_capacity_decisions_total` tracks admit, queue, defer, throttle, preempt, shed, and deny.
- Metric `healthcare_capacity_queue_depth` tracks tenant, route, replay, review, and audit queues.
- Metric `healthcare_capacity_wait_seconds` tracks queue wait by priority class.
- Metric `healthcare_capacity_reserve_spend_total` tracks emergency and compliance reserve use.
- Metric `healthcare_capacity_preemptions_total` tracks lower-priority pauses.
- Metric `healthcare_capacity_shed_total` tracks rejected elective work.
- Dashboard shows tenant cost and capacity together.
- Dashboard shows local domain throughput.
- Dashboard shows source-system burst pressure.
- Dashboard shows destination ACK backlog.
- Dashboard shows audit-chain lag.
- Dashboard shows patient-match backlog.
- Alerts fire on emergency reserve exhaustion.
- Alerts fire on sustained clinical route queue growth.
- Alerts fire on replay freshness breach.

## 15. Failure Modes
- Admission controller unavailable admits emergency with degraded evidence and queues elective work.
- Queue store unavailable denies elective and protects interactive traffic.
- Audit-chain lag pauses terminal writes.
- Provider credential pool exhaustion queues provider calls.
- Destination ACK backlog pauses affected route group.
- Source-system burst triggers source pause.
- Patient-match backlog queues ambiguous rows only.
- Residency local-only cell saturation queues payload work locally.
- Budget hard throttle sheds elective work.
- Transport retry storm reduces route concurrency.
- HPA lag triggers soft queue before shedding.
- Dashboard outage does not block admission decisions.

## 16. Rollback
- Disable shedding first.
- Preserve queueing and observe decisions.
- Restore prior tenant weights.
- Restore prior route limits.
- Restore prior replay partition limits.
- Drain emergency reserve audit evidence.
- Resume paused elective jobs gradually.
- Recompute capacity decision evidence from audit events.
- Notify SRE and tenant owners when rollback changes wait time.
- Keep clinical operations protected during rollback.

## 17. Acceptance Evidence
- The IP cites `capacity-model.md`.
- The IP cites cost-budget and multi-region docs.
- The IP cites replay and cost IPs.
- The IP defines admission dimensions.
- The IP defines priority classes.
- The IP defines decisions and fairness.
- The IP defines emergency and compliance reserves.
- The IP defines replay and cost coupling.
- The IP defines dashboard and SLO hooks.
- The IP includes all seven named benchmark families.
- The IP keeps ADR-0321 referenced but unmodified.

## 18. Done Criteria
- Interactive admission fixture covers admit, queue, and deny.
- Route fixture covers destination ACK backlog.
- Replay fixture covers partition pause and resume.
- Patient-match fixture isolates ambiguous rows.
- Emergency fixture uses reserve capacity.
- Compliance fixture uses pack-aware reserve.
- Budget fixture changes elective priority.
- Residency fixture blocks forbidden cell borrowing.
- Dashboard fixture shows decision and queue dimensions.
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
- Surface evidence: `microservices/healthcare-integration/IP-018-capacity-admission-control.md:16` - - microservices/healthcare-integration/multi-region.md; `microservices/healthcare-integration/IP-018-capacity-admission-control.md:205` - - Replay freshness SLO tracks admission delay..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-018-capacity-admission-control.md:15` - - microservices/healthcare-integration/cost-budget.md; `microservices/healthcare-integration/IP-018-capacity-admission-control.md:19` - - microservices/healthcare-integration/IP-017-cost-budget-enforcer.md.
