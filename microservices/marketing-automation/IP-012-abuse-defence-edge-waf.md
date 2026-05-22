---
doc_class: ImplementationPlan
ip_id: IP-012-abuse-defence-edge-waf
microservice: marketing-automation
bounded_contexts: [form, landing-page, webhook-subscription, email-tracking, chatflow]
related_adrs: [ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0297, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-security
tenant_class_aware: true
---

# IP-012: Abuse Defence Edge WAF

## A. Problem

Marketing Automation accepts public traffic through forms, landing pages, tracking pixels, webhook callbacks, chatflows, and mobile SDK events. The stamped IP did not distinguish those surfaces. The real gap is bot and abuse control: public lead capture must block credential stuffing, form spam, tracking-pixel floods, webhook replay, and deliverability reputation attacks without blocking emergency-services bypass traffic described in the service policies.

## B. Approach

Bind `iac/edge-waf.yaml`, `iac/ech-config.yaml`, `policy/abuse-defence.cedar`, `dashboards/abuse-defence-outcomes.json`, and public endpoint contracts into one admission chain. Edge WAF performs cheap bot, replay, and rate checks; Cedar performs tenant and purpose checks; usecase logic persists only accepted command receipts.

## C. Deliverables

| Artifact | Change |
|---|---|
| `iac/edge-waf.yaml` | Add rules for form submit, tracking pixel, webhook callback, and chatflow public ingress. |
| `policy/abuse-defence.cedar` | Preserve bot-score deny and emergency-services exception; add action names for public marketing surfaces. |
| `dashboards/abuse-defence-outcomes.json` | Break out deny reasons by surface and tenant class. |
| `runbooks/bad-audience-import-rollback.md` | Add abuse-driven import quarantine path. |
| `contracts/openapi-v1.yaml` | Mark public submission operations with rate-limit and replay headers when those endpoints are added. |

## D. Implementation

1. Inventory public ingress from IP-031..IP-055: forms, landing pages, tracking, webhooks, chatflow, survey, mobile SDK.
2. Configure WAF rule groups for bot score, signed token replay, tenant cap exhaustion, malicious attachment, and webhook timestamp skew.
3. Pass WAF results into Cedar context as `bot_score`, `replay_detected`, `surface`, and `tenant_class`.
4. Keep `audience_type == "EMERGENCY_SERVICES"` bypass narrow and audit-event-gated through `policy/emergency-services-bypass.cedar`.
5. Add dashboard panels for false-positive release, deny count, and top abusive source prefixes.
6. Add tests for bot_score >= 95 deny and emergency-services audited allow.
7. Document rollback: WAF rule can be disabled per rule id, not per-service, and must leave Cedar default-deny active.

## E. Acceptance

- `cargo run -p oya-dev-cli -- gate validate edge-waf --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate policy-authorization --microservice marketing-automation`
- `kubectl apply --dry-run=server -f microservices/marketing-automation/iac/edge-waf.yaml`
- Manual evidence: every public marketing surface has a WAF rule and audit outcome dimension.

## F. Evidence

- Local IaC: `iac/edge-waf.yaml`, `iac/ech-config.yaml`.
- Local policy: `policy/abuse-defence.cedar`.
- Local dashboard: `dashboards/abuse-defence-outcomes.json`.
- Local runbooks: `bad-audience-import-rollback.md`, `webhook-signature-failure.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Public forms, tracking, and chatflows receive explicit bot controls. |
| Adobe Marketo Engage | Program and form abuse defences are separated from campaign logic. |
| Mailchimp | Audience import and public signup abuse can be quarantined with audit proof. |

## H. Local Traceability

- Edge file: `iac/edge-waf.yaml`.
- Transport file: `iac/ech-config.yaml`.
- Policy file: `policy/abuse-defence.cedar`.
- Policy file: `policy/emergency-services-bypass.cedar`.
- Dashboard: `dashboards/abuse-defence-outcomes.json`.
- Public surface: form submission.
- Public surface: landing page conversion.
- Public surface: email tracking pixel.
- Public surface: webhook callback.
- Public surface: chatflow.
- Public surface: mobile SDK event ingest.
- Cedar context: `bot_score`.
- Cedar context: `replay_detected`.
- Cedar context: `surface`.
- Runbook: `bad-audience-import-rollback.md`.
- Runbook: `webhook-signature-failure.md`.
- Failure state: disabling WAF cannot disable Cedar default deny.
- Failure state: emergency bypass without audit event is a blocker.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
