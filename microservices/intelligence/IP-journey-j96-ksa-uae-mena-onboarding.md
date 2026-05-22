---
doc_class: Implementation-Plan
ip_id: IP-journey-j96-ksa-uae-mena-onboarding
journey_ref: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/
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

# IP - intelligence role in j96 KSA and UAE MENA tenant onboarding

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j96-ksa-uae-mena-tenant-onboarding. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
The service participates in KSA-PDPL + UAE-PDPL; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. KSA PDPL Royal Decree M/19 Article 5 lawful basis and consent principles.
- 2. KSA PDPL Article 6 processing without consent exceptions.
- 3. KSA PDPL Article 18 data subject rights and controller response duties.
- 4. KSA PDPL Article 20 personal data breach notification to the competent authority.
- 5. KSA PDPL Article 29 transfer or disclosure of personal data outside the Kingdom.
- 6. SDAIA Regulation on Personal Data Transfer Outside the Kingdom implementing PDPL Article 29.
- 7. NDMO National Data Governance Interim Regulations data classification and data sharing controls.
- 8. UAE Federal Decree-Law No. 45 of 2021 Article 4 data subject rights.
- 9. UAE PDPL Articles 22 and 23 cross-border transfer controls.
- 10. UAE PDPL Article 24 personal data security and breach notification obligations.

## Acceptance criteria

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | Arabic signup classification: Intelligence classifies/summarizes `Arabic/English onboarding intent classifier` for `MENA tenant operator` under `KSA-PDPL`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-001` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |
| 2 | Sovereign cell placement: Intelligence classifies/summarizes `pack routing explanation` for `MENA tenant operator` under `KSA-NDMO`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-002` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |
| 3 | UAE branch transfer review: Intelligence classifies/summarizes `transfer-risk summary` for `MENA tenant operator` under `UAE-PDPL`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-003` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |
| 4 | Bilingual access response: Intelligence classifies/summarizes `localized response draft with citations` for `MENA tenant operator` under `KSA-PDPL/UAE-PDPL`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-004` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |
| 5 | Provider deny: Intelligence classifies/summarizes `refusal when non-permitted provider requested` for `MENA tenant operator` under `KSA/UAE`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-005` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |
| 6 | Audit packet: Intelligence classifies/summarizes `sealed audit-tap readback` for `MENA tenant operator` under `KSA/UAE`. | Cedar `journey.j96.intelligence.execute` admits only matching tenant/pack; `EVT-J96-INTELLIGENCE-006` is sealed; counterpart equivalence: OneTrust regional privacy pack + Azure UAE/KSA residency controls. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j96.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_MENA_TENANT_ADMIN" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j96" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("KSA-NDMO")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J96-INTELLIGENCE-001 | Arabic signup classification | journey_id, tenant_id, service=intelligence, pack_id=KSA-PDPL, article_ref="PDPL transparent notice", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |
| EVT-J96-INTELLIGENCE-002 | Sovereign cell placement | journey_id, tenant_id, service=intelligence, pack_id=KSA-NDMO, article_ref="KSA data residency / NDMO classification", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |
| EVT-J96-INTELLIGENCE-003 | UAE branch transfer review | journey_id, tenant_id, service=intelligence, pack_id=UAE-PDPL, article_ref="UAE PDPL transfer controls", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |
| EVT-J96-INTELLIGENCE-004 | Bilingual access response | journey_id, tenant_id, service=intelligence, pack_id=KSA-PDPL/UAE-PDPL, article_ref="data-subject access rights", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |
| EVT-J96-INTELLIGENCE-005 | Provider deny | journey_id, tenant_id, service=intelligence, pack_id=KSA/UAE, article_ref="regional provider allowlist", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |
| EVT-J96-INTELLIGENCE-006 | Audit packet | journey_id, tenant_id, service=intelligence, pack_id=KSA/UAE, article_ref="regulator-ready evidence", cedar_decision_id, evidence_hash, counterpart="OneTrust regional privacy pack + Azure UAE/KSA residency controls" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire Arabic signup classification through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `PDPL transparent notice`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-001`. |
| 2 | api-rest | Wire Sovereign cell placement through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `KSA data residency / NDMO classification`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-002`. |
| 3 | api-async | Wire UAE branch transfer review through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `UAE PDPL transfer controls`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-003`. |
| 4 | proto | Wire Bilingual access response through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `data-subject access rights`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-004`. |
| 5 | observability | Wire Provider deny through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `regional provider allowlist`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-005`. |
| 6 | runbook | Wire Audit packet through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `regulator-ready evidence`, verifies Cedar default-deny, and seals `EVT-J96-INTELLIGENCE-006`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j96-ksa-uae-mena-onboarding.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
