# api-gateway — Competitor parity matrix

**Hyperscaler precedents.**

| Capability | oyatie | AWS API Gateway | Cloudflare | Apigee | Kong | Envoy Gateway |
|---|---|---|---|---|---|---|
| HTTP/3 + QUIC default | Yes (h3 → h2 → h1.1) | h2 default | Yes (h3 default) | h2 default | h2 default | h3 supported |
| TLS 1.3 floor | MUST | Optional | MUST | Optional | Optional | Optional |
| ECH (RFC 9460) | MUST | No | Yes | No | No | Roadmap |
| PQC hybrid (MLKEM-768) | MUST | Roadmap | Yes | Roadmap | No | Roadmap |
| Bot-management ML | Yes (in-house + Cloudflare) | AWS WAF + 3rd party | Yes (native) | No | No | No |
| Per-tenant rate-limit | Yes (Cedar-driven) | Yes (per-key) | Yes (per-rule) | Yes | Yes | Yes |
| Multi-region Anycast | Yes (24 cells) | Yes | Yes | Yes (limited) | DIY | DIY |
| Sov-cell-tier-0 | Yes (KR/CN/AE/KSA/EU/IL5-6) | partial | partial | partial | DIY | DIY |
| SPIFFE workload identity | Yes (mTLS upstream) | IAM SigV4 | mTLS | OIDC | mTLS | mTLS |
| Cedar-gated admission | Yes (caller-side library) | IAM policy | Workers script | OAuth scope | DSL | RBAC |
| Audit-chain Merkle-sealed | Yes (per-event signed) | CloudTrail | Stream | Stream | Stream | Stream |
| Blue/green + canary | Yes (per-route weighted) | Stage variables | Workers | Yes | Yes | Yes |
| Circuit-breaker per-upstream | Yes (per-cell, per-µservice) | Limited | Workers | Yes | Yes | Yes |
| Anti-scrape adaptive challenge | Yes (8 controls) | WAF rules | Turnstile + Bot Management | Optional | Optional | Optional |
| Honeypot route + canary payload | Yes | DIY | DIY | DIY | DIY | DIY |
| Per-tenant per-purpose consent (GDPR) | Yes (X-Oya-Consent-State) | DIY | DIY | DIY | DIY | DIY |
| OpenAPI 3.2.0 contract | Yes | 3.0 | 3.1 | 3.0 | 3.0 | 3.0 |
| AsyncAPI 3.1.0 contract | Yes | partial (SQS) | n/a | n/a | n/a | n/a |
| proto3 gRPC management plane | Yes | No (REST + AWS API) | No (Workers + API) | REST | REST | REST + gRPC |

## A — Why we are above parity

- **ECH MUST.** Most peers consider ECH optional or roadmap. We treat it as floor — sov-cell jurisdictions (KR/CN/EU) increasingly require SNI privacy.
- **PQC MUST.** Record-now-decrypt-later attacks already in play by nation-state adversaries; we are FedRAMP-PMO-ready on day-one.
- **Caller-side Cedar library.** No hot-path network call to policy engine; sub-1ms p99 admission eval.
- **Sov-cell-tier-0 day-one.** AWS / Cloudflare offer sov-cell post-hoc; we ship day-one (per ADR-0250 build-ahead-of-certification).
- **8-control anti-scrape baseline.** No peer ships a full 8-row anti-scrape baseline; ours is mandatory per documentation-rigor.md §3.2.3.

## B — Why we are at parity

- Per-tenant rate-limit (everyone supports this).
- Multi-region Anycast (everyone supports this).
- mTLS upstream (everyone supports this).

## C — Where peers exceed (gaps to close)

- **Native Cloudflare Workers.** Cloudflare's Worker model gives ms-edge custom logic; we use Wasm filters in Envoy (equivalent functionality, similar latency).
- **AWS native IAM SigV4 for service-to-service.** AWS auth model is more ergonomic for AWS-native consumers; we offer SPIFFE + OIDC + SigV4 compat for migration.

## D — References

- AWS API Gateway technical brief 2024
- Cloudflare API Shield + Workers + Bot Management 2024
- Apigee X technical brief 2024
- Kong Gateway 2025
- Envoy Gateway 1.2 release notes 2025
