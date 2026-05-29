# IP-039 ITSM csat-survey

Service: itsm
ChangeSet scope: microservices/itsm/IP-039-csat-survey.md
Counterparts displaced: ServiceNow Survey, Jira Service Management CSAT, Freshservice CSAT
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328

## Objective
- O-001: Collect customer satisfaction (CSAT), customer effort (CES), and net promoter (NPS) scores after relevant lifecycle transitions.
- O-002: Bind survey responses to the originating ticket/principal so analytics can compute per-agent + per-team trends.
- O-003: Respect per-pack PII rules; pseudonymize for GDPR / KR-PIPA tenants where requested.

## Survey kinds
- SK-001: CSAT post-incident close (5-point Likert).
- SK-002: CES customer effort score (7-point).
- SK-003: NPS periodic (0-10 with verbatim).

## Delivery channels
- DC-001: Email via mail µservice.
- DC-002: Personal Messenger via meet+messenger substrate (MLS RFC 9420 per ADR-0246).
- DC-003: In-app widget via portal (IP-031) and mobile (IP-032).

## Tenant invariants
- T-001: Surveys are emitted only after ticket closure event.
- T-002: One CSAT per ticket; multiple NPS allowed but capped by per-tenant cadence rule.

## Cedar policy
- C-001: `policy/csat-survey-authorization.cedar` permits `survey.respond` only by the requester or delegated principal.

## Privacy
- P-001: Per-pack pseudonymization: GDPR / KR-PIPA tenants can elect aggregate-only reporting where the respondent id is replaced by a per-period pseudonym.
- P-002: Free-text verbatim opt-in; default off for HIPAA.

## Tenant-class behavior
- TC-001: demo_trial: surveys available; deflection-only mode (no agent-level reporting).
- TC-002: paid: full agent-level + team-level analytics.

## Acceptance evidence
- E-001: openslo: survey_send_p95_ms ≤ 1500.
- E-002: cargo test for the response state machine.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: CSAT survey.
- REST/API anchor: csat send/collect route.
- Policy anchor: policy/csat-survey-authorization.cedar.
- SLO/dashboard anchor: survey send p95 <= 1500ms.
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
| ServiceNow ITSM | customer satisfaction loop under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

### Additional promotion guard
- Promotion also requires an owner-named residual-risk row when any referenced policy file is not yet implemented.
- Promotion also requires one synthetic-tenant fixture and one cross-tenant denial fixture.
- Promotion also requires a rollback flag name that can be toggled without disabling core incident handling.
- Promotion also requires evidence that ServiceNow, Jira Service Management, and Freshservice ids remain aliases.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-039-csat-survey.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-039-csat-survey.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
