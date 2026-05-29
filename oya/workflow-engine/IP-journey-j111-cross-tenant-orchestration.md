---
doc_class: Implementation-Plan
ip_id: IP-journey-j111-cross-tenant-orchestration
journey_ref: docs/user-journeys/j111-staffing-agency-as-tenant-facilitator/
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0263-observability-emission-contract
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-tenant-hierarchy
  - ADR-0314-marketplace-universal-deal-settlement-substrate
planned_enforcement_ref: oya-governance-doc-rigor
---

# IP - workflow-engine role in j111: Staffing agency as tenant facilitator

Role: cross-tenant-orchestration.

Journey purpose: A staffing-agency tenant sources workers from Community, places them at KrampusCorp, ConstructionCo,
and HealthcareSystem-Megacorp, and receives Stripe facilitator commissions.

## Scope

workflow-engine owns only the cross-tenant-orchestration slice for j111. It does not absorb another service
responsibility, does not bypass Cedar, and does not write into another tenant-owned store without an explicit grant.

## Acceptance criteria

1. workflow-engine exposes or consumes the typed j111 contract without ad hoc string parsing.
2. Every state-changing path evaluates Cedar and records the permit id.
3. Every mutation emits an ADR-0263 observability event with audit_id linkage.
4. Rollback exists for each reversible state and pause exists for irreversible state.
5. Cross-tenant reads require explicit tenant pair and purpose.
6. Personal-tenant data is default-deny unless the personal tenant owner consents.
7. The implementation maps to one of the ADR-0105 canonical layers.
8. The test plan includes success, expired-permit, outage, and residency-hold cases.

## Bespoke journey rows

These rows replace the prior 68-row generated loop. Each row is grounded in `contracts/openapi/workflow-engine.yaml`, `contracts/asyncapi/workflow-events.yaml`, `contracts/proto/workflow-engine.proto`, `policy/tenant-scope.cedar`, `policy/spec-integrity.md` and the journey counterpart context.

| Row | Source trigger | Actor / Cedar probe | State effect and evidence | Counterpart equivalence |
|---|---|---|---|---|
| 01 placement workflow start | agency submits candidate placement for client. | `startWorkflowRun` pins agency, worker, and client tenant refs plus purpose. | WorkflowStarted provides placement correlation id. | matches Bullhorn placement workflow. |
| 02 candidate consent | worker accepts personal-to-work sharing consent. | `SignalWorkflowRun` advances only with consent grant id in payload. | StepCompleted records consent audit hash. | matches DocuSign consent envelope completion. |
| 03 client approval | client tenant approves placement. | Cedar verifies client operator can signal the placement resource. | StepCompleted carries client approval id and permit decision. | matches Workday requisition approval. |
| 04 assignment activation | identity activates client-scoped access. | workflow-engine emits event only; identity owns principal update. | audit seal links assignment id and run id. | matches Okta group assignment workflow. |
| 05 agency substitution | candidate withdraws and agency substitutes alternate. | run pauses, records substitution reason, and reuses idempotency key for alternate branch. | StepRetried evidence prevents duplicate placement. | matches Temporal child-workflow replacement. |
| 06 timesheet cadence | weekly timesheet reminder fires. | StepStarted timer event emits to tenant-hashed channel; subscriber idempotency is required. | deadline slack and event replay offset are evidence. | matches Bullhorn/Workday timesheet cadence. |
| 07 boundary denial | client tries to inspect agency margin notes. | tenant-scope Cedar denies non-resource-tenant read. | denial counter and audit event prove separation. | matches Auth0 organization claim boundary. |
| 08 debug replay | agency audits disputed placement date. | ReplayDebuggerBackend streams snapshots without resending access grants. | ReplaySession proves deterministic activation date. | matches Temporal replay. |

Rows deleted as un-grounded: the former 68 numbered deliverables rotated the same Change/Contract/Cedar/Observability/Failure/Verification labels across path numbers without additional source artifacts. The eight rows above preserve the grounded workflow states and remove speculative path fan-out.
## Dependencies and non-goals

- Depends on community through a typed contract only; no shared table or hidden callback is allowed.
- Depends on identity through a typed contract only; no shared table or hidden callback is allowed.
- Depends on payments through a typed contract only; no shared table or hidden callback is allowed.
- Depends on tenancy through a typed contract only; no shared table or hidden callback is allowed.

## Done evidence

- Journey README links this IP from docs/user-journeys/j111-staffing-agency-as-tenant-facilitator/README.md.
- Integration test plan names workflow-engine in at least one positive and one failure-injection case.
- Schema docs include the fields this service owns for j111.
- Multispectrum evidence records the doc-only change class.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j111-cross-tenant-orchestration.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j111-cross-tenant-orchestration.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
