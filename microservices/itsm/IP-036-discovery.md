# IP-036 ITSM discovery

Service: itsm
ChangeSet scope: microservices/itsm/IP-036-discovery.md
Counterparts displaced: ServiceNow Discovery (separate product line), Jira Service Management Assets discovery, Freshservice Probe Discovery
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Auto-populate the CMDB by running tenant-scoped discovery agents against SNMP, WMI, SSH, Kubernetes label scrapers, and cloud-provider inventory APIs.
- O-002: Reduce CMDB stale-CI rate below 5% (vs ServiceNow Discovery ~15% on equivalent perimeter).
- O-003: Hand discovered candidates to the cmdb µservice for merge + reconciliation per IP-027.

## Discovery agent kinds
- DA-001: `snmp_scanner` — network gear inventory.
- DA-002: `wmi_scrape` — Windows endpoints.
- DA-003: `ssh_inventory` — Linux endpoints.
- DA-004: `k8s_label_scrape` — workload + service inventory.
- DA-005: `aws_resource_inventory` — EC2 / RDS / S3 / IAM.
- DA-006: `oci_resource_inventory` — Compute / Object / Vault.
- DA-007: `azure_resource_inventory_optional` — VMs / Storage / Key Vault.
- DA-008: `cloud_compute_internal` — Oyatie's own cloud-compute µservice as inventory source.

## Tenant invariants
- T-001: Discovery credentials are per-tenant; never shared.
- T-002: Discovered CIs are emitted only into the tenant's home cell.
- T-003: Cross-tenant CI merge is forbidden.

## Output format
- OF-001: Each candidate carries `tenant_id`, `discovery_principal_id`, `source_agent_kind`, `source_system_ref`, `confidence`, `discovered_at_epoch`.
- OF-002: Confidence is a closed enum: `high`, `medium`, `low`.
- OF-003: Low-confidence candidates require human approval before becoming a CI.

## Tenant-class behavior
- TC-001: demo_trial: discovery agents disabled per ADR-0331 (manual entry only; 200-CI cap).
- TC-002: paid: discovery agents enabled; per-usage metered as `cmdb_cis_discovered`.

## Implementation sequence
- I-001: Implement agent dispatch via worker layer.
- I-002: Wire Cedar gate for credential issuance.
- I-003: Emit candidates to cmdb µservice via AsyncAPI.
- I-004: Reconciliation runs in cmdb µservice (not in ITSM) per substrate boundary.

## Acceptance evidence
- E-001: openslo: discovery_agent_cycle_p95_min ≤ 30.
- E-002: cargo test for the candidate emission shape.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: discovery.
- REST/API anchor: discovery cycle worker.
- Policy anchor: policy/discovery-authorization.cedar.
- SLO/dashboard anchor: discovery cycle p95 <= 30m.
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
| ServiceNow ITSM | asset and service discovery under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-036-discovery.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-036-discovery.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-036-discovery.md` matched [`metered`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-036-discovery.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
