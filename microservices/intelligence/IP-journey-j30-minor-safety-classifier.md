---
doc_class: ImplementationPlan
shape: Plan
journey_id: j30
microservice: intelligence
role: minor-safety-classifier
status: Accepted
date: 2026-05-20
authority_tier: 2
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0263
  - ADR-0273
  - ADR-0292
  - ADR-0297
  - ADR-0299
---

# IP j30 - intelligence - minor-safety-classifier

## A. Intent
Implement `minor-safety-classifier` for `shorts-creator-first-post` without modifying PRDs, ADRs, standards, or ARCHITECTURE.md. The slice stays in ADR-0131 flat layout and ADR-0105 layer ownership.

Journey summary: Yejin daughter posts a first short under KOSA-tier defaults with minor protection and appealable moderation.

## B. Boundaries
- Owns: `intelligence` responsibility only.
- Consumes: typed capabilities from shorts, identity, community.
- Does not own unrelated polish, cross-service schema rewrites, or new ADR decisions.
- Must cite ADR-0244, ADR-0263, ADR-0273, ADR-0297, ADR-0299, and ADR-0292 when active.

## C. Layer-specific substance

| Layer surface | Specific intelligence responsibility | Grounded artifact |
|---|---|---|
| policy | Evaluate tenant/audience/purpose before any model call for `minor-safety-classifier`. | `microservices/intelligence/policy/dispatch-authorization.cedar`, `tenant-scope.cedar`, `abuse-defence.cedar` |
| api-rest | Accept the journey prompt through the existing dispatch envelope rather than inventing a journey endpoint. | `microservices/intelligence/contracts/openapi/intelligence-v1.yaml` `POST /dispatch` |
| api-grpc | Preserve idempotent readback and audit lookup through typed proto messages. | `microservices/intelligence/contracts/proto/intelligence-v1.proto` `Dispatch.Issue`, `Dispatch.Get`, `Dispatch.GetAuditTapRecord` |
| eventing | Emit dispatch/refusal/eval/audit events consumed by audit-chain, finops, and SLO workers. | `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml` |
| observability | Track low-cardinality journey metrics and policy-denied counters; no raw content labels. | `microservices/intelligence/dashboards/intelligence-overview.json`, `slos/*` |
| runbook | Use existing provider/refusal/audit runbooks for operator recovery. | `runbooks/provider-rate-limit-saturation.md`, `runbooks/audit-row-forgery-detected.md`, `runbooks/refusal-false-positive-cascade.md` |

## D. Journey execution rows

| Journey row | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| First short preflight | `shorts` submits a draft video caption/transcript for first post | `ConsumerEndUser` minor principal under guardian-linked tenant; `dispatch-authorization.cedar` tenant match plus `IP-024-minor-protection-wiring.md` consent guard | `POST /dispatch` purpose=`shorts.minor_safety_preflight`, modality=`video` or `multi` | returns safety band and explanation; Shorts owns publish/hold state | `intelligence/dispatch.completed` includes output_hash and latency | matches YouTube Kids upload preflight safety check |
| Guardian consent gap | minor post request lacks guardian/KOSA consent grant | `ConsumerEndUser` with minor audience context; `refusal-baseline.cedar` and minor protection policy return consent_missing/coppa refusal | `Dispatch.Issue` envelope omits `consent_grant_id` | publish path halts with localized safe copy, not exception semantics | `intelligence/dispatch.refused` records gate and refusal_reason | matches TikTok Family Pairing consent gate |
| Unsafe comment prompt | caption or prompt contains self-harm or grooming indicators | `ConsumerEndUser` creator with untrusted content flagged; `refusal-baseline.cedar` self_harm/coppa floor applies before provider call | `PromptPart.untrusted_content=true` and purpose=`shorts.safety_classify` | refusal plus crisis-safe wrap where allowed; Community owns escalation ticket | `DispatchRefusedPayload.gate=refusal-baseline` and audit signature | matches YouTube youth-safety escalation handoff |
| Emergency exception | life-safety report is submitted through a crisis tenant | `EMERGENCY_SERVICES` principal with valid attestation; emergency policy bypasses bot/rate/score gates but keeps audit emission | `critical-path-emergency-services.cedar` dispatch path | classification proceeds without user challenge; audit remains tenant-scoped | `AbuseDefenceEmergencyServiceBypass` audit class in policy comments plus dispatch event | matches crisis-line moderation bypass constraints |
| Appeal readback | guardian appeals a blocked first post | guardian-linked `TenantPrincipal` scoped to child tenant; `auditor-scope.cedar`/tenant scope prevents unrelated child data reads | `GET /audit-tap/{envelope_id}` and `Eval.GetRecord` read prior verdict | appeal packet includes model, prompt_template_hash, output_hash, and reason | `intelligence/eval.recorded` and audit-tap record prove verdict lineage | matches YouTube moderation appeal evidence packet |
| Abuse-defense trip | automated scrape/spoof pattern hits minor content endpoint | unattested automated client; `abuse-defence.cedar` forbids dispatch before provider routing | `DispatchEnvelope.request_meta` with blank user-agent or blocked IP reputation | request is refused; no model inference or content retention beyond audit minimum | `intelligence/dispatch.refused` gate=`abuse-defence` | matches Cloudflare Bot Management front-door refusal semantics |
| Pack routing | regional pack demands on-device or pack-local provider for minor content | `ConsumerEndUser` under pack-specific tenant; `provider-routing.cedar` refuses non-pack provider | `RoutingDecision.provider` from `provider-routing.cedar` permitted providers | route uses local/on-device provider or returns provider_saturated refusal | `intelligence/routing.decided` dimensions provider/model/region | matches Apple on-device child-safety routing constraints |
| Post-publish drift sample | sampled post later receives safety reports | `FoundryAgent` eval worker with internal-foundry audience; `ci-scope.cedar` limits aggregate metric reads | `Eval.GetRecord` plus sealed dispatch outcome replay | adds golden-set candidate; Shorts/Community own enforcement action | `oya_intelligence_*` eval and refusal metrics update | matches TikTok trust-and-safety model feedback loop |

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
| counterpart parity | evidence packet maps to YouTube Kids/KOSA youth-safety classification and TikTok minor-safety moderation flows |

## H. Observability
| Signal | Type | Budget |
|---|---|---:|
| `j30.intelligence.minor-safety-classifier.request_total` | counter | 200 |
| `j30.intelligence.minor-safety-classifier.latency_ms` | histogram | 200 |
| `j30.intelligence.minor-safety-classifier.policy_denied_total` | counter | 200 |
| `j30.intelligence.minor-safety-classifier.rollback_total` | counter | 200 |

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

The previous numbered appendix checklist repeated one generic instruction and is deleted as ungrounded scaffold. The retained implementation surface is the eight journey rows in §D plus the focused tests in §G. Counterpart equivalence: YouTube Kids/KOSA youth-safety classification and TikTok minor-safety moderation flows.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j30-minor-safety-classifier.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j30-minor-safety-classifier.md` matched `emission, finops`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
