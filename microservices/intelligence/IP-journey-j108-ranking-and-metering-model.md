---
doc_class: Implementation-Plan
ip_id: IP-journey-j108-ranking-and-metering-model
journey_ref: docs/user-journeys/j108-supplier-rating-and-marketplace-discovery/
microservice: intelligence
status: draft
date: 2026-05-20
authority_tier: 3
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0263-observability-emission-contract
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-tenant-hierarchy
  - ADR-0314-marketplace-universal-deal-settlement-substrate
planned_enforcement_ref: oya-governance-doc-rigor
---

# IP - intelligence role in j108: Supplier rating and marketplace discovery

Role: ranking-and-metering-model.

Journey purpose: KrampusCorp rates AcmeRawMaterials, the rating feeds marketplace ranking, and other buyers discover
vendors through rating-weighted trust signals.

## Scope

intelligence owns only the ranking-and-metering-model slice for j108. It does not absorb another service responsibility,
does not bypass Cedar, and does not write into another tenant-owned store without an explicit grant.

## Acceptance criteria

1. intelligence exposes or consumes the typed j108 contract without ad hoc string parsing.
2. Every state-changing path evaluates Cedar and records the permit id.
3. Every mutation emits an ADR-0263 observability event with audit_id linkage.
4. Rollback exists for each reversible state and pause exists for irreversible state.
5. Cross-tenant reads require explicit tenant pair and purpose.
6. Personal-tenant data is default-deny unless the personal tenant owner consents.
7. The implementation maps to one of the ADR-0105 canonical layers.
8. The test plan includes success, expired-permit, outage, and residency-hold cases.

## Atomic deliverables

| Deliverable | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| Rating submission admitted | marketplace accepts AcmeRawMaterials rating from KrampusCorp | `TenantPrincipal` for buyer tenant; `dispatch-authorization.cedar` admits buyer tenant only | `POST /dispatch` purpose=`marketplace.rating.normalize` with tenant pair + deal_set_id | normalizes rating explanation; marketplace owns stored rating | `dispatch.completed` + audit_tap_record_id | matches AWS Marketplace seller-rating submission boundary |
| Counterparty read check | ranking model needs supplier context | `FoundryAgent` ranking worker; tenant pair and grant_id required; no implicit cross-tenant reads | Cedar `journey.j108.intelligence.ranking_and_metering_model.read_counterparty` | returns feature vector summary, not raw supplier private data | `CrossTenantBoundaryDenied` when grant missing | matches Salesforce marketplace partner-data sharing consent |
| Metering signal emitted | rating affects marketplace discovery score | `FoundryAgent` with internal-foundry audience; `provider-routing.cedar` restricts pack provider | AsyncAPI `intelligence/eval.recorded` and `routing.decided` for model choice | emits low-cardinality ranking feature and cost record | `CostRecord` plus `EvalRecord` retained | matches Google Cloud Marketplace metering/ranking evidence |
| Abuse/spam rating rejected | rating text has scrape/spam or prompt-injection signal | `TenantPrincipal` buyer actor; `abuse-defence.cedar` or `refusal-baseline.cedar` denies before model call | `PromptPart.untrusted_content=true` in `DispatchEnvelope` | rating is held for reviewer; no rank boost applied | `dispatch.refused` gate label and prompt-injection event if detected | matches G2/Capterra review-fraud moderation |
| Provider fallback preserves rank determinism | primary model provider saturated during discovery refresh | `FoundryAgent` ranking worker; `provider-routing.cedar` allows only pack-valid endpoint | `Providers.Health` plus `RoutingDecision` | ranking explanation records provider/model and deterministic seed | `routing.decided` captures provider/model/latency/cost | matches enterprise LLM provider failover controls |
| Auditor evidence packet | supplier disputes a rank change | `Auditor` scoped to buyer/supplier tenant pair; `auditor-scope.cedar` read-only scoped_tenants window | `GET /audit-tap/{envelope_id}` and `Eval.GetRecord` | packet shows rating input hash, feature summary, model route, and audit seal | `AuditTapRecordRef` and `EvalRecord` prove chain | matches marketplace seller appeal evidence |
| Rollback false rank boost | reviewer finds manipulated rating after publish | `FoundryAgent` plus marketplace compensating actor; Intelligence emits eval correction; marketplace owns rank rollback | original envelope_id and idempotency key | rank feature invalidated append-only, no audit deletion | rollback metric + corrected audit-tap event | matches Amazon Marketplace seller-rating correction |
| Personal/work boundary | personal buyer identity attempts work-tenant rating action | `ConsumerEndUser` with personal tenant context; `dispatch-authorization.cedar` and ADR-0311 boundary deny | `DispatchEnvelope.tenant_id` and `audience_tag` mismatch probe | dispatch refused; no marketplace rank feature emitted | `dispatch.refused` plus tenant mismatch evidence | matches LinkedIn personal/company context split |

## Dependencies and non-goals

- Depends on marketplace through a typed contract only; no shared table or hidden callback is allowed.
- Depends on community through a typed contract only; no shared table or hidden callback is allowed.
- Depends on identity through a typed contract only; no shared table or hidden callback is allowed.

## Done evidence

- Journey README links this IP from docs/user-journeys/j108-supplier-rating-and-marketplace-discovery/README.md.
- Integration test plan names intelligence in at least one positive and one failure-injection case.
- Schema docs include the fields this service owns for j108.
- Multispectrum evidence records the doc-only change class.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j108-ranking-and-metering-model.md` matched `cost, emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
