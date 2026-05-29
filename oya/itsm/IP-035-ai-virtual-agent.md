# IP-035 ITSM ai-virtual-agent

Service: itsm
ChangeSet scope: microservices/itsm/IP-035-ai-virtual-agent.md
Counterparts displaced: ServiceNow Now Assist (Virtual Agent), Atlassian Intelligence for JSM, Freshservice Freddy AI
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0255, ADR-0263, ADR-0328

## Objective
- O-001: L1 deflection chatbot that resolves common requester questions without involving an agent.
- O-002: When uncertainty is above threshold, warm-transfer to the agent workspace with a summarized context block.
- O-003: Tenant-isolated; per-tenant fine-tuning of the deflection model.

## Conversation flow
- CF-001: Greet (template scoped to tenant brand).
- CF-002: Classify intent (intelligence µservice classifier).
- CF-003: Retrieve KB answer (RAG via IP-034).
- CF-004: Offer answer with confidence score.
- CF-005: If confirmed → close conversation + emit deflection event.
- CF-006: If not confirmed or low confidence → open ticket with summary attached.

## Refusal + safety
- RS-001: Tenant pack overlays gate output (e.g., HIPAA pack disables free-text PII recall).
- RS-002: Cedar gate `ai-virtual-agent-authorization.cedar` checks per-tenant model approval state.

## Tenant invariants
- T-001: Each conversation is tagged `tenant_id`, `requester_principal_id`, `audience_type=REQUESTER`.
- T-002: Conversation transcripts are tenant-private; no cross-tenant training.

## Performance + metering
- PM-001: p95 first-token latency ≤ 800ms (paid).
- PM-002: Meter: `ai_deflection_attempts_per_month` (per-usage).
- PM-003: demo_trial cap: 200 attempts/month per ADR-0331.

## Implementation sequence
- I-001: Wire intelligence µservice chat-completion endpoint.
- I-002: Bind KB retrieval via IP-034.
- I-003: Implement Cedar gate.
- I-004: Emit audit events per turn.
- I-005: Ship to portal + mobile + agent workspace.

## Acceptance evidence
- E-001: openslo: ai_va_first_token_p95_ms ≤ 800 (paid).
- E-002: openslo: deflection_rate ≥ 0.35 (paid).
- E-003: cargo test for the conversation state machine.

## Out of scope
- OoS-001: Cross-tenant fine-tuning shared models — never.
- OoS-002: Pure rules-based bots without intelligence µservice — superseded by ADR-0255.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: AI virtual agent.
- REST/API anchor: ai-va converse route.
- Policy anchor: policy/ai-virtual-agent-authorization.cedar.
- SLO/dashboard anchor: deflection attempt and confidence metrics.
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
| ServiceNow ITSM | virtual agent deflection under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-035-ai-virtual-agent.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-035-ai-virtual-agent.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
