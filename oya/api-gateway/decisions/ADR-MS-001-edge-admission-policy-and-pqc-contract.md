---
id: ADR-MS-001
title: Edge admission, Cedar gating, and PQC negotiation contract for api-gateway
status: Proposed
date: 2026-05-20
microservice: api-gateway
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway
  - ADR-0090-hyper-canonical-http-backbone
  - ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation
decision_owner: axis-edge + council-architecture
---

# ADR-MS-001: Edge admission, Cedar gating, and PQC negotiation contract for api-gateway

## Context

- Pressure name: north-south trust compression.
- `api-gateway` is the first service boundary for browser, mobile, webhook, SDK, tenant-admin, and machine clients.
- A gateway that only routes requests would collapse identity, policy, TLS, abuse defence, and audit into downstream services.
- The service PRD and architecture surfaces make `api-gateway` responsible for request admission before downstream mutation.
- The OpenAPI contract exposes `POST /edge/admission` with operation id `admitEdgeRequest`.
- The AsyncAPI contract emits `RequestAdmitted` and `RequestDenied`.
- Local capability records include `north-south-request-admission`, `edge-cedar-eval`, `tls-handshake-terminate`, and `canary-route-shift`.
- Local policy files include `route-authorization.cedar`, `rate-limit.cedar`, `tenant-scope.cedar`, `tls-policy.cedar`, and `sov-cloud-overlay.cedar`.
- Local policy files also include `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, and `public-read.cedar`.
- Local SLOs include edge availability, p50, p95, p99 latency, HTTP/3 negotiation rate, PQC negotiation rate, and TLS handshake success.
- The architecture notes bind the service to tenant lifecycle, identity, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac.
- Constraint name: gateway/service-mesh separation.
- ADR-0182 separates north-south gateway concerns from east-west service mesh concerns.
- The gateway must not become the service mesh or an internal service-to-service policy engine.
- The gateway must terminate public TLS, apply caller admission, and emit evidence before forwarding.
- Downstream service mesh retains east-west mTLS, workload authorization, and mesh-level retry behavior.
- Constraint name: fail-closed mutation path.
- If tenant projection is stale, the gateway applies the most restrictive policy.
- If Cedar fragments mismatch, the gateway fails closed for mutating actions.
- If audit-chain backpressure threatens evidence loss, the gateway stops high-risk mutations.
- Constraint name: cryptographic transition pressure.
- The service already tracks TLS handshake success, HTTP/3 negotiation, and PQC negotiation.
- The edge must advertise post-quantum hybrid negotiation where supported without breaking ordinary TLS 1.3 clients.
- The edge must keep ECH, HSTS, CSP, WAF, and SPIFFE evidence connected to the admission decision.
- Constraint name: hot-tenant isolation.
- A DDoS, bot storm, emergency event, or misconfigured client must not starve unrelated tenants or cells.
- Per-tenant rate limits, abuse score, route sensitivity, pack residency, and canary cohort must be evaluated before admission.
- The final decision needs to be observable with low-cardinality metrics and high-fidelity signed audit events.

## Decision

- Decision name: evidence-first edge admission.
- `api-gateway` will treat `POST /edge/admission` as the canonical north-south decision point.
- The gateway will execute Cedar policy before downstream routing for every mutating request.
- The gateway will execute Cedar policy before read requests that cross tenant, pack, or administrative boundary.
- The gateway will emit `RequestAdmitted` only after tenant scope, identity, route policy, rate limit, abuse, TLS, and residency checks pass.
- The gateway will emit `RequestDenied` for every deny, challenge, fail-closed, or degraded-mode decision.
- The gateway will forward only a signed `AdmissionDecision` envelope to downstream services.
- The envelope includes `tenant_id_hash`, `principal_id`, `route_id`, `method`, `risk_tier`, `policy_version`, `decision`, `evidence_id`, and `traceparent`.
- The gateway will not forward raw bot signals, raw tenant ids in metrics, raw credentials, or full request bodies into audit metrics.
- The gateway will keep tenant ids in signed evidence while metrics use bounded route, pack, cell, and risk labels.
- The gateway will enforce route policies from `policy/route-authorization.cedar`.
- The gateway will enforce tenant policies from `policy/tenant-scope.cedar`.
- The gateway will enforce rate-limit policies from `policy/rate-limit.cedar`.
- The gateway will enforce TLS posture from `policy/tls-policy.cedar`.
- The gateway will enforce sovereign overlays from `policy/sov-cloud-overlay.cedar`.
- The gateway will use cloud-secrets references for signing keys and certificate rotation material.
- The gateway will terminate TLS 1.3 and HTTP/3 at the north-south edge.
- The gateway will prefer hybrid `X25519MLKEM768` when client and cell policy allow it.
- The gateway will fall back to ordinary TLS 1.3 without downgrading route policy.
- The gateway will keep ECH advertised for eligible public hostnames.
- The gateway will require admission evidence before canary cohort shift.
- The gateway will keep edge p50 latency good when <=50ms.
- The gateway will keep edge p95 latency good when <=200ms.
- The gateway will keep edge p99 latency good when <=500ms.
- The gateway will keep edge availability >=99.99% over a rolling 30-day window.
- The gateway will keep TLS handshake success >=99.95%.
- The gateway will keep HTTP/3 negotiation target at 80% where client support permits.
- The gateway will keep PQC negotiation target at 10% during progressive cryptographic rollout.
- The gateway will use local degraded-mode policy when policy-engine is unavailable.
- Degraded mode serves safe reads, denies high-risk writes, and emits signed evidence.
- The gateway will not treat missing audit evidence as a successful request.

## Alternatives Considered

### Alternative 1: Route first and authorize downstream

- Pros: smaller gateway implementation.
- Pros: lets every service own domain-specific authorization.
- Cons: downstream services receive unadmitted traffic.
- Cons: duplicated north-south checks across services.
- Cons: denial evidence becomes inconsistent.
- Cons: hot-tenant protection happens too late.
- Rejected because the edge must provide one admission envelope before routing.

### Alternative 2: Put all policy in the service mesh

- Pros: one internal policy plane for service-to-service traffic.
- Pros: strong workload identity semantics.
- Cons: mesh is not the public edge.
- Cons: HTTP/3, ECH, WAF, route canary, and public client concerns do not belong there.
- Cons: ADR-0182 separates north-south gateway from east-west mesh.
- Rejected because mesh-only policy cannot satisfy public edge admission.

### Alternative 3: Use a managed cloud API gateway as authority

- Pros: mature product and easy first deploy.
- Pros: built-in rate limits and TLS support.
- Cons: policy and evidence semantics differ per provider.
- Cons: pack residency and cell failover are hard to prove uniformly.
- Cons: Cedar fragments and audit-chain evidence become adapter-specific.
- Rejected because Oyatie needs one provider-neutral admission contract.

### Alternative 4: Allow downstream services to bypass gateway for trusted clients

- Pros: lower latency for internal automation.
- Pros: fewer edge dependencies for private traffic.
- Cons: "trusted client" becomes an escape hatch.
- Cons: tenant and pack evidence becomes incomplete.
- Cons: abuse defence and route audit no longer see all north-south traffic.
- Rejected because public and tenant-facing north-south traffic must be admitted once.

### Alternative 5: Require PQC for all clients immediately

- Pros: strongest transition posture.
- Pros: simple compliance story.
- Cons: client support is not universal.
- Cons: emergency access and older SDKs would fail abruptly.
- Cons: progressive rollout metrics would be impossible.
- Rejected because hybrid negotiation must be measured before enforcement expands.

## Consequences

### Positive

- Downstream services receive one signed admission envelope instead of re-parsing edge context.
- Tenant, route, rate, TLS, and abuse decisions become traceable in the audit chain.
- Canary routing can be policy-gated before traffic shift.
- HTTP/3 and PQC rollout become measurable through service-local SLOs.
- Edge failures can be separated from service-mesh and domain-service failures.
- Hot tenants can be isolated through route and tenant rate decisions.
- Deny decisions produce specific evidence instead of vague 403 or timeout responses.
- Gateway dashboards can show TLS, bot, rate, and route health without raw tenant cardinality.

### Negative

- The gateway becomes a high-value service requiring strict change control.
- Bad Cedar fragments can deny valid traffic until rollback.
- Edge latency budget must absorb policy evaluation and evidence emission.
- Cryptographic rollout requires compatibility monitoring across client families.
- Emergency traffic modes need explicit policies rather than manual operator bypass.
- Degraded mode adds operational complexity for policy-engine and audit-chain outages.
- Edge admission tests must cover route families, not just one happy path.

### Neutral

- Service mesh remains authoritative for east-west mTLS and workload identity.
- Downstream services still own domain-specific business invariants.
- WAF and DDoS controls remain edge-adjacent but do not replace Cedar authorization.
- Public health and contract reads may stay anonymous under `public-read.cedar`.
- Route cache is allowed when the policy fragment version and tenant projection are current.

### Follow-up work

- Add admission-envelope compatibility tests for all SDK and webhook clients.
- Add route sensitivity registry for emergency, financial, secret, and ordinary traffic.
- Add dashboard panels for degraded-mode reads and fail-closed writes.
- Add PQC cohort rollout notes to `microservices/api-gateway/migration-playbooks/`.
- Add canary shift proof that `RequestAdmitted` evidence exists before traffic movement.
- Add cell-evac replay fixtures for home-cell failover without residency violation.

## Implementation Notes

### Data Shapes

- `EdgeAdmissionRequest` fields: `tenant_id`, `principal_id`, `method`, `host`, `path`, `route_id`, `body_hash`, `idempotency_key`, `traceparent`, `client_tls`.
- `ClientTlsContext` fields: `tls_version`, `cipher_suite`, `ech_offered`, `h3_negotiated`, `pqc_group`, `sni`, `certificate_fingerprint`.
- `RoutePolicyContext` fields: `route_id`, `route_class`, `risk_tier`, `allowed_methods`, `requires_auth`, `requires_pack`, `canary_cohort`.
- `RateLimitContext` fields: `tenant_id_hash`, `route_id`, `bucket_key`, `limit_per_minute`, `burst`, `remaining`, `reset_at`.
- `AbuseContext` fields: `bot_score_bucket`, `waf_rule_id`, `honeypot_hit`, `credential_stuffing_signal`, `challenge_state`.
- `AdmissionDecision` fields: `decision`, `reason_code`, `policy_version`, `evidence_id`, `route_id`, `forward_cluster`, `headers_to_forward`, `ttl_ms`.
- `RequestAdmitted` fields: `tenant_id_hash`, `route_id`, `policy_version`, `latency_ms`, `tls_posture`, `canary_cohort`, `evidence_id`.
- `RequestDenied` fields: `tenant_id_hash`, `route_id`, `reason_code`, `policy_version`, `challenge_required`, `evidence_id`.

### API Endpoints

- `POST /edge/admission` is the only public admission API.
- `POST /edge/admission` returns `decision=admit`, `decision=deny`, or `decision=challenge`.
- `POST /edge/admission` returns 200 for an evaluated decision envelope.
- `POST /edge/admission` returns 503 only when the gateway cannot safely emit evidence.
- Public traffic must not call downstream service APIs before this endpoint evaluates.
- Internal route reloads are separate operator actions and not part of this public contract.
- Metrics export stays in the service-local telemetry path and not in admission response bodies.

### Cedar Policies

- `policy/route-authorization.cedar` maps route id, method, principal, tenant, and pack to allow or deny.
- `policy/rate-limit.cedar` maps route class and tenant bucket to rate decisions.
- `policy/tenant-scope.cedar` rejects tenant mismatch and missing tenant context.
- `policy/tls-policy.cedar` rejects stale TLS versions and unsafe cipher suites.
- `policy/sov-cloud-overlay.cedar` rejects cross-pack routing that violates residency.
- `policy/abuse-defence.cedar` challenges bot-score and spoofing patterns before forwarding.
- `policy/public-read.cedar` allows only safe public documentation, health, ready, and metrics surfaces.
- `policy/auditor-scope.cedar` allows evidence review without traffic mutation authority.
- `policy/ci-scope.cedar` permits contract and policy validation without production route mutation.

### SLO Targets

- `edge-availability.openslo.yaml`: 30-day edge availability >=99.99%.
- `edge-latency-p50.openslo.yaml`: 50% of gateway-side requests <=50ms.
- `edge-latency-p95.openslo.yaml`: 95% of gateway-side requests <=200ms.
- `edge-latency-p99.openslo.yaml`: 99% of gateway-side requests <=500ms.
- `tls-handshake-success.openslo.yaml`: TLS handshake success >=99.95%.
- `h3-negotiation-rate.openslo.yaml`: HTTP/3 negotiation target 80% where eligible.
- `pqc-negotiation-rate.openslo.yaml`: PQC negotiation target 10% during rollout.

## Verification

- Unit test `admission_envelope_requires_tenant_principal_route_and_trace`.
- Unit test `admission_envelope_redacts_request_body`.
- Unit test `tls_context_rejects_tls12_for_high_risk_routes`.
- Unit test `pqc_group_fallback_does_not_bypass_policy`.
- Unit test `rate_bucket_key_excludes_raw_tenant_id`.
- Cedar test `route_authorization_denies_missing_principal`.
- Cedar test `tenant_scope_denies_cross_tenant_route`.
- Cedar test `rate_limit_denies_bucket_exhaustion`.
- Cedar test `tls_policy_denies_stale_cipher`.
- Cedar test `sov_cloud_overlay_denies_cross_pack_failover`.
- Cedar test `abuse_defence_challenges_bot_score_high`.
- Contract test `api-gateway.openapi.yaml_contains_edge_admission`.
- Contract test `api-gateway.asyncapi.yaml_emits_admitted_and_denied`.
- Integration test `request_admitted_emits_audit_evidence_before_forward`.
- Integration test `request_denied_emits_reason_code_and_policy_version`.
- Integration test `policy_engine_unavailable_enters_degraded_mode`.
- Integration test `audit_backpressure_blocks_high_risk_mutation`.
- Integration test `canary_shift_requires_admission_evidence`.
- Load test `edge_latency_p99_under_500ms_with_policy_eval`.
- Load test `rate_limit_hot_tenant_does_not_starve_control_tenant`.
- Load test `tls_handshake_success_above_9995`.
- Load test `h3_negotiation_above_80_for_eligible_clients`.
- Rollout test `pqc_negotiation_above_10_in_canary_cohort`.
- Chaos test `stale_tenant_projection_applies_most_restrictive_policy`.
- Chaos test `cedar_fragment_mismatch_rolls_back_to_prior_fragment`.
- Chaos test `regional_outage_preserves_residency_boundary`.
- Metric `oya_api_gateway_latency_seconds_bucket`.
- Metric `oya_api_gateway_request_admission_total`.
- Metric `oya_api_gateway_request_denied_total`.
- Metric `oya_api_gateway_tls_handshake_success_ratio`.
- Metric `oya_api_gateway_h3_negotiation_ratio`.
- Metric `oya_api_gateway_pqc_negotiation_ratio`.
- Metric `oya_api_gateway_degraded_mode_total`.
- Dashboard `dashboards/edge-overview.json`.
- Dashboard `dashboards/rate-limit-hits.json`.
- Dashboard `dashboards/tls-health.json`.
- Dashboard `dashboards/bot-score-distribution.json`.
- Runbook check `runbooks/ddos-mitigation.md` covers hot-tenant isolation.
- Runbook check `runbooks/edge-admission-regression.md` covers policy rollback.
- Runbook check `runbooks/cell-evac.md` covers residency-safe failover.
- Promotion gate blocks if edge p99 exceeds 500ms during admission load test.
- Promotion gate blocks if any high-risk mutation lacks `RequestAdmitted` evidence.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0044: Service mesh Istio ambient and Envoy Gateway.
- Oyatie ADR-0090: Hyper canonical HTTP backbone.
- Oyatie ADR-0182: API gateway north-south vs service mesh east-west separation.
- Envoy Proxy documentation: HTTP connection manager, rate limit filter, access log, and TLS configuration.
- Cedar policy language documentation.
- RFC 8446: The Transport Layer Security Protocol Version 1.3.
- RFC 9000: QUIC A UDP-Based Multiplexed and Secure Transport.
- RFC 9114: HTTP/3.
- RFC 9110: HTTP Semantics.
- RFC 9204: QPACK for HTTP/3.
- RFC 9325: Recommendations for Secure Use of TLS and DTLS.
- NIST FIPS 203: Module-Lattice-Based Key-Encapsulation Mechanism Standard.
- NIST SP 800-207: Zero Trust Architecture.
- Google SRE Workbook: Handling overload and multi-window burn rate alerts.
- AWS Builders Library: Using shuffle sharding to isolate workloads.
