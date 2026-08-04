---
doc_class: Evidence
status: contract-snapshot
source_task: t_994be41f
generated_at_utc: 2026-07-01T13:22:15Z
claim_ceiling: contract/spec encoding only; no runtime Cloudflare, Envoy, rustls, cert-manager, DNS, ECH key rotation, PQC certificate issuance, aws-lc-rs provider installation, live handshake, rollout, or production-readiness claim
---

# TLS-001 HTTP/3 strict TLS/ECH/PQC posture snapshot

This snapshot records the bounded transport-security posture encoded for one route: `POST /edge/admission` in `oya/api-gateway/contracts/api-gateway.openapi.yaml` (`route_id=edge.admission.v1`). The change is intentionally a contract/spec slice only. It does not apply Kubernetes resources, mutate Cloudflare or Envoy runtime configuration, rotate ECH keys, issue PQC certificates, prove a live TLS handshake, or promote a runtime enforcement gate.

## Authority posture

- Accepted authority: `docs/decisions/ADR-0506-aws-lc-rs-canonical-crypto-provider.md:55-77` makes `aws-lc-rs` the canonical Phase-1 crypto backend and rustls/hyper-rustls provider alignment target.
- Contextual authority: `docs/decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md` is Proposed. It is used only as the source for the target posture fields, not as binding runtime/product/cloud mutation authority.
- Kanban reconciliation guardrail preserved: runtime/product/cloud mutation or enforcement promotion still requires accepted ADR/root-pointer authority or an explicit follow-up Kanban elevation.

## Route posture encoded

The OpenAPI operation `POST /edge/admission` now carries `x-oyatie-transport-security-posture` with:

- `endpoint_coverage`: api-gateway external tenant API surface, north-south public route, `api.oyatie.com`, ECH required, PQC hybrid required, TLS 1.2 grace ineligible.
- `http3_fallback`: preferred `h3` with fallback `[h3, h2, http/1.1]`, 12.5s maximum total fallback budget, canonical `Alt-Svc` declaration (`h3=":443"; ma=86400, h3-29=":443"; ma=86400`), and `protocol_fallback` downgrade log event.
- `strict_tls13`: TLSv1.3 min/max, the closed cipher-suite list (`TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`, `TLS_AES_128_GCM_SHA256`), TLSv1.2 and older forbidden, 0-RTT and renegotiation disallowed.
- `ech`: ECH enabled target, <=24h key rotation target, 48h overlap, X25519 / HKDF-SHA256 / AES-128-GCM HPKE tuple, DNS HTTPS/SVCB distribution, <=3600s DNS TTL, GREASE retry config handling, and plaintext-SNI fallback observability.
- `pqc_hybrid`: `x25519mlkem768` first in supported groups, classical `x25519` fallback, `ed25519+ml_dsa_65` CertificateVerify signature target, and PQC negotiation/fallback metrics.
- `crypto_provider_alignment`: `aws-lc-rs` as the Rust crypto provider, prod `ring` disallowed for this route posture, with `oya-crypto` retained as the Tier-4 future destination after ADR-0506 gates.

## Current-state inventory read but not mutated

- `oya/api-gateway/iac/envoy-config.yaml:26-47` already shows TLSv1.3 min/max, strict TLS 1.3 cipher-suite inventory, `X25519MLKEM768` preference, and a separate HTTP/3 listener stub at `:443/UDP` (`lines 112-120`). This was read as inventory only.
- `oya/api-gateway/iac/ech-config.yaml:8-28` already shows ECH DNS HTTPS/SVCB inventory for `api.oyatie.com`, but it currently documents `ech-config-rotation-days: "90"`; the new route posture therefore records a reconciliation requirement before any live ECH compliance claim.
- `oya/api-gateway/iac/pqc-cert.yaml:8-34` already shows `ed25519+ml_dsa_65`, `api.oyatie.com`, `X25519MLKEM768`, and related signature preferences as inventory only.
- `Cargo.toml:374-411` already shows the workspace `aws-lc-rs` dependency and `hyper-rustls` configured with the `aws-lc-rs` feature.

## Non-claims and follow-up before runtime promotion

- No runtime edge/gateway/cloud resource has been changed or applied.
- No live TLS, QUIC, ECH, PQC, DNS, certificate, provider-installation, observability, rollout, rollback, or production-readiness evidence is claimed.
- The existing ECH inventory's 90-day rotation note conflicts with the encoded <=24h target and must be reconciled by a future runtime/config lane before any live ECH compliance claim.
- ADR-0354 remains contextual until accepted/elevated; this slice does not make its Proposed status binding beyond this bounded posture annotation.

## Runtime design-only reconciliation guard

- Runtime design-only status: `oya/api-gateway/iac/ech-config.yaml` remains unchanged inventory documenting a 90-day rotation while the contract target remains <=24h.
- Runtime promotion remains blocked because no accepted ADR/root-pointer authority currently elevates the Proposed ADR-0354 ECH rotation cadence into runtime authority.
- No live ECH compliance claim is permitted from this snapshot or the OpenAPI posture while that mismatch exists.
- Before runtime promotion, a follow-up lane must provide accepted ADR/root-pointer authority or replace the target authority, change the runtime ConfigMap/IaC under that authority, and record rollout, rollback, and observability checks for ECHConfigList publication, DNS HTTPS/SVCB propagation, retry_configs rate, and plaintext-SNI fallback events.
- Rollback requirement: document restoring the previous ECHConfigList and safely disabling or reverting the stricter rotation job before any runtime apply.
- Observability requirement: prove rotation-job success/failure telemetry, DNS propagation visibility, and fallback event counters before claiming live compliance.

## Verification expectations

- Parse `oya/api-gateway/contracts/api-gateway.openapi.yaml` as YAML.
- Assert `POST /edge/admission` has `x-oyatie-transport-security-posture`.
- Assert accepted authority for the posture is limited to ADR-0506 and ADR-0354 is listed as `contextual_not_binding` with a `proposed_adr_guardrail`.
- Assert the route posture includes HTTP/3 fallback, strict TLS1.3 suite allowlist, ECH endpoint coverage, `x25519mlkem768` first supported group, `ed25519+ml_dsa_65`, and `aws-lc-rs` provider alignment.
- Assert the runtime reconciliation guard rejects a 90-day rotation versus <=24h target mismatch unless the posture records `runtime_promotion_blocked: true`, `no_live_ech_compliance_claim: true`, and accepted-authority requirements for rollout/rollback/observability.
- Assert no `*.generated.json` file is touched by this slice.
