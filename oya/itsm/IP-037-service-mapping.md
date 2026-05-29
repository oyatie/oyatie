# IP-037 ITSM service-mapping

Service: itsm
ChangeSet scope: microservices/itsm/IP-037-service-mapping.md
Counterparts displaced: ServiceNow Service Mapping (top-down), BMC Helix Service Mapping, Atlassian Insight Service Graph
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Compute top-down dependency graphs from declared business services through application + middleware + infrastructure layers down to CMDB CIs.
- O-002: 3-hop traversal p99 ≤ 380ms vs ServiceNow ~1400ms (advantage factor 3.7×).
- O-003: Detect relation drift; trigger remediation tickets via the workflow-designer capability.

## Mapping operations
- MO-001: `declare_business_service` — name + owner + criticality.
- MO-002: `traverse_application_layer` — find owning application CIs.
- MO-003: `traverse_middleware_layer` — find DBs, queues, caches.
- MO-004: `traverse_infrastructure_layer` — find hosts, networks.
- MO-005: `render_dependency_graph` — produce a directed acyclic graph snapshot.
- MO-006: `detect_relation_drift` — compare current vs last snapshot; emit drift events.

## Graph storage
- GS-001: PostgreSQL JSONB plus recursive CTE traversal (per `capacity-model.md`).
- GS-002: Per-tenant graph isolation at the schema level.
- GS-003: Drift events are projected into the ontology µservice for downstream consumers.

## Tenant invariants
- T-001: Graph traversal is tenant-bounded; no edge crosses tenants.
- T-002: Business service declaration requires `audience_type=ITIL_OPERATOR`.

## Tenant-class behavior
- TC-001: demo_trial: service-mapping disabled per ADR-0331.
- TC-002: paid: enabled; per-usage metered as `service_map_traversals`.

## Performance plan
- PP-001: Index per-tenant on (source_ci_id, target_ci_id, relation_kind).
- PP-002: Cache 3-hop neighborhoods per business service (TTL 60s).
- PP-003: Cold p99 ≤ 380ms is the contract; warm p99 ≤ 60ms.

## Acceptance evidence
- E-001: openslo: service_mapping_3hop_p99_ms ≤ 380 (paid).
- E-002: cargo test for the recursive traversal correctness.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: service mapping.
- REST/API anchor: service-map traversal query.
- Policy anchor: policy/service-mapping-authorization.cedar.
- SLO/dashboard anchor: 3-hop p99 <= 380ms.
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
| ServiceNow ITSM | dependency mapping under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-037-service-mapping.md` matched [`p99`, `SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-037-service-mapping.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-037-service-mapping.md` matched [`metered`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-037-service-mapping.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
