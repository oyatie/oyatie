---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
status: draft
date: 2026-05-20
microservice: finops-portal
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

# IP - finops-portal role in j91 US state money transmitter licensing for Yejin

## Scope

finops-portal owns licensing cost, bond threshold, audit cost, and regulator fee operations for j91-us-state-money-transmitter-licensing. The slice is a flat per-microservice implementation plan under microservices/finops-portal/, matching ADR-0131.
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

1. finops-portal implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-FINOPS_PORTAL-001, and fails closed on Cedar deny.
2. finops-portal implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-FINOPS_PORTAL-002, and fails closed on Cedar deny.
3. finops-portal implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-FINOPS_PORTAL-003, and fails closed on Cedar deny.
4. finops-portal implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-FINOPS_PORTAL-004, and fails closed on Cedar deny.
5. finops-portal implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-FINOPS_PORTAL-005, and fails closed on Cedar deny.
6. finops-portal implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-FINOPS_PORTAL-006, and fails closed on Cedar deny.
7. finops-portal implements threshold detection for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-FINOPS_PORTAL-007, and fails closed on Cedar deny.
8. finops-portal implements state license gap analysis for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-FINOPS_PORTAL-008, and fails closed on Cedar deny.
9. finops-portal implements surety bond packet for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-FINOPS_PORTAL-009, and fails closed on Cedar deny.
10. finops-portal implements NMLS evidence upload for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-FINOPS_PORTAL-010, and fails closed on Cedar deny.
11. finops-portal implements Cedar-gated payment throttling for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-FINOPS_PORTAL-011, and fails closed on Cedar deny.
12. finops-portal implements regulator renewal calendar for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-FINOPS_PORTAL-012, and fails closed on Cedar deny.
13. finops-portal implements threshold detection for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-FINOPS_PORTAL-013, and fails closed on Cedar deny.
14. finops-portal implements state license gap analysis for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-FINOPS_PORTAL-014, and fails closed on Cedar deny.
15. finops-portal implements surety bond packet for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-FINOPS_PORTAL-015, and fails closed on Cedar deny.
16. finops-portal implements NMLS evidence upload for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-FINOPS_PORTAL-016, and fails closed on Cedar deny.
17. finops-portal implements Cedar-gated payment throttling for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-FINOPS_PORTAL-017, and fails closed on Cedar deny.
18. finops-portal implements regulator renewal calendar for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-FINOPS_PORTAL-018, and fails closed on Cedar deny.
19. finops-portal implements threshold detection for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-FINOPS_PORTAL-019, and fails closed on Cedar deny.
20. finops-portal implements state license gap analysis for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-FINOPS_PORTAL-020, and fails closed on Cedar deny.
21. finops-portal implements surety bond packet for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-FINOPS_PORTAL-021, and fails closed on Cedar deny.
22. finops-portal implements NMLS evidence upload for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-FINOPS_PORTAL-022, and fails closed on Cedar deny.
23. finops-portal implements Cedar-gated payment throttling for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-FINOPS_PORTAL-023, and fails closed on Cedar deny.
24. finops-portal implements regulator renewal calendar for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-FINOPS_PORTAL-024, and fails closed on Cedar deny.
25. finops-portal implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-FINOPS_PORTAL-025, and fails closed on Cedar deny.
26. finops-portal implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-FINOPS_PORTAL-026, and fails closed on Cedar deny.
27. finops-portal implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-FINOPS_PORTAL-027, and fails closed on Cedar deny.
28. finops-portal implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-FINOPS_PORTAL-028, and fails closed on Cedar deny.
29. finops-portal implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-FINOPS_PORTAL-029, and fails closed on Cedar deny.
30. finops-portal implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-FINOPS_PORTAL-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j91.finops_portal.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2C_SIDE_BUSINESS_OPERATOR" &&
  resource.service == "finops-portal" &&
  resource.journey_id == "j91" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("US-MSB")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J91-FINOPS_PORTAL-001 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-002 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-003 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-004 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-005 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-006 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-007 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-008 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-009 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-010 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-011 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-012 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-013 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-014 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-015 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-016 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-017 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-018 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-019 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-020 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-021 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-022 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-023 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-024 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-025 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-026 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-027 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-028 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-029 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-030 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-031 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-032 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-033 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-034 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-035 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-036 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-037 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-038 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-039 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-040 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-041 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-042 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-043 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-044 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-045 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-046 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-047 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-048 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-049 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-050 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-051 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-052 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-053 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-054 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-055 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-056 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-057 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-058 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-059 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-060 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-061 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-062 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-063 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-064 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-065 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-066 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-067 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-068 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-069 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-070 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-071 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-072 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-073 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-074 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-075 | surety bond packet | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-076 | NMLS evidence upload | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-077 | Cedar-gated payment throttling | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-078 | regulator renewal calendar | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-079 | threshold detection | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-FINOPS_PORTAL-080 | state license gap analysis | journey_id, tenant_id, service=finops-portal, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-001 sealed |
| 2 | edge | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-002 sealed |
| 3 | api-rest | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-003 sealed |
| 4 | api-async | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-004 sealed |
| 5 | adapter | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-005 sealed |
| 6 | usecase | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-006 sealed |
| 7 | domain | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-007 sealed |
| 8 | kernel | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-008 sealed |
| 9 | policy | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-009 sealed |
| 10 | eventing | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-010 sealed |
| 11 | observability | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-011 sealed |
| 12 | iac | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-012 sealed |
| 13 | evidence | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-013 sealed |
| 14 | experience | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-014 sealed |
| 15 | edge | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-015 sealed |
| 16 | api-rest | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-016 sealed |
| 17 | api-async | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-017 sealed |
| 18 | adapter | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-018 sealed |
| 19 | usecase | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-019 sealed |
| 20 | domain | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-020 sealed |
| 21 | kernel | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-021 sealed |
| 22 | policy | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-022 sealed |
| 23 | eventing | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-023 sealed |
| 24 | observability | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-024 sealed |
| 25 | iac | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-025 sealed |
| 26 | evidence | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-026 sealed |
| 27 | experience | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-027 sealed |
| 28 | edge | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-028 sealed |
| 29 | api-rest | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-029 sealed |
| 30 | api-async | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-030 sealed |
| 31 | adapter | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-031 sealed |
| 32 | usecase | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-032 sealed |
| 33 | domain | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-033 sealed |
| 34 | kernel | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-034 sealed |
| 35 | policy | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-035 sealed |
| 36 | eventing | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-036 sealed |
| 37 | observability | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-037 sealed |
| 38 | iac | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-038 sealed |
| 39 | evidence | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-039 sealed |
| 40 | experience | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-040 sealed |
| 41 | edge | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-041 sealed |
| 42 | api-rest | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-042 sealed |
| 43 | api-async | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-043 sealed |
| 44 | adapter | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-044 sealed |
| 45 | usecase | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-045 sealed |
| 46 | domain | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-046 sealed |
| 47 | kernel | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-047 sealed |
| 48 | policy | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-048 sealed |
| 49 | eventing | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-049 sealed |
| 50 | observability | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-050 sealed |
| 51 | iac | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-051 sealed |
| 52 | evidence | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-052 sealed |
| 53 | experience | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-053 sealed |
| 54 | edge | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-054 sealed |
| 55 | api-rest | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-055 sealed |
| 56 | api-async | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-056 sealed |
| 57 | adapter | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-057 sealed |
| 58 | usecase | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-058 sealed |
| 59 | domain | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-059 sealed |
| 60 | kernel | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-060 sealed |
| 61 | policy | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-061 sealed |
| 62 | eventing | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-062 sealed |
| 63 | observability | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-063 sealed |
| 64 | iac | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-064 sealed |
| 65 | evidence | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-065 sealed |
| 66 | experience | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-066 sealed |
| 67 | edge | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-067 sealed |
| 68 | api-rest | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-068 sealed |
| 69 | api-async | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-069 sealed |
| 70 | adapter | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-070 sealed |
| 71 | usecase | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-071 sealed |
| 72 | domain | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-072 sealed |
| 73 | kernel | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-073 sealed |
| 74 | policy | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-074 sealed |
| 75 | eventing | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-075 sealed |
| 76 | observability | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-076 sealed |
| 77 | iac | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-077 sealed |
| 78 | evidence | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-078 sealed |
| 79 | experience | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-079 sealed |
| 80 | edge | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-080 sealed |
| 81 | api-rest | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-081 sealed |
| 82 | api-async | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-082 sealed |
| 83 | adapter | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-083 sealed |
| 84 | usecase | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-084 sealed |
| 85 | domain | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-085 sealed |
| 86 | kernel | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-086 sealed |
| 87 | policy | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-087 sealed |
| 88 | eventing | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-088 sealed |
| 89 | observability | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-089 sealed |
| 90 | iac | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-090 sealed |
| 91 | evidence | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-091 sealed |
| 92 | experience | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-092 sealed |
| 93 | edge | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-093 sealed |
| 94 | api-rest | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-094 sealed |
| 95 | api-async | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-095 sealed |
| 96 | adapter | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-096 sealed |
| 97 | usecase | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-097 sealed |
| 98 | domain | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-098 sealed |
| 99 | kernel | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-099 sealed |
| 100 | policy | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-100 sealed |
| 101 | eventing | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-101 sealed |
| 102 | observability | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-102 sealed |
| 103 | iac | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-103 sealed |
| 104 | evidence | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-104 sealed |
| 105 | experience | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-105 sealed |
| 106 | edge | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-106 sealed |
| 107 | api-rest | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-107 sealed |
| 108 | api-async | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-108 sealed |
| 109 | adapter | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-109 sealed |
| 110 | usecase | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-110 sealed |
| 111 | domain | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-111 sealed |
| 112 | kernel | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-112 sealed |
| 113 | policy | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-FINOPS_PORTAL-TASK-113 sealed |
| 114 | eventing | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-FINOPS_PORTAL-TASK-114 sealed |
| 115 | observability | finops-portal threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-FINOPS_PORTAL-TASK-115 sealed |
| 116 | iac | finops-portal state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-FINOPS_PORTAL-TASK-116 sealed |
| 117 | evidence | finops-portal surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-FINOPS_PORTAL-TASK-117 sealed |
| 118 | experience | finops-portal NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-FINOPS_PORTAL-TASK-118 sealed |
| 119 | edge | finops-portal Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-FINOPS_PORTAL-TASK-119 sealed |
| 120 | api-rest | finops-portal regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-FINOPS_PORTAL-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in finops-portal; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles threshold detection at ADR-0105 layer experience; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-ANALYTICS-001. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles state license gap analysis at ADR-0105 layer edge; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-API_GATEWAY-002. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles surety bond packet at ADR-0105 layer api-rest; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-APPLICATION-003. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles NMLS evidence upload at ADR-0105 layer api-async; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-AUDIT_CHAIN-004. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles Cedar-gated payment throttling at ADR-0105 layer adapter; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CALENDAR-005. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles regulator renewal calendar at ADR-0105 layer usecase; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CELL-006. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles threshold detection at ADR-0105 layer domain; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-CLOUD_IAC-007. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles state license gap analysis at ADR-0105 layer kernel; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-CLOUD_K8S-008. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles surety bond packet at ADR-0105 layer policy; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-CLOUD_SECRETS-009. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles NMLS evidence upload at ADR-0105 layer eventing; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-COMMS_EMAIL-010. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles Cedar-gated payment throttling at ADR-0105 layer observability; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-COMMUNITY-011. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles regulator renewal calendar at ADR-0105 layer iac; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-COMPLIANCE-012. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles threshold detection at ADR-0105 layer evidence; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CONNECT-013. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles state license gap analysis at ADR-0105 layer experience; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CONSENT_GRAPH-014. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles surety bond packet at ADR-0105 layer edge; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-DEVELOPER_SDK-015. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles NMLS evidence upload at ADR-0105 layer api-rest; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-DOCS-016. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles Cedar-gated payment throttling at ADR-0105 layer api-async; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-DRIVE-017. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles regulator renewal calendar at ADR-0105 layer adapter; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-FEATURE_FLAGS-018. Service finops-portal remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
