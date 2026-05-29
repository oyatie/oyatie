# IP-031 ITSM self-service-portal

Service: itsm
ChangeSet scope: microservices/itsm/IP-031-self-service-portal.md
Counterparts displaced: ServiceNow Service Portal (Washington DC), Jira Service Management Customer Portal, Freshservice Self-Service
Binding ADRs: ADR-0064, ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0328, ADR-0331

## Objective
- O-001: Provide a tenant-scoped, Cedar-gated self-service portal that lets a requester browse the knowledge base, create a ticket, look up assets, submit a service-catalog request, and check ticket status without operator help.
- O-002: Drive a target 35% L0 deflection rate by serving AI Virtual Agent (IP-035) responses + knowledge-base (IP-034) RAG retrieval before opening a ticket.
- O-003: Honor pack overlays (SOC-2, ISO-27001, GDPR, KR-PIPA, HIPAA) at the portal boundary; the portal must not surface a ticket field that a pack would later redact.
- O-004: Stay readable on mobile devices via the Mobile ITSM capability (IP-032); deep links into native bundles where installed.
- O-005: Emit one audit-chain event per portal action (view article, file ticket, lookup asset).

## Problem framing
- P-001: ServiceNow Service Portal customizes via Now Experience UI Framework; the portal logic gets sticky inside a vendor runtime.
- P-002: JSM Customer Portal exposes per-project request forms and burdens the tenant admin with per-form Cedar-equivalent rules.
- P-003: Freshservice Self-Service deflection is workflow-glued; the AI rerank is opaque to the tenant.
- P-004: Without a tenant-isolated rerank, knowledge-base answers can leak cross-tenant article fragments.
- P-005: Without principal scoping at the portal, requester-portal logic ends up duplicating agent-workspace logic.

## Surface
- S-001: REST endpoints under `/api/v1/itsm/self-service-portal/` for `articles.search`, `tickets.create`, `tickets.status`, `assets.lookup`, `requests.submit`.
- S-002: Pages: home, browse KB, request catalog, my tickets, my assets, my notifications.
- S-003: Audience type: `REQUESTER`.
- S-004: Data classes touched: `kb_article`, `incident_ticket`, `service_request`, `cmdb_ci_summary`.

## Architecture
- A-001: Portal frontend bundle lives in `frontend/web/itsm-self-service/` (TypeScript-free per language policy; built via Rust-emitted WASM front end runtime).
- A-002: Server-side rendering and JSON API both terminate at the ITSM µservice REST handler.
- A-003: Knowledge retrieval is served by the knowledge-base capability via intelligence µservice substrate.

## Tenant invariants
- T-001: Every portal action carries `tenant_id`, `requester_principal_id`, `audience_type=REQUESTER`, `purpose`, `data_class`.
- T-002: A requester can list only tickets where the requester principal equals the calling principal (or a delegated_admin_grant_id includes the requester).
- T-003: Asset lookup returns CMDB CIs filtered by ownership scope, never the global tenant CI set.

## Cedar policy
- C-001: `policy/self-service-portal-authorization.cedar` default-denies all actions; explicit permits for `portal.articles.search`, `portal.tickets.create`, `portal.tickets.status`, `portal.assets.lookup`, `portal.requests.submit`.
- C-002: Permits require `principal.audience_type == REQUESTER` and `principal.tenant_id == resource.tenant_id`.
- C-003: Article retrieval permits gate on `resource.data_class == kb_article` AND `resource.tenant_id == principal.tenant_id`.

## Deflection mechanic
- D-001: When a requester types into the ticket-create description, the portal calls AI Virtual Agent + Knowledge Base before committing the ticket.
- D-002: If the top-3 retrieved articles cross the per-tenant relevance threshold, the portal offers them inline.
- D-003: If the requester marks any article as "this answers my question", the portal logs a `self_service_deflection_success` audit event and does not open a ticket.
- D-004: If the requester proceeds, the portal opens an incident or service request with the AI summarization attached.

## Tenant-class behavior
- TC-001: demo_trial: portal available; deflection mechanic active; KB capped at 50 articles per ADR-0331; ticket cap 500/month.
- TC-002: paid: portal available with no caps; per-seat licensing for the embedded agent surface; pack overlays activatable.

## Observability (ADR-0263)
- OB-001: Audit events: `self-service-portal.viewed`, `articles.searched`, `articles.viewed`, `tickets.created_from_portal`, `tickets.status_viewed`, `assets.lookup`, `requests.submitted`.
- OB-002: Metrics: deflection_rate, avg_articles_returned_per_search, p95_search_latency_ms, ticket_create_from_portal_ratio.
- OB-003: Traces: every portal request carries an OpenTelemetry trace context that the substrate µservices propagate.

## Failure modes
- F-001: Intelligence µservice unavailable → portal degrades to keyword search; deflection ratio drops; banner shown to requester.
- F-002: Knowledge base empty → portal still allows ticket creation; emits `kb_empty_self_service` audit event.
- F-003: Tenant in residency-restricted pack tries to retrieve cross-region article → deny + audit `pack_residency_block`.

## Implementation sequence
- I-001: Wire REST handlers for the 5 portal endpoints into the ITSM REST adapter.
- I-002: Implement Cedar policy + bind to authorization gate.
- I-003: Integrate KB retrieval; pass principal scope and tenant_id.
- I-004: Integrate AI Virtual Agent invocation; bound by attempt cap per tenant class.
- I-005: Emit audit events; verify via audit-chain capability.
- I-006: Ship the portal frontend bundle; route through Mobile ITSM where bundle is installed.

## Acceptance evidence
- E-001: cargo test for the REST handler.
- E-002: openslo: portal_p95_latency_ms ≤ 600 paid; ≤ 2000 demo_trial.
- E-003: openslo: deflection_rate ≥ 0.30 paid; ≥ 0.15 demo_trial.
- E-004: audit-chain replay test verifies one event per portal action.

## Rollback
- R-001: Feature flag `itsm.self_service_portal.enabled` set to false reverts the portal to a 503 page; the µservice continues serving agent surfaces.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: self-service portal.
- REST/API anchor: /api/v1/itsm/self-service-portal/*.
- Policy anchor: policy/self-service-portal-authorization.cedar.
- SLO/dashboard anchor: portal_p95_latency_ms and deflection_rate.
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
| ServiceNow ITSM | requester portal and deflection under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-031-self-service-portal.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-031-self-service-portal.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
