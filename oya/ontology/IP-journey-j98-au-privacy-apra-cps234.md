---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
status: draft
date: 2026-05-20
microservice: ontology
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

# IP - ontology role in j98 Australian Privacy Act and APRA CPS 234 tenant onboarding

## Scope

ontology owns typed entity graph, obligation relationships, and policy decision explainability for j98-au-privacy-apra-cps-234-tenant. The slice is a flat per-microservice implementation plan under microservices/ontology/, matching ADR-0131.
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

1. ontology implements AU tenant eligibility for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-ONTOLOGY-001, and fails closed on Cedar deny.
2. ontology implements APP notice and consent bind for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-ONTOLOGY-002, and fails closed on Cedar deny.
3. ontology implements IRAP PROTECTED cell placement for j98, cites APP 5 notification of collection, emits EVT-J98-ONTOLOGY-003, and fails closed on Cedar deny.
4. ontology implements CPS 234 asset classification for j98, cites APP 6 use or disclosure, emits EVT-J98-ONTOLOGY-004, and fails closed on Cedar deny.
5. ontology implements APRA notification drill for j98, cites APP 8 cross-border disclosure, emits EVT-J98-ONTOLOGY-005, and fails closed on Cedar deny.
6. ontology implements OAIC breach packet rehearsal for j98, cites APP 11 security of personal information, emits EVT-J98-ONTOLOGY-006, and fails closed on Cedar deny.
7. ontology implements AU tenant eligibility for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-ONTOLOGY-007, and fails closed on Cedar deny.
8. ontology implements APP notice and consent bind for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-ONTOLOGY-008, and fails closed on Cedar deny.
9. ontology implements IRAP PROTECTED cell placement for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-ONTOLOGY-009, and fails closed on Cedar deny.
10. ontology implements CPS 234 asset classification for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-ONTOLOGY-010, and fails closed on Cedar deny.
11. ontology implements APRA notification drill for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-ONTOLOGY-011, and fails closed on Cedar deny.
12. ontology implements OAIC breach packet rehearsal for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-ONTOLOGY-012, and fails closed on Cedar deny.
13. ontology implements AU tenant eligibility for j98, cites APP 5 notification of collection, emits EVT-J98-ONTOLOGY-013, and fails closed on Cedar deny.
14. ontology implements APP notice and consent bind for j98, cites APP 6 use or disclosure, emits EVT-J98-ONTOLOGY-014, and fails closed on Cedar deny.
15. ontology implements IRAP PROTECTED cell placement for j98, cites APP 8 cross-border disclosure, emits EVT-J98-ONTOLOGY-015, and fails closed on Cedar deny.
16. ontology implements CPS 234 asset classification for j98, cites APP 11 security of personal information, emits EVT-J98-ONTOLOGY-016, and fails closed on Cedar deny.
17. ontology implements APRA notification drill for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-ONTOLOGY-017, and fails closed on Cedar deny.
18. ontology implements OAIC breach packet rehearsal for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-ONTOLOGY-018, and fails closed on Cedar deny.
19. ontology implements AU tenant eligibility for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-ONTOLOGY-019, and fails closed on Cedar deny.
20. ontology implements APP notice and consent bind for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-ONTOLOGY-020, and fails closed on Cedar deny.
21. ontology implements IRAP PROTECTED cell placement for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-ONTOLOGY-021, and fails closed on Cedar deny.
22. ontology implements CPS 234 asset classification for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-ONTOLOGY-022, and fails closed on Cedar deny.
23. ontology implements APRA notification drill for j98, cites APP 5 notification of collection, emits EVT-J98-ONTOLOGY-023, and fails closed on Cedar deny.
24. ontology implements OAIC breach packet rehearsal for j98, cites APP 6 use or disclosure, emits EVT-J98-ONTOLOGY-024, and fails closed on Cedar deny.
25. ontology implements AU tenant eligibility for j98, cites APP 8 cross-border disclosure, emits EVT-J98-ONTOLOGY-025, and fails closed on Cedar deny.
26. ontology implements APP notice and consent bind for j98, cites APP 11 security of personal information, emits EVT-J98-ONTOLOGY-026, and fails closed on Cedar deny.
27. ontology implements IRAP PROTECTED cell placement for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-ONTOLOGY-027, and fails closed on Cedar deny.
28. ontology implements CPS 234 asset classification for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-ONTOLOGY-028, and fails closed on Cedar deny.
29. ontology implements APRA notification drill for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-ONTOLOGY-029, and fails closed on Cedar deny.
30. ontology implements OAIC breach packet rehearsal for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-ONTOLOGY-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j98.ontology.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_AU_FINANCIAL_SERVICES_ADMIN" &&
  resource.service == "ontology" &&
  resource.journey_id == "j98" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("AU-PRIVACY-ACT")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J98-ONTOLOGY-001 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-002 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-003 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-004 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-005 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-006 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-007 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-008 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-009 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-010 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-011 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-012 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-013 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-014 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-015 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-016 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-017 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-018 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-019 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-020 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-021 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-022 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-023 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-024 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-025 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-026 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-027 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-028 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-029 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-030 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-031 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-032 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-033 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-034 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-035 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-036 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-037 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-038 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-039 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-040 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-041 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-042 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-043 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-044 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-045 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-046 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-047 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-048 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-049 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-050 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-051 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-052 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-053 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-054 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-055 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-056 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-057 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-058 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-059 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-060 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-061 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-062 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-063 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-064 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-065 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-066 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-067 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-068 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-069 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-070 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-071 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-072 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-073 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-074 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-075 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-076 | CPS 234 asset classification | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-077 | APRA notification drill | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-078 | OAIC breach packet rehearsal | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-079 | AU tenant eligibility | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-ONTOLOGY-080 | APP notice and consent bind | journey_id, tenant_id, service=ontology, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-001 sealed |
| 2 | edge | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-002 sealed |
| 3 | api-rest | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-003 sealed |
| 4 | api-async | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-004 sealed |
| 5 | adapter | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-005 sealed |
| 6 | usecase | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-006 sealed |
| 7 | domain | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-007 sealed |
| 8 | kernel | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-008 sealed |
| 9 | policy | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-009 sealed |
| 10 | eventing | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-010 sealed |
| 11 | observability | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-011 sealed |
| 12 | iac | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-012 sealed |
| 13 | evidence | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-013 sealed |
| 14 | experience | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-014 sealed |
| 15 | edge | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-015 sealed |
| 16 | api-rest | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-016 sealed |
| 17 | api-async | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-017 sealed |
| 18 | adapter | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-018 sealed |
| 19 | usecase | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-019 sealed |
| 20 | domain | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-020 sealed |
| 21 | kernel | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-021 sealed |
| 22 | policy | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-022 sealed |
| 23 | eventing | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-023 sealed |
| 24 | observability | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-024 sealed |
| 25 | iac | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-025 sealed |
| 26 | evidence | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-026 sealed |
| 27 | experience | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-027 sealed |
| 28 | edge | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-028 sealed |
| 29 | api-rest | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-029 sealed |
| 30 | api-async | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-030 sealed |
| 31 | adapter | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-031 sealed |
| 32 | usecase | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-032 sealed |
| 33 | domain | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-033 sealed |
| 34 | kernel | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-034 sealed |
| 35 | policy | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-035 sealed |
| 36 | eventing | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-036 sealed |
| 37 | observability | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-037 sealed |
| 38 | iac | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-038 sealed |
| 39 | evidence | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-039 sealed |
| 40 | experience | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-040 sealed |
| 41 | edge | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-041 sealed |
| 42 | api-rest | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-042 sealed |
| 43 | api-async | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-043 sealed |
| 44 | adapter | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-044 sealed |
| 45 | usecase | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-045 sealed |
| 46 | domain | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-046 sealed |
| 47 | kernel | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-047 sealed |
| 48 | policy | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-048 sealed |
| 49 | eventing | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-049 sealed |
| 50 | observability | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-050 sealed |
| 51 | iac | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-051 sealed |
| 52 | evidence | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-052 sealed |
| 53 | experience | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-053 sealed |
| 54 | edge | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-054 sealed |
| 55 | api-rest | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-055 sealed |
| 56 | api-async | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-056 sealed |
| 57 | adapter | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-057 sealed |
| 58 | usecase | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-058 sealed |
| 59 | domain | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-059 sealed |
| 60 | kernel | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-060 sealed |
| 61 | policy | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-061 sealed |
| 62 | eventing | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-062 sealed |
| 63 | observability | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-063 sealed |
| 64 | iac | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-064 sealed |
| 65 | evidence | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-065 sealed |
| 66 | experience | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-066 sealed |
| 67 | edge | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-067 sealed |
| 68 | api-rest | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-068 sealed |
| 69 | api-async | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-069 sealed |
| 70 | adapter | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-070 sealed |
| 71 | usecase | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-071 sealed |
| 72 | domain | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-072 sealed |
| 73 | kernel | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-073 sealed |
| 74 | policy | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-074 sealed |
| 75 | eventing | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-075 sealed |
| 76 | observability | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-076 sealed |
| 77 | iac | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-077 sealed |
| 78 | evidence | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-078 sealed |
| 79 | experience | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-079 sealed |
| 80 | edge | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-080 sealed |
| 81 | api-rest | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-081 sealed |
| 82 | api-async | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-082 sealed |
| 83 | adapter | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-083 sealed |
| 84 | usecase | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-084 sealed |
| 85 | domain | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-085 sealed |
| 86 | kernel | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-086 sealed |
| 87 | policy | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-087 sealed |
| 88 | eventing | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-088 sealed |
| 89 | observability | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-089 sealed |
| 90 | iac | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-090 sealed |
| 91 | evidence | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-091 sealed |
| 92 | experience | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-092 sealed |
| 93 | edge | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-093 sealed |
| 94 | api-rest | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-094 sealed |
| 95 | api-async | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-095 sealed |
| 96 | adapter | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-096 sealed |
| 97 | usecase | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-097 sealed |
| 98 | domain | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-098 sealed |
| 99 | kernel | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-099 sealed |
| 100 | policy | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-100 sealed |
| 101 | eventing | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-101 sealed |
| 102 | observability | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-102 sealed |
| 103 | iac | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-103 sealed |
| 104 | evidence | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-104 sealed |
| 105 | experience | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-105 sealed |
| 106 | edge | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-106 sealed |
| 107 | api-rest | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-107 sealed |
| 108 | api-async | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-108 sealed |
| 109 | adapter | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-109 sealed |
| 110 | usecase | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-110 sealed |
| 111 | domain | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-ONTOLOGY-TASK-111 sealed |
| 112 | kernel | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-ONTOLOGY-TASK-112 sealed |
| 113 | policy | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-ONTOLOGY-TASK-113 sealed |
| 114 | eventing | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-ONTOLOGY-TASK-114 sealed |
| 115 | observability | ontology AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-ONTOLOGY-TASK-115 sealed |
| 116 | iac | ontology APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-ONTOLOGY-TASK-116 sealed |
| 117 | evidence | ontology IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-ONTOLOGY-TASK-117 sealed |
| 118 | experience | ontology CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-ONTOLOGY-TASK-118 sealed |
| 119 | edge | ontology APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-ONTOLOGY-TASK-119 sealed |
| 120 | api-rest | ontology OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-ONTOLOGY-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in ontology; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles AU tenant eligibility at ADR-0105 layer experience; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-ANALYTICS-001. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles APP notice and consent bind at ADR-0105 layer edge; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-API_GATEWAY-002. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles IRAP PROTECTED cell placement at ADR-0105 layer api-rest; citation: APP 5 notification of collection; evidence: EVT-J98-APPLICATION-003. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles CPS 234 asset classification at ADR-0105 layer api-async; citation: APP 6 use or disclosure; evidence: EVT-J98-AUDIT_CHAIN-004. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles APRA notification drill at ADR-0105 layer adapter; citation: APP 8 cross-border disclosure; evidence: EVT-J98-CALENDAR-005. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles OAIC breach packet rehearsal at ADR-0105 layer usecase; citation: APP 11 security of personal information; evidence: EVT-J98-CELL-006. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles AU tenant eligibility at ADR-0105 layer domain; citation: APP 12 access and APP 13 correction; evidence: EVT-J98-CLOUD_IAC-007. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles APP notice and consent bind at ADR-0105 layer kernel; citation: Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; evidence: EVT-J98-CLOUD_K8S-008. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles IRAP PROTECTED cell placement at ADR-0105 layer policy; citation: APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; evidence: EVT-J98-CLOUD_SECRETS-009. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles CPS 234 asset classification at ADR-0105 layer eventing; citation: APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; evidence: EVT-J98-COMMS_EMAIL-010. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles APRA notification drill at ADR-0105 layer observability; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-COMMUNITY-011. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles OAIC breach packet rehearsal at ADR-0105 layer iac; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-COMPLIANCE-012. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles AU tenant eligibility at ADR-0105 layer evidence; citation: APP 5 notification of collection; evidence: EVT-J98-CONNECT-013. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles APP notice and consent bind at ADR-0105 layer experience; citation: APP 6 use or disclosure; evidence: EVT-J98-CONSENT_GRAPH-014. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles IRAP PROTECTED cell placement at ADR-0105 layer edge; citation: APP 8 cross-border disclosure; evidence: EVT-J98-DEVELOPER_SDK-015. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles CPS 234 asset classification at ADR-0105 layer api-rest; citation: APP 11 security of personal information; evidence: EVT-J98-DOCS-016. Service ontology remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
