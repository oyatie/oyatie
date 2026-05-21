# IP-043 ITSM performance-analytics

Service: itsm
ChangeSet scope: microservices/itsm/IP-043-performance-analytics.md
Counterparts displaced: ServiceNow Performance Analytics, Atlassian Reports for JSM, Freshservice Analytics Plus
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Surface KPI dashboards for service-desk leadership: MTTR, MTBF, change success rate, CAB throughput, CSAT, SLA breach rate, CMDB health, AI deflection rate.
- O-002: Route data via the analytics + observability µservices; ITSM owns the catalog of KPIs but not the data warehouse.
- O-003: Provide benchmark comparison views against industry standards.

## KPI packs
- KP-001: incident_lifecycle (MTTR / MTBF / volume).
- KP-002: change_success_rate (successful_changes / total).
- KP-003: cab_throughput (changes_approved_per_period).
- KP-004: csat_trend (rolling 30d).
- KP-005: sla_breach_rate (breach_count / total_clocks).
- KP-006: cmdb_health_score (% verified within retention window).
- KP-007: ai_deflection_rate (deflected / requester_intents).

## Visualization kinds
- VK-001: Scorecard.
- VK-002: Trend line.
- VK-003: SLA compliance heatmap.
- VK-004: Waterfall change throughput.
- VK-005: Benchmark compare (vs ServiceNow / JSM / Freshservice industry medians).

## Tenant invariants
- T-001: KPI computation runs in the analytics µservice; ITSM only emits raw events.
- T-002: Cross-tenant aggregation forbidden.

## Tenant-class behavior
- TC-001: demo_trial: performance analytics disabled per ADR-0331.
- TC-002: paid: enabled with 1y historical retention.

## Acceptance evidence
- E-001: openslo: kpi_render_p95_ms ≤ 1500.
- E-002: cargo test for the KPI event emitter.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: performance analytics.
- REST/API anchor: kpi snapshot route.
- Policy anchor: policy/performance-analytics-authorization.cedar.
- SLO/dashboard anchor: kpi render p95 <= 1500ms.
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
| ServiceNow ITSM | ITSM analytics dashboards under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

### Additional promotion guard
- Promotion also requires an owner-named residual-risk row when any referenced policy file is not yet implemented.
- Promotion also requires one synthetic-tenant fixture and one cross-tenant denial fixture.
- Promotion also requires a rollback flag name that can be toggled without disabling core incident handling.
- Promotion also requires evidence that ServiceNow, Jira Service Management, and Freshservice ids remain aliases.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-043-performance-analytics.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-043-performance-analytics.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
