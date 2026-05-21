---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
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

# IP - intelligence role in j98 Australian Privacy Act and APRA CPS 234 tenant onboarding

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j98-au-privacy-apra-cps-234-tenant. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
The service participates in AU-Privacy + APRA-CPS-234; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information.
- 2. APP 3 collection of solicited personal information.
- 3. APP 5 notification of collection.
- 4. APP 6 use or disclosure.
- 5. APP 8 cross-border disclosure.
- 6. APP 11 security of personal information.
- 7. APP 12 access and APP 13 correction.
- 8. Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification.
- 9. APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls.
- 10. APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification.

## Acceptance criteria

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | APP notice classifier: Intelligence classifies/summarizes `notice/purpose summary` for `Australian operator` under `AU-Privacy`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-001` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |
| 2 | CPS 234 control mapping: Intelligence classifies/summarizes `control evidence classifier` for `Australian operator` under `AU-APRA`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-002` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |
| 3 | Notifiable breach triage: Intelligence classifies/summarizes `breach-risk summary and reviewer escalation` for `Australian operator` under `AU-Privacy`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-003` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |
| 4 | Outsourcing/provider route: Intelligence classifies/summarizes `provider allowlist/refusal decision` for `Australian operator` under `AU-APRA`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-004` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |
| 5 | Access/correction draft: Intelligence classifies/summarizes `localized response draft` for `Australian operator` under `AU-Privacy`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-005` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |
| 6 | Audit evidence readback: Intelligence classifies/summarizes `sealed audit-tap ref` for `Australian operator` under `AU-APRA`. | Cedar `journey.j98.intelligence.execute` admits only matching tenant/pack; `EVT-J98-INTELLIGENCE-006` is sealed; counterpart equivalence: APRA CPS 234 control assessment + Microsoft Purview privacy workflow. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j98.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_AU_FINANCIAL_SERVICES_ADMIN" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j98" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("AU-PRIVACY-ACT")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J98-INTELLIGENCE-001 | APP notice classifier | journey_id, tenant_id, service=intelligence, pack_id=AU-Privacy, article_ref="Australian Privacy Principles", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |
| EVT-J98-INTELLIGENCE-002 | CPS 234 control mapping | journey_id, tenant_id, service=intelligence, pack_id=AU-APRA, article_ref="APRA CPS 234 information security capability", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |
| EVT-J98-INTELLIGENCE-003 | Notifiable breach triage | journey_id, tenant_id, service=intelligence, pack_id=AU-Privacy, article_ref="NDB scheme", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |
| EVT-J98-INTELLIGENCE-004 | Outsourcing/provider route | journey_id, tenant_id, service=intelligence, pack_id=AU-APRA, article_ref="APRA third-party security control", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |
| EVT-J98-INTELLIGENCE-005 | Access/correction draft | journey_id, tenant_id, service=intelligence, pack_id=AU-Privacy, article_ref="APP access and correction rights", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |
| EVT-J98-INTELLIGENCE-006 | Audit evidence readback | journey_id, tenant_id, service=intelligence, pack_id=AU-APRA, article_ref="APRA inspection evidence", cedar_decision_id, evidence_hash, counterpart="APRA CPS 234 control assessment + Microsoft Purview privacy workflow" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire APP notice classifier through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `Australian Privacy Principles`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-001`. |
| 2 | api-rest | Wire CPS 234 control mapping through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `APRA CPS 234 information security capability`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-002`. |
| 3 | api-async | Wire Notifiable breach triage through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `NDB scheme`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-003`. |
| 4 | proto | Wire Outsourcing/provider route through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `APRA third-party security control`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-004`. |
| 5 | observability | Wire Access/correction draft through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `APP access and correction rights`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-005`. |
| 6 | runbook | Wire Audit evidence readback through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `APRA inspection evidence`, verifies Cedar default-deny, and seals `EVT-J98-INTELLIGENCE-006`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j98-au-privacy-apra-cps234.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
