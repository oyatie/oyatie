# IP-038 ITSM predictive-intelligence

Service: itsm
ChangeSet scope: microservices/itsm/IP-038-predictive-intelligence.md
Counterparts displaced: ServiceNow Predictive Intelligence, Atlassian Intelligence (predictive), Freshservice Freddy Predict
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0255, ADR-0263, ADR-0328

## Objective
- O-001: Auto-classify incoming tickets, suggest assignment groups, predict priority, cluster similar incidents, and estimate resolution time.
- O-002: Tenant-isolated training data; per-tenant fine-tunes via the intelligence µservice.
- O-003: Every prediction carries top-k features explanation + audit trail.

## Inference kinds
- IK-001: `ticket_category_classification` — multi-class classifier.
- IK-002: `assignment_group_routing` — multi-class classifier with confidence threshold.
- IK-003: `priority_suggestion` — ordinal classifier.
- IK-004: `similar_incident_clustering` — embedding-based cluster join.
- IK-005: `resolution_time_estimate` — regressor.

## Training plan
- TP-001: Training data is tenant-private incident history.
- TP-002: Re-train cadence: nightly per tenant when ≥ 500 new tickets accumulate.
- TP-003: Model versioning is tracked via the intelligence µservice model registry.

## Refusal + safety
- RS-001: When the classifier confidence is below per-tenant threshold, predictive intelligence emits `low_confidence` and the ticket falls through to manual triage.

## Tenant invariants
- T-001: Predictive features used: ticket text, requester role, CI tag set. No principal personal data beyond requester id.
- T-002: Per-pack redaction applies before any training step.

## Tenant-class behavior
- TC-001: demo_trial: predictive disabled per ADR-0331.
- TC-002: paid: enabled; per-usage metered as `predictions_emitted`.

## Acceptance evidence
- E-001: openslo: prediction_p95_ms ≤ 500.
- E-002: classifier f1 ≥ 0.85 on tenant-held-out set.
- E-003: cargo test for the prediction emission shape.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: predictive intelligence.
- REST/API anchor: ticket classification route.
- Policy anchor: policy/predictive-intelligence-authorization.cedar.
- SLO/dashboard anchor: classification latency and explanation coverage.
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
| ServiceNow ITSM | assignment intelligence under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

### Additional promotion guard
- Promotion also requires an owner-named residual-risk row when any referenced policy file is not yet implemented.
- Promotion also requires one synthetic-tenant fixture and one cross-tenant denial fixture.
- Promotion also requires a rollback flag name that can be toggled without disabling core incident handling.
- Promotion also requires evidence that ServiceNow, Jira Service Management, and Freshservice ids remain aliases.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-038-predictive-intelligence.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-038-predictive-intelligence.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-038-predictive-intelligence.md` matched [`metered`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-038-predictive-intelligence.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
