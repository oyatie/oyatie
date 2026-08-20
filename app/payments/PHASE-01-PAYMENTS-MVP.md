---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-payments-mvp
status: Proposed
entry_gate: |
  PRD-payments (Slice 8) merged; ADR-0244 + ADR-0245 + ADR-0246 + ADR-0248 + ADR-0251
  + ADR-0253 + ADR-0254 + ADR-0255 §D-4 accepted; tenancy + cloud-secrets + cloud-iam
  + policy-engine + observability + governance + notifications µservices live
  on `dev` ≥ 1 week each.
exit_gate: |
  All 25 IPs (IP-001..IP-025) merged; oya-governance-doc-set-payments
  green; charge-api availability SLO holds ≥99.95% for ≥30d on staging;
  Stripe platform-facilitator sub-merchant onboarding e2e validated;
  first non-trivial consumer (messenger sticker store + cloud-billing usage
  invoicing) wired and traffic-on-staging green; PCI-DSS L1 v4 control
  matrix evidence complete and signed by ops-security; KR-FSS pack-overlay
  audit-trail evidence complete.
depends_on:
  - milestone: M01-foundation
    phase: P14-cloud-secrets
    reason: OpenBao must be live for PSP credential storage
  - milestone: M01-foundation
    phase: P13-policy-engine
    reason: Cedar evaluation substrate must be live
  - milestone: M01-foundation
    phase: P11-tenancy
    reason: Tenant model + audience_type + compliance_packs[] must be live
  - milestone: M01-foundation
    phase: P15-cloud-iam
    reason: Principal + SVID issuance must be live
owner_team: axis-payments + council-finance + ops-fraud + ops-treasury
date: 2026-05-20
related_adrs:
  - ADR-0145
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0248
  - ADR-0251
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0263
  - ADR-0292
related_specs:
  - /specs/microservices/payments.json
  - /specs/tenant-model.json
  - /specs/compliance-pack-matrix.json
doc_status: published
diataxis_quadrant: how-to
---

# P01-payments-mvp: Payments substrate MVP

> Land the payments µservice end-to-end — Stripe platform-facilitator + sub-merchant + subscription + one-time + payout — with first non-trivial consumers (messenger sticker store + cloud-billing usage invoicing) traffic-on-staging. Compliance posture: PCI-DSS L1 v4 certification-ready + KR-FSS pack overlay wired + EU PSD2 / SCA wired.

---

## §A. Purpose

This phase ships the canonical Stripe platform-facilitator shape for oyatie. The phase advances master-plan principles:

- **Hero-substrate first** (ADR-0245) — payments serves every monetisation surface; product µservices never call PSPs directly.
- **Tenant-scoped money** (ADR-0244) — every charge / refund / payout / sub-merchant row carries tenant context.
- **Provider-BYOK** (ADR-0255 §D-4) — every tenant brings their own Stripe / Adyen / Toss account; oyatie holds zero PSP credentials.
- **Build-ahead-of-certification** (ADR-0250) — PCI-DSS L1 v4 + KR-FSS + EU PSD2 / SCA certified-shape day one.
- **No-stub, no-deferral** — every PSP adapter listed in scope is fully wired by exit-gate; no FUTURE markers.

The phase is part of M02-foundation because every consumer-facing product surface that monetises depends on it.

## §B. Scope

### In-scope

| BC | Bounded contexts | Crates per BC |
|---|---|---|
| `charge` | One-time charge / authorise / capture / void; payment-method tokenisation. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `grpc`, `worker`, `app`, `sdk` |
| `refund` | Full / partial refund; refund-reason taxonomy. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `worker`, `app` |
| `payout` | Bank-account verification + payout scheduling + cooling-period. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `worker`, `app` |
| `dispute` | Chargeback lifecycle + evidence + representment-bundle. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `worker`, `app` |
| `subscription-lifecycle` | Recurring billing + dunning + trial + upgrade / downgrade / cancel. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `worker`, `app` |
| `sub-merchant` | Stripe sub-merchant + KYC / KYB onboarding. | `domain`, `kernel`, `usecase`, `adapter`, `rest`, `worker`, `app` |
| `kyc-kyb` | Document collection + verification + restricted-reason taxonomy. | shared with sub-merchant |
| `settlement` | Daily reconciliation vs PSP settlement report. | `domain`, `usecase`, `adapter`, `worker`, `app` |

### Per-PSP adapter coverage

| PSP | Region | Adapter crate | MVP scope |
|---|---|---|---|
| Stripe | US / EU / global | `oya-payments-adapter-stripe` | Full: charges / refunds / payouts / disputes / subscriptions / sub-merchant. |
| Adyen | EU / interchange-plus | `oya-payments-adapter-adyen` | Full: charges / refunds / payouts / MarketPay. |
| Toss Payments | KR | `oya-payments-adapter-toss` | Charges / refunds / payouts (KR-FSS-licensed). |
| KakaoPay | KR | `oya-payments-adapter-kakaopay` | Charges / refunds (wallet only). |
| LINE Pay | JP / TW / TH | `oya-payments-adapter-line-pay` | Charges / refunds. |
| WeChat Pay | CN | `oya-payments-adapter-wechat-pay` | Charges only (CN-PIPL data-residency). |
| Alipay | CN / global | `oya-payments-adapter-alipay` | Charges only. |

### Compliance pack overlays

- `pack-pci-dss-l1-v4` — mandatory; PAN / PIN never touch oyatie systems (PSP-tokenised path only).
- `pack-kr-fss` — Korean Financial Supervisory Service oversight; KR audit-trail pull surface.
- `pack-eu-psd2-sca` — Strong Customer Authentication; per-tenant SCA challenge wiring.
- `pack-us-state-mtl` — per-US-state money-transmitter licence; per-state restricted-reason taxonomy.
- `pack-ccpa-cpra-2023` — California consumer rights expansion.
- `pack-au-aml-ctf` — AML / CTF Australia.
- `pack-br-lgpd-finance` — BACEN + LGPD financial-sector overlay.
- `pack-in-rbi` — RBI payment-aggregator licensing.
- `pack-cn-pipl-2021` — CN data-residency for WeChat Pay / Alipay.
- `pack-coppa-minor-refusal` — refuse all payments by users <13 per ADR-0292.

### First non-trivial consumers (must be wired by exit-gate)

1. **messenger sticker store** (B2C-personal MVP) — sticker-pack purchase → Stripe-adapter → audit-chain.
2. **cloud-billing usage invoicing** (B2B-work MVP) — usage-metered invoice → Stripe-adapter → audit-chain.

### Out-of-scope (deferred to subsequent phases per master-plan-sequencing)

- Crypto / on-chain settlement (Wave-4 plus).
- Buy-now-pay-later via Klarna / Afterpay (Wave-4).
- Recurring crypto payouts (Wave-5+).
- Multi-currency netting + on-platform FX (Wave-4).
- Embedded-finance partner-bank (Wave-5+ — requires bank-as-a-service licence).
- Tax computation engine (handled by `cloud-billing-tax` µservice already live).
- Marketplace revshare auto-disbursement to >100k creators (Wave-4 scale phase).

## §C. Implementation Plans

| IP | Title | Single-PR-sized |
|---|---|---|
| IP-001 | Domain layer — charge | Yes |
| IP-002 | Domain layer — refund | Yes |
| IP-003 | Domain layer — payout | Yes |
| IP-004 | Domain layer — dispute | Yes |
| IP-005 | Domain layer — subscription | Yes |
| IP-006 | Domain layer — sub-merchant | Yes |
| IP-007 | Kernel — psp-router | Yes |
| IP-008 | Usecase — charge | Yes |
| IP-009 | Usecase — refund | Yes |
| IP-010 | Usecase — payout | Yes |
| IP-011 | Adapter — Stripe | Yes |
| IP-012 | Adapter — Adyen | Yes |
| IP-013 | Adapter — Toss | Yes |
| IP-014 | Adapter — KakaoPay | Yes |
| IP-015 | Adapter — LINE Pay | Yes |
| IP-016 | Adapter — WeChat Pay | Yes |
| IP-017 | REST surface (OpenAPI 3.2.0) | Yes |
| IP-018 | gRPC surface (proto3) | Yes |
| IP-019 | Worker — webhook handler | Yes |
| IP-020 | App — bootstrap | Yes |
| IP-021 | Data-residency routing | Yes |
| IP-022 | KYB onboarding flow | Yes |
| IP-023 | Abuse-defence wiring | Yes |
| IP-024 | Compliance pack overlay — PCI | Yes |
| IP-025 | Compliance pack overlay — KR-FSS | Yes |

Sequencing: domain → kernel → usecase → adapter → REST / gRPC / worker → app. Each IP is single-PR-sized. Acceptance: `cargo nextest run --workspace -p oya-payments-* --release` + lane-green on `oya-governance-doc-set-payments`.

## §D. Risk register

| # | Risk | Likelihood | Impact | Mitigation |
|---:|---|:---:|:---:|---|
| R1 | PSP-credential leak in audit-chain | low | catastrophic | Provider-BYOK + OpenBao + sidecar; never log raw credentials per ADR-0296. |
| R2 | Double-charge from race in PSP retry | medium | high | Idempotency-key UNIQUE constraint per `(tenant_id, idempotency_key)`; 24h replay window. |
| R3 | Dispute-evidence loss | low | high | Evidence stored in immutable object-storage; Merkle-sealed audit per ADR-0028. |
| R4 | KR-FSS audit findings on Q2 2027 pre-cert | medium | medium | Day-one wiring per ADR-0250; mock audit at staging milestone. |
| R5 | PCI scope creep (PAN in logs) | medium | catastrophic | Lint + Cedar gate + redaction-at-OTel-SDK; never accept PAN in any code path outside the PSP tokenisation hop. |
| R6 | Webhook-replay forge | low | high | HMAC + idempotency-key + replay-window ≤5min. |
| R7 | Sub-merchant ToS-violation undetected | medium | medium | Periodic KYC re-verification + ML scoring of sub-merchant transaction patterns. |
| R8 | Cross-tenant payout misroute | low | catastrophic | Default-deny Cedar + payout-eligibility CHECK in DDL + dual-signoff on payout >$10k per tenant policy. |
| R9 | EU PSD2 SCA exemption misuse | medium | high | Per-tenant exemption-rule cedar gate; audit-trail every exemption claim. |
| R10 | CN-PIPL non-compliant data flow for WeChat / Alipay | low | catastrophic | CN-cell-only deployment for CN flows; data-residency Cedar gate. |

## §E. SLO targets at exit

| SLO | Target | Window |
|---|---:|---|
| Charge-API availability | ≥99.95% | 30d rolling |
| Charge-API p99 latency | ≤500ms | 30d rolling |
| Refund-API availability | ≥99.9% | 30d rolling |
| Payout-completion-success | ≥99.9% | 24h rolling |
| Dispute-response-latency | ≤24h p95 | 30d rolling |
| Webhook-delivery-success (outbound) | ≥99.99% within 5min retries | 24h rolling |

## §F. References

- [`PRD.md`](PRD.md).
- [`ARCHITECTURE.md`](ARCHITECTURE.md).
- [`compliance.md`](compliance.md).
- [`threat-model.md`](threat-model.md).
- [ADR-0244 — tenant scoping](../../docs/decisions/ADR-0702-identity-authz-live-apex.md).
- [ADR-0251 — compliance packs](../../docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md).
- [ADR-0255 §D-4 — provider-BYOK](../../docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md).
- [ADR-0263 — observability emission](../../docs/decisions/ADR-0706-observability-live-apex.md).
- [ADR-0292 — minor-protection](../../docs/adr-archive/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md).
- Stripe docs — `stripe.com/docs/connect`.
- Adyen MarketPay docs — `docs.adyen.com/marketpay`.
- KR-FSS oversight — `fss.or.kr` (Korean Financial Supervisory Service).
- PCI-DSS L1 v4 — `pcisecuritystandards.org/pci_security/`.
- EU PSD2 + SCA — `eba.europa.eu` (European Banking Authority).
