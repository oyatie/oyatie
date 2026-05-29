---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
status: draft
date: 2026-05-20
microservice: api-gateway
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

# IP - api-gateway role in j98 Australian Privacy Act and APRA CPS 234 tenant onboarding

## Scope

api-gateway owns pack-aware ingress, route admission, and OpenAPI 3.2.0 response shaping for j98-au-privacy-apra-cps-234-tenant. The slice is a flat per-microservice implementation plan under microservices/api-gateway/, matching ADR-0131.
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

1. api-gateway implements AU tenant eligibility for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-API_GATEWAY-001, and fails closed on Cedar deny.
2. api-gateway implements APP notice and consent bind for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-API_GATEWAY-002, and fails closed on Cedar deny.
3. api-gateway implements IRAP PROTECTED cell placement for j98, cites APP 5 notification of collection, emits EVT-J98-API_GATEWAY-003, and fails closed on Cedar deny.
4. api-gateway implements CPS 234 asset classification for j98, cites APP 6 use or disclosure, emits EVT-J98-API_GATEWAY-004, and fails closed on Cedar deny.
5. api-gateway implements APRA notification drill for j98, cites APP 8 cross-border disclosure, emits EVT-J98-API_GATEWAY-005, and fails closed on Cedar deny.
6. api-gateway implements OAIC breach packet rehearsal for j98, cites APP 11 security of personal information, emits EVT-J98-API_GATEWAY-006, and fails closed on Cedar deny.
7. api-gateway implements AU tenant eligibility for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-API_GATEWAY-007, and fails closed on Cedar deny.
8. api-gateway implements APP notice and consent bind for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-API_GATEWAY-008, and fails closed on Cedar deny.
9. api-gateway implements IRAP PROTECTED cell placement for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-API_GATEWAY-009, and fails closed on Cedar deny.
10. api-gateway implements CPS 234 asset classification for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-API_GATEWAY-010, and fails closed on Cedar deny.
11. api-gateway implements APRA notification drill for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-API_GATEWAY-011, and fails closed on Cedar deny.
12. api-gateway implements OAIC breach packet rehearsal for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-API_GATEWAY-012, and fails closed on Cedar deny.
13. api-gateway implements AU tenant eligibility for j98, cites APP 5 notification of collection, emits EVT-J98-API_GATEWAY-013, and fails closed on Cedar deny.
14. api-gateway implements APP notice and consent bind for j98, cites APP 6 use or disclosure, emits EVT-J98-API_GATEWAY-014, and fails closed on Cedar deny.
15. api-gateway implements IRAP PROTECTED cell placement for j98, cites APP 8 cross-border disclosure, emits EVT-J98-API_GATEWAY-015, and fails closed on Cedar deny.
16. api-gateway implements CPS 234 asset classification for j98, cites APP 11 security of personal information, emits EVT-J98-API_GATEWAY-016, and fails closed on Cedar deny.
17. api-gateway implements APRA notification drill for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-API_GATEWAY-017, and fails closed on Cedar deny.
18. api-gateway implements OAIC breach packet rehearsal for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-API_GATEWAY-018, and fails closed on Cedar deny.
19. api-gateway implements AU tenant eligibility for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-API_GATEWAY-019, and fails closed on Cedar deny.
20. api-gateway implements APP notice and consent bind for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-API_GATEWAY-020, and fails closed on Cedar deny.
21. api-gateway implements IRAP PROTECTED cell placement for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-API_GATEWAY-021, and fails closed on Cedar deny.
22. api-gateway implements CPS 234 asset classification for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-API_GATEWAY-022, and fails closed on Cedar deny.
23. api-gateway implements APRA notification drill for j98, cites APP 5 notification of collection, emits EVT-J98-API_GATEWAY-023, and fails closed on Cedar deny.
24. api-gateway implements OAIC breach packet rehearsal for j98, cites APP 6 use or disclosure, emits EVT-J98-API_GATEWAY-024, and fails closed on Cedar deny.
25. api-gateway implements AU tenant eligibility for j98, cites APP 8 cross-border disclosure, emits EVT-J98-API_GATEWAY-025, and fails closed on Cedar deny.
26. api-gateway implements APP notice and consent bind for j98, cites APP 11 security of personal information, emits EVT-J98-API_GATEWAY-026, and fails closed on Cedar deny.
27. api-gateway implements IRAP PROTECTED cell placement for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-API_GATEWAY-027, and fails closed on Cedar deny.
28. api-gateway implements CPS 234 asset classification for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-API_GATEWAY-028, and fails closed on Cedar deny.
29. api-gateway implements APRA notification drill for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-API_GATEWAY-029, and fails closed on Cedar deny.
30. api-gateway implements OAIC breach packet rehearsal for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-API_GATEWAY-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j98.api_gateway.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_AU_FINANCIAL_SERVICES_ADMIN" &&
  resource.service == "api-gateway" &&
  resource.journey_id == "j98" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("AU-PRIVACY-ACT")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J98-API_GATEWAY-001 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-002 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-003 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-004 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-005 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-006 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-007 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-008 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-009 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-010 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-011 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-012 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-013 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-014 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-015 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-016 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-017 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-018 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-019 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-020 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-021 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-022 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-023 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-024 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-025 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-026 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-027 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-028 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-029 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-030 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-031 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-032 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-033 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-034 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-035 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-036 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-037 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-038 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-039 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-040 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-041 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-042 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-043 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-044 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-045 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-046 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-047 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-048 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-049 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-050 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-051 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-052 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-053 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-054 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-055 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-056 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-057 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-058 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-059 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-060 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-061 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-062 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-063 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-064 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-065 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-066 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-067 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-068 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-069 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-070 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-071 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-072 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-073 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-074 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-075 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-076 | CPS 234 asset classification | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-077 | APRA notification drill | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-078 | OAIC breach packet rehearsal | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-079 | AU tenant eligibility | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-API_GATEWAY-080 | APP notice and consent bind | journey_id, tenant_id, service=api-gateway, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-001 sealed |
| 2 | edge | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-002 sealed |
| 3 | api-rest | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-003 sealed |
| 4 | api-async | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-004 sealed |
| 5 | adapter | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-005 sealed |
| 6 | usecase | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-006 sealed |
| 7 | domain | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-007 sealed |
| 8 | kernel | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-008 sealed |
| 9 | policy | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-009 sealed |
| 10 | eventing | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-010 sealed |
| 11 | observability | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-011 sealed |
| 12 | iac | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-012 sealed |
| 13 | evidence | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-013 sealed |
| 14 | experience | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-014 sealed |
| 15 | edge | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-015 sealed |
| 16 | api-rest | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-016 sealed |
| 17 | api-async | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-017 sealed |
| 18 | adapter | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-018 sealed |
| 19 | usecase | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-019 sealed |
| 20 | domain | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-020 sealed |
| 21 | kernel | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-021 sealed |
| 22 | policy | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-022 sealed |
| 23 | eventing | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-023 sealed |
| 24 | observability | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-024 sealed |
| 25 | iac | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-025 sealed |
| 26 | evidence | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-026 sealed |
| 27 | experience | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-027 sealed |
| 28 | edge | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-028 sealed |
| 29 | api-rest | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-029 sealed |
| 30 | api-async | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-030 sealed |
| 31 | adapter | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-031 sealed |
| 32 | usecase | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-032 sealed |
| 33 | domain | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-033 sealed |
| 34 | kernel | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-034 sealed |
| 35 | policy | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-035 sealed |
| 36 | eventing | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-036 sealed |
| 37 | observability | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-037 sealed |
| 38 | iac | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-038 sealed |
| 39 | evidence | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-039 sealed |
| 40 | experience | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-040 sealed |
| 41 | edge | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-041 sealed |
| 42 | api-rest | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-042 sealed |
| 43 | api-async | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-043 sealed |
| 44 | adapter | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-044 sealed |
| 45 | usecase | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-045 sealed |
| 46 | domain | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-046 sealed |
| 47 | kernel | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-047 sealed |
| 48 | policy | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-048 sealed |
| 49 | eventing | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-049 sealed |
| 50 | observability | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-050 sealed |
| 51 | iac | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-051 sealed |
| 52 | evidence | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-052 sealed |
| 53 | experience | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-053 sealed |
| 54 | edge | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-054 sealed |
| 55 | api-rest | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-055 sealed |
| 56 | api-async | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-056 sealed |
| 57 | adapter | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-057 sealed |
| 58 | usecase | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-058 sealed |
| 59 | domain | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-059 sealed |
| 60 | kernel | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-060 sealed |
| 61 | policy | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-061 sealed |
| 62 | eventing | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-062 sealed |
| 63 | observability | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-063 sealed |
| 64 | iac | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-064 sealed |
| 65 | evidence | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-065 sealed |
| 66 | experience | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-066 sealed |
| 67 | edge | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-067 sealed |
| 68 | api-rest | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-068 sealed |
| 69 | api-async | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-069 sealed |
| 70 | adapter | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-070 sealed |
| 71 | usecase | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-071 sealed |
| 72 | domain | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-072 sealed |
| 73 | kernel | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-073 sealed |
| 74 | policy | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-074 sealed |
| 75 | eventing | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-075 sealed |
| 76 | observability | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-076 sealed |
| 77 | iac | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-077 sealed |
| 78 | evidence | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-078 sealed |
| 79 | experience | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-079 sealed |
| 80 | edge | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-080 sealed |
| 81 | api-rest | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-081 sealed |
| 82 | api-async | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-082 sealed |
| 83 | adapter | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-083 sealed |
| 84 | usecase | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-084 sealed |
| 85 | domain | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-085 sealed |
| 86 | kernel | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-086 sealed |
| 87 | policy | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-087 sealed |
| 88 | eventing | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-088 sealed |
| 89 | observability | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-089 sealed |
| 90 | iac | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-090 sealed |
| 91 | evidence | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-091 sealed |
| 92 | experience | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-092 sealed |
| 93 | edge | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-093 sealed |
| 94 | api-rest | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-094 sealed |
| 95 | api-async | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-095 sealed |
| 96 | adapter | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-096 sealed |
| 97 | usecase | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-097 sealed |
| 98 | domain | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-098 sealed |
| 99 | kernel | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-099 sealed |
| 100 | policy | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-100 sealed |
| 101 | eventing | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-101 sealed |
| 102 | observability | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-102 sealed |
| 103 | iac | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-103 sealed |
| 104 | evidence | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-104 sealed |
| 105 | experience | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-105 sealed |
| 106 | edge | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-106 sealed |
| 107 | api-rest | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-107 sealed |
| 108 | api-async | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-108 sealed |
| 109 | adapter | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-109 sealed |
| 110 | usecase | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-110 sealed |
| 111 | domain | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-API_GATEWAY-TASK-111 sealed |
| 112 | kernel | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-API_GATEWAY-TASK-112 sealed |
| 113 | policy | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-API_GATEWAY-TASK-113 sealed |
| 114 | eventing | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-API_GATEWAY-TASK-114 sealed |
| 115 | observability | api-gateway AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-API_GATEWAY-TASK-115 sealed |
| 116 | iac | api-gateway APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-API_GATEWAY-TASK-116 sealed |
| 117 | evidence | api-gateway IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-API_GATEWAY-TASK-117 sealed |
| 118 | experience | api-gateway CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-API_GATEWAY-TASK-118 sealed |
| 119 | edge | api-gateway APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-API_GATEWAY-TASK-119 sealed |
| 120 | api-rest | api-gateway OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-API_GATEWAY-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in api-gateway; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles AU tenant eligibility at ADR-0105 layer experience; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-ANALYTICS-001. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles APP notice and consent bind at ADR-0105 layer edge; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-API_GATEWAY-002. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles IRAP PROTECTED cell placement at ADR-0105 layer api-rest; citation: APP 5 notification of collection; evidence: EVT-J98-APPLICATION-003. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles CPS 234 asset classification at ADR-0105 layer api-async; citation: APP 6 use or disclosure; evidence: EVT-J98-AUDIT_CHAIN-004. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles APRA notification drill at ADR-0105 layer adapter; citation: APP 8 cross-border disclosure; evidence: EVT-J98-CALENDAR-005. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles OAIC breach packet rehearsal at ADR-0105 layer usecase; citation: APP 11 security of personal information; evidence: EVT-J98-CELL-006. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles AU tenant eligibility at ADR-0105 layer domain; citation: APP 12 access and APP 13 correction; evidence: EVT-J98-CLOUD_IAC-007. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles APP notice and consent bind at ADR-0105 layer kernel; citation: Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; evidence: EVT-J98-CLOUD_K8S-008. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles IRAP PROTECTED cell placement at ADR-0105 layer policy; citation: APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; evidence: EVT-J98-CLOUD_SECRETS-009. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles CPS 234 asset classification at ADR-0105 layer eventing; citation: APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; evidence: EVT-J98-COMMS_EMAIL-010. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles APRA notification drill at ADR-0105 layer observability; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-COMMUNITY-011. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles OAIC breach packet rehearsal at ADR-0105 layer iac; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-COMPLIANCE-012. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles AU tenant eligibility at ADR-0105 layer evidence; citation: APP 5 notification of collection; evidence: EVT-J98-CONNECT-013. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles APP notice and consent bind at ADR-0105 layer experience; citation: APP 6 use or disclosure; evidence: EVT-J98-CONSENT_GRAPH-014. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles IRAP PROTECTED cell placement at ADR-0105 layer edge; citation: APP 8 cross-border disclosure; evidence: EVT-J98-DEVELOPER_SDK-015. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles CPS 234 asset classification at ADR-0105 layer api-rest; citation: APP 11 security of personal information; evidence: EVT-J98-DOCS-016. Service api-gateway remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.

## Wave 15 counterpart anchor

GitHub and GitLab are the grep-recognized API-ingress counterparts for this preserved journey IP: the gateway work must keep route admission, webhooks, rate limits, TLS, canary routing, abuse defense, and emergency bypass controls explicit at the north-south edge.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-journey-j98-au-privacy-apra-cps234.md`.
