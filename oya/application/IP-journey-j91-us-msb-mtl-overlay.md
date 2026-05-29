---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
status: draft
date: 2026-05-20
microservice: application
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

# IP - application role in j91 US state money transmitter licensing for Yejin

## Scope

application owns user-facing shell state, locale copy routing, and session affordances for j91-us-state-money-transmitter-licensing. The slice is a flat per-microservice implementation plan under microservices/application/, matching ADR-0131.
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

1. application implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-APPLICATION-001, and fails closed on Cedar deny.
2. application implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-APPLICATION-002, and fails closed on Cedar deny.
3. application implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-APPLICATION-003, and fails closed on Cedar deny.
4. application implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-APPLICATION-004, and fails closed on Cedar deny.
5. application implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-APPLICATION-005, and fails closed on Cedar deny.
6. application implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-APPLICATION-006, and fails closed on Cedar deny.
7. application implements threshold detection for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-APPLICATION-007, and fails closed on Cedar deny.
8. application implements state license gap analysis for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-APPLICATION-008, and fails closed on Cedar deny.
9. application implements surety bond packet for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-APPLICATION-009, and fails closed on Cedar deny.
10. application implements NMLS evidence upload for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-APPLICATION-010, and fails closed on Cedar deny.
11. application implements Cedar-gated payment throttling for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-APPLICATION-011, and fails closed on Cedar deny.
12. application implements regulator renewal calendar for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-APPLICATION-012, and fails closed on Cedar deny.
13. application implements threshold detection for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-APPLICATION-013, and fails closed on Cedar deny.
14. application implements state license gap analysis for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-APPLICATION-014, and fails closed on Cedar deny.
15. application implements surety bond packet for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-APPLICATION-015, and fails closed on Cedar deny.
16. application implements NMLS evidence upload for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-APPLICATION-016, and fails closed on Cedar deny.
17. application implements Cedar-gated payment throttling for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-APPLICATION-017, and fails closed on Cedar deny.
18. application implements regulator renewal calendar for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-APPLICATION-018, and fails closed on Cedar deny.
19. application implements threshold detection for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-APPLICATION-019, and fails closed on Cedar deny.
20. application implements state license gap analysis for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-APPLICATION-020, and fails closed on Cedar deny.
21. application implements surety bond packet for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-APPLICATION-021, and fails closed on Cedar deny.
22. application implements NMLS evidence upload for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-APPLICATION-022, and fails closed on Cedar deny.
23. application implements Cedar-gated payment throttling for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-APPLICATION-023, and fails closed on Cedar deny.
24. application implements regulator renewal calendar for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-APPLICATION-024, and fails closed on Cedar deny.
25. application implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-APPLICATION-025, and fails closed on Cedar deny.
26. application implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-APPLICATION-026, and fails closed on Cedar deny.
27. application implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-APPLICATION-027, and fails closed on Cedar deny.
28. application implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-APPLICATION-028, and fails closed on Cedar deny.
29. application implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-APPLICATION-029, and fails closed on Cedar deny.
30. application implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-APPLICATION-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j91.application.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2C_SIDE_BUSINESS_OPERATOR" &&
  resource.service == "application" &&
  resource.journey_id == "j91" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("US-MSB")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J91-APPLICATION-001 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-002 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-003 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-004 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-005 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-006 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-007 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-008 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-009 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-010 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-011 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-012 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-013 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-014 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-015 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-016 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-017 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-018 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-019 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-020 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-021 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-022 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-023 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-024 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-025 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-026 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-027 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-028 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-029 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-030 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-031 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-032 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-033 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-034 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-035 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-036 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-037 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-038 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-039 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-040 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-041 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-042 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-043 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-044 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-045 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-046 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-047 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-048 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-049 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-050 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-051 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-052 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-053 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-054 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-055 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-056 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-057 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-058 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-059 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-060 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-061 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-062 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-063 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-064 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-065 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-066 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-067 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-068 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-069 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-070 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-071 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-072 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-073 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-074 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-075 | surety bond packet | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-076 | NMLS evidence upload | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-077 | Cedar-gated payment throttling | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-078 | regulator renewal calendar | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-079 | threshold detection | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-APPLICATION-080 | state license gap analysis | journey_id, tenant_id, service=application, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-001 sealed |
| 2 | edge | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-002 sealed |
| 3 | api-rest | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-003 sealed |
| 4 | api-async | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-004 sealed |
| 5 | adapter | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-005 sealed |
| 6 | usecase | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-006 sealed |
| 7 | domain | application threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-007 sealed |
| 8 | kernel | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-008 sealed |
| 9 | policy | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-009 sealed |
| 10 | eventing | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-010 sealed |
| 11 | observability | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-011 sealed |
| 12 | iac | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-012 sealed |
| 13 | evidence | application threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-013 sealed |
| 14 | experience | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-014 sealed |
| 15 | edge | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-015 sealed |
| 16 | api-rest | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-016 sealed |
| 17 | api-async | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-017 sealed |
| 18 | adapter | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-018 sealed |
| 19 | usecase | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-019 sealed |
| 20 | domain | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-020 sealed |
| 21 | kernel | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-021 sealed |
| 22 | policy | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-022 sealed |
| 23 | eventing | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-023 sealed |
| 24 | observability | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-024 sealed |
| 25 | iac | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-025 sealed |
| 26 | evidence | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-026 sealed |
| 27 | experience | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-027 sealed |
| 28 | edge | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-028 sealed |
| 29 | api-rest | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-029 sealed |
| 30 | api-async | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-030 sealed |
| 31 | adapter | application threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-031 sealed |
| 32 | usecase | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-032 sealed |
| 33 | domain | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-033 sealed |
| 34 | kernel | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-034 sealed |
| 35 | policy | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-035 sealed |
| 36 | eventing | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-036 sealed |
| 37 | observability | application threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-037 sealed |
| 38 | iac | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-038 sealed |
| 39 | evidence | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-039 sealed |
| 40 | experience | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-040 sealed |
| 41 | edge | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-041 sealed |
| 42 | api-rest | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-042 sealed |
| 43 | api-async | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-043 sealed |
| 44 | adapter | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-044 sealed |
| 45 | usecase | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-045 sealed |
| 46 | domain | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-046 sealed |
| 47 | kernel | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-047 sealed |
| 48 | policy | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-048 sealed |
| 49 | eventing | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-049 sealed |
| 50 | observability | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-050 sealed |
| 51 | iac | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-051 sealed |
| 52 | evidence | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-052 sealed |
| 53 | experience | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-053 sealed |
| 54 | edge | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-054 sealed |
| 55 | api-rest | application threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-055 sealed |
| 56 | api-async | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-056 sealed |
| 57 | adapter | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-057 sealed |
| 58 | usecase | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-058 sealed |
| 59 | domain | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-059 sealed |
| 60 | kernel | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-060 sealed |
| 61 | policy | application threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-061 sealed |
| 62 | eventing | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-062 sealed |
| 63 | observability | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-063 sealed |
| 64 | iac | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-064 sealed |
| 65 | evidence | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-065 sealed |
| 66 | experience | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-066 sealed |
| 67 | edge | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-067 sealed |
| 68 | api-rest | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-068 sealed |
| 69 | api-async | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-069 sealed |
| 70 | adapter | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-070 sealed |
| 71 | usecase | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-071 sealed |
| 72 | domain | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-072 sealed |
| 73 | kernel | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-073 sealed |
| 74 | policy | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-074 sealed |
| 75 | eventing | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-075 sealed |
| 76 | observability | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-076 sealed |
| 77 | iac | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-077 sealed |
| 78 | evidence | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-078 sealed |
| 79 | experience | application threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-079 sealed |
| 80 | edge | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-080 sealed |
| 81 | api-rest | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-081 sealed |
| 82 | api-async | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-082 sealed |
| 83 | adapter | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-083 sealed |
| 84 | usecase | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-084 sealed |
| 85 | domain | application threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-085 sealed |
| 86 | kernel | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-086 sealed |
| 87 | policy | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-087 sealed |
| 88 | eventing | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-088 sealed |
| 89 | observability | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-089 sealed |
| 90 | iac | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-090 sealed |
| 91 | evidence | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-091 sealed |
| 92 | experience | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-092 sealed |
| 93 | edge | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-093 sealed |
| 94 | api-rest | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-094 sealed |
| 95 | api-async | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-095 sealed |
| 96 | adapter | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-096 sealed |
| 97 | usecase | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-097 sealed |
| 98 | domain | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-098 sealed |
| 99 | kernel | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-099 sealed |
| 100 | policy | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-100 sealed |
| 101 | eventing | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-101 sealed |
| 102 | observability | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-102 sealed |
| 103 | iac | application threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-103 sealed |
| 104 | evidence | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-104 sealed |
| 105 | experience | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-105 sealed |
| 106 | edge | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-106 sealed |
| 107 | api-rest | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-107 sealed |
| 108 | api-async | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-108 sealed |
| 109 | adapter | application threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-109 sealed |
| 110 | usecase | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-110 sealed |
| 111 | domain | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-111 sealed |
| 112 | kernel | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-112 sealed |
| 113 | policy | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-APPLICATION-TASK-113 sealed |
| 114 | eventing | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-APPLICATION-TASK-114 sealed |
| 115 | observability | application threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-APPLICATION-TASK-115 sealed |
| 116 | iac | application state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-APPLICATION-TASK-116 sealed |
| 117 | evidence | application surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-APPLICATION-TASK-117 sealed |
| 118 | experience | application NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-APPLICATION-TASK-118 sealed |
| 119 | edge | application Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-APPLICATION-TASK-119 sealed |
| 120 | api-rest | application regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-APPLICATION-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in application; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles threshold detection at ADR-0105 layer experience; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-ANALYTICS-001. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles state license gap analysis at ADR-0105 layer edge; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-API_GATEWAY-002. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles surety bond packet at ADR-0105 layer api-rest; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-APPLICATION-003. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles NMLS evidence upload at ADR-0105 layer api-async; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-AUDIT_CHAIN-004. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles Cedar-gated payment throttling at ADR-0105 layer adapter; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CALENDAR-005. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles regulator renewal calendar at ADR-0105 layer usecase; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CELL-006. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles threshold detection at ADR-0105 layer domain; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-CLOUD_IAC-007. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles state license gap analysis at ADR-0105 layer kernel; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-CLOUD_K8S-008. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles surety bond packet at ADR-0105 layer policy; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-CLOUD_SECRETS-009. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles NMLS evidence upload at ADR-0105 layer eventing; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-COMMS_EMAIL-010. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles Cedar-gated payment throttling at ADR-0105 layer observability; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-COMMUNITY-011. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles regulator renewal calendar at ADR-0105 layer iac; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-COMPLIANCE-012. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles threshold detection at ADR-0105 layer evidence; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CONNECT-013. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles state license gap analysis at ADR-0105 layer experience; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CONSENT_GRAPH-014. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles surety bond packet at ADR-0105 layer edge; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-DEVELOPER_SDK-015. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles NMLS evidence upload at ADR-0105 layer api-rest; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-DOCS-016. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles Cedar-gated payment throttling at ADR-0105 layer api-async; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-DRIVE-017. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles regulator renewal calendar at ADR-0105 layer adapter; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-FEATURE_FLAGS-018. Service application remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
