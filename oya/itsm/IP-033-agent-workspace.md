# IP-033 ITSM agent-workspace

Service: itsm
ChangeSet scope: microservices/itsm/IP-033-agent-workspace.md
Counterparts displaced: ServiceNow Agent Workspace (Configurable Workspace), Jira Service Management Queues, Freshservice Agent Console
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Provide a single modern UI for service-desk agents that surfaces the active ticket + related CMDB CIs + similar KB articles + the SLA clock + the agent-assistant panel.
- O-002: Reduce average handle time (AHT) versus ServiceNow Agent Workspace by 30% via tighter latency budgets and inline AI assist.
- O-003: Support multi-pane layouts (priority queue / detail / context / assistant) with keyboard-first navigation.

## Panes
- PA-001: Inbox priority queue (sorted by SLA-clock-to-breach).
- PA-002: Active ticket detail (incident / problem / change / service request).
- PA-003: Related CMDB panel (auto-populated via service-mapping capability).
- PA-004: Similar KB articles panel (RAG via knowledge-base + intelligence).
- PA-005: SLA clock strip (live ticks via WebSocket on top of HTTP/3 transport).
- PA-006: Agent assistant panel (drafts replies; suggests resolution).

## Tenant invariants
- T-001: Agents see only tickets in their assigned support groups within their tenant.
- T-002: Cross-tenant agent (managed-service-provider) requires `delegated_admin_grant_id`.
- T-003: AI assistant invocations are tenant-isolated; never leak cross-tenant prompts.

## Cedar policy
- C-001: `policy/agent-workspace-authorization.cedar` default-denies; explicit permits scoped by `principal.audience_type == AGENT_RESPONDER`.

## SLA clock strip
- SC-001: Real-time clock updates pushed via the IP-030 SLA breach remediation loop.
- SC-002: Visual cues at T-15min, T-5min, breached.
- SC-003: Click-through to escalation-policy bounded context if breach is imminent.

## Keyboard shortcuts
- KS-001: `j/k` move queue cursor; `space` open ticket; `r` reply; `e` escalate; `c` change-link; `?` help.

## Tenant-class behavior
- TC-001: demo_trial: per-seat cap 3 agents per ADR-0331.
- TC-002: paid: per-seat metered.

## Implementation sequence
- I-001: Ship REST + WebSocket endpoints under `/api/v1/itsm/agent-workspace/`.
- I-002: Bind Cedar; implement support-group scoping.
- I-003: Wire CMDB + KB + SLA + assistant fetchers.
- I-004: Implement keyboard navigation in the frontend bundle.

## Acceptance evidence
- E-001: openslo: agent_workspace_action_p95_ms ≤ 250.
- E-002: openslo: agent_workspace_inbox_render_p95_ms ≤ 400.
- E-003: cargo test for the REST + WebSocket handlers.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: agent workspace.
- REST/API anchor: agent action route.
- Policy anchor: policy/agent-workspace-authorization.cedar.
- SLO/dashboard anchor: agent action p95 <= 250ms.
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
| ServiceNow ITSM | agent operator console under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-033-agent-workspace.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-033-agent-workspace.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-033-agent-workspace.md` matched [`metered`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-033-agent-workspace.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
