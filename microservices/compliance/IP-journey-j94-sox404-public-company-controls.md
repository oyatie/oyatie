---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
status: draft
date: 2026-05-20
microservice: compliance
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

# IP - compliance role in j94 SOX 404 public-company controls for Marcus

## Scope

compliance owns pack activation, regulator article mapping, and auditor portal evidence inventory for j94-sox-404-public-company-controls. The slice is a flat per-microservice implementation plan under microservices/compliance/, matching ADR-0131.
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

1. compliance implements control inventory import for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-COMPLIANCE-001, and fails closed on Cedar deny.
2. compliance implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-COMPLIANCE-002, and fails closed on Cedar deny.
3. compliance implements quarterly evidence close for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-COMPLIANCE-003, and fails closed on Cedar deny.
4. compliance implements management certification packet for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-COMPLIANCE-004, and fails closed on Cedar deny.
5. compliance implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-COMPLIANCE-005, and fails closed on Cedar deny.
6. compliance implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-COMPLIANCE-006, and fails closed on Cedar deny.
7. compliance implements control inventory import for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-COMPLIANCE-007, and fails closed on Cedar deny.
8. compliance implements segregation-of-duties graph for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-COMPLIANCE-008, and fails closed on Cedar deny.
9. compliance implements quarterly evidence close for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-COMPLIANCE-009, and fails closed on Cedar deny.
10. compliance implements management certification packet for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-COMPLIANCE-010, and fails closed on Cedar deny.
11. compliance implements external auditor read-only portal for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-COMPLIANCE-011, and fails closed on Cedar deny.
12. compliance implements whistleblower protected intake for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-COMPLIANCE-012, and fails closed on Cedar deny.
13. compliance implements control inventory import for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-COMPLIANCE-013, and fails closed on Cedar deny.
14. compliance implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-COMPLIANCE-014, and fails closed on Cedar deny.
15. compliance implements quarterly evidence close for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-COMPLIANCE-015, and fails closed on Cedar deny.
16. compliance implements management certification packet for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-COMPLIANCE-016, and fails closed on Cedar deny.
17. compliance implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-COMPLIANCE-017, and fails closed on Cedar deny.
18. compliance implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-COMPLIANCE-018, and fails closed on Cedar deny.
19. compliance implements control inventory import for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-COMPLIANCE-019, and fails closed on Cedar deny.
20. compliance implements segregation-of-duties graph for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-COMPLIANCE-020, and fails closed on Cedar deny.
21. compliance implements quarterly evidence close for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-COMPLIANCE-021, and fails closed on Cedar deny.
22. compliance implements management certification packet for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-COMPLIANCE-022, and fails closed on Cedar deny.
23. compliance implements external auditor read-only portal for j94, cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection, emits EVT-J94-COMPLIANCE-023, and fails closed on Cedar deny.
24. compliance implements whistleblower protected intake for j94, cites SEC Rule 21F-17 anti-impediment to whistleblower communication, emits EVT-J94-COMPLIANCE-024, and fails closed on Cedar deny.
25. compliance implements control inventory import for j94, cites Sarbanes-Oxley Act section 302 issuer officer certifications, emits EVT-J94-COMPLIANCE-025, and fails closed on Cedar deny.
26. compliance implements segregation-of-duties graph for j94, cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting, emits EVT-J94-COMPLIANCE-026, and fails closed on Cedar deny.
27. compliance implements quarterly evidence close for j94, cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation, emits EVT-J94-COMPLIANCE-027, and fails closed on Cedar deny.
28. compliance implements management certification packet for j94, cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting, emits EVT-J94-COMPLIANCE-028, and fails closed on Cedar deny.
29. compliance implements external auditor read-only portal for j94, cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation, emits EVT-J94-COMPLIANCE-029, and fails closed on Cedar deny.
30. compliance implements whistleblower protected intake for j94, cites Sarbanes-Oxley Act section 802 records destruction penalties, emits EVT-J94-COMPLIANCE-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j94.compliance.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2B_PUBLIC_COMPANY_EXECUTIVE" &&
  resource.service == "compliance" &&
  resource.journey_id == "j94" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("SOX-404")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J94-COMPLIANCE-001 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-002 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-003 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-004 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-005 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-006 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-007 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-008 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-009 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-010 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-011 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-012 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-013 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-014 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-015 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-016 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-017 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-018 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-019 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-020 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-021 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-022 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-023 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-024 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-025 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-026 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-027 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-028 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-029 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-030 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-031 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-032 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-033 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-034 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-035 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-036 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-037 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-038 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-039 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-040 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-041 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-042 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-043 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-044 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-045 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-046 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-047 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-048 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-049 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-050 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-051 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-052 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-053 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-054 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-055 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-056 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-057 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-058 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-059 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-060 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-061 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-062 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-063 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-064 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-065 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-066 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-067 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-068 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-069 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-070 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-071 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-072 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-073 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-074 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-075 | quarterly evidence close | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-076 | management certification packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-077 | external auditor read-only portal | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-078 | whistleblower protected intake | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-079 | control inventory import | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J94-COMPLIANCE-080 | segregation-of-duties graph | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-001 sealed |
| 2 | edge | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-002 sealed |
| 3 | api-rest | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-003 sealed |
| 4 | api-async | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-004 sealed |
| 5 | adapter | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-005 sealed |
| 6 | usecase | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-006 sealed |
| 7 | domain | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-007 sealed |
| 8 | kernel | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-008 sealed |
| 9 | policy | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-009 sealed |
| 10 | eventing | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-010 sealed |
| 11 | observability | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-011 sealed |
| 12 | iac | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-012 sealed |
| 13 | evidence | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-013 sealed |
| 14 | experience | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-014 sealed |
| 15 | edge | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-015 sealed |
| 16 | api-rest | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-016 sealed |
| 17 | api-async | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-017 sealed |
| 18 | adapter | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-018 sealed |
| 19 | usecase | compliance control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-019 sealed |
| 20 | domain | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-020 sealed |
| 21 | kernel | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-021 sealed |
| 22 | policy | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-022 sealed |
| 23 | eventing | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-023 sealed |
| 24 | observability | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-024 sealed |
| 25 | iac | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-025 sealed |
| 26 | evidence | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-026 sealed |
| 27 | experience | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-027 sealed |
| 28 | edge | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-028 sealed |
| 29 | api-rest | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-029 sealed |
| 30 | api-async | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-030 sealed |
| 31 | adapter | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-031 sealed |
| 32 | usecase | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-032 sealed |
| 33 | domain | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-033 sealed |
| 34 | kernel | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-034 sealed |
| 35 | policy | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-035 sealed |
| 36 | eventing | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-036 sealed |
| 37 | observability | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-037 sealed |
| 38 | iac | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-038 sealed |
| 39 | evidence | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-039 sealed |
| 40 | experience | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-040 sealed |
| 41 | edge | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-041 sealed |
| 42 | api-rest | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-042 sealed |
| 43 | api-async | compliance control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-043 sealed |
| 44 | adapter | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-044 sealed |
| 45 | usecase | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-045 sealed |
| 46 | domain | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-046 sealed |
| 47 | kernel | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-047 sealed |
| 48 | policy | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-048 sealed |
| 49 | eventing | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-049 sealed |
| 50 | observability | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-050 sealed |
| 51 | iac | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-051 sealed |
| 52 | evidence | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-052 sealed |
| 53 | experience | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-053 sealed |
| 54 | edge | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-054 sealed |
| 55 | api-rest | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-055 sealed |
| 56 | api-async | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-056 sealed |
| 57 | adapter | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-057 sealed |
| 58 | usecase | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-058 sealed |
| 59 | domain | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-059 sealed |
| 60 | kernel | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-060 sealed |
| 61 | policy | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-061 sealed |
| 62 | eventing | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-062 sealed |
| 63 | observability | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-063 sealed |
| 64 | iac | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-064 sealed |
| 65 | evidence | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-065 sealed |
| 66 | experience | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-066 sealed |
| 67 | edge | compliance control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-067 sealed |
| 68 | api-rest | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-068 sealed |
| 69 | api-async | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-069 sealed |
| 70 | adapter | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-070 sealed |
| 71 | usecase | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-071 sealed |
| 72 | domain | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-072 sealed |
| 73 | kernel | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-073 sealed |
| 74 | policy | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-074 sealed |
| 75 | eventing | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-075 sealed |
| 76 | observability | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-076 sealed |
| 77 | iac | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-077 sealed |
| 78 | evidence | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-078 sealed |
| 79 | experience | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-079 sealed |
| 80 | edge | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-080 sealed |
| 81 | api-rest | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-081 sealed |
| 82 | api-async | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-082 sealed |
| 83 | adapter | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-083 sealed |
| 84 | usecase | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-084 sealed |
| 85 | domain | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-085 sealed |
| 86 | kernel | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-086 sealed |
| 87 | policy | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-087 sealed |
| 88 | eventing | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-088 sealed |
| 89 | observability | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-089 sealed |
| 90 | iac | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-090 sealed |
| 91 | evidence | compliance control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-091 sealed |
| 92 | experience | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-092 sealed |
| 93 | edge | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-093 sealed |
| 94 | api-rest | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-094 sealed |
| 95 | api-async | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-095 sealed |
| 96 | adapter | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-096 sealed |
| 97 | usecase | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-097 sealed |
| 98 | domain | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-098 sealed |
| 99 | kernel | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-099 sealed |
| 100 | policy | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-100 sealed |
| 101 | eventing | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-101 sealed |
| 102 | observability | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-102 sealed |
| 103 | iac | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-103 sealed |
| 104 | evidence | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-104 sealed |
| 105 | experience | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-105 sealed |
| 106 | edge | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-106 sealed |
| 107 | api-rest | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-107 sealed |
| 108 | api-async | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-108 sealed |
| 109 | adapter | compliance control inventory import support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-109 sealed |
| 110 | usecase | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-110 sealed |
| 111 | domain | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-111 sealed |
| 112 | kernel | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-112 sealed |
| 113 | policy | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 302 issuer officer certifications; audit EVT-J94-COMPLIANCE-TASK-113 sealed |
| 114 | eventing | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-114 sealed |
| 115 | observability | compliance control inventory import support with pack SOX-404 | Unit/integration check cites 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; audit EVT-J94-COMPLIANCE-TASK-115 sealed |
| 116 | iac | compliance segregation-of-duties graph support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; audit EVT-J94-COMPLIANCE-TASK-116 sealed |
| 117 | evidence | compliance quarterly evidence close support with pack SOX-404 | Unit/integration check cites Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; audit EVT-J94-COMPLIANCE-TASK-117 sealed |
| 118 | experience | compliance management certification packet support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites Sarbanes-Oxley Act section 802 records destruction penalties; audit EVT-J94-COMPLIANCE-TASK-118 sealed |
| 119 | edge | compliance external auditor read-only portal support with pack SOX-404 | Unit/integration check cites Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; audit EVT-J94-COMPLIANCE-TASK-119 sealed |
| 120 | api-rest | compliance whistleblower protected intake support with pack DODD-FRANK-WHISTLEBLOWER | Unit/integration check cites SEC Rule 21F-17 anti-impediment to whistleblower communication; audit EVT-J94-COMPLIANCE-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in compliance; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles control inventory import at ADR-0105 layer experience; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-ANALYTICS-001. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles segregation-of-duties graph at ADR-0105 layer edge; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-API_GATEWAY-002. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles quarterly evidence close at ADR-0105 layer api-rest; citation: 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; evidence: EVT-J94-APPLICATION-003. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles management certification packet at ADR-0105 layer api-async; citation: Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; evidence: EVT-J94-AUDIT_CHAIN-004. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles external auditor read-only portal at ADR-0105 layer adapter; citation: Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; evidence: EVT-J94-CALENDAR-005. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles whistleblower protected intake at ADR-0105 layer usecase; citation: Sarbanes-Oxley Act section 802 records destruction penalties; evidence: EVT-J94-CELL-006. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles control inventory import at ADR-0105 layer domain; citation: Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; evidence: EVT-J94-CLOUD_IAC-007. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles segregation-of-duties graph at ADR-0105 layer kernel; citation: SEC Rule 21F-17 anti-impediment to whistleblower communication; evidence: EVT-J94-CLOUD_K8S-008. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles quarterly evidence close at ADR-0105 layer policy; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-CLOUD_SECRETS-009. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles management certification packet at ADR-0105 layer eventing; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-COMMS_EMAIL-010. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles external auditor read-only portal at ADR-0105 layer observability; citation: 15 U.S.C. 7262 SOX 404 management assessment and auditor attestation; evidence: EVT-J94-COMMUNITY-011. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles whistleblower protected intake at ADR-0105 layer iac; citation: Exchange Act Rules 13a-15 and 15d-15 internal control over financial reporting; evidence: EVT-J94-COMPLIANCE-012. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles control inventory import at ADR-0105 layer evidence; citation: Sarbanes-Oxley Act section 806 whistleblower anti-retaliation; evidence: EVT-J94-CONNECT-013. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles segregation-of-duties graph at ADR-0105 layer experience; citation: Sarbanes-Oxley Act section 802 records destruction penalties; evidence: EVT-J94-CONSENT_GRAPH-014. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles quarterly evidence close at ADR-0105 layer edge; citation: Dodd-Frank Act section 922 and 15 U.S.C. 78u-6 SEC whistleblower incentives and protection; evidence: EVT-J94-DEVELOPER_SDK-015. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles management certification packet at ADR-0105 layer api-rest; citation: SEC Rule 21F-17 anti-impediment to whistleblower communication; evidence: EVT-J94-DOCS-016. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles external auditor read-only portal at ADR-0105 layer api-async; citation: Sarbanes-Oxley Act section 302 issuer officer certifications; evidence: EVT-J94-DRIVE-017. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles whistleblower protected intake at ADR-0105 layer adapter; citation: Sarbanes-Oxley Act section 404(a) management assessment of internal control over financial reporting; evidence: EVT-J94-FEATURE_FLAGS-018. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-journey-j94-sox404-public-company-controls.md` matched `financial`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-journey-j94-sox404-public-company-controls.md` matched `emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
