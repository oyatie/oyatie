---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
status: draft
date: 2026-05-20
microservice: slides
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

# IP - slides role in j94 SOX 404 public-company controls for Marcus

## Scope

slides owns board/audit committee decks and regulator presentation packets for j94-sox-404-public-company-controls. The slice is a flat per-microservice implementation plan under microservices/slides/, matching ADR-0131.
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

1. slides implements control inventory import for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-SLIDES-001, and fails closed on Cedar deny.
2. slides implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-SLIDES-002, and fails closed on Cedar deny.
3. slides implements quarterly evidence close for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-SLIDES-003, and fails closed on Cedar deny.
4. slides implements management certification packet for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-SLIDES-004, and fails closed on Cedar deny.
5. slides implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-SLIDES-005, and fails closed on Cedar deny.
6. slides implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-SLIDES-006, and fails closed on Cedar deny.
7. slides implements control inventory import for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-SLIDES-007, and fails closed on Cedar deny.
8. slides implements segregation-of-duties graph for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-SLIDES-008, and fails closed on Cedar deny.
9. slides implements quarterly evidence close for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-SLIDES-009, and fails closed on Cedar deny.
10. slides implements management certification packet for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-SLIDES-010, and fails closed on Cedar deny.
11. slides implements external auditor read-only portal for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-SLIDES-011, and fails closed on Cedar deny.
12. slides implements whistleblower protected intake for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-SLIDES-012, and fails closed on Cedar deny.
13. slides implements control inventory import for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-SLIDES-013, and fails closed on Cedar deny.
14. slides implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-SLIDES-014, and fails closed on Cedar deny.
15. slides implements quarterly evidence close for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-SLIDES-015, and fails closed on Cedar deny.
16. slides implements management certification packet for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-SLIDES-016, and fails closed on Cedar deny.
17. slides implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-SLIDES-017, and fails closed on Cedar deny.
18. slides implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-SLIDES-018, and fails closed on Cedar deny.
19. slides implements control inventory import for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-SLIDES-019, and fails closed on Cedar deny.
20. slides implements segregation-of-duties graph for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-SLIDES-020, and fails closed on Cedar deny.
21. slides implements quarterly evidence close for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-SLIDES-021, and fails closed on Cedar deny.
22. slides implements management certification packet for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-SLIDES-022, and fails closed on Cedar deny.
23. slides implements external auditor read-only portal for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-SLIDES-023, and fails closed on Cedar deny.
24. slides implements whistleblower protected intake for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-SLIDES-024, and fails closed on Cedar deny.
25. slides implements control inventory import for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-SLIDES-025, and fails closed on Cedar deny.
26. slides implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-SLIDES-026, and fails closed on Cedar deny.
27. slides implements quarterly evidence close for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-SLIDES-027, and fails closed on Cedar deny.
28. slides implements management certification packet for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-SLIDES-028, and fails closed on Cedar deny.
29. slides implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-SLIDES-029, and fails closed on Cedar deny.
30. slides implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-SLIDES-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j94.slides.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_PUBLIC_COMPANY_EXECUTIVE" &&
  resource.service == "slides" &&
  resource.journey_id == "j94" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("SOX-404")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J94-SLIDES-001 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-002 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-003 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-004 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-005 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-006 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-007 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-008 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-009 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-010 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-011 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-012 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-013 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-014 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-015 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-016 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-017 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-018 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-019 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-020 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-021 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-022 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-023 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-024 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-025 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-026 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-027 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-028 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-029 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-030 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-031 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-032 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-033 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-034 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-035 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-036 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-037 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-038 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-039 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-040 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-041 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-042 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-043 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-044 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-045 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-046 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-047 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-048 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-049 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-050 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-051 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-052 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-053 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-054 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-055 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-056 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-057 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-058 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-059 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-060 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-061 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-062 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-063 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-064 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-065 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-066 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-067 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-068 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-069 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-070 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-071 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-072 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-073 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-074 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-075 | quarterly evidence close | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-076 | management certification packet | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-077 | external auditor read-only portal | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-078 | whistleblower protected intake | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-079 | control inventory import | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-SLIDES-080 | segregation-of-duties graph | journey_id, tenant_id, service=slides, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-001 sealed |
| 2 | edge | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-002 sealed |
| 3 | api-rest | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-003 sealed |
| 4 | api-async | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-004 sealed |
| 5 | adapter | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-005 sealed |
| 6 | usecase | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-006 sealed |
| 7 | domain | slides control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-007 sealed |
| 8 | kernel | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-008 sealed |
| 9 | policy | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-009 sealed |
| 10 | eventing | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-010 sealed |
| 11 | observability | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-011 sealed |
| 12 | iac | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-012 sealed |
| 13 | evidence | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-013 sealed |
| 14 | experience | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-014 sealed |
| 15 | edge | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-015 sealed |
| 16 | api-rest | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-016 sealed |
| 17 | api-async | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-017 sealed |
| 18 | adapter | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-018 sealed |
| 19 | usecase | slides control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-019 sealed |
| 20 | domain | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-020 sealed |
| 21 | kernel | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-021 sealed |
| 22 | policy | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-022 sealed |
| 23 | eventing | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-023 sealed |
| 24 | observability | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-024 sealed |
| 25 | iac | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-025 sealed |
| 26 | evidence | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-026 sealed |
| 27 | experience | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-027 sealed |
| 28 | edge | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-028 sealed |
| 29 | api-rest | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-029 sealed |
| 30 | api-async | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-030 sealed |
| 31 | adapter | slides control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-031 sealed |
| 32 | usecase | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-032 sealed |
| 33 | domain | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-033 sealed |
| 34 | kernel | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-034 sealed |
| 35 | policy | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-035 sealed |
| 36 | eventing | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-036 sealed |
| 37 | observability | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-037 sealed |
| 38 | iac | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-038 sealed |
| 39 | evidence | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-039 sealed |
| 40 | experience | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-040 sealed |
| 41 | edge | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-041 sealed |
| 42 | api-rest | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-042 sealed |
| 43 | api-async | slides control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-043 sealed |
| 44 | adapter | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-044 sealed |
| 45 | usecase | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-045 sealed |
| 46 | domain | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-046 sealed |
| 47 | kernel | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-047 sealed |
| 48 | policy | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-048 sealed |
| 49 | eventing | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-049 sealed |
| 50 | observability | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-050 sealed |
| 51 | iac | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-051 sealed |
| 52 | evidence | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-052 sealed |
| 53 | experience | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-053 sealed |
| 54 | edge | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-054 sealed |
| 55 | api-rest | slides control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-055 sealed |
| 56 | api-async | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-056 sealed |
| 57 | adapter | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-057 sealed |
| 58 | usecase | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-058 sealed |
| 59 | domain | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-059 sealed |
| 60 | kernel | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-060 sealed |
| 61 | policy | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-061 sealed |
| 62 | eventing | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-062 sealed |
| 63 | observability | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-063 sealed |
| 64 | iac | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-064 sealed |
| 65 | evidence | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-065 sealed |
| 66 | experience | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-066 sealed |
| 67 | edge | slides control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-067 sealed |
| 68 | api-rest | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-068 sealed |
| 69 | api-async | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-069 sealed |
| 70 | adapter | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-070 sealed |
| 71 | usecase | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-071 sealed |
| 72 | domain | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-072 sealed |
| 73 | kernel | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-073 sealed |
| 74 | policy | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-074 sealed |
| 75 | eventing | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-075 sealed |
| 76 | observability | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-076 sealed |
| 77 | iac | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-077 sealed |
| 78 | evidence | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-078 sealed |
| 79 | experience | slides control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-079 sealed |
| 80 | edge | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-080 sealed |
| 81 | api-rest | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-081 sealed |
| 82 | api-async | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-082 sealed |
| 83 | adapter | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-083 sealed |
| 84 | usecase | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-084 sealed |
| 85 | domain | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-085 sealed |
| 86 | kernel | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-086 sealed |
| 87 | policy | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-087 sealed |
| 88 | eventing | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-088 sealed |
| 89 | observability | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-089 sealed |
| 90 | iac | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-090 sealed |
| 91 | evidence | slides control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-091 sealed |
| 92 | experience | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-092 sealed |
| 93 | edge | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-093 sealed |
| 94 | api-rest | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-094 sealed |
| 95 | api-async | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-095 sealed |
| 96 | adapter | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-096 sealed |
| 97 | usecase | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-097 sealed |
| 98 | domain | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-098 sealed |
| 99 | kernel | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-099 sealed |
| 100 | policy | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-100 sealed |
| 101 | eventing | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-101 sealed |
| 102 | observability | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-102 sealed |
| 103 | iac | slides control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-103 sealed |
| 104 | evidence | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-104 sealed |
| 105 | experience | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-105 sealed |
| 106 | edge | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-106 sealed |
| 107 | api-rest | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-107 sealed |
| 108 | api-async | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-108 sealed |
| 109 | adapter | slides control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-109 sealed |
| 110 | usecase | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-110 sealed |
| 111 | domain | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-111 sealed |
| 112 | kernel | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-112 sealed |
| 113 | policy | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-SLIDES-TASK-113 sealed |
| 114 | eventing | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-SLIDES-TASK-114 sealed |
| 115 | observability | slides control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-SLIDES-TASK-115 sealed |
| 116 | iac | slides segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-SLIDES-TASK-116 sealed |
| 117 | evidence | slides quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-SLIDES-TASK-117 sealed |
| 118 | experience | slides management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-SLIDES-TASK-118 sealed |
| 119 | edge | slides external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-SLIDES-TASK-119 sealed |
| 120 | api-rest | slides whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-SLIDES-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in slides; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles control inventory import at ADR-0105 layer experience; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-ANALYTICS-001. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles segregation-of-duties graph at ADR-0105 layer edge; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-API_GATEWAY-002. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles quarterly evidence close at ADR-0105 layer api-rest; citation: 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; evidence: EVT-J94-APPLICATION-003. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles management certification packet at ADR-0105 layer api-async; citation: Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; evidence: EVT-J94-AUDIT_CHAIN-004. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles external auditor read-only portal at ADR-0105 layer adapter; citation: Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; evidence: EVT-J94-CALENDAR-005. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles whistleblower protected intake at ADR-0105 layer usecase; citation: Sarbanes-Oxley Act section 802 records destruction penalties; evidence: EVT-J94-CELL-006. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles control inventory import at ADR-0105 layer domain; citation: Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; evidence: EVT-J94-CLOUD_IAC-007. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles segregation-of-duties graph at ADR-0105 layer kernel; citation: SEC Rule 21F-17 anti-impediment to whistleblower communication; evidence: EVT-J94-CLOUD_K8S-008. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles quarterly evidence close at ADR-0105 layer policy; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-CLOUD_SECRETS-009. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles management certification packet at ADR-0105 layer eventing; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-COMMS_EMAIL-010. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles external auditor read-only portal at ADR-0105 layer observability; citation: 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; evidence: EVT-J94-COMMUNITY-011. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles whistleblower protected intake at ADR-0105 layer iac; citation: Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; evidence: EVT-J94-COMPLIANCE-012. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles control inventory import at ADR-0105 layer evidence; citation: Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; evidence: EVT-J94-CONNECT-013. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles segregation-of-duties graph at ADR-0105 layer experience; citation: Sarbanes-Oxley Act section 802 records destruction penalties; evidence: EVT-J94-CONSENT_GRAPH-014. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles quarterly evidence close at ADR-0105 layer edge; citation: Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; evidence: EVT-J94-DEVELOPER_SDK-015. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles management certification packet at ADR-0105 layer api-rest; citation: SEC Rule 21F-17 anti-impediment to whistleblower communication; evidence: EVT-J94-DOCS-016. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles external auditor read-only portal at ADR-0105 layer api-async; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-DRIVE-017. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles whistleblower protected intake at ADR-0105 layer adapter; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-FEATURE_FLAGS-018. Service slides remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
