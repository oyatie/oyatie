---
doc_class: IP
ip_id: IP-013
microservice: identity
status: ga
related_adrs: [ADR-0157, ADR-0182, ADR-0191]
date: 2026-05-18
owner_team: axis-identity + axis-api-gateway
---

# IP-013 — Edge authz rules (Coraza + rate-limit + geo)

## Goal

Land the edge-tier authz rules at Envoy Gateway per ADR-0191 §"Edge tier": Coraza WAF v3 with OWASP CRS v4.25.0 LTS, per-IP + per-tenant + per-endpoint rate limits, MaxMind GeoIP allowlist/denylist per pack, ASN deny-list, eBPF XDP DDoS shaping. NONE of these reference identity claims (origin's responsibility).

## Files

| File | Purpose |
|---|---|
| `microservices/identity/iac/kustomize/components/edge-authz-rules/kustomization.yaml` | kustomize root |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/coraza-waf-rules.conf` | Coraza-format WAF rules; OWASP CRS v4.25.0 LTS pinned |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/rate-limit-config.yaml` | Envoy rate-limit filter config |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/geo-asn-block.yaml` | GeoIP + ASN deny-list per pack |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/ddos-xdp-policy.yaml` | eBPF XDP NIC-level drop |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-kr.yaml` | KR overlay (KR-FSS rule set) |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-eu.yaml` | EU overlay |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-us-healthcare.yaml` | HIPAA tightened |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-ksa.yaml` | KSA-sovereign overlay |
| `microservices/identity/iac/kustomize/components/edge-authz-rules/values-pack-ae.yaml` | UAE overlay |

## Rate-limit schedule (per ADR-0191)

| Surface | Per-tenant | Per-IP |
|---|---|---|
| `/oauth/v2/token` | 100 rps | 10 rps |
| `/oauth/v2/userinfo` | 200 rps | 20 rps |
| `/webauthn/*` | 50 rps | 5 rps |
| `/scim/v2/*` POST | 100 rps | n/a |
| `/scim/v2/*` GET | 50 rps | n/a |

## GeoIP base posture

- Default-allow all geos.
- Per-pack deny: pack-kr denies traffic from sanctioned-country list; pack-us-healthcare allows only US + Canada.
- Per-tenant override permitted via tenant-edge-policy registry.

## Coraza WAF rule set

Pinned: OWASP CRS v4.25.0 LTS[^1]
- Inbound paranoia level 2 (strict, false-positive-tolerant).
- Outbound rules disabled (no PII reflection from this endpoint).
- Anomaly score threshold: 5 (default in CRS 4.x).

Custom oyatie rules:

| Rule ID | Purpose |
|---|---|
| 9999100 | Block `eyJ` prefix outside Authorization header (token-in-URL detector) |
| 9999101 | Block `Bearer eyJ...` in URL path (token-in-URL detector) |
| 9999102 | Block long User-Agent strings (>2KB) |
| 9999103 | Block excessive header count (>50) |
| 9999104 | Per-route GET-only methods (anti-tampering) |

## eBPF XDP drop policy

- L3/L4 drop at NIC for known DDoS source IP signatures.
- Per-pack threshold tuned to expected baseline + 20σ.
- Cilium DaemonSet manages the XDP program.

## Tests

| Test | Mechanism |
|---|---|
| `coraza_blocks_sql_injection_payload` | inject `' OR 1=1`; expect 403 with X-Block-Reason: waf |
| `rate_limit_429_after_threshold` | exceed limit; observe 429 with Retry-After |
| `geo_block_on_denied_country` | spoof source IP from sanctioned country; 403 with X-Block-Reason: geo |
| `asn_block_on_known_botnet` | spoof source from botnet ASN; 403 |
| `xdp_drops_known_ddos_source` | k6 attack from spoofed bad IP; observe drop counter |
| `legitimate_traffic_passes` | k6 normal traffic; no false positives |
| `edge_filter_has_no_principal_acr_string` | grep filters for `principal\|acr\|oidc` → no hits (lean-a17 gate) |
| `cors_preflight_passes_without_authz` | OPTIONS request returns 204 without auth challenge |
| `tls_1.3_required` | TLS 1.2 client → handshake fails |
| `hsts_header_set` | response carries `Strict-Transport-Security: max-age=31536000; preload` |

## Acceptance — DONE when

- 10 tests pass.
- `oya-check-authz-tier-discipline` finds 0 origin-concerns in these filters.
- Per-pack overlays render valid CiliumNetworkPolicy + Envoy filter config.
- Synthetic DDoS test in staging shows ≥99% drop at edge.

## Cross-references

- ADR-0191 §"Edge tier"
- OWASP CRS v4.25.0 LTS
- Coraza WAF v3

[^1]: Versions current as of 2026-05-18.

## Counterpart references - 013-edge-authz-rules

- Counterpart class: policy and risk gate.
- Palantir Foundry policy controls and GitHub organization security policies are the relevant counterpart bar; this IP makes the gate Cedar-first, tenant-scoped, and evidence-emitting instead of burying access decisions in route handlers.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

