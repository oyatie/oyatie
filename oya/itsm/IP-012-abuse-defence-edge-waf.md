---
doc_class: IP
ip_id: IP-012-abuse-defence-edge-waf
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + security-edge
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/policy/abuse-defence.cedar
  - microservices/itsm/dashboards/abuse-defence-outcomes.json
  - microservices/itsm/contracts/openapi-v1.yaml
  - microservices/itsm/manifest.json
---

# IP-012 ITSM Abuse Defence and Edge WAF

## A. Problem
Requester portals, ticket creation, KB search, mobile acknowledgement, and service-catalog requests are abuse-prone. The stamped IP mixed generic WAF language with ITSM actions and did not protect the clean critical path.

The gap is an edge + Cedar defense that throttles suspicious automation while preserving emergency incident creation and on-call acknowledgement.

## B. Approach
Classify edge actions by ITSM risk:

| Surface | Clean-path behavior | Suspicious behavior |
|---|---|---|
| `/v1/incidents` | no friction for authenticated tenant caller | rate limit, step-up, audit |
| portal ticket create | CAPTCHA/step-up only after bot score threshold | queue or deny |
| KB search | allow with request budget | throttle high-cardinality scraping |
| mobile ack | never CAPTCHA an on-call page ack | require signed device token |
| service catalog publish | require Cedar and admin audience | deny requester audience |

Use `policy/abuse-defence.cedar` for service-local authorization and WAF rules at the ingress layer.

## C. Deliverables
- Edge WAF rule inventory for ITSM REST paths in `contracts/openapi-v1.yaml`.
- Cedar action mapping for abuse-defense decisions in `policy/abuse-defence.cedar`.
- Dashboard metrics in `dashboards/abuse-defence-outcomes.json`.
- Runbook entry for false-positive handling.
- Tests that clean P1 incident open and mobile ack do not receive user-hostile friction.

## D. Implementation
1. Enumerate public REST paths and classify by requester/operator/admin audience.
2. Add WAF rules for volumetric ticket creation, KB scrape patterns, malformed ids, and credential stuffing signals.
3. Carry `bot_score`, `rate_limit_bucket`, and `device_attestation` into Cedar context.
4. Add allowlist behavior for emergency/P1 paths: log and rate-limit, but do not block authenticated on-call acknowledge solely for generic bot suspicion.
5. Add denial/refusal audit events with redacted client metadata.
6. Add dashboard panels for clean-path latency, challenge rate, denial rate, and false-positive reopen count.
7. Add tests for requester ticket spam denial and operator P1 path preservation.
8. Document rollback as disabling individual WAF rule groups, not turning off Cedar gates.

## E. Acceptance
- Clean authenticated incident creation remains within ITSM p95 latency budget.
- Suspicious portal abuse is denied or challenged with audit evidence.
- WAF rules do not become authorization; Cedar remains the final gate.
- ServiceNow/Jira/Freshservice import adapters are not accidentally rate-limited by requester rules.

## F. Evidence
- `policy/abuse-defence.cedar` exists for ITSM.
- `dashboards/abuse-defence-outcomes.json` exists.
- `contracts/openapi-v1.yaml` exposes `invokeItsmAction` and capability listing.
- ADR-0145 and ADR-0263 govern abuse defense and evidence behavior.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow portal hardening | Edge rules plus Cedar context instead of opaque platform throttles |
| Jira Service Management customer portal limits | Requester abuse controls without breaking operator emergency flows |
| Freshservice self-service controls | Dashboarded false-positive and denial evidence |

## H. Cold-start buildability notes
- Classify endpoints before writing any ingress rules.
- Keep WAF as signal and throttle; Cedar remains authorization.
- Test clean-path P1 creation before abuse denial tests.
- Use bot score as context, not a direct domain invariant.
- Do not challenge mobile page acknowledgement with CAPTCHA.
- Add false-positive review metrics with operator owner.
- Keep importer traffic in a separate bucket from requester portal traffic.
- Redact IP/user-agent details in audit evidence.
- Roll back individual WAF rule groups rather than disabling all abuse controls.
- Validate OpenAPI paths before naming ingress rules.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
