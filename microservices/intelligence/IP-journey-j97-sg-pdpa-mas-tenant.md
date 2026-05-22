---
doc_class: Implementation-Plan
ip_id: IP-journey-j97-sg-pdpa-mas-tenant
journey_ref: docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/
status: draft
date: 2026-05-20
microservice: intelligence
flat_layout_adr: ADR-0131
related_adrs:
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0263-observability-emission-contract
  - ADR-0131-per-microservice-flat-layout
  - ADR-0105-thirteen-layer-canonical-enum
---

# IP - intelligence role in j97 Singapore PDPA and MAS tenant onboarding

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j97-sg-pdpa-mas-singapore-tenant. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
The service participates in SG-PDPA + MAS; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. Singapore PDPA section 11 accountability.
- 2. Singapore PDPA sections 13 to 17 consent, purpose, and withdrawal duties.
- 3. Singapore PDPA section 20 notification of purposes.
- 4. Singapore PDPA section 21 access and correction.
- 5. Singapore PDPA section 24 protection obligation.
- 6. Singapore PDPA section 25 retention limitation.
- 7. Singapore PDPA section 26 transfer limitation.
- 8. Singapore PDPA section 26A data breach notification.
- 9. MAS Notice on Technology Risk Management incident reporting provisions for relevant incidents.
- 10. MAS Notice 658 cybersecurity overlay as tenant pack citation in this journey brief.

## Acceptance criteria

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | PDPA consent-purpose screen: Intelligence classifies/summarizes `processing-purpose classifier` for `Singapore operator` under `SG-PDPA`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-001` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |
| 2 | MAS outsourcing classifier: Intelligence classifies/summarizes `provider and subprocessor risk summary` for `Singapore operator` under `SG-MAS`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-002` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |
| 3 | Breach-notification triage: Intelligence classifies/summarizes `incident summary with reviewer escalation` for `Singapore operator` under `SG-PDPA`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-003` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |
| 4 | Financial data route check: Intelligence classifies/summarizes `pack-local provider decision` for `Singapore operator` under `SG-MAS`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-004` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |
| 5 | Access request draft: Intelligence classifies/summarizes `response draft with citation spans` for `Singapore operator` under `SG-PDPA`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-005` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |
| 6 | Audit evidence readback: Intelligence classifies/summarizes `audit-tap reference and signature` for `Singapore operator` under `SG-MAS`. | Cedar `journey.j97.intelligence.execute` admits only matching tenant/pack; `EVT-J97-INTELLIGENCE-006` is sealed; counterpart equivalence: MAS TRM evidence workflow + OneTrust PDPA response flow. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j97.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_SINGAPORE_FINTECH_ADMIN" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j97" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("SG-PDPA")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J97-INTELLIGENCE-001 | PDPA consent-purpose screen | journey_id, tenant_id, service=intelligence, pack_id=SG-PDPA, article_ref="Singapore PDPA consent/notification", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |
| EVT-J97-INTELLIGENCE-002 | MAS outsourcing classifier | journey_id, tenant_id, service=intelligence, pack_id=SG-MAS, article_ref="MAS TRM / outsourcing risk", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |
| EVT-J97-INTELLIGENCE-003 | Breach-notification triage | journey_id, tenant_id, service=intelligence, pack_id=SG-PDPA, article_ref="PDPA breach notification", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |
| EVT-J97-INTELLIGENCE-004 | Financial data route check | journey_id, tenant_id, service=intelligence, pack_id=SG-MAS, article_ref="MAS technology risk controls", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |
| EVT-J97-INTELLIGENCE-005 | Access request draft | journey_id, tenant_id, service=intelligence, pack_id=SG-PDPA, article_ref="PDPA access/correction rights", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |
| EVT-J97-INTELLIGENCE-006 | Audit evidence readback | journey_id, tenant_id, service=intelligence, pack_id=SG-MAS, article_ref="MAS inspection evidence", cedar_decision_id, evidence_hash, counterpart="MAS TRM evidence workflow + OneTrust PDPA response flow" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire PDPA consent-purpose screen through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `Singapore PDPA consent/notification`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-001`. |
| 2 | api-rest | Wire MAS outsourcing classifier through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `MAS TRM / outsourcing risk`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-002`. |
| 3 | api-async | Wire Breach-notification triage through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `PDPA breach notification`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-003`. |
| 4 | proto | Wire Financial data route check through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `MAS technology risk controls`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-004`. |
| 5 | observability | Wire Access request draft through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `PDPA access/correction rights`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-005`. |
| 6 | runbook | Wire Audit evidence readback through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `MAS inspection evidence`, verifies Cedar default-deny, and seals `EVT-J97-INTELLIGENCE-006`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j97-sg-pdpa-mas-tenant.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
