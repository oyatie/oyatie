---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
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

# IP - intelligence role in j94 SOX 404 public-company controls for Marcus

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j94-sox-404-public-company-controls. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
The service participates in SOX-404 + Dodd-Frank; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. Sarbanes-Oxley Act section 302 issuer officer certifications.
- 2. Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting.
- 3. 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation.
- 4. Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting.
- 5. Sarbanes-Oxley Act section 806 whistleblower anti-retaliation.
- 6. Sarbanes-Oxley Act section 802 records destruction penalties.
- 7. Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection.
- 8. SEC Rule 21F-17 anti-impediment to whistleblower communication.

## Acceptance criteria

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | Control narrative classification: Intelligence classifies/summarizes `control-description summarizer` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-001` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |
| 2 | Deficiency severity draft: Intelligence classifies/summarizes `risk explanation requiring human sign-off` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-002` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |
| 3 | Evidence completeness check: Intelligence classifies/summarizes `missing-artifact classifier` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-003` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |
| 4 | Change-control explanation: Intelligence classifies/summarizes `change-ticket summary with audit hashes` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-004` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |
| 5 | Auditor packet readback: Intelligence classifies/summarizes `scoped audit-tap retrieval` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-005` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |
| 6 | False-positive correction: Intelligence classifies/summarizes `append-only eval correction after reviewer override` for `Marcus` under `US-SOX404`. | Cedar `journey.j94.intelligence.execute` admits only matching tenant/pack; `EVT-J94-INTELLIGENCE-006` is sealed; counterpart equivalence: Workiva SOX control evidence + ServiceNow GRC issue workflow. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j94.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_PUBLIC_COMPANY_EXECUTIVE" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j94" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("SOX-404")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J94-INTELLIGENCE-001 | Control narrative classification | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX 404 management assessment", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |
| EVT-J94-INTELLIGENCE-002 | Deficiency severity draft | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX material weakness/significant deficiency review", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |
| EVT-J94-INTELLIGENCE-003 | Evidence completeness check | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX audit evidence sufficiency", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |
| EVT-J94-INTELLIGENCE-004 | Change-control explanation | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX ITGC change-management control", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |
| EVT-J94-INTELLIGENCE-005 | Auditor packet readback | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX external auditor evidence request", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |
| EVT-J94-INTELLIGENCE-006 | False-positive correction | journey_id, tenant_id, service=intelligence, pack_id=US-SOX404, article_ref="SOX remediation tracking", cedar_decision_id, evidence_hash, counterpart="Workiva SOX control evidence + ServiceNow GRC issue workflow" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire Control narrative classification through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOX 404 management assessment`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-001`. |
| 2 | api-rest | Wire Deficiency severity draft through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOX material weakness/significant deficiency review`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-002`. |
| 3 | api-async | Wire Evidence completeness check through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOX audit evidence sufficiency`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-003`. |
| 4 | proto | Wire Change-control explanation through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOX ITGC change-management control`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-004`. |
| 5 | observability | Wire Auditor packet readback through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `SOX external auditor evidence request`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-005`. |
| 6 | runbook | Wire False-positive correction through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `SOX remediation tracking`, verifies Cedar default-deny, and seals `EVT-J94-INTELLIGENCE-006`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j94-sox404-public-company-controls.md` matched `financial`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j94-sox404-public-company-controls.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
