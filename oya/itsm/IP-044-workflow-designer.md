# IP-044 ITSM workflow-designer

Service: itsm
ChangeSet scope: microservices/itsm/IP-044-workflow-designer.md
Counterparts displaced: ServiceNow Flow Designer, Jira Service Management Automation, Freshservice Orchestration Center
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0247, ADR-0249, ADR-0263, ADR-0328

## Objective
- O-001: Provide a visual no-code workflow builder that publishes templates to the workflow-engine µservice.
- O-002: Sustain 800 workflows/sec throughput vs ServiceNow Flow Designer ~120/sec (7× advantage).
- O-003: Allow tenants to publish workflow templates to the Oyatie marketplace with DealSet settlement.

## Editor primitives
- EP-001: Trigger event (ticket created / changed / closed; external webhook; scheduled cron).
- EP-002: Decision branch (Cedar-evaluated condition).
- EP-003: Approval gate (Cedar-policy with named approvers).
- EP-004: Call substrate action (any allow-listed substrate API).
- EP-005: Call marketplace action pack (any installed marketplace listing).
- EP-006: Emit audit event.
- EP-007: Rollback compensation step (paired with each mutating action).

## Cross-cell orchestration
- CC-001: ITSM owns template authoring + storage.
- CC-002: workflow-engine µservice owns runtime + per-cell execution.
- CC-003: Foundry absorption per ADR-0247: workflow templates live in the workflow-engine µservice (no separate foundry runtime).

## Tenant invariants
- T-001: Each template is tenant-scoped; marketplace publication is opt-in.
- T-002: Cedar policy gates template publication.

## Marketplace integration
- MI-001: Templates can be listed in the marketplace; revenue-share applies per ADR-0314 DealSet settlement.

## Tenant-class behavior
- TC-001: demo_trial: workflow execution cap 1000/month per ADR-0331.
- TC-002: paid: per-usage metered as `workflow_executions_per_month`.

## Performance claim
- PC-001: sustained 800 workflows/sec.
- PC-002: ServiceNow Flow Designer ~120/sec.
- PC-003: Advantage factor: 7.

## Acceptance evidence
- E-001: openslo: workflow_designer_publish_p95_ms ≤ 600.
- E-002: workflow-engine load test: ≥ 800 wf/s sustained.
- E-003: cargo test for template state machine.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: workflow designer.
- REST/API anchor: workflow template publish route.
- Policy anchor: policy/workflow-designer-authorization.cedar.
- SLO/dashboard anchor: workflow publish and 800 wf/s throughput.
- Counterpart pressure: ServiceNow ITSM, Jira Service Management, and Freshservice all expose this class of ITSM surface; Oyatie closes the gap with tenant scope, Cedar, audit-chain evidence, and pack overlays.

### Implementation detail that must exist before promotion
- Define the command DTO with tenant_id, principal_id, audience_type, purpose, data_class, and audit_event_class fields.
- Bind the command to a Capability or an adjacent bounded-context action instead of adding a free-form route.
- Evaluate Cedar before any repository write, external provider call, workflow-engine dispatch, or audit success event.
- Emit an ADR-0263 audit event for success and a distinct denial event for policy, budget, residency, or capacity refusal.
- Carry home_cell, jurisdiction_code, and pack ids through the request context before data leaves the home cell.
- Use existing ITSM source files as the first implementation surface: src/domain/mod.rs, src/usecase/mod.rs, src/adapter/mod.rs, and tests/integration.rs.
- Keep source-system identifiers from ServiceNow, Jira, or Freshservice as aliases only; they cannot authorize Oyatie actions.
- Preserve demo_trial and paid behavior from manifest.json; demo caps must be tested separately from paid behavior.
- Add dashboard evidence before calling the feature production-ready.
- Add rollback that disables this capability without disabling incident open, change approval, SLA recompute, or audit publication.

### Acceptance evidence to add
- Unit or integration test proving the clean allow path succeeds for a synthetic tenant.
- Negative test proving cross-tenant access is denied before mutation.
- Negative test proving missing pack/residency context fails closed where the capability touches protected data.
- Contract test or schema validation for the REST/event/RPC surface used by this capability.
- Audit replay check proving one success event is emitted for each successful command.
- Dashboard or OpenSLO check proving latency/error-budget evidence is available.
- Counterpart parity row explaining the ServiceNow/Jira/Freshservice behavior being displaced.
- Residual-risk note if a referenced runtime module, route, or Cedar entity is not yet implemented.

### Counterpart comparison
| Counterpart | Why this IP is not a clone |
|---|---|
| ServiceNow ITSM | workflow design and automation under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-044-workflow-designer.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-044-workflow-designer.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-044-workflow-designer.md` matched [`metered`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-044-workflow-designer.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
