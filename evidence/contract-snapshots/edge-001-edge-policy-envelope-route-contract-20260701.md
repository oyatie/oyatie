---
doc_class: Evidence
status: contract-snapshot
source_task: t_751d2107
generated_at_utc: 2026-07-01T09:24:34Z
claim_ceiling: spec/contract encoding only; no runtime Cloudflare, Envoy, Coraza, rate-limit, Cedar, emergency-services, disaster-mode, live audit, rollout, or production-readiness claim
---

# EDGE-001 edge/network abuse throttle life-safety route contract snapshot

This snapshot records the bounded authority and verification basis for the one-route edge policy envelope encoded in `oya/api-gateway/contracts/api-gateway.openapi.yaml` for `POST /edge/admission` (`route_id=edge.admission.v1`). The repository mutation stays at the OpenAPI contract layer and does not apply Kubernetes resources, update Cloudflare/Envoy/Coraza runtime policy, wire rate-limit stores, change Cedar fragments, provision emergency-services registries, or claim live audit/SLO/rollout evidence.

## Accepted authority used for hard contract fields

- `docs/decisions/ADR-0177-internal-external-api-surface-separation.md:61-77` requires every OpenAPI route to declare public/internal surface classification, with external access to internal-classified routes returning 404.
- `docs/decisions/ADR-0177-internal-external-api-surface-separation.md:94-125` separates public ingress isolation, internal mesh-only access, public/internal dashboards, SDK generation, and documentation surfaces.
- `docs/decisions/ADR-0178-layered-throttling-tiers.md:63-90` defines the four throttle layers evaluated outermost-first: per-IP, per-API-key, per-user, and per-tenant; denial short-circuits and returns 429 plus `Retry-After`.
- `docs/decisions/ADR-0178-layered-throttling-tiers.md:92-152` requires headroom headers for all four layers and per-layer denial/headroom observability.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:40-60` assigns north-south public ingress to Envoy Gateway and east-west internal traffic to Cilium + Istio Ambient with zero overlap.
- `docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md:127-132` keeps public gateway policy at the gateway and Cedar `ext_authz` inside the mesh waypoint.
- `docs/decisions/ADR-0191-edge-authz-tier-vs-origin-cedar-pdp.md:34-84` defines the strict edge-vs-origin authorization split: edge owns IP/ASN/geo/rate/WAF/bot/DDoS/TLS, origin owns identity/PAR/tenant/residency/time/ACR/data-class/purpose/idempotency.
- `docs/decisions/ADR-0191-edge-authz-tier-vs-origin-cedar-pdp.md:86-96` requires edge denies and origin denies to emit distinct audit classes and forbids fail-open origin authorization.

## Context-only Proposed ADRs

The existing Kanban reconciliation comment warned that ADR-0253, ADR-0297, ADR-0298, and ADR-0306 are Proposed. EDGE-001 therefore records their fields as an audit envelope and future-runtime context only. The OpenAPI extension explicitly marks them `contextual_not_binding` and says they require a root pointer, accepted ADR, or follow-up Kanban elevation before runtime/product/cloud mutation.

- `docs/decisions/ADR-0253-network-topology-edge-service-mesh.md:323-365` describes the Cloudflare/edge POP WAF, DDoS, bot-mitigation, TLS, and HTTP/3 context around the accepted gateway/mesh route.
- `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md:426-491` describes the anti-bot, anti-spoof, and anti-scrape matrix across Tier-0 edge, per-service, and Cedar policy.
- `docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md:493-546` rejects single-signal default blocking and default-path CAPTCHA, so the route contract records composite scoring and challenge-only-when-suspicious behavior.
- `docs/decisions/ADR-0298-emergency-services-bypass-life-safety.md:631-685` describes emergency-services attestation headers, first-fragment Cedar composition, distinct audit classes, 200ms p99 budget, 60s revocation, and no-rate-limit invariant.
- `docs/decisions/ADR-0306-disaster-mode-cell-resilience.md:514-624` describes load-shed tiers while preserving emergency-services availability at every tier; the route contract records non-emergency 503 load-shed and emergency bypass availability.

## Route contract encoded

The OpenAPI operation `POST /edge/admission` now carries `x-oyatie-edge-policy-envelope` with these contract fields:

- `public_internal_classification`: marks the route as `api_surface: public`, `public_hostname: api.oyatie.com`, `Public-Preview`, public-SDK scoped, customer-visible, 404 for internal-route misrouting, and cluster-internal mesh handoff only.
- `layered_throttling`: records ADR-0178 order (`per_ip`, `per_api_key`, `per_user`, `per_tenant`), short-circuit semantics, `Retry-After`, `oya-throttle-class`, all four headroom headers, default budgets, counter-store classes, and observability requirements.
- `edge_vs_origin_authz`: records the ADR-0191 no-overlap split, edge-owned concerns and `EdgeDeny` reasons, origin-owned identity/PAR/tenant/residency/ACR/data-class/purpose/idempotency concerns, and fail-closed `ext_authz`.
- `abuse_defence_controls`: records the ADR-0297 context-only anti-bot/anti-spoof/anti-scrape families, the no-single-signal/default-CAPTCHA guardrail, audit event class names, and read-only runtime inventory references to current `edge-waf.yaml` and `cloudflare-config.yaml`.
- `emergency_services_bypass_audit_envelope`: records optional `x-oya-emergency-attestation` and `x-oya-emergency-pack` headers, bypass-before-throttle/abuse/load-shed ordering, no-rate-limit invariant, deferred abuse-control audit, emergency audit event classes, correlation fields, p99 budget, revocation window, and all-load-shed-tier availability.
- Response/header contract: `429` now exposes `Retry-After`, `oya-throttle-class`, and all four headroom headers; `503` now represents non-emergency disaster/load-shed denial with `Retry-After` and `X-Oya-Load-Shed-Tier`.

## Current-state inventory read but not mutated

- `oya/api-gateway/iac/edge-waf.yaml:1-91` already contains anti-bot, anti-spoof, anti-scrape, anti-cache-poisoning WAF rule examples marked as ADR-0297 in flight.
- `oya/api-gateway/iac/cloudflare-config.yaml:1-69` already contains TLS/HTTP3/PQC, bot fight, WAF, DDoS, and rate-limit example configuration marked as ADR-0297 in flight + ADR-0253.
- `oya/api-gateway/capabilities/edge-cedar-eval.yaml:1-86` already references `rate-limit.cedar` and `abuse-defence.cedar` fragments as capability inventory.

This task did not validate those runtime files against a live edge, did not apply them, and did not change them.

## Non-claims and required follow-up before runtime promotion

- No runtime Cloudflare, Envoy Gateway, Coraza, Cilium, Istio, SPIRE/SPIFFE, Cedar PDP, Valkey/Redis, OpenBao, audit-chain, or OTel collector evidence is claimed.
- No emergency-services principal registry, attestation verifier, revocation polling, PSAP/SIP/NCMEC/988/J-ALERT path, or disaster-mode substrate is implemented by this slice.
- No production-readiness, hyperscaler maturity, SLO, rollout, rollback, browser/user-story, or live observability evidence is claimed.
- Future runtime lanes must either elevate/accept the Proposed ADRs or replace them with accepted authority before mutating product/cloud behavior, then prove a real request path emits the throttle, edge/origin authz, abuse-defence, emergency-bypass, trace, and audit evidence.

## Verification expectations

- Parse `oya/api-gateway/contracts/api-gateway.openapi.yaml` as YAML.
- Assert `POST /edge/admission` has both `x-oyatie-gateway-mesh-boundary` and `x-oyatie-edge-policy-envelope`.
- Assert accepted authority is limited to ADR-0177, ADR-0178, ADR-0182, and ADR-0191 for hard edge-policy fields.
- Assert Proposed ADR-0253, ADR-0297, ADR-0298, and ADR-0306 are contextual-only and protected by `proposed_adr_guardrail`.
- Assert the edge envelope includes public/internal classification, four-layer throttle order, edge-vs-origin authz split, abuse-defence families, emergency-services bypass audit envelope, 429 throttle headers, and 503 non-emergency load-shed headers.
- Assert no `*.generated.json` file is touched by this slice.
