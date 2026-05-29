---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
status: draft
date: 2026-05-20
microservice: workflow-studio
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

# IP - workflow-studio role in j98 Australian Privacy Act and APRA CPS 234 tenant onboarding

## Scope

workflow-studio owns no-code workflow authoring and visual policy preview for tenant admins for j98-au-privacy-apra-cps-234-tenant. The slice is a flat per-microservice implementation plan under microservices/workflow-studio/, matching ADR-0131.
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

1. workflow-studio implements AU tenant eligibility for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-WORKFLOW_STUDIO-001, and fails closed on Cedar deny.
2. workflow-studio implements APP notice and consent bind for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-WORKFLOW_STUDIO-002, and fails closed on Cedar deny.
3. workflow-studio implements IRAP PROTECTED cell placement for j98, cites APP 5 notification of collection, emits EVT-J98-WORKFLOW_STUDIO-003, and fails closed on Cedar deny.
4. workflow-studio implements CPS 234 asset classification for j98, cites APP 6 use or disclosure, emits EVT-J98-WORKFLOW_STUDIO-004, and fails closed on Cedar deny.
5. workflow-studio implements APRA notification drill for j98, cites APP 8 cross-border disclosure, emits EVT-J98-WORKFLOW_STUDIO-005, and fails closed on Cedar deny.
6. workflow-studio implements OAIC breach packet rehearsal for j98, cites APP 11 security of personal information, emits EVT-J98-WORKFLOW_STUDIO-006, and fails closed on Cedar deny.
7. workflow-studio implements AU tenant eligibility for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-WORKFLOW_STUDIO-007, and fails closed on Cedar deny.
8. workflow-studio implements APP notice and consent bind for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-WORKFLOW_STUDIO-008, and fails closed on Cedar deny.
9. workflow-studio implements IRAP PROTECTED cell placement for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-WORKFLOW_STUDIO-009, and fails closed on Cedar deny.
10. workflow-studio implements CPS 234 asset classification for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-WORKFLOW_STUDIO-010, and fails closed on Cedar deny.
11. workflow-studio implements APRA notification drill for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-WORKFLOW_STUDIO-011, and fails closed on Cedar deny.
12. workflow-studio implements OAIC breach packet rehearsal for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-WORKFLOW_STUDIO-012, and fails closed on Cedar deny.
13. workflow-studio implements AU tenant eligibility for j98, cites APP 5 notification of collection, emits EVT-J98-WORKFLOW_STUDIO-013, and fails closed on Cedar deny.
14. workflow-studio implements APP notice and consent bind for j98, cites APP 6 use or disclosure, emits EVT-J98-WORKFLOW_STUDIO-014, and fails closed on Cedar deny.
15. workflow-studio implements IRAP PROTECTED cell placement for j98, cites APP 8 cross-border disclosure, emits EVT-J98-WORKFLOW_STUDIO-015, and fails closed on Cedar deny.
16. workflow-studio implements CPS 234 asset classification for j98, cites APP 11 security of personal information, emits EVT-J98-WORKFLOW_STUDIO-016, and fails closed on Cedar deny.
17. workflow-studio implements APRA notification drill for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-WORKFLOW_STUDIO-017, and fails closed on Cedar deny.
18. workflow-studio implements OAIC breach packet rehearsal for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-WORKFLOW_STUDIO-018, and fails closed on Cedar deny.
19. workflow-studio implements AU tenant eligibility for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-WORKFLOW_STUDIO-019, and fails closed on Cedar deny.
20. workflow-studio implements APP notice and consent bind for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-WORKFLOW_STUDIO-020, and fails closed on Cedar deny.
21. workflow-studio implements IRAP PROTECTED cell placement for j98, cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information, emits EVT-J98-WORKFLOW_STUDIO-021, and fails closed on Cedar deny.
22. workflow-studio implements CPS 234 asset classification for j98, cites APP 3 collection of solicited personal information, emits EVT-J98-WORKFLOW_STUDIO-022, and fails closed on Cedar deny.
23. workflow-studio implements APRA notification drill for j98, cites APP 5 notification of collection, emits EVT-J98-WORKFLOW_STUDIO-023, and fails closed on Cedar deny.
24. workflow-studio implements OAIC breach packet rehearsal for j98, cites APP 6 use or disclosure, emits EVT-J98-WORKFLOW_STUDIO-024, and fails closed on Cedar deny.
25. workflow-studio implements AU tenant eligibility for j98, cites APP 8 cross-border disclosure, emits EVT-J98-WORKFLOW_STUDIO-025, and fails closed on Cedar deny.
26. workflow-studio implements APP notice and consent bind for j98, cites APP 11 security of personal information, emits EVT-J98-WORKFLOW_STUDIO-026, and fails closed on Cedar deny.
27. workflow-studio implements IRAP PROTECTED cell placement for j98, cites APP 12 access and APP 13 correction, emits EVT-J98-WORKFLOW_STUDIO-027, and fails closed on Cedar deny.
28. workflow-studio implements CPS 234 asset classification for j98, cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification, emits EVT-J98-WORKFLOW_STUDIO-028, and fails closed on Cedar deny.
29. workflow-studio implements APRA notification drill for j98, cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls, emits EVT-J98-WORKFLOW_STUDIO-029, and fails closed on Cedar deny.
30. workflow-studio implements OAIC breach packet rehearsal for j98, cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification, emits EVT-J98-WORKFLOW_STUDIO-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j98.workflow_studio.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_AU_FINANCIAL_SERVICES_ADMIN" &&
  resource.service == "workflow-studio" &&
  resource.journey_id == "j98" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("AU-PRIVACY-ACT")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J98-WORKFLOW_STUDIO-001 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-002 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-003 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-004 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-005 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-006 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-007 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-008 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-009 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-010 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-011 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-012 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-013 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-014 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-015 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-016 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-017 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-018 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-019 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-020 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-021 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-022 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-023 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-024 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-025 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-026 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-027 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-028 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-029 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-030 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-031 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-032 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-033 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-034 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-035 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-036 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-037 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-038 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-039 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-040 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-041 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-042 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-043 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-044 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-045 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-046 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-047 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-048 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-049 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-050 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-051 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-052 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-053 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-054 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-055 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-056 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-057 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-058 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-059 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-060 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-061 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-062 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-063 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-064 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-065 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-066 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-067 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-068 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-069 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-070 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-071 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-072 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-073 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-074 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-075 | IRAP PROTECTED cell placement | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-076 | CPS 234 asset classification | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-077 | APRA notification drill | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-078 | OAIC breach packet rehearsal | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-079 | AU tenant eligibility | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J98-WORKFLOW_STUDIO-080 | APP notice and consent bind | journey_id, tenant_id, service=workflow-studio, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-001 sealed |
| 2 | edge | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-002 sealed |
| 3 | api-rest | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-003 sealed |
| 4 | api-async | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-004 sealed |
| 5 | adapter | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-005 sealed |
| 6 | usecase | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-006 sealed |
| 7 | domain | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-007 sealed |
| 8 | kernel | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-008 sealed |
| 9 | policy | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-009 sealed |
| 10 | eventing | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-010 sealed |
| 11 | observability | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-011 sealed |
| 12 | iac | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-012 sealed |
| 13 | evidence | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-013 sealed |
| 14 | experience | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-014 sealed |
| 15 | edge | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-015 sealed |
| 16 | api-rest | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-016 sealed |
| 17 | api-async | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-017 sealed |
| 18 | adapter | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-018 sealed |
| 19 | usecase | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-019 sealed |
| 20 | domain | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-020 sealed |
| 21 | kernel | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-021 sealed |
| 22 | policy | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-022 sealed |
| 23 | eventing | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-023 sealed |
| 24 | observability | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-024 sealed |
| 25 | iac | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-025 sealed |
| 26 | evidence | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-026 sealed |
| 27 | experience | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-027 sealed |
| 28 | edge | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-028 sealed |
| 29 | api-rest | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-029 sealed |
| 30 | api-async | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-030 sealed |
| 31 | adapter | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-031 sealed |
| 32 | usecase | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-032 sealed |
| 33 | domain | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-033 sealed |
| 34 | kernel | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-034 sealed |
| 35 | policy | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-035 sealed |
| 36 | eventing | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-036 sealed |
| 37 | observability | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-037 sealed |
| 38 | iac | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-038 sealed |
| 39 | evidence | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-039 sealed |
| 40 | experience | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-040 sealed |
| 41 | edge | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-041 sealed |
| 42 | api-rest | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-042 sealed |
| 43 | api-async | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-043 sealed |
| 44 | adapter | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-044 sealed |
| 45 | usecase | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-045 sealed |
| 46 | domain | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-046 sealed |
| 47 | kernel | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-047 sealed |
| 48 | policy | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-048 sealed |
| 49 | eventing | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-049 sealed |
| 50 | observability | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-050 sealed |
| 51 | iac | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-051 sealed |
| 52 | evidence | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-052 sealed |
| 53 | experience | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-053 sealed |
| 54 | edge | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-054 sealed |
| 55 | api-rest | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-055 sealed |
| 56 | api-async | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-056 sealed |
| 57 | adapter | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-057 sealed |
| 58 | usecase | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-058 sealed |
| 59 | domain | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-059 sealed |
| 60 | kernel | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-060 sealed |
| 61 | policy | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-061 sealed |
| 62 | eventing | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-062 sealed |
| 63 | observability | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-063 sealed |
| 64 | iac | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-064 sealed |
| 65 | evidence | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-065 sealed |
| 66 | experience | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-066 sealed |
| 67 | edge | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-067 sealed |
| 68 | api-rest | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-068 sealed |
| 69 | api-async | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-069 sealed |
| 70 | adapter | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-070 sealed |
| 71 | usecase | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-071 sealed |
| 72 | domain | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-072 sealed |
| 73 | kernel | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-073 sealed |
| 74 | policy | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-074 sealed |
| 75 | eventing | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-075 sealed |
| 76 | observability | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-076 sealed |
| 77 | iac | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-077 sealed |
| 78 | evidence | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-078 sealed |
| 79 | experience | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-079 sealed |
| 80 | edge | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-080 sealed |
| 81 | api-rest | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-081 sealed |
| 82 | api-async | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-082 sealed |
| 83 | adapter | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-083 sealed |
| 84 | usecase | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-084 sealed |
| 85 | domain | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-085 sealed |
| 86 | kernel | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-086 sealed |
| 87 | policy | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-087 sealed |
| 88 | eventing | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-088 sealed |
| 89 | observability | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-089 sealed |
| 90 | iac | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-090 sealed |
| 91 | evidence | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-091 sealed |
| 92 | experience | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-092 sealed |
| 93 | edge | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-093 sealed |
| 94 | api-rest | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-094 sealed |
| 95 | api-async | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-095 sealed |
| 96 | adapter | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-096 sealed |
| 97 | usecase | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-097 sealed |
| 98 | domain | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-098 sealed |
| 99 | kernel | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-099 sealed |
| 100 | policy | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-100 sealed |
| 101 | eventing | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-101 sealed |
| 102 | observability | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-102 sealed |
| 103 | iac | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-103 sealed |
| 104 | evidence | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-104 sealed |
| 105 | experience | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-105 sealed |
| 106 | edge | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-106 sealed |
| 107 | api-rest | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-107 sealed |
| 108 | api-async | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-108 sealed |
| 109 | adapter | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-109 sealed |
| 110 | usecase | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-110 sealed |
| 111 | domain | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-111 sealed |
| 112 | kernel | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 3 collection of solicited personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-112 sealed |
| 113 | policy | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APP 5 notification of collection; audit EVT-J98-WORKFLOW_STUDIO-TASK-113 sealed |
| 114 | eventing | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 6 use or disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-114 sealed |
| 115 | observability | workflow-studio AU tenant eligibility support with pack AU-PRIVACY-ACT | Unit/integration check cites APP 8 cross-border disclosure; audit EVT-J98-WORKFLOW_STUDIO-TASK-115 sealed |
| 116 | iac | workflow-studio APP notice and consent bind support with pack APRA-CPS-234 | Unit/integration check cites APP 11 security of personal information; audit EVT-J98-WORKFLOW_STUDIO-TASK-116 sealed |
| 117 | evidence | workflow-studio IRAP PROTECTED cell placement support with pack AU-IRAP-PROTECTED | Unit/integration check cites APP 12 access and APP 13 correction; audit EVT-J98-WORKFLOW_STUDIO-TASK-117 sealed |
| 118 | experience | workflow-studio CPS 234 asset classification support with pack AU-PRIVACY-ACT | Unit/integration check cites Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-118 sealed |
| 119 | edge | workflow-studio APRA notification drill support with pack APRA-CPS-234 | Unit/integration check cites APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; audit EVT-J98-WORKFLOW_STUDIO-TASK-119 sealed |
| 120 | api-rest | workflow-studio OAIC breach packet rehearsal support with pack AU-IRAP-PROTECTED | Unit/integration check cites APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; audit EVT-J98-WORKFLOW_STUDIO-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in workflow-studio; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles AU tenant eligibility at ADR-0105 layer experience; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-ANALYTICS-001. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles APP notice and consent bind at ADR-0105 layer edge; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-API_GATEWAY-002. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles IRAP PROTECTED cell placement at ADR-0105 layer api-rest; citation: APP 5 notification of collection; evidence: EVT-J98-APPLICATION-003. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles CPS 234 asset classification at ADR-0105 layer api-async; citation: APP 6 use or disclosure; evidence: EVT-J98-AUDIT_CHAIN-004. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles APRA notification drill at ADR-0105 layer adapter; citation: APP 8 cross-border disclosure; evidence: EVT-J98-CALENDAR-005. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles OAIC breach packet rehearsal at ADR-0105 layer usecase; citation: APP 11 security of personal information; evidence: EVT-J98-CELL-006. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles AU tenant eligibility at ADR-0105 layer domain; citation: APP 12 access and APP 13 correction; evidence: EVT-J98-CLOUD_IAC-007. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles APP notice and consent bind at ADR-0105 layer kernel; citation: Privacy Act 1988 Part IIIC sections 26WE and 26WK eligible data breach assessment and notification; evidence: EVT-J98-CLOUD_K8S-008. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles IRAP PROTECTED cell placement at ADR-0105 layer policy; citation: APRA CPS 234 paragraphs 13 to 21 governance, capability, policy, classification, and controls; evidence: EVT-J98-CLOUD_SECRETS-009. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles CPS 234 asset classification at ADR-0105 layer eventing; citation: APRA CPS 234 paragraphs 35 and 36 incident and material control weakness notification; evidence: EVT-J98-COMMS_EMAIL-010. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles APRA notification drill at ADR-0105 layer observability; citation: Privacy Act 1988 Schedule 1 APP 1 open and transparent management of personal information; evidence: EVT-J98-COMMUNITY-011. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles OAIC breach packet rehearsal at ADR-0105 layer iac; citation: APP 3 collection of solicited personal information; evidence: EVT-J98-COMPLIANCE-012. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles AU tenant eligibility at ADR-0105 layer evidence; citation: APP 5 notification of collection; evidence: EVT-J98-CONNECT-013. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles APP notice and consent bind at ADR-0105 layer experience; citation: APP 6 use or disclosure; evidence: EVT-J98-CONSENT_GRAPH-014. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles IRAP PROTECTED cell placement at ADR-0105 layer edge; citation: APP 8 cross-border disclosure; evidence: EVT-J98-DEVELOPER_SDK-015. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles CPS 234 asset classification at ADR-0105 layer api-rest; citation: APP 11 security of personal information; evidence: EVT-J98-DOCS-016. Service workflow-studio remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/workflow-studio/IP-journey-j98-au-privacy-apra-cps234.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/workflow-studio/IP-journey-j98-au-privacy-apra-cps234.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/capacity-model.md`, `microservices/workflow-studio/compliance.md`, `microservices/workflow-studio/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-journey-j98-au-privacy-apra-cps234.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
