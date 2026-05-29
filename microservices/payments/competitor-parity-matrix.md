---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + council-product + ops-treasury
related_adrs: [ADR-0249]
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/cost-budget.md
diataxis_quadrant: reference
doc_status: published
---

# Competitor Parity Matrix — payments µservice

> Stripe vs Adyen vs Toss vs PayPal vs Coinbase Commerce vs Square — feature parity matrix per ADR-0249 multi-category marketplace doctrine.

---

## §1. Competitor map

| Competitor | Position | Strengths | Weaknesses for our use-case |
|---|---|---|---|
| **Stripe** | The platform-facilitator benchmark standard | Connect, SCA, MarketPay, dunning, fraud-Radar, Identity, Treasury, Atlas | US-base latency for KR / JP; no domestic KR-FSS-licence |
| **Adyen** | EU-native enterprise payments | Interchange-plus pricing; MarketPay; eCom + retail unified; strong fraud-prevention | Lower brand-recognition with SMB tenants |
| **Toss Payments** | KR-native payments leader | KR-FSS-licensed; KR card-network direct; Toss Pay wallet | KR-only; KRW-only |
| **PayPal Business** | Consumer-recognised checkout brand | Brand + buyer-protection | Higher fees; weaker developer experience; complex Connect-equivalent (PayPal Marketplaces) |
| **Square** | SMB + retail | POS hardware; small-merchant tier; Square Capital | Limited Connect-like marketplace surface |
| **Coinbase Commerce** | Crypto checkout | Native BTC/ETH/USDC; 1.0% flat fee | Volatility; regulatory uncertainty per region |
| **Klarna / Afterpay** (post-MVP) | BNPL | Higher consumer-AOV | Per-region licence; not MVP scope |
| **WeChat Pay / Alipay** | CN-domestic + cross-border | Massive CN reach | Per-CN-licence; PIPL data-residency constraint |
| **KakaoPay** | KR-mobile-wallet | KR-Kakao-Talk integration | KR-only |
| **LINE Pay** | JP / TW / TH mobile | LINE messenger integration | Per-region; JPY / TWD / THB |
| **Chargebee** | Subscription billing platform | Subscription + dunning + invoicing focused | Wraps PSPs; not a PSP itself |
| **Recurly** | Subscription billing platform | Subscription-only | Wraps PSPs |
| **Paddle** | Merchant-of-record (MoR) | Tax + compliance handled by Paddle | We are facilitator, not MoR |
| **AWS Marketplace** | Usage-metered B2B billing | Marketplace primitive | Closed-AWS-ecosystem |
| **Apple Pay / Google Pay** | Wallet | Consumer-side wallet | Not a PSP; layers on top of card-networks |

## §2. Feature parity matrix — core payment primitives

| Feature | Stripe | Adyen | Toss | PayPal | Square | Coinbase | **Oyatie payments** |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| One-time charge | yes | yes | yes | yes | yes | yes | **yes** (M02) |
| Refund (full + partial) | yes | yes | yes | yes | yes | partial | **yes** (M02) |
| Dispute / chargeback flow | yes | yes | yes | yes | yes | no | **yes** (M02) |
| Payout to bank | yes | yes | yes | yes | yes | no | **yes** (M02) |
| Sub-merchant onboarding (Connect-equivalent) | yes (Connect) | yes (MarketPay) | partial | partial | no | no | **yes** (M02) |
| Subscription billing | yes (Billing) | partial | no | partial | partial | no | **yes** (M02) |
| Smart-retry dunning | yes | yes | no | weak | weak | no | **yes** (M02 — library-first; superior to Stripe Billing per cost-model) |
| Multi-PSP routing | no (vendor lock) | no | no | no | no | no | **yes** (unique to oyatie — per-tenant policy) |
| Provider-BYOK (tenant brings own PSP account) | n/a | n/a | n/a | n/a | n/a | n/a | **yes** (per ADR-0255 §D-4 — unique to oyatie) |
| PCI-DSS L1 v4 | yes | yes | yes | yes | yes | n/a | **yes (target Q4 2026)** |
| KR-FSS licence | partial (Korea-Stripe-equivalent) | no | yes | no | no | no | **inherited via Toss** |
| EU PSD2 SCA support | yes | yes | n/a | yes | yes | n/a | **yes** (M02) |
| Marketplace revshare automation | yes (Express) | yes (MarketPay) | no | yes | no | no | **yes** (M02 per ADR-0249) |
| Cross-currency settlement | yes | yes | no | yes | yes | yes | **yes** (M02 — Stripe + Adyen native) |
| Idempotency-key API | yes | yes | partial | weak | yes | partial | **yes** |
| Webhook HMAC signing | yes | yes | yes | yes | yes | yes | **yes** |
| Card-Present (POS) | yes (Terminal) | yes (Adyen POS) | partial | yes | yes (POS) | no | **out-of-scope MVP (post-Wave-4)** |
| Crypto checkout | partial (Crypto on-ramp) | no | no | partial | no | yes | **out-of-scope MVP (post-Wave-4)** |
| BNPL | yes (Klarna integration) | yes | partial | yes (Pay-in-4) | yes | no | **post-MVP (Wave-4)** |
| Apple Pay / Google Pay wallet | yes | yes | yes | yes | yes | yes | **yes** (M02 via PSP-pass-through) |
| Tap-to-Pay | yes (Stripe Terminal) | yes | no | yes (Zettle) | yes | no | **out-of-scope MVP** |

## §3. Feature parity matrix — marketplace surfaces (ADR-0249 multi-category)

| Marketplace category | Stripe | Adyen MarketPay | Paddle MoR | **Oyatie payments** |
|---|:--:|:--:|:--:|:--:|
| Plugin / app store | yes | yes | yes | **yes** (M02) |
| Workflow templates | partial | partial | partial | **yes** (M02 per ADR-0249) |
| Agent templates | no | no | no | **yes** (M02 — oyatie-first category) |
| Model rental | no | no | no | **yes** (M02 — oyatie-first category) |
| Dataset access | no | no | no | **yes** (M02 — oyatie-first category) |
| Creator tipping | partial | partial | no | **yes** (M02) |
| Sticker / digital-goods | partial | partial | no | **yes** (M02) |
| Sub-merchant payouts | yes | yes | yes (MoR-aggregated) | **yes** (M02) |
| Cross-category revshare | partial | partial | no | **yes (ADR-0249 unique)** |

## §4. Compliance parity matrix

| Pack | Stripe | Adyen | Toss | KakaoPay | LINE Pay | WeChat | Alipay | **Oyatie** |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| PCI-DSS L1 v4 | yes | yes | yes | yes | yes | yes | yes | **yes (Q4 2026)** |
| KR-FSS | partial | no | yes | yes | no | no | no | **yes (via Toss)** |
| EU PSD2 SCA | yes | yes | n/a | n/a | partial | n/a | n/a | **yes** |
| US state MTL | yes | yes | n/a | n/a | n/a | n/a | partial | **yes (per-state per-tenant)** |
| CN PIPL | n/a | n/a | n/a | n/a | n/a | yes | yes | **yes (CN-cell hard-pin)** |
| AU AML / CTF | yes | yes | n/a | n/a | n/a | n/a | n/a | **yes (M02)** |
| BR LGPD + BACEN | yes | yes | n/a | n/a | n/a | n/a | n/a | **yes (M02)** |
| IN RBI | partial | partial | n/a | n/a | n/a | n/a | n/a | **yes (M02)** |
| COPPA <13 refusal | n/a (PSP-level) | n/a | n/a | n/a | n/a | n/a | n/a | **yes (per ADR-0292)** |
| KOSA tier 14-17 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | **yes (per ADR-0292)** |

## §5. Pricing parity (consumer-facing only — wholesale rates negotiated with PSP)

| Surface | Stripe (list) | Adyen (interchange-plus mid) | Toss | **Oyatie list (pass-through + platform-fee 0-2%)** |
|---|---:|---:|---:|---:|
| US card (online) | 2.9% + $0.30 | ~1.4% + $0.10 (large) | n/a | **list = tenant's PSP list + 0-2% platform-fee** |
| EU card | 2.9% + €0.25 | ~1.0% + €0.10 (large) | n/a | **list = tenant's PSP list + 0-2% platform-fee** |
| KR card | n/a | n/a | 2.5% + ₩60 | **list = Toss list + 0-2% platform-fee** |
| Sub-merchant payout | $0.25 + 0.25% | €0.25 | ₩500 | **list = tenant's PSP list + 0-2% platform-fee** |

## §6. Strategic differentiation

What oyatie payments uniquely offers:

1. **Multi-PSP routing** — no other facilitator routes per-tenant across Stripe + Adyen + Toss + KakaoPay + LINE Pay + WeChat + Alipay. Each tenant picks their own PSP-set with fail-over policy.
2. **Provider-BYOK** — tenant brings their own PSP account; oyatie holds zero PSP credentials. Eliminates the per-PSP per-tenant onboarding-against-our-account that competitors require.
3. **Day-one pack-overlay** per ADR-0250 — every certification target is wired day one; QSA + KR-FSS audits validate the shape rather than require architectural change.
4. **Audit-chain Merkle seal** per ADR-0028 — unique to oyatie; competitors have audit logs but not Merkle-sealed chains with per-µservice signing keys.
5. **Cedar universal gate** per ADR-0243 — every action gated by policy, evaluated library-first, soak-window-protected, drift-detected.
6. **Cross-category marketplace** per ADR-0249 — plugins / apps / workflows / agents / models / datasets, all with payments support, all with per-tenant revshare.
7. **Self-modification awareness** per ADR-0247 — Foundry can modify Cedar fragments (with soak + attestation); never modifies contracts / human-authored docs.
8. **HTTP/3 + ECH + PQC default** per ADR-0253 — first-class post-quantum cryptography in payments substrate.

## §7. Anti-features (what we explicitly do NOT do)

- We are NOT a Merchant-of-Record (MoR). Paddle is a MoR; we are a facilitator above per-tenant-owned PSP accounts.
- We do NOT own platform-master-PSP-account that aggregates funds (provider-BYOK only).
- We do NOT mark up PSP fees (the platform fee is a separate line-item; PSP fees are pass-through).
- We do NOT operate as a card-issuer (Stripe Issuing, Adyen Capital are out-of-scope).
- We do NOT operate as a card-acquirer (we always sit above PSP, never become a card-network member).
- We do NOT bypass SCA / KYC (compliance-pack overlays enforce them by default).

## §8. References

- Stripe docs — `stripe.com/docs/connect`.
- Adyen MarketPay docs — `docs.adyen.com/marketpay`.
- Toss Payments docs — `docs.tosspayments.com`.
- Paddle merchant-of-record — `paddle.com/billing`.
- Coinbase Commerce — `commerce.coinbase.com`.
- [`PRD.md`](PRD.md).
- [`cost-budget.md`](cost-budget.md).
- [`compliance.md`](compliance.md).
- [ADR-0249 — multi-category marketplace](../../docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md).
- [ADR-0250 — build-ahead-of-certification](../../docs/decisions/ADR-0250-build-ahead-of-certification.md).
- [ADR-0255 §D-4 — provider-BYOK](../../docs/decisions/ADR-0255-intelligence-two-layer-substrate.md).
