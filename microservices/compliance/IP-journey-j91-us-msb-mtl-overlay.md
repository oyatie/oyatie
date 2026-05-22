---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
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

# IP - compliance role in j91 US state money transmitter licensing for Yejin

## Scope

compliance owns pack activation, regulator article mapping, and auditor portal evidence inventory for j91-us-state-money-transmitter-licensing. The slice is a flat per-microservice implementation plan under microservices/compliance/, matching ADR-0131.
The service participates in US-MSB + per-state MTLs; exact article anchors are inherited from the journey and repeated below for implementer cold-start buildability.

## Exact regulatory anchors

- 1. 31 CFR 1010.100(ff) money transmitter definition.
- 2. 31 CFR 1022.210 money services business anti-money-laundering program.
- 3. 31 CFR 1022.320 suspicious activity reporting for money services businesses.
- 4. California Financial Code section 2030 license requirement and section 2037 surety/securities obligation.
- 5. New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding.
- 6. Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security.
- 7. Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security.
- 8. Washington RCW 19.230.030 license required and 19.230.050 surety bond.

## Acceptance criteria

1. compliance implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-COMPLIANCE-001, and fails closed on Cedar deny.
2. compliance implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-COMPLIANCE-002, and fails closed on Cedar deny.
3. compliance implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-COMPLIANCE-003, and fails closed on Cedar deny.
4. compliance implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-COMPLIANCE-004, and fails closed on Cedar deny.
5. compliance implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-COMPLIANCE-005, and fails closed on Cedar deny.
6. compliance implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-COMPLIANCE-006, and fails closed on Cedar deny.
7. compliance implements threshold detection for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-COMPLIANCE-007, and fails closed on Cedar deny.
8. compliance implements state license gap analysis for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-COMPLIANCE-008, and fails closed on Cedar deny.
9. compliance implements surety bond packet for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-COMPLIANCE-009, and fails closed on Cedar deny.
10. compliance implements NMLS evidence upload for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-COMPLIANCE-010, and fails closed on Cedar deny.
11. compliance implements Cedar-gated payment throttling for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-COMPLIANCE-011, and fails closed on Cedar deny.
12. compliance implements regulator renewal calendar for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-COMPLIANCE-012, and fails closed on Cedar deny.
13. compliance implements threshold detection for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-COMPLIANCE-013, and fails closed on Cedar deny.
14. compliance implements state license gap analysis for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-COMPLIANCE-014, and fails closed on Cedar deny.
15. compliance implements surety bond packet for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-COMPLIANCE-015, and fails closed on Cedar deny.
16. compliance implements NMLS evidence upload for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-COMPLIANCE-016, and fails closed on Cedar deny.
17. compliance implements Cedar-gated payment throttling for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-COMPLIANCE-017, and fails closed on Cedar deny.
18. compliance implements regulator renewal calendar for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-COMPLIANCE-018, and fails closed on Cedar deny.
19. compliance implements threshold detection for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-COMPLIANCE-019, and fails closed on Cedar deny.
20. compliance implements state license gap analysis for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-COMPLIANCE-020, and fails closed on Cedar deny.
21. compliance implements surety bond packet for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-COMPLIANCE-021, and fails closed on Cedar deny.
22. compliance implements NMLS evidence upload for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-COMPLIANCE-022, and fails closed on Cedar deny.
23. compliance implements Cedar-gated payment throttling for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-COMPLIANCE-023, and fails closed on Cedar deny.
24. compliance implements regulator renewal calendar for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-COMPLIANCE-024, and fails closed on Cedar deny.
25. compliance implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-COMPLIANCE-025, and fails closed on Cedar deny.
26. compliance implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-COMPLIANCE-026, and fails closed on Cedar deny.
27. compliance implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-COMPLIANCE-027, and fails closed on Cedar deny.
28. compliance implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-COMPLIANCE-028, and fails closed on Cedar deny.
29. compliance implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-COMPLIANCE-029, and fails closed on Cedar deny.
30. compliance implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-COMPLIANCE-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j91.compliance.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2C_SIDE_BUSINESS_OPERATOR" &&
  resource.service == "compliance" &&
  resource.journey_id == "j91" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("US-MSB")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J91-COMPLIANCE-001 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-002 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-003 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-004 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-005 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-006 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-007 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-008 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-009 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-010 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-011 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-012 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-013 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-014 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-015 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-016 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-017 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-018 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-019 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-020 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-021 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-022 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-023 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-024 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-025 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-026 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-027 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-028 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-029 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-030 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-031 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-032 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-033 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-034 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-035 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-036 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-037 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-038 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-039 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-040 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-041 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-042 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-043 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-044 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-045 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-046 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-047 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-048 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-049 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-050 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-051 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-052 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-053 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-054 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-055 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-056 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-057 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-058 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-059 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-060 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-061 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-062 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-063 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-064 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-065 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-066 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-067 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-068 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-069 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-070 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-071 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-072 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-073 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-074 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-075 | surety bond packet | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-076 | NMLS evidence upload | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-077 | Cedar-gated payment throttling | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-078 | regulator renewal calendar | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-079 | threshold detection | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-COMPLIANCE-080 | state license gap analysis | journey_id, tenant_id, service=compliance, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-001 sealed |
| 2 | edge | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-002 sealed |
| 3 | api-rest | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-003 sealed |
| 4 | api-async | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-004 sealed |
| 5 | adapter | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-005 sealed |
| 6 | usecase | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-006 sealed |
| 7 | domain | compliance threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-007 sealed |
| 8 | kernel | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-008 sealed |
| 9 | policy | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-009 sealed |
| 10 | eventing | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-010 sealed |
| 11 | observability | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-011 sealed |
| 12 | iac | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-012 sealed |
| 13 | evidence | compliance threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-013 sealed |
| 14 | experience | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-014 sealed |
| 15 | edge | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-015 sealed |
| 16 | api-rest | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-016 sealed |
| 17 | api-async | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-017 sealed |
| 18 | adapter | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-018 sealed |
| 19 | usecase | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-019 sealed |
| 20 | domain | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-020 sealed |
| 21 | kernel | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-021 sealed |
| 22 | policy | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-022 sealed |
| 23 | eventing | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-023 sealed |
| 24 | observability | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-024 sealed |
| 25 | iac | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-025 sealed |
| 26 | evidence | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-026 sealed |
| 27 | experience | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-027 sealed |
| 28 | edge | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-028 sealed |
| 29 | api-rest | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-029 sealed |
| 30 | api-async | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-030 sealed |
| 31 | adapter | compliance threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-031 sealed |
| 32 | usecase | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-032 sealed |
| 33 | domain | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-033 sealed |
| 34 | kernel | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-034 sealed |
| 35 | policy | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-035 sealed |
| 36 | eventing | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-036 sealed |
| 37 | observability | compliance threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-037 sealed |
| 38 | iac | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-038 sealed |
| 39 | evidence | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-039 sealed |
| 40 | experience | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-040 sealed |
| 41 | edge | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-041 sealed |
| 42 | api-rest | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-042 sealed |
| 43 | api-async | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-043 sealed |
| 44 | adapter | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-044 sealed |
| 45 | usecase | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-045 sealed |
| 46 | domain | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-046 sealed |
| 47 | kernel | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-047 sealed |
| 48 | policy | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-048 sealed |
| 49 | eventing | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-049 sealed |
| 50 | observability | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-050 sealed |
| 51 | iac | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-051 sealed |
| 52 | evidence | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-052 sealed |
| 53 | experience | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-053 sealed |
| 54 | edge | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-054 sealed |
| 55 | api-rest | compliance threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-055 sealed |
| 56 | api-async | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-056 sealed |
| 57 | adapter | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-057 sealed |
| 58 | usecase | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-058 sealed |
| 59 | domain | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-059 sealed |
| 60 | kernel | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-060 sealed |
| 61 | policy | compliance threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-061 sealed |
| 62 | eventing | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-062 sealed |
| 63 | observability | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-063 sealed |
| 64 | iac | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-064 sealed |
| 65 | evidence | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-065 sealed |
| 66 | experience | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-066 sealed |
| 67 | edge | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-067 sealed |
| 68 | api-rest | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-068 sealed |
| 69 | api-async | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-069 sealed |
| 70 | adapter | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-070 sealed |
| 71 | usecase | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-071 sealed |
| 72 | domain | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-072 sealed |
| 73 | kernel | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-073 sealed |
| 74 | policy | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-074 sealed |
| 75 | eventing | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-075 sealed |
| 76 | observability | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-076 sealed |
| 77 | iac | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-077 sealed |
| 78 | evidence | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-078 sealed |
| 79 | experience | compliance threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-079 sealed |
| 80 | edge | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-080 sealed |
| 81 | api-rest | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-081 sealed |
| 82 | api-async | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-082 sealed |
| 83 | adapter | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-083 sealed |
| 84 | usecase | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-084 sealed |
| 85 | domain | compliance threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-085 sealed |
| 86 | kernel | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-086 sealed |
| 87 | policy | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-087 sealed |
| 88 | eventing | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-088 sealed |
| 89 | observability | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-089 sealed |
| 90 | iac | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-090 sealed |
| 91 | evidence | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-091 sealed |
| 92 | experience | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-092 sealed |
| 93 | edge | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-093 sealed |
| 94 | api-rest | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-094 sealed |
| 95 | api-async | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-095 sealed |
| 96 | adapter | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-096 sealed |
| 97 | usecase | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-097 sealed |
| 98 | domain | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-098 sealed |
| 99 | kernel | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-099 sealed |
| 100 | policy | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-100 sealed |
| 101 | eventing | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-101 sealed |
| 102 | observability | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-102 sealed |
| 103 | iac | compliance threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-103 sealed |
| 104 | evidence | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-104 sealed |
| 105 | experience | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-105 sealed |
| 106 | edge | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-106 sealed |
| 107 | api-rest | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-107 sealed |
| 108 | api-async | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-108 sealed |
| 109 | adapter | compliance threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-109 sealed |
| 110 | usecase | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-110 sealed |
| 111 | domain | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-111 sealed |
| 112 | kernel | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-112 sealed |
| 113 | policy | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-COMPLIANCE-TASK-113 sealed |
| 114 | eventing | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-COMPLIANCE-TASK-114 sealed |
| 115 | observability | compliance threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-COMPLIANCE-TASK-115 sealed |
| 116 | iac | compliance state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-COMPLIANCE-TASK-116 sealed |
| 117 | evidence | compliance surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-COMPLIANCE-TASK-117 sealed |
| 118 | experience | compliance NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-COMPLIANCE-TASK-118 sealed |
| 119 | edge | compliance Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-COMPLIANCE-TASK-119 sealed |
| 120 | api-rest | compliance regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-COMPLIANCE-TASK-120 sealed |

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
- IP invariant 001: analytics handles threshold detection at ADR-0105 layer experience; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-ANALYTICS-001. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles state license gap analysis at ADR-0105 layer edge; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-API_GATEWAY-002. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles surety bond packet at ADR-0105 layer api-rest; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-APPLICATION-003. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles NMLS evidence upload at ADR-0105 layer api-async; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-AUDIT_CHAIN-004. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles Cedar-gated payment throttling at ADR-0105 layer adapter; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CALENDAR-005. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles regulator renewal calendar at ADR-0105 layer usecase; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CELL-006. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles threshold detection at ADR-0105 layer domain; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-CLOUD_IAC-007. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles state license gap analysis at ADR-0105 layer kernel; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-CLOUD_K8S-008. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles surety bond packet at ADR-0105 layer policy; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-CLOUD_SECRETS-009. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles NMLS evidence upload at ADR-0105 layer eventing; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-COMMS_EMAIL-010. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles Cedar-gated payment throttling at ADR-0105 layer observability; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-COMMUNITY-011. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles regulator renewal calendar at ADR-0105 layer iac; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-COMPLIANCE-012. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles threshold detection at ADR-0105 layer evidence; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CONNECT-013. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles state license gap analysis at ADR-0105 layer experience; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CONSENT_GRAPH-014. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles surety bond packet at ADR-0105 layer edge; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-DEVELOPER_SDK-015. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles NMLS evidence upload at ADR-0105 layer api-rest; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-DOCS-016. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles Cedar-gated payment throttling at ADR-0105 layer api-async; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-DRIVE-017. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles regulator renewal calendar at ADR-0105 layer adapter; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-FEATURE_FLAGS-018. Service compliance remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-journey-j91-us-msb-mtl-overlay.md` matched `payment`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-journey-j91-us-msb-mtl-overlay.md` matched `emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
