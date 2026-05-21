---
doc_class: ImplementationPlan
shape: Plan
journey_id: j22
microservice: intelligence
role: spam-classification
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

# IP j22 - intelligence - spam-classification

## A. Intent
Implement `spam-classification` for `personal-mail-inbox-first-week` without modifying PRDs, ADRs, standards, or ARCHITECTURE.md. The slice stays in ADR-0131 flat layout and ADR-0105 layer ownership.

Journey summary: Yejin uses Mail for a week, classifies spam, organizes folders, and unsubscribes without leaking personal mail into work context.

## B. Boundaries
- Owns: `intelligence` responsibility only.
- Consumes: typed capabilities from mail, identity, observability.
- Does not own unrelated polish, cross-service schema rewrites, or new ADR decisions.
- Must cite ADR-0244, ADR-0263, ADR-0273, ADR-0297, ADR-0299, and ADR-0292 when active.

## C. Layer-specific substance

| Layer surface | Specific intelligence responsibility | Grounded artifact |
|---|---|---|
| policy | Evaluate tenant/audience/purpose before any model call for `spam-classification`. | `microservices/intelligence/policy/dispatch-authorization.cedar`, `tenant-scope.cedar`, `abuse-defence.cedar` |
| api-rest | Accept the journey prompt through the existing dispatch envelope rather than inventing a journey endpoint. | `microservices/intelligence/contracts/openapi/intelligence-v1.yaml` `POST /dispatch` |
| api-grpc | Preserve idempotent readback and audit lookup through typed proto messages. | `microservices/intelligence/contracts/proto/intelligence-v1.proto` `Dispatch.Issue`, `Dispatch.Get`, `Dispatch.GetAuditTapRecord` |
| eventing | Emit dispatch/refusal/eval/audit events consumed by audit-chain, finops, and SLO workers. | `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml` |
| observability | Track low-cardinality journey metrics and policy-denied counters; no raw content labels. | `microservices/intelligence/dashboards/intelligence-overview.json`, `slos/*` |
| runbook | Use existing provider/refusal/audit runbooks for operator recovery. | `runbooks/provider-rate-limit-saturation.md`, `runbooks/audit-row-forgery-detected.md`, `runbooks/refusal-false-positive-cascade.md` |

## D. Journey execution rows

| Journey row | Source trigger | Actor | Contract / Cedar probe | State effect | Evidence touch | Counterpart |
|---|---|---|---|---|---|---|
| Mailbox ingest candidate | `mail` emits a new personal inbox message with SPF/DKIM/DMARC verdicts | `ConsumerEndUser` in the same personal tenant; `dispatch-authorization.cedar` admits only matching `tenant_id`; `refusal-baseline.cedar` screens abuse/prompt-injection signals | `POST /dispatch` envelope purpose=`mail.spam_classify` with `untrusted_content=true` prompt part | `DispatchOutcome.output.eval_score` stores spam confidence; Mail owns folder mutation | `intelligence/dispatch.completed` plus `audit_tap_record_id` sealed for the message hash | matches Gmail spam classifier verdict handoff |
| Phishing suspicion | message URL reputation or sender-auth failure reaches Mail | `ConsumerEndUser` mailbox rule executor; `abuse-defence.cedar` denies spoofed audience tags and blocked source reputation | `Dispatch.Issue` with purpose=`mail.phishing_explain` and provider_hint=`auto` | returns localized explanation and `RefusalDecision` when prompt injection is detected | `intelligence/dispatch.refused` or completed event with prompt/output hashes | matches Microsoft Defender phishing-confidence handoff |
| Unsubscribe assist | user selects unsubscribe on a graymail candidate | `ConsumerEndUser` acting on own tenant-scoped mailbox; `tenant-scope.cedar` requires consent_grant_id and remaining AI budget | `POST /dispatch` generates safe unsubscribe summary, not the mailbox mutation | Mail service receives a draft action recommendation with citation spans | `audit-tap.committed` evidence links envelope_id to consent_grant_id | matches Gmail unsubscribe recommendation boundary |
| Appeal false positive | user marks a message as not spam | `ConsumerEndUser` owner of the personal mailbox; `auditor-scope.cedar` prevents unrelated audit reads | `Dispatch.Get` retrieves the prior outcome and `Eval.GetRecord` reads the eval score | eval record is marked for golden-set review; no audit deletion occurs | `intelligence/eval.recorded` metric drives false-positive SLO | matches Outlook junk-mail appeal feedback loop |
| Bulk sender trend | several similar senders cross a tenant-local spam threshold | `FoundryAgent` with `spiffe://oyatie/foundry/*` for aggregate evaluation only; `ci-scope.cedar` and tenant aggregation rules prevent customer audit-tap disclosure | proto `Eval.GetRecord` reads sealed outcomes by envelope_id | updates model-eval backlog; Mail/community own any sender blocking | `oya_intelligence_*` CI metrics and audit seal prove no cross-tenant read | matches Microsoft Defender campaign clustering without exposing mailbox content |
| Provider route fallback | primary provider saturation during first-week classification burst | `ConsumerEndUser` dispatch via platform-default credential; `provider-routing.cedar` refuses disallowed cross-pack provider endpoints | `Providers.Health` and `RoutingDecision.provider` choose permitted pack provider | routing decision changes provider/model while preserving envelope_id idempotency | `intelligence/routing.decided` records provider, model, latency, cost | matches OpenAI/Anthropic enterprise routing observability expectations |
| Audit review | privacy reviewer requests evidence for one spam verdict | `Auditor` scoped to the tenant engagement window; `auditor-scope.cedar` permits read-only audit artifact access | `GET /audit-tap/{envelope_id}` / `Dispatch.GetAuditTapRecord` | returns sealed audit-tap ref; no message body replay unless scoped artifact permits it | `audit_tap_record_id` and Ed25519 signature validate chain inclusion | matches Google Workspace audit-log evidence retrieval |
| Rollback | Mail reverses a spam-folder move after user correction | `ConsumerEndUser` or Mail compensating worker; default-deny remains active; no direct folder mutation from Intelligence | original `envelope_id` is referenced; Intelligence emits explanation/eval correction only | compensating event links original idempotency key and corrected score | `rollback_total` metric and audit seal prove append-only correction | matches Microsoft 365 not-junk recovery model |

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
| counterpart parity | evidence packet maps to Gmail spam/phishing classification and Microsoft Defender for Office anti-spam verdict flows |

## H. Observability
| Signal | Type | Budget |
|---|---|---:|
| `j22.intelligence.spam-classification.request_total` | counter | 200 |
| `j22.intelligence.spam-classification.latency_ms` | histogram | 200 |
| `j22.intelligence.spam-classification.policy_denied_total` | counter | 200 |
| `j22.intelligence.spam-classification.rollback_total` | counter | 200 |

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

The previous numbered appendix checklist repeated one generic instruction and is deleted as ungrounded scaffold. The retained implementation surface is the eight journey rows in §D plus the focused tests in §G. Counterpart equivalence: Gmail spam/phishing classification and Microsoft Defender for Office anti-spam verdict flows.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-journey-j22-spam-classification.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-journey-j22-spam-classification.md` matched `cost, finops`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
