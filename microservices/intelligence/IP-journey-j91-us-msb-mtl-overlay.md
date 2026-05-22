---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
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

# IP - intelligence role in j91 US state money transmitter licensing for Yejin

## Scope

intelligence owns classification assistance, policy summarization, and human-reviewed inference surfaces for j91-us-state-money-transmitter-licensing. The slice is a flat per-microservice implementation plan under microservices/intelligence/, matching ADR-0131.
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

| # | Specific journey action | Acceptance evidence |
|---:|---|---|
| 1 | MSB threshold screen: Intelligence classifies/summarizes `money-transmission facts from onboarding/payment plan` for `Yejin` under `US-MSB`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-001` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 2 | AML program summary: Intelligence classifies/summarizes `AML-control narrative requiring compliance review` for `Yejin` under `US-MSB`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-002` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 3 | SAR trigger explanation: Intelligence classifies/summarizes `suspicious activity summary without filing ownership` for `Yejin` under `US-MSB`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-003` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 4 | California license gap: Intelligence classifies/summarizes `state-required license/surety evidence gap` for `Yejin` under `US-CA-MTL`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-004` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 5 | New York bond evidence: Intelligence classifies/summarizes `bonding/security packet summary` for `Yejin` under `US-NY-MTL`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-005` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 6 | Texas permissible investment check: Intelligence classifies/summarizes `state availability and reserve explanation` for `Yejin` under `US-TX-MTL`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-006` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |
| 7 | Florida/Washington renewal brief: Intelligence classifies/summarizes `renewal-calendar evidence summary` for `Yejin` under `US-FL-MTL, US-WA-MTL`. | Cedar `journey.j91.intelligence.execute` admits only matching tenant/pack; `EVT-J91-INTELLIGENCE-007` is sealed; counterpart equivalence: FinCEN BSA E-Filing + NMLS state license workflow. |

## Contracts

- REST ingress conforms to OpenAPI 3.2.0 schema in the journey schemas directory.
- Event publication conforms to AsyncAPI 3.1.0 channel in the journey schemas directory.
- Internal RPC conforms to proto3 message names PackOverlayAction and PackOverlayDecision.
- State grammar conforms to BNF v4.1 and maps to ADR-0105 13-layer canonical enum.

## Cedar fragments

```cedar
permit (principal == User, action == Action::"journey.j91.intelligence.execute", resource is JourneyPackAction) when {
  principal.audience_type == "B2C_SIDE_BUSINESS_OPERATOR" &&
  resource.service == "intelligence" &&
  resource.journey_id == "j91" &&
  context.authentication_method == "webauthn" &&
  context.audit_session_open == true &&
  context.tenant.compliance_pack_active("US-MSB")
};
```

## ADR-0263 event classes

| Event class | Trigger | Dimensions |
|---|---|---|
| EVT-J91-INTELLIGENCE-001 | MSB threshold screen | journey_id, tenant_id, service=intelligence, pack_id=US-MSB, article_ref="31 CFR 1010.100(ff)", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-002 | AML program summary | journey_id, tenant_id, service=intelligence, pack_id=US-MSB, article_ref="31 CFR 1022.210", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-003 | SAR trigger explanation | journey_id, tenant_id, service=intelligence, pack_id=US-MSB, article_ref="31 CFR 1022.320", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-004 | California license gap | journey_id, tenant_id, service=intelligence, pack_id=US-CA-MTL, article_ref="CA Fin. Code §§2030/2037", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-005 | New York bond evidence | journey_id, tenant_id, service=intelligence, pack_id=US-NY-MTL, article_ref="NY Banking Law Art. 13-B §§641/643", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-006 | Texas permissible investment check | journey_id, tenant_id, service=intelligence, pack_id=US-TX-MTL, article_ref="Texas Finance Code §§151.302/151.308", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |
| EVT-J91-INTELLIGENCE-007 | Florida/Washington renewal brief | journey_id, tenant_id, service=intelligence, pack_id=US-FL-MTL, US-WA-MTL, article_ref="Fla. Ch. 560 / WA RCW 19.230", cedar_decision_id, evidence_hash, counterpart="FinCEN BSA E-Filing + NMLS state license workflow" |

## Build tasks

| Task | Layer | Deliverable | Verification |
|---:|---|---|---|
| 1 | policy | Wire MSB threshold screen through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `31 CFR 1010.100(ff)`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-001`. |
| 2 | api-rest | Wire AML program summary through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `31 CFR 1022.210`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-002`. |
| 3 | api-async | Wire SAR trigger explanation through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `31 CFR 1022.320`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-003`. |
| 4 | proto | Wire California license gap through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `CA Fin. Code §§2030/2037`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-004`. |
| 5 | observability | Wire New York bond evidence through `POST /dispatch`, `GET /audit-tap/{envelope_id}`, `Dispatch.Issue`, `Dispatch.GetAuditTapRecord`, and AsyncAPI `intelligence/audit-tap.committed`; do not invent a journey-specific endpoint. | Contract row cites `NY Banking Law Art. 13-B §§641/643`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-005`. |
| 6 | runbook | Wire Texas permissible investment check through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `Texas Finance Code §§151.302/151.308`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-006`. |
| 7 | evidence | Wire Florida/Washington renewal brief through `Dispatch.GetAuditTapRecord`, `Eval.GetRecord`, and existing intelligence runbooks; do not invent a journey-specific endpoint. | Contract row cites `Fla. Ch. 560 / WA RCW 19.230`, verifies Cedar default-deny, and seals `EVT-J91-INTELLIGENCE-007`. |

## Failure modes and rollback

| Failure | Recovery | Evidence |
|---|---|---|
| Cedar deny | Treat as policy success; return localized refusal or reviewer-escalation packet, no provider call. | `dispatch.refused` event with gate and refusal_reason. |
| Provider unavailable | Use `Providers.Health` and `provider-routing.cedar` fallback; if no permitted provider remains, refuse closed. | `routing.decided` or `dispatch.failed` with provider/model/region. |
| Audit seal failure | Pause downstream action; do not emit unsealed regulatory evidence. | `audit-row-forgery-detected.md` runbook and missing `audit_tap_record_id` alert. |
| Wrong pack applied | Emit append-only correction linked to original idempotency key; never delete audit rows. | rollback metric plus corrected audit-tap record. |

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j91-us-msb-mtl-overlay.md` matched `payment`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j91-us-msb-mtl-overlay.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
