# IP-041 ITSM sla-engine

Service: itsm
ChangeSet scope: microservices/itsm/IP-041-sla-engine.md
Counterparts displaced: ServiceNow SLA Engine, Jira Service Management SLA, Freshservice SLA Policies
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0252, ADR-0263, ADR-0328

## Objective
- O-001: Define SLA / OLA / UC (underpinning contract) per service-catalog item and per ticket category.
- O-002: Drive event-driven breach detection via IP-030; target p99 detection latency 15s vs ServiceNow ~120s (8× advantage).
- O-003: Bind breach escalation to the escalation-policy bounded context.

## Operations
- OP-001: `declare_sla_contract` — bind to catalog item or category; declare clock policy.
- OP-002: `declare_ola_contract` — internal team contract.
- OP-003: `declare_uc_underpinning_contract` — third-party underpinning contract.
- OP-004: `start_clock` — at ticket creation or specified event.
- OP-005: `pause_clock_per_business_hours_or_blocker` — business hours, awaiting requester, paused per pack rule.
- OP-006: `resume_clock` — when paused condition lifts.
- OP-007: `detect_breach_event_driven` — IP-030 worker emits breach events the moment clock crosses threshold.
- OP-008: `escalate_on_breach` — calls escalation-policy bounded context.
- OP-009: `seal_breach_evidence_to_audit_chain` — immutable evidence.

## Causality
- C-001: Default clock is HLC (hybrid logical clock) per ADR-0252 KS#12.
- C-002: TrueTime opt-in for fin-grade tenants; required by FedRAMP-High pack.

## Tenant invariants
- T-001: Every clock state carries `tenant_id`, `ticket_id`, `sla_contract_id`.
- T-002: Clock recomputation never hides breached time (matches domain invariant `sla_monotonic`).

## Performance claim
- PC-001: Event-driven detection p99 ≤ 15000ms.
- PC-002: ServiceNow ITSM scheduled-recompute p99 ~120000ms.
- PC-003: Advantage factor: 8.

## Acceptance evidence
- E-001: openslo: sla_breach_detection_p99_ms ≤ 15000.
- E-002: cargo property test: monotonic clock invariant.
- E-003: chaos drill: pause/resume cycle.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: SLA engine.
- REST/API anchor: sla detect/recompute route.
- Policy anchor: policies/local-sla-recompute-guard.cedar.
- SLO/dashboard anchor: breach detection p99 <= 15s.
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
| ServiceNow ITSM | SLA breach prevention under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

### Additional promotion guard
- Promotion also requires an owner-named residual-risk row when any referenced policy file is not yet implemented.
- Promotion also requires one synthetic-tenant fixture and one cross-tenant denial fixture.
- Promotion also requires a rollback flag name that can be toggled without disabling core incident handling.
- Promotion also requires evidence that ServiceNow, Jira Service Management, and Freshservice ids remain aliases.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-041-sla-engine.md` matched [`p99`, `SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-041-sla-engine.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
