---
doc_class: Implementation-Plan
ip_id: IP-journey-j95-iso27001-soc2-annual-audit
journey_ref: docs/user-journeys/j95-iso-27001-soc-2-annual-audit/
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

# IP - intelligence role in j95 Combined ISO 27001, ISO 22301, and SOC 2 annual audit for Marcus

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j95-iso-27001-soc-2-annual-audit. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
The service participates in ISO-27001 + ISO-22301 + SOC-2-T2; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. ISO/IEC 27001:2022 clauses 4 through 10 and Annex A controls A.5, A.6, A.7, A.8.
- 2. ISO/IEC 27002:2022 Annex A implementation guidance for organizational, people, physical, and technological controls.
- 3. ISO 22301:2019 clauses 8.4 incident response, 8.5 business continuity plans, and 8.6 exercise program.
- 4. AICPA SOC 2 Trust Services Criteria CC1 through CC9.
- 5. SOC 2 availability criteria A1.1 through A1.3.
- 6. SOC 2 confidentiality criteria C1.1 through C1.2.
- 7. SOC 2 processing integrity PI1.1 through PI1.5.
- 8. SOC 2 privacy criteria P1.1 through P8.1.

## Acceptance criteria

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | Control evidence classifier: Intelligence classifies/summarizes `artifact-to-control mapping` for `Marcus` under `ISO27001-SOC2`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-001` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |
| 2 | BCP evidence summary: Intelligence classifies/summarizes `business-continuity summary from runbook refs` for `Marcus` under `ISO22301`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-002` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |
| 3 | Auditor question draft: Intelligence classifies/summarizes `answer draft with citation spans only` for `Marcus` under `SOC2`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-003` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |
| 4 | Policy drift detection: Intelligence classifies/summarizes `changed-policy classifier and reviewer escalation` for `Marcus` under `ISO27001`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-004` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |
| 5 | Audit-tap readback: Intelligence classifies/summarizes `sealed dispatch/eval/audit reference` for `Marcus` under `SOC2`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-005` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |
| 6 | Provider incident summary: Intelligence classifies/summarizes `provider-outage runbook summary` for `Marcus` under `ISO27001-SOC2`. | Cedar `journey.j95.intelligence.execute` admits only matching tenant/pack; `EVT-J95-INTELLIGENCE-006` is sealed; counterpart equivalence: Vanta/Drata control evidence collection + ISO auditor packet. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j95.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_SECURITY_COMPLIANCE_LEAD" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j95" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("ISO-27001-2022")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J95-INTELLIGENCE-001 | Control evidence classifier | journey_id, tenant_id, service=intelligence, pack_id=ISO27001-SOC2, article_ref="ISO 27001 Annex A / SOC 2 CC evidence", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |
| EVT-J95-INTELLIGENCE-002 | BCP evidence summary | journey_id, tenant_id, service=intelligence, pack_id=ISO22301, article_ref="ISO 22301 continuity evidence", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |
| EVT-J95-INTELLIGENCE-003 | Auditor question draft | journey_id, tenant_id, service=intelligence, pack_id=SOC2, article_ref="SOC 2 auditor request", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |
| EVT-J95-INTELLIGENCE-004 | Policy drift detection | journey_id, tenant_id, service=intelligence, pack_id=ISO27001, article_ref="ISO 27001 policy review", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |
| EVT-J95-INTELLIGENCE-005 | Audit-tap readback | journey_id, tenant_id, service=intelligence, pack_id=SOC2, article_ref="SOC 2 evidence integrity", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |
| EVT-J95-INTELLIGENCE-006 | Provider incident summary | journey_id, tenant_id, service=intelligence, pack_id=ISO27001-SOC2, article_ref="ISO/SOC vendor risk evidence", cedar_decision_id, evidence_hash, counterpart="Vanta/Drata control evidence collection + ISO auditor packet" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire Control evidence classifier through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `ISO 27001 Annex A / SOC 2 CC evidence`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-001`. |
| 2 | api-rest | Wire BCP evidence summary through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `ISO 22301 continuity evidence`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-002`. |
| 3 | api-async | Wire Auditor question draft through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOC 2 auditor request`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-003`. |
| 4 | proto | Wire Policy drift detection through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `ISO 27001 policy review`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-004`. |
| 5 | observability | Wire Audit-tap readback through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOC 2 evidence integrity`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-005`. |
| 6 | runbook | Wire Provider incident summary through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `ISO/SOC vendor risk evidence`, verifies Cedar default-deny, and seals `EVT-J95-INTELLIGENCE-006`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j95-iso27001-soc2-annual-audit.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
