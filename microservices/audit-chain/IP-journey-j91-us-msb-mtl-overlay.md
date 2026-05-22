---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
status: draft
date: 2026-05-20
microservice: audit-chain
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

# IP - audit-chain role in j91 US state money transmitter licensing for Yejin

## Scope

audit-chain owns ADR-0263 event class sealing, Merkle anchoring, and regulator evidence proofs for j91-us-state-money-transmitter-licensing. The slice is a flat per-microservice implementation plan under microservices/audit-chain/, matching ADR-0131.
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

1. audit-chain implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-AUDIT_CHAIN-001, and fails closed on Cedar deny.
2. audit-chain implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-AUDIT_CHAIN-002, and fails closed on Cedar deny.
3. audit-chain implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-AUDIT_CHAIN-003, and fails closed on Cedar deny.
4. audit-chain implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-AUDIT_CHAIN-004, and fails closed on Cedar deny.
5. audit-chain implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-AUDIT_CHAIN-005, and fails closed on Cedar deny.
6. audit-chain implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-AUDIT_CHAIN-006, and fails closed on Cedar deny.
7. audit-chain implements threshold detection for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-AUDIT_CHAIN-007, and fails closed on Cedar deny.
8. audit-chain implements state license gap analysis for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-AUDIT_CHAIN-008, and fails closed on Cedar deny.
9. audit-chain implements surety bond packet for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-AUDIT_CHAIN-009, and fails closed on Cedar deny.
10. audit-chain implements NMLS evidence upload for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-AUDIT_CHAIN-010, and fails closed on Cedar deny.
11. audit-chain implements Cedar-gated payment throttling for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-AUDIT_CHAIN-011, and fails closed on Cedar deny.
12. audit-chain implements regulator renewal calendar for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-AUDIT_CHAIN-012, and fails closed on Cedar deny.
13. audit-chain implements threshold detection for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-AUDIT_CHAIN-013, and fails closed on Cedar deny.
14. audit-chain implements state license gap analysis for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-AUDIT_CHAIN-014, and fails closed on Cedar deny.
15. audit-chain implements surety bond packet for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-AUDIT_CHAIN-015, and fails closed on Cedar deny.
16. audit-chain implements NMLS evidence upload for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-AUDIT_CHAIN-016, and fails closed on Cedar deny.
17. audit-chain implements Cedar-gated payment throttling for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-AUDIT_CHAIN-017, and fails closed on Cedar deny.
18. audit-chain implements regulator renewal calendar for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-AUDIT_CHAIN-018, and fails closed on Cedar deny.
19. audit-chain implements threshold detection for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-AUDIT_CHAIN-019, and fails closed on Cedar deny.
20. audit-chain implements state license gap analysis for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-AUDIT_CHAIN-020, and fails closed on Cedar deny.
21. audit-chain implements surety bond packet for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-AUDIT_CHAIN-021, and fails closed on Cedar deny.
22. audit-chain implements NMLS evidence upload for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-AUDIT_CHAIN-022, and fails closed on Cedar deny.
23. audit-chain implements Cedar-gated payment throttling for j91, cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security, emits EVT-J91-AUDIT_CHAIN-023, and fails closed on Cedar deny.
24. audit-chain implements regulator renewal calendar for j91, cites Washington RCW 19.230.030 license required and 19.230.050 surety bond, emits EVT-J91-AUDIT_CHAIN-024, and fails closed on Cedar deny.
25. audit-chain implements threshold detection for j91, cites 31 CFR 1010.100(ff) money transmitter definition, emits EVT-J91-AUDIT_CHAIN-025, and fails closed on Cedar deny.
26. audit-chain implements state license gap analysis for j91, cites 31 CFR 1022.210 money services business anti-money-laundering program, emits EVT-J91-AUDIT_CHAIN-026, and fails closed on Cedar deny.
27. audit-chain implements surety bond packet for j91, cites 31 CFR 1022.320 suspicious activity reporting for money services businesses, emits EVT-J91-AUDIT_CHAIN-027, and fails closed on Cedar deny.
28. audit-chain implements NMLS evidence upload for j91, cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation, emits EVT-J91-AUDIT_CHAIN-028, and fails closed on Cedar deny.
29. audit-chain implements Cedar-gated payment throttling for j91, cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding, emits EVT-J91-AUDIT_CHAIN-029, and fails closed on Cedar deny.
30. audit-chain implements regulator renewal calendar for j91, cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security, emits EVT-J91-AUDIT_CHAIN-030, and fails closed on Cedar deny.

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j91.audit_chain.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2C_SIDE_BUSINESS_OPERATOR" &&
  resource.service == "audit-chain" &&
  resource.journey_id == "j91" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("US-MSB")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J91-AUDIT_CHAIN-001 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-002 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-003 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-004 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-005 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-006 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-007 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-008 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-009 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-010 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-011 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-012 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-013 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-014 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-015 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-016 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-017 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-018 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-019 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-020 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-021 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-022 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-023 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-024 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-025 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-026 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-027 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-028 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-029 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-030 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-031 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-032 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-033 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-034 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-035 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-036 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-037 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-038 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-039 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-040 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-041 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-042 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-043 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-044 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-045 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-046 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-047 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-048 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-049 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-050 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-051 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-052 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-053 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-054 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-055 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-056 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-057 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-058 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-059 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-060 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-061 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-062 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-063 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-064 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-065 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-066 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-067 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-068 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-069 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-070 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-071 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-072 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-073 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-074 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-075 | surety bond packet | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-076 | NMLS evidence upload | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-077 | Cedar-gated payment throttling | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-078 | regulator renewal calendar | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-079 | threshold detection | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |
| EVT-J91-AUDIT_CHAIN-080 | state license gap analysis | journey_id, tenant_id, service=audit-chain, pack_id, article_ref, cedar_decision_id, evidence_hash |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | experience | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-001 sealed |
| 2 | edge | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-002 sealed |
| 3 | api-rest | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-003 sealed |
| 4 | api-async | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-004 sealed |
| 5 | adapter | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-005 sealed |
| 6 | usecase | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-006 sealed |
| 7 | domain | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-007 sealed |
| 8 | kernel | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-008 sealed |
| 9 | policy | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-009 sealed |
| 10 | eventing | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-010 sealed |
| 11 | observability | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-011 sealed |
| 12 | iac | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-012 sealed |
| 13 | evidence | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-013 sealed |
| 14 | experience | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-014 sealed |
| 15 | edge | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-015 sealed |
| 16 | api-rest | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-016 sealed |
| 17 | api-async | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-017 sealed |
| 18 | adapter | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-018 sealed |
| 19 | usecase | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-019 sealed |
| 20 | domain | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-020 sealed |
| 21 | kernel | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-021 sealed |
| 22 | policy | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-022 sealed |
| 23 | eventing | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-023 sealed |
| 24 | observability | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-024 sealed |
| 25 | iac | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-025 sealed |
| 26 | evidence | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-026 sealed |
| 27 | experience | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-027 sealed |
| 28 | edge | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-028 sealed |
| 29 | api-rest | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-029 sealed |
| 30 | api-async | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-030 sealed |
| 31 | adapter | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-031 sealed |
| 32 | usecase | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-032 sealed |
| 33 | domain | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-033 sealed |
| 34 | kernel | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-034 sealed |
| 35 | policy | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-035 sealed |
| 36 | eventing | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-036 sealed |
| 37 | observability | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-037 sealed |
| 38 | iac | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-038 sealed |
| 39 | evidence | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-039 sealed |
| 40 | experience | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-040 sealed |
| 41 | edge | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-041 sealed |
| 42 | api-rest | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-042 sealed |
| 43 | api-async | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-043 sealed |
| 44 | adapter | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-044 sealed |
| 45 | usecase | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-045 sealed |
| 46 | domain | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-046 sealed |
| 47 | kernel | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-047 sealed |
| 48 | policy | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-048 sealed |
| 49 | eventing | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-049 sealed |
| 50 | observability | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-050 sealed |
| 51 | iac | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-051 sealed |
| 52 | evidence | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-052 sealed |
| 53 | experience | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-053 sealed |
| 54 | edge | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-054 sealed |
| 55 | api-rest | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-055 sealed |
| 56 | api-async | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-056 sealed |
| 57 | adapter | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-057 sealed |
| 58 | usecase | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-058 sealed |
| 59 | domain | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-059 sealed |
| 60 | kernel | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-060 sealed |
| 61 | policy | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-061 sealed |
| 62 | eventing | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-062 sealed |
| 63 | observability | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-063 sealed |
| 64 | iac | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-064 sealed |
| 65 | evidence | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-065 sealed |
| 66 | experience | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-066 sealed |
| 67 | edge | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-067 sealed |
| 68 | api-rest | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-068 sealed |
| 69 | api-async | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-069 sealed |
| 70 | adapter | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-070 sealed |
| 71 | usecase | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-071 sealed |
| 72 | domain | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-072 sealed |
| 73 | kernel | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-073 sealed |
| 74 | policy | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-074 sealed |
| 75 | eventing | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-075 sealed |
| 76 | observability | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-076 sealed |
| 77 | iac | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-077 sealed |
| 78 | evidence | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-078 sealed |
| 79 | experience | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-079 sealed |
| 80 | edge | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-080 sealed |
| 81 | api-rest | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-081 sealed |
| 82 | api-async | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-082 sealed |
| 83 | adapter | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-083 sealed |
| 84 | usecase | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-084 sealed |
| 85 | domain | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-085 sealed |
| 86 | kernel | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-086 sealed |
| 87 | policy | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-087 sealed |
| 88 | eventing | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-088 sealed |
| 89 | observability | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-089 sealed |
| 90 | iac | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-090 sealed |
| 91 | evidence | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-091 sealed |
| 92 | experience | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-092 sealed |
| 93 | edge | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-093 sealed |
| 94 | api-rest | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-094 sealed |
| 95 | api-async | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-095 sealed |
| 96 | adapter | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-096 sealed |
| 97 | usecase | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-097 sealed |
| 98 | domain | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-098 sealed |
| 99 | kernel | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-099 sealed |
| 100 | policy | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-100 sealed |
| 101 | eventing | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-101 sealed |
| 102 | observability | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-102 sealed |
| 103 | iac | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-103 sealed |
| 104 | evidence | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-104 sealed |
| 105 | experience | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-105 sealed |
| 106 | edge | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-106 sealed |
| 107 | api-rest | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-107 sealed |
| 108 | api-async | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-108 sealed |
| 109 | adapter | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-109 sealed |
| 110 | usecase | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-110 sealed |
| 111 | domain | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-111 sealed |
| 112 | kernel | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-112 sealed |
| 113 | policy | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites 31 CFR 1010.100(ff) money transmitter definition; audit EVT-J91-AUDIT_CHAIN-TASK-113 sealed |
| 114 | eventing | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites 31 CFR 1022.210 money services business anti-money-laundering program; audit EVT-J91-AUDIT_CHAIN-TASK-114 sealed |
| 115 | observability | audit-chain threshold detection support with pack US-MSB | Unit/integration check cites 31 CFR 1022.320 suspicious activity reporting for money services businesses; audit EVT-J91-AUDIT_CHAIN-TASK-115 sealed |
| 116 | iac | audit-chain state license gap analysis support with pack US-CA-MTL | Unit/integration check cites California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; audit EVT-J91-AUDIT_CHAIN-TASK-116 sealed |
| 117 | evidence | audit-chain surety bond packet support with pack US-NY-MTL | Unit/integration check cites New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; audit EVT-J91-AUDIT_CHAIN-TASK-117 sealed |
| 118 | experience | audit-chain NMLS evidence upload support with pack US-TX-MTL | Unit/integration check cites Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; audit EVT-J91-AUDIT_CHAIN-TASK-118 sealed |
| 119 | edge | audit-chain Cedar-gated payment throttling support with pack US-FL-MTL | Unit/integration check cites Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; audit EVT-J91-AUDIT_CHAIN-TASK-119 sealed |
| 120 | api-rest | audit-chain regulator renewal calendar support with pack US-WA-MTL | Unit/integration check cites Washington RCW 19.230.030 license required and 19.230.050 surety bond; audit EVT-J91-AUDIT_CHAIN-TASK-120 sealed |

## Failure modes and rollback

- FM-001: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-002: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-003: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-004: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-005: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-006: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-007: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-008: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-009: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-010: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-011: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-012: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-013: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-014: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-015: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-016: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-017: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-018: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-019: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-020: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-021: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-022: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-023: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-024: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-025: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-026: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-027: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-028: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-029: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-030: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-031: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-032: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-033: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-034: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-035: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-036: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-037: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-038: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-039: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-040: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-041: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-042: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-043: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-044: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-045: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-046: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-047: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-048: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-049: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-050: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-051: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-052: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-053: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-054: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-055: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-056: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-057: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-058: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-059: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-060: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-061: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-062: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-063: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-064: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-065: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-066: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-067: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-068: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-069: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-070: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-071: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-072: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-073: Cedar deny in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-074: pack version stale in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-075: cell certification missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-076: event seal timeout in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-077: OpenBao key expired in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-078: cross-pack conflict in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-079: tenant scope mismatch in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- FM-080: article map missing in audit-chain; rollback is append-only compensation, no evidence deletion, workflow-engine resumes from idempotency key.
- IP invariant 001: analytics handles threshold detection at ADR-0105 layer experience; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-ANALYTICS-001. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 002: api-gateway handles state license gap analysis at ADR-0105 layer edge; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-API_GATEWAY-002. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 003: application handles surety bond packet at ADR-0105 layer api-rest; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-APPLICATION-003. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 004: audit-chain handles NMLS evidence upload at ADR-0105 layer api-async; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-AUDIT_CHAIN-004. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 005: calendar handles Cedar-gated payment throttling at ADR-0105 layer adapter; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CALENDAR-005. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 006: cell handles regulator renewal calendar at ADR-0105 layer usecase; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CELL-006. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 007: cloud-iac handles threshold detection at ADR-0105 layer domain; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-CLOUD_IAC-007. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 008: cloud-k8s handles state license gap analysis at ADR-0105 layer kernel; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-CLOUD_K8S-008. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 009: cloud-secrets handles surety bond packet at ADR-0105 layer policy; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-CLOUD_SECRETS-009. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 010: comms-email handles NMLS evidence upload at ADR-0105 layer eventing; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-COMMS_EMAIL-010. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 011: community handles Cedar-gated payment throttling at ADR-0105 layer observability; citation: 31 CFR 1022.320 suspicious activity reporting for money services businesses; evidence: EVT-J91-COMMUNITY-011. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 012: compliance handles regulator renewal calendar at ADR-0105 layer iac; citation: California Financial Code section 2030 license requirement and section 2037 surety/securities obligation; evidence: EVT-J91-COMPLIANCE-012. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 013: connect handles threshold detection at ADR-0105 layer evidence; citation: New York Banking Law Article 13-B sections 641 licensing and 643 security/bonding; evidence: EVT-J91-CONNECT-013. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 014: consent-graph handles state license gap analysis at ADR-0105 layer experience; citation: Texas Finance Code Chapter 151 sections 151.302 license and 151.308 security; evidence: EVT-J91-CONSENT_GRAPH-014. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 015: developer-sdk handles surety bond packet at ADR-0105 layer edge; citation: Florida Statutes Chapter 560 sections 560.125 licensure and 560.209 permissible investments/security; evidence: EVT-J91-DEVELOPER_SDK-015. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 016: docs handles NMLS evidence upload at ADR-0105 layer api-rest; citation: Washington RCW 19.230.030 license required and 19.230.050 surety bond; evidence: EVT-J91-DOCS-016. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 017: drive handles Cedar-gated payment throttling at ADR-0105 layer api-async; citation: 31 CFR 1010.100(ff) money transmitter definition; evidence: EVT-J91-DRIVE-017. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.
- IP invariant 018: feature-flags handles regulator renewal calendar at ADR-0105 layer adapter; citation: 31 CFR 1022.210 money services business anti-money-laundering program; evidence: EVT-J91-FEATURE_FLAGS-018. Service audit-chain remains inside its boundary and does not edit shared ADRs, standards, PRDs, or ARCHITECTURE files.

## Wave 15 counterpart evidence note

This IP is checked against `microservices/audit-chain/competitor-parity-matrix.md` and `microservices/audit-chain/feature-parity-matrix-2026-05-20.md`, not against line count. For the `j91 us msb mtl overlay` slice, the relevant counterpart gap is AWS CloudTrail / Google Cloud Audit Logs / Microsoft Purview Audit parity for searchable immutable audit history, plus Oyatie's additional tenant-verifiable Merkle proof path. The GitHub-pinned root and key manifests from `policy/seal-integrity.md` SI-04 and SI-11 are the evidence channel this implementation must preserve; if the slice cannot publish or verify through that channel, it remains below the Wave 15 substance bar.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-journey-j91-us-msb-mtl-overlay.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/audit-chain/IP-journey-j91-us-msb-mtl-overlay.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/audit-chain/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
