# IP-040 ITSM walk-up-experience

Service: itsm
ChangeSet scope: microservices/itsm/IP-040-walk-up-experience.md
Counterparts displaced: ServiceNow Walk-Up Experience, Freshservice Walk-Up
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Provide a kiosk surface for in-office help requests; the requester checks in at a lobby kiosk, joins a queue, and an agent receives the pickup notification.
- O-002: Capture face-to-face resolution outcomes so the audit-chain has visibility into in-person fixes that bypass tickets.
- O-003: Exit-csat collected at the kiosk on resolution.

## Surface
- S-001: Kiosk frontend lives in `frontend/winui-3/walk-up-kiosk/` (WinUI 3 C# / .NET per frontend allowlist) for Windows-based lobby terminals.
- S-002: Optional macOS Apple Silicon M5+ kiosk via Swift bundle.
- S-003: REST endpoints under `/api/v1/itsm/walk-up/`.

## Tenant invariants
- T-001: Kiosk is tied to a `location_id` registered against the tenant.
- T-002: Requester check-in requires SSO via identity µservice.

## Flows
- F-001: Kiosk check-in (badge tap or QR).
- F-002: Queue position display.
- F-003: Agent pickup notification.
- F-004: Face-to-face resolution capture.
- F-005: Exit CSAT.

## Tenant-class behavior
- TC-001: demo_trial: walk-up disabled per ADR-0331.
- TC-002: paid: enabled.

## Acceptance evidence
- E-001: openslo: kiosk_checkin_p95_ms ≤ 700.
- E-002: cargo test for the queue state machine.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: walk-up experience.
- REST/API anchor: walk-up check-in route.
- Policy anchor: policy/walk-up-experience-authorization.cedar.
- SLO/dashboard anchor: check-in p95 <= 700ms.
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
| ServiceNow ITSM | walk-up service desk under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

### Additional promotion guard
- Promotion also requires an owner-named residual-risk row when any referenced policy file is not yet implemented.
- Promotion also requires one synthetic-tenant fixture and one cross-tenant denial fixture.
- Promotion also requires a rollback flag name that can be toggled without disabling core incident handling.
- Promotion also requires evidence that ServiceNow, Jira Service Management, and Freshservice ids remain aliases.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-040-walk-up-experience.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-040-walk-up-experience.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
