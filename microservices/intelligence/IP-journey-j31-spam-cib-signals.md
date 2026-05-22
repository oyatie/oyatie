---
doc_class: ImplementationPlan
shape: Plan
journey_id: j31
microservice: intelligence
role: spam-cib-signals
status: Accepted
date: 2026-05-20
authority_tier: 2
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0263
  - ADR-0273
  - ADR-0297
  - ADR-0299
---

# IP j31 - intelligence - spam-cib-signals

## A. Intent
Implement `spam-cib-signals` for `social-broadcast-vs-DM` without modifying PRDs, ADRs, standards, or ARCHITECTURE.md. The slice stays in ADR-0131 flat layout and ADR-0105 layer ownership.

Journey summary: Yejin posts a public Social update about her side business using the same human identity as DM-mode Messenger but a broadcast context.

## B. Boundaries
- Owns: `intelligence` responsibility only.
- Consumes: typed capabilities from social, identity, community.
- Does not own unrelated polish, cross-service schema rewrites, or new ADR decisions.
- Must cite ADR-0244, ADR-0263, ADR-0273, ADR-0297, ADR-0299, and ADR-0292 when active.

## C. Layer-specific substance

| Layer surface | Specific intelligence responsibility | Grounded artifact |
|---|---|---|
| policy | Evaluate tenant/audience/purpose before any model call for `spam-cib-signals`. | `microservices/intelligence/policy/dispatch-authorization.cedar`, `tenant-scope.cedar`, `abuse-defence.cedar` |
| api-rest | Accept the journey prompt through the existing dispatch envelope rather than inventing a journey endpoint. | `microservices/intelligence/contracts/openapi/intelligence-v1.yaml` `POST /dispatch` |
| api-grpc | Preserve idempotent readback and audit lookup through typed proto messages. | `microservices/intelligence/contracts/proto/intelligence-v1.proto` `Dispatch.Issue`, `Dispatch.Get`, `Dispatch.GetAuditTapRecord` |
| eventing | Emit dispatch/refusal/eval/audit events consumed by audit-chain, finops, and SLO workers. | `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml` |
| observability | Track low-cardinality journey metrics and policy-denied counters; no raw content labels. | `microservices/intelligence/dashboards/intelligence-overview.json`, `slos/*` |
| runbook | Use existing provider/refusal/audit runbooks for operator recovery. | `runbooks/provider-rate-limit-saturation.md`, `runbooks/audit-row-forgery-detected.md`, `runbooks/refusal-false-positive-cascade.md` |

## D. Journey execution rows

| Journey row | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| Broadcast draft classify | `social` submits a public post draft with side-business language | `ConsumerEndUser` whose identity also uses private DM mode; `dispatch-authorization.cedar` binds `X-Tenant-Id` to JWT tenant and audience tag | `POST /dispatch` purpose=`social.broadcast_spam_cib_classify` | returns spam/CIB risk score; Social owns publish/rank decision | `intelligence/dispatch.completed` stores prompt_hash/output_hash | matches Meta CIB classifier verdict handoff |
| DM/broadcast boundary | same human identity switches from Messenger DM to public Social broadcast | `ConsumerEndUser` with different context_id; `tenant-scope.cedar` forbids cross-context retrieval | `tenant-scope.cedar` checks context_id and consent_grant_id | no private DM content enters broadcast classifier prompt | `audit-tap.committed` proves envelope prompt refs exclude DM context | matches LinkedIn public/private context boundary |
| Spam burst | many near-duplicate posts appear from a tenant or source cluster | `FoundryAgent` abuse-eval worker; `ci-scope.cedar` allows aggregate `oya_intelligence_*` metrics only | `Eval.GetRecord` over sealed envelope ids, not raw cross-tenant content | flags campaign candidate for Community; no account action inside Intelligence | `intelligence/eval.recorded` and low-cardinality metrics | matches Meta spam campaign clustering |
| Prompt injection in post text | post body tries to override classifier instructions | `ConsumerEndUser` authoring public content; `refusal-baseline.cedar` prompt_injection_detected branch can refuse | `PromptPart.untrusted_content=true` and dispatch purpose includes CIB | returns refusal or low-confidence verdict with citation to unsafe span | `intelligence/prompt-injection.detected` event when classifier flags it | matches OpenAI moderation prompt-injection guardrail |
| Provider route deny | pack policy forbids selected outbound provider for social text | `ConsumerEndUser` under pack-specific tenant; `provider-routing.cedar` refuses cross-pack endpoint | `provider-routing.cedar` evaluates `route_to_provider` | routing falls back to permitted provider or returns provider_saturated | `intelligence/routing.decided` captures provider/model/region | matches enterprise LLM provider allowlist controls |
| Appeal packet | author appeals downranking or refusal | `ConsumerEndUser` owner plus scoped reviewer; `auditor-scope.cedar` read-only evidence scope | `Dispatch.Get` and `GET /audit-tap/{envelope_id}` | appeal includes risk score, refusal reason if any, prompt/output hashes | `audit_tap_record_id` and eval score prove lineage | matches Meta content-moderation appeal evidence |
| Counterparty tenant read blocked | marketplace/community reviewer requests unrelated tenant context | `Auditor` or service principal outside scoped_tenants; `auditor-scope.cedar` and tenant-scope forbid read | `Dispatch.GetAuditTapRecord` attempted with wrong tenant scope | request is denied; no CIB evidence leaks across tenants | `dispatch.refused` or policy-denied metric increments | matches Google Workspace tenant audit isolation |
| Rollback of false positive | Community restores post after CIB false-positive review | `FoundryAgent` eval worker plus Social compensating actor; Intelligence updates eval/golden-set signal only; Social owns restored rank | original envelope_id and idempotency key are referenced | append-only correction event links verdict and appeal outcome | `rollback_total`, `eval.recorded`, and audit seal validate correction | matches Meta false-positive recovery loop |

## E. Contract work
| Surface | Delta |
|---|---|
| OpenAPI 3.2.0 | request, response, and error envelope with tenant_id and idempotency key |
| AsyncAPI 3.1.0 | journey event and compensating rollback event |
| proto3 | internal RPC only when library-first cannot carry the call |
| JSON Schema | shared journey contract under docs/user-journeys |
| Cedar v4.2 | default-deny, explicit allow, abuse-defence branch |

## F. ADR adherence answers

| Authority | Service answer |
|---|---|
| ADR-0244 | Tenant ID in `DispatchEnvelope` and `X-Tenant-Id` must match the principal tenant before dispatch. |
| ADR-0263 | Every admitted, refused, routed, eval, and audit-tap event uses AsyncAPI `intelligence-events-v1.yaml` channels. |
| ADR-0273 | Mail/social signed-payload evidence stays upstream; Intelligence receives hashes/citations, not mailbox ownership. |
| ADR-0292 | Minor or personal-context access is default-deny without explicit consent or guardian scope. |
| ADR-0297 | `abuse-defence.cedar` blocks bot/spoof/scrape patterns before provider routing. |
| ADR-0299 | Appeals and refusals are success states with audit evidence, not silent exceptions. |
| ADR-0311 | Personal/work or public/private identity boundaries are preserved through `context_id`/tenant scope. |
| ADR-0324 | No numbered placeholder tasks remain; deleted rows were ungrounded scaffold expansions. |

## G. Tests

| Test focus | Evidence |
|---|---|
| positive dispatch | `POST /dispatch` or `Dispatch.Issue` returns completed outcome with audit_tap_record_id |
| tenant denial | mismatched `X-Tenant-Id`/principal tenant is refused by `dispatch-authorization.cedar` |
| abuse denial | spoof/scrape/prompt-injection path emits `dispatch.refused` with gate label |
| provider fallback | `Providers.Health` plus `provider-routing.cedar` chooses permitted provider or refuses |
| audit readback | `GET /audit-tap/{envelope_id}` succeeds only for scoped auditor/principal |
| eval feedback | `Eval.GetRecord` records false-positive/false-negative review signal |
| rollback | compensating event references original idempotency key; audit row is not deleted |
| counterpart parity | evidence packet maps to Meta coordinated inauthentic behavior signal review and LinkedIn spam/broadcast ranking safeguards |

## H. Observability
| Signal | Type | Budget |
|---|---|---:|
| `j31.intelligence.spam-cib-signals.request_total` | counter | 200 |
| `j31.intelligence.spam-cib-signals.latency_ms` | histogram | 200 |
| `j31.intelligence.spam-cib-signals.policy_denied_total` | counter | 200 |
| `j31.intelligence.spam-cib-signals.rollback_total` | counter | 200 |

## I. Rollback
Rollback is a compensating event with the original idempotency key, not audit deletion. User copy names the object and action, gives safe retry, and records appeal routing when policy denied the action.

## J. Done definition
- Contract validates.
- Tests cover positive, negative, resilience, rollback paths.
- Audit event appears in ADR-0263 registry follow-up.
- Metrics and traces keep cardinality budget.
- No cross-tenant read or write occurs.
- No placeholder tokens remain.

## Appendix A. Substance-pass deletion note

The previous numbered appendix checklist repeated one generic instruction and is deleted as ungrounded scaffold. The retained implementation surface is the eight journey rows in §D plus the focused tests in §G. Counterpart equivalence: Meta coordinated inauthentic behavior signal review and LinkedIn spam/broadcast ranking safeguards.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j31-spam-cib-signals.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j31-spam-cib-signals.md` matched `finops`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
