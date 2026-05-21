---
microservice: connect
doc_class: CompetitorParityMatrix
date: 2026-05-20
owner_team: axis-integration + product
status: Accepted
related_adrs: [ADR-0249]
doc_status: published
---

# Competitor Parity Matrix — connect

Hyperscaler precedents: **Zapier**, **n8n**, **Workato**, **Boomi**, **MuleSoft**, **Tray.io**, **Pipedream**, **AWS EventBridge**.

## Feature parity

| Feature | Zapier | n8n | Workato | Boomi | MuleSoft | Tray.io | Pipedream | AWS EventBridge | oyatie connect (target) |
|---|---|---|---|---|---|---|---|---|---|
| Connector count (catalog) | 6,000+ | 400+ | 1,000+ | 1,000+ | 500+ | 600+ | 1,000+ | (event-driven) | **500 at GA; 1,000 at M02; 5,000 at M03** |
| OAuth broker | Yes | Yes (limited) | Yes | Yes | Yes | Yes | Yes | (not exposed) | **Yes — provider-credential BYOK (ADR-0255 §D-4) + shared** |
| Webhook receiver | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes — per-tenant DNS** |
| Webhook HMAC verify | Some | Some | Yes | Yes | Yes | Yes | Yes | Yes | **Yes — required, constant-time** |
| Data mapper (visual) | Limited | Yes | Yes | Yes | Yes | Yes | Yes (code) | (event transform) | **Yes — visual + AI-assisted (M02)** |
| Schema-drift detection | No | No | Yes | Yes | Yes | Limited | No | No | **Yes — hourly + on-demand** |
| Retry + DLQ | Basic | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes — per-wiring config** |
| Circuit-breaker (vendor outage) | No | No | Yes | Yes | Yes | Yes | No | (via Step Fn) | **Yes — ADR-0145 invariant** |
| Audit chain (Merkle-sealed) | No | No | Limited | Yes | Yes | Limited | No | (via CloudTrail) | **Yes — ADR-0263** |
| Per-tenant provider-credential BYOK | No | Self-host | Yes | Yes | Yes | Yes | No | Yes (IAM) | **Yes — ADR-0255 §D-4** |
| WCAG 2.2 AA compliance | Partial | Self-host | Yes | Partial | Yes | Yes | Partial | (no UI) | **Yes — required** |
| WebAuthn step-up auth | No | No | Limited | Yes | Yes | No | No | (via IAM) | **Yes — ADR-0297** |
| Marketplace publishing | Yes | Yes | Limited | Limited | Yes | Yes | Yes | No | **Yes — multi-category (ADR-0249)** |
| Per-tenant DNS for webhooks | No | Self-host | No | No | Yes | No | No | No | **Yes — ADR-0273** |
| Anti-bot baseline (UX-floor) | Limited | No | Limited | Yes | Yes | Limited | No | (via WAF) | **Yes — §3.2.3 + UX-floor invariants** |
| HTTP/3 + QUIC + ECH + PQC | No | No | No | Limited | Limited | No | No | (CloudFront) | **Yes — ADR-0253** |
| Sovereign-cell residency (KR, EU, CN) | Limited | Self-host | Yes (EU/JP/AU) | Yes | Yes | Limited | No | Yes (regions) | **Yes — pack overlays** |
| Open-source-friendly | No | **Yes** | No | No | No | No | No | No | Substrate is proprietary; SDK/connector specs OSS |

## Differentiation

oyatie connect's deltas vs the hyperscaler bar:
1. **UX-floor enforced abuse-defence**: documentation-rigor §3.2.3 — default-path latency ≤2ms p99 (CI-gated). No competitor enforces this.
2. **Per-tenant DNS + ECH (ADR-0273 + ADR-0253)**: Stripe-tier transport rigor that competitors don't match.
3. **Cedar default-deny + library-first dispatch (ADR-0246)**: every action gate is policy-evaluated; no implicit trust.
4. **Audit chain Merkle-seal (ADR-0263)**: Boomi has audit logs, but oyatie's chain is Merkle-sealed with cosign signatures — non-repudiable.
5. **Cellular shuffle-sharding (ADR-0248)**: AWS-Lambda-equivalent isolation that prevents noisy-neighbor cascades.

## Where oyatie lags (intentional, scoped to M02+)

- Catalog count at GA (500 vs Zapier 6,000) — closed by M03 via marketplace MPO onboarding.
- AI-assisted data mapping — depends on intelligence µservice GA (M02).
- Workflow-engine visual editor parity with Zapier multi-step zaps — depends on workflow-studio µservice (parallel track).

## References

- ADR-0249 multi-category marketplace
- ADR-0297 abuse-defence baseline
- Hyperscaler citations: zapier.com, n8n.io, workato.com, boomi.com, mulesoft.com, tray.io, pipedream.com, aws.amazon.com/eventbridge
