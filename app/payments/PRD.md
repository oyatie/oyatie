---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-payments
microservice: payments
status: Proposed
sales_segment: shared-substrate-and-product
tier: hero-substrate
tier_subtype: substrate-payments
tier_classification_rationale: |
  The payments µservice is the shared monetisation substrate that powers every
  oyatie revenue surface — B2C consumer purchases (sticker packs, creator
  subscriptions, premium handles, in-app one-shots), B2B subscription billing
  (per-seat SaaS, usage-metered substrate plans), marketplace facilitator
  patterns for the plugin-app-store / shorts / community-tipping surfaces,
  escrow + payout for connect-bridge flows, and regulated multi-PSP routing
  (Stripe + Adyen + Toss + KakaoPay + LINE Pay + WeChat Pay + Alipay). It is
  hero-substrate (not hero-product) because the surfaces that touch money sit
  in their respective product µservices (messenger sticker store, shorts
  creator-tipping, plugin-app-store checkout, cloud-billing usage invoicing);
  this µservice is the load-bearing substrate that all of them depend on.
keystone-bundle: 2026-05-20-foundational-doctrine
milestone_first_ship: M02-foundation
related_adrs:
  - ADR-0008
  - ADR-0028
  - ADR-0049
  - ADR-0056
  - ADR-0105
  - ADR-0117
  - ADR-0131
  - ADR-0140
  - ADR-0145
  - ADR-0148
  - ADR-0179
  - ADR-0182
  - ADR-0183
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0249
  - ADR-0251
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs:
  - /specs/microservices/payments.json
  - /specs/tenant-model.json
  - /specs/compliance-pack-matrix.json
  - /specs/per-microservice-flat-layout.json
related_memories:
  - tenant-as-universal-scoping-primitive
  - cedar-as-universal-gate
  - byok-everywhere-credentials
  - build-ahead-of-certification
  - compliance-pack-primitive
  - multi-category-marketplace-doctrine
  - quality-performance-scalability-bar
date: 2026-05-20
owner_team: axis-payments + council-finance + ops-fraud + ops-treasury
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
  - oyatie-internal-tenant
benchmarks:
  - stripe
  - adyen
  - braintree
  - checkout-com
  - toss-payments
  - kakaopay
  - line-pay
  - wechat-pay
  - alipay
  - square
  - paddle
  - chargebee
  - recurly
  - paypal-business
---

# PRD-payments: Multi-PSP, Multi-Currency, Marketplace-Facilitator Payment Substrate

> Hero substrate. Powers every oyatie monetisation surface — B2C personal (sticker packs, creator subs, premium handles), B2B (per-seat subscriptions, usage-metered invoices), and marketplace-facilitator flows (plugin-app-store, shorts creator tipping, community super-chats). Per ADR-0244 every payment row carries tenant context; per ADR-0255 §D-4 provider-BYOK extends to tenant-owned provider accounts (tenant's own Stripe / Adyen account). Per ADR-0251 compliance packs (PCI-DSS L1 v4, KR-FSS, EU SCA + PSD2, COPPA refusal, KOSA tier) attach per tenant + cell.

---

## A. Problem

### A.1 Why payments needs its own µservice

Every product surface that monetises must touch payments, but no product owns payments end-to-end. A naïve approach lets each product (messenger sticker store, shorts tipping, plugin-app-store checkout, cloud-billing invoicing) call Stripe directly. That violates:

- **ADR-0145 inter-microservice communication** — direct gRPC + 3 invariants, no shared-secret coupling to PSP SDKs.
- **ADR-0244 tenant-as-universal-scoping** — per-product credential sprawl produces N PSP accounts per tenant.
- **ADR-0251 compliance-pack primitive** — per-product PCI scope explodes the audit surface from 1 µservice to N.
- **ADR-0255 §D-4 provider-BYOK SecretReference model** — without a substrate, tenant-owned Stripe accounts cannot be uniformly enforced.
- **`feedback_quality_performance_scalability_bar`** — Stripe parity (P99 ≤ 200ms charge, 99.999% availability) requires concentrated engineering.

The payments µservice consolidates: PSP routing, tenant-scoped vaulting, idempotency, retries, dispute handling, refunds, payouts, tax calculation, subscription state-machine, escrow, marketplace-facilitator settlement, audit-chain emission. Every monetising product surface calls this µservice; no product surface calls a PSP directly. PSP-direct calls are refused by the LEAN-A2 lane.

### A.2 What competitors get wrong

Hyperscaler precedent: **Stripe Connect** (platform-facilitator pattern) + **Adyen MarketPay** (marketplace settlement) + **Toss Payments Sub-Merchant** (KR regulator-mandated platform pattern) — every successful multi-tenant platform converges on the same shape. Failure modes observed:

1. **Slack 2019** — sticker-pack tipping launched on raw Stripe Charges; per-creator payouts required a manual sub-merchant onboarding flow that took weeks. Fix: marketplace-facilitator (Stripe Express) from day-one.
2. **Discord Nitro 2020-2022** — single PSP (Stripe) created KR + JP + CN payment-friction gaps; users dropped at checkout because KakaoPay / LINE Pay / Alipay were missing. Fix: multi-PSP routing with locale-aware preference.
3. **Substack 2023** — per-creator tax-1099 generation was bolted onto the subscription engine; year-end tax season caused 5× incident load. Fix: tax-registration is a first-class entity, not an afterthought.
4. **Patreon 2024** — per-tenant_class subscription churn was opaque because subscription state lived in Stripe rather than the platform's own ledger. Fix: own the subscription state machine; PSP is a transport.

The oyatie payments µservice avoids all four by design (see §D Functional Requirements, §E.4 Subscription State Machine, §E.6 Tax).

### A.3 What "good" looks like

An oyatie tenant whose product touches money — a creator selling a sticker pack, a SaaS company billing $99/seat/month, a healthcare clinic charging insurance copay, a marketplace platform settling to vendors — calls one of three substrate APIs:

- `POST /v1/charges` (one-shot)
- `POST /v1/subscriptions` (recurring)
- `POST /v1/transfers` (marketplace payout)

…and receives a typed receipt + an audit-chain seal + a webhook-stream subscription. The substrate handles: PSP selection by `(tenant_pack, currency, payment_method)`, idempotency by `Idempotency-Key`, retry on transient PSP failure, SCA challenge if EU + threshold, KR-FSS event emission if tenant_pack=kr, tax computation per jurisdiction, ledger journal entry, audit seal. The tenant never touches PSP credentials, PCI cardholder data, or webhook signature verification.

---

## B. Target Users (Personas)

### B.1 B2C personas

#### Persona B2C-1 — "Ji-min, Korean university student buying KakaoTalk-class stickers"
- **Goals**: buy a 2,000 KRW sticker pack for her oyatie messenger; pay via KakaoPay (her default); receive sticker pack instantly; no card details required.
- **Frustrations**: foreign apps that don't accept KakaoPay; double-charge on retry; opaque foreign-currency conversion; KR-FSS-mandated 1-tap consent skip on every purchase.
- **Tech comfort**: high (iOS native messenger, multi-app pay stack: KakaoPay + Toss + Naver Pay).
- **Locale + device**: ko-KR, KST, iPhone 15 + Apple Watch + Galaxy Tab; mobile-first; in-app webview checkout.

#### Persona B2C-2 — "Marcus, German indie creator monetising shorts"
- **Goals**: receive €0.50 super-chats from viewers during his live stream; receive monthly payout to his Sparkasse SEPA IBAN; pay German income tax via auto-1099-equivalent.
- **Frustrations**: 7-day payout holds; PayPal cross-border 4% margin; opaque Stripe Express onboarding; no auto-tax-report.
- **Tech comfort**: high (creator-economy native; Stripe Connect, Patreon, Twitch Bits).
- **Locale + device**: de-DE, CEST, MacBook Pro + iPhone; desktop creator-dashboard primary.

#### Persona B2C-3 — "Sofia, Brazilian retiree subscribing to a hobbyist creator"
- **Goals**: subscribe at R$ 19/mo via PIX or Boleto Bancário; cancel any time; never see foreign-currency fees.
- **Frustrations**: credit-card-only platforms (low CC penetration in BR); failed PIX retries; opaque cancellation flow; subscription auto-renews despite cancel.
- **Tech comfort**: medium (mobile-first; bank-app native; reluctant to enter card data).
- **Locale + device**: pt-BR, BRT, Android mid-range + occasional desktop; native browser + PIX QR-scan.

#### Persona B2C-4 — "Alex (13) and parent guardian, COPPA-refusal case"
- **Goals**: Alex wants to buy a sticker pack; the system MUST refuse because <13; parent receives a notification; no payment flow proceeds.
- **Frustrations**: platforms that silently age-gate without parental notification; platforms that succeed with card-on-file then need refund.
- **Tech comfort**: Alex high (digital-native), parent medium.
- **Locale + device**: en-US, mobile + console.

### B.2 B2B personas

#### Persona B2B-1 — "Marketplace Mai, B2B platform operator on oyatie"
- **Goals**: run a marketplace where vendors sell to buyers; oyatie holds funds (escrow) then pays vendors weekly; oyatie keeps 10% facilitator fee; vendors are sub-merchants on Mai's Stripe account.
- **Frustrations**: PSP onboarding friction; per-vendor KYC; chargeback liability ambiguity; settlement currency conversion losses.
- **Tech comfort**: high (operator dashboard; Stripe veteran; SaaS founder).
- **Locale + device**: en-US + zh-Hans (vendors in CN), PT/SGT, desktop primary; admin console.

#### Persona B2B-2 — "Finance Felix, B2B SaaS controller billing per-seat"
- **Goals**: bill $99/seat/month for 500 seats; invoice via Stripe Invoicing OR auto-charge stored card; support proration on mid-cycle seat-add; export to NetSuite via journal-export API; close-the-books month-end.
- **Frustrations**: pricing changes mid-cycle; proration miscalculation; refund vs credit-memo confusion; tax-jurisdiction mismatch.
- **Tech comfort**: medium-high (NetSuite + QuickBooks; familiar with Stripe Invoicing + Chargebee).
- **Locale + device**: en-US, ET, desktop primary; spreadsheet-heavy workflow.

#### Persona B2B-3 — "Clinic Coordinator Caroline, healthcare tenant on `pack-us-healthcare`"
- **Goals**: collect $35 copay from patient at appointment time; charge insurer via 837P claim (out-of-scope; payments substrate only handles the patient-portion); HIPAA-safe receipt; no PHI in PSP payload.
- **Frustrations**: PSP webhook payloads that leak PHI; PCI + HIPAA dual-audit overhead; copay charged before insurance adjudicates (over-collection).
- **Tech comfort**: medium (EHR + Stripe Terminal).
- **Locale + device**: en-US, CT, tablet at front-desk + desktop billing-portal.

### B.3 Internal persona

#### Persona INT-1 — "Treasurer Tom, oyatie ops-treasury team"
- **Goals**: reconcile platform-fee receipts daily; project monthly settlement obligations; trigger payouts to oyatie's corporate banks; respond to KR-FSS regulator audit pulls.
- **Frustrations**: PSP webhook delays; per-pack settlement currency mismatch; manual journal entries.
- **Tech comfort**: very high (internal ops dashboard; SQL-fluent).

---

## C. User Stories

Stories are oyatie-payments specific. They do NOT duplicate `docs/user-stories/b2c-consumer-surfaces.md` or `b2b-work-surfaces.md`; they reference them by ID and add NEW payment-specific stories.

### US-payments-01 — Korean sticker-pack one-shot via KakaoPay
- **As** Ji-min (B2C-1)
- **I want** to tap "Buy" on a 2,000 KRW sticker pack and pay via KakaoPay
- **so that** I receive the sticker pack within 3 seconds without entering card details.
- **Acceptance criteria**:
  1. Tapping "Buy" presents a KakaoPay QR + push to her phone within 1 second.
  2. After KakaoPay confirms (Cedar policy `payments::pack_kr::psp_route` permits), receipt appears within 3 seconds.
  3. Receipt MUST include: amount (2,000 KRW), PSP=KakaoPay, idempotency-key, tax-line (KR VAT inclusive), audit-chain seal id.
  4. Charge MUST be idempotent on retry (duplicate tap = single charge).
- **Accessibility AC**: receipt screen passes WCAG 2.2 AA; screen-reader announces amount + receipt-id; high-contrast mode available.
- **i18n AC**: amount localized as `2,000원`; receipt PDF in ko-KR; KR-FSS-mandated consent string in ko-KR.

### US-payments-02 — Indie creator monthly subscription
- **As** Marcus (B2C-2)
- **I want** viewers to subscribe at €5/mo via SEPA Direct Debit or card
- **so that** I receive a predictable monthly payout net of platform fees.
- **Acceptance criteria**:
  1. Subscriber enters IBAN OR card; SCA challenge triggered if amount × annual ≥ €30 (PSD2 RTS threshold).
  2. Subscription state machine moves `pending → active`; first charge succeeds within 5s; subsequent monthly charges retry on failure per `payments::retry_ladder::standard`.
  3. Payout to Marcus's Stripe Express account triggered on `payout_schedule=weekly`; first payout T+7 days.
  4. Cancellation by subscriber moves state to `cancelled_at_period_end`; no further charges.
- **Accessibility AC**: subscription flow keyboard-navigable end-to-end; SCA challenge UI passes WCAG 2.2 AA.
- **i18n AC**: amounts in EUR with German decimal comma (`5,00 €`); SCA challenge text in de-DE; IBAN field accepts SEPA format.

### US-payments-03 — Brazilian retiree PIX subscription
- **As** Sofia (B2C-3)
- **I want** to subscribe at R$ 19/mo via PIX with auto-renew
- **so that** I never enter card data and always pay via my bank's PIX UI.
- **Acceptance criteria**:
  1. PIX QR + key (`copy-paste`) presented within 1s; payment confirmed within 30s of bank-app confirmation.
  2. Auto-renew on 30-day cycle uses stored `pix_dict_alias`; subscriber confirms first renewal explicitly per BCB regulation.
  3. Boleto Bancário fallback offered if PIX fails 3 consecutive times.
  4. Cancellation removes stored alias; subscriber receives confirmation email + SMS within 5 minutes.
- **Accessibility AC**: PIX QR also offered as accessible copy-paste string; QR scanning timeout extends for low-vision mode.
- **i18n AC**: amounts in BRL (`R$ 19,00`); receipt PDF in pt-BR; BCB-mandated disclosures included.

### US-payments-04 — COPPA <13 refusal
- **As** Alex (B2C-4)
- **I want** the system to refuse my purchase attempt
- **so that** no charge proceeds and my parent is notified.
- **Acceptance criteria**:
  1. Cedar policy `payments::age_assurance::coppa_refuse` evaluates `subject.age_class == "under_13"` → forbid.
  2. UI displays a refusal screen with parent-guardian contact prompt; no PSP call is made.
  3. Audit-chain emits `PaymentRefused{reason="coppa_age", subject_hashed_id, ...}`.
  4. Parent notification dispatched via the messenger µservice within 60 seconds; SMS fallback if no messenger handle.
- **Accessibility AC**: refusal screen has clear non-blame copy; parent-guidance link is descriptive.
- **i18n AC**: refusal copy localized per locale; parent notification respects parent's locale.

### US-payments-05 — KOSA tier (14-17)
- **As** a 15-year-old user attempting to subscribe
- **I want** the system to enforce KOSA-tier limits (≤$10/mo spend cap; parental notification on subscribe)
- **so that** spend is bounded and the parent is informed.
- **Acceptance criteria**:
  1. Cedar `payments::age_assurance::kosa_minor_tier` returns context `{spend_cap_monthly_usd: 10, parental_notification: required}`.
  2. Charge amount ≤ cap proceeds with parental notification webhook.
  3. Charge amount > cap is refused with explanatory screen.
  4. Monthly aggregate spend tracked per subject; cap enforced as a rolling 30-day window.
- **Accessibility AC**: clear cap-disclosure screen pre-purchase; opt-out path visible.
- **i18n AC**: cap currency conversion explained per locale.

### US-payments-06 — Marketplace facilitator settlement (Marketplace Mai)
- **As** Marketplace Mai (B2B-1)
- **I want** buyers to pay $100 for a vendor's good; oyatie holds the funds; weekly payout sends $90 to vendor and $10 to my marketplace
- **so that** I never touch buyer card data and vendor 1099 reports are automatic.
- **Acceptance criteria**:
  1. `POST /v1/charges` with `application_fee=10_00` (10% in cents) + `transfer_data.destination=<vendor_connect_account>` succeeds.
  2. Charge state machine: `pending → succeeded → captured → in_escrow → ready_for_payout`.
  3. Weekly payout cron transfers $90 to vendor's Stripe Express balance; $10 to Mai's platform balance.
  4. Year-end 1099-K generated per vendor jurisdiction (US: IRS 1099-K threshold; KR: NTS annual filing; etc.).
- **Accessibility AC**: vendor onboarding flow passes WCAG 2.2 AA; 1099-K PDF accessibility-compliant.
- **i18n AC**: vendor sees amounts in their settlement currency; receipts in vendor locale.

### US-payments-07 — Per-seat SaaS billing with proration
- **As** Finance Felix (B2B-2)
- **I want** to add 50 seats mid-cycle on day 15 of 30
- **so that** I am charged ($99 × 50 × 15/30) = $2,475 prorated on the next invoice.
- **Acceptance criteria**:
  1. `POST /v1/subscription_items` with `quantity_delta=+50` returns a typed proration receipt.
  2. Next invoice contains a `proration_credit_unused_time` line + `proration_charge_remaining_time` line.
  3. Stripe Tax (or equivalent) computes US sales tax per shipping address; line item visible.
  4. Webhook `invoice.upcoming` fires 7 days before invoice; webhook `invoice.paid` fires after settlement.
- **Accessibility AC**: invoice PDF accessibility-compliant (PDF/UA); table headers labeled.
- **i18n AC**: amounts in USD with `$2,475.00`; proration explanation copy in en-US.

### US-payments-08 — Clinic copay with HIPAA-safe receipt
- **As** Caroline (B2B-3)
- **I want** the patient's $35 copay to charge without PHI in the PSP payload
- **so that** PCI scope and HIPAA scope are not co-mingled.
- **Acceptance criteria**:
  1. Charge metadata MUST NOT contain PHI fields; only `tenant_id`, `appointment_id` (opaque), `payment_purpose_class`.
  2. Receipt to patient via messenger µservice contains appointment confirmation but no diagnostic codes.
  3. BAA in place between oyatie ↔ PSP for the `pack-us-healthcare` pack.
  4. Audit-chain emits a HIPAA `Disclosure` record per 45 CFR §164.528 even though disclosure is operational.
- **Accessibility AC**: receipt accessible to patient on web + mobile + email.
- **i18n AC**: en-US clinic only at M02; ES-US extended in M04.

### US-payments-09 — DSAR right-to-erasure with retention conflict
- **As** a EU consumer requesting deletion of all personal data
- **I want** my email, name, address removed
- **and** my payment-card token removed
- **so that** GDPR Art. 17 is honored — except where Member State law mandates 7-year financial-audit retention
- **Acceptance criteria**:
  1. DSAR cascade reaches payments µservice with `subject_pseudonym_id`.
  2. Email + name + address fields tombstoned within 30 days.
  3. Payment-card token revoked at PSP; PSP-side card detail purged.
  4. Charge ledger entries retained 7 years per local commercial code; `subject_pseudonym_id` ↔ subject mapping broken via key rotation per ADR-0255 §D-9.
  5. DSAR-completion report cites which fields were redacted and which were retained with legal basis.
- **Accessibility AC**: DSAR report PDF accessible (PDF/UA).
- **i18n AC**: report in subject's locale.

### US-payments-10 — Refund flow with partial amount
- **As** a customer support agent
- **I want** to refund $7.50 of a $25.00 charge
- **so that** the partial refund is captured + accounted in the ledger.
- **Acceptance criteria**:
  1. `POST /v1/refunds` with `amount=7_50` + reason code succeeds.
  2. Charge state moves to `partial_refunded`; remaining net = $17.50.
  3. Stripe Tax (or equivalent) reverses tax-portion proportionally.
  4. Webhook `refund.created` fires; subscriber receives notification + updated receipt PDF.
- **Accessibility AC**: refund UI keyboard-navigable; refunded-amount visible in receipt.
- **i18n AC**: amounts in original-charge currency; locale follows tenant default.

### US-payments-11 — Dispute resolution (chargeback)
- **As** a creator (B2C-2) whose customer disputed a $30 charge
- **I want** to upload evidence (delivery confirmation, communications log)
- **so that** the dispute is contested via the PSP within 7 calendar days.
- **Acceptance criteria**:
  1. `POST /v1/disputes/<id>/evidence` accepts JSON + file attachments (receipts, emails, screenshots).
  2. PSP-specific evidence shape generated (Stripe Disputes shape, Adyen NotificationItem, etc.).
  3. Dispute state machine: `needs_response → under_review → won|lost`.
  4. On lose, charge state moves to `lost_dispute`; ledger reversal posted; webhook fires.
- **Accessibility AC**: evidence upload UI supports drag-drop + click-upload + paste; alt-text required for image evidence.
- **i18n AC**: evidence narrative supports the dispute-resolution PSP locale.

### US-payments-12 — KR-FSS audit pull
- **As** a KR-FSS regulator
- **I want** to request all `pack-kr` tenant transactions for tenant `acme-kr` between 2026-01-01 and 2026-03-31
- **so that** I can review for anti-fraud anomalies.
- **Acceptance criteria**:
  1. Internal `/internal/audit/regulator-pull` endpoint authorised via Cedar `payments::regulator::kr_fss_pull` (4-eye approval).
  2. Output is a sealed, Ed25519-signed JSON bundle with all `payments::Charge`, `payments::Refund`, `payments::Dispute`, `payments::Payout` rows.
  3. Output includes a Merkle-root proof of completeness vs the audit-chain.
  4. SLA: bundle ready within 4 business hours; ≤1 day for ≥10M-row pulls.
- **Accessibility AC**: bundle includes a human-readable summary CSV (UTF-8 BOM for Excel compatibility).
- **i18n AC**: KR-FSS-mandated columns labeled in ko-KR with English back-fill.

### US-payments-13 — Tenant brings own Stripe (provider-BYOK)
- **As** Marketplace Mai (B2B-1)
- **I want** to connect my own Stripe account (not oyatie's)
- **so that** payouts hit MY bank balance and oyatie sees only platform-fee transfers.
- **Acceptance criteria**:
  1. `POST /v1/tenant/psp-account/connect` initiates Stripe OAuth Standard flow.
  2. After OAuth, tenant's `payments::Tenant.psp_account_ref` references a SecretReference per ADR-0255 §D-4 (oyatie holds no Stripe API key in plaintext).
  3. All charges + payouts route through Mai's account; oyatie collects platform-fee transfers via Stripe `application_fee`.
  4. Disconnect flow: 30-day grace; old charges still settle; new charges refused after disconnect.
- **Accessibility AC**: OAuth flow accessible; consent screen WCAG 2.2 AA.
- **i18n AC**: Stripe OAuth UI inherits Mai's locale.

### US-payments-14 — Multi-PSP locale-aware routing
- **As** a Japanese consumer
- **I want** the system to default to LINE Pay (my native wallet)
- **but** offer Stripe (card) as fallback
- **so that** my payment friction is minimized.
- **Acceptance criteria**:
  1. Routing table `payments::psp_route::ja_jp` returns `[LINE_PAY, STRIPE]` ordered.
  2. UI presents LINE Pay first; Stripe second; both labeled clearly.
  3. If LINE Pay fails, user can retry with Stripe in one tap.
  4. Routing decisions are audit-chain-logged with `route_score` for analytics.
- **Accessibility AC**: payment-method picker is screen-reader friendly with clear labels.
- **i18n AC**: ja-JP locale; LINE Pay UI in Japanese.

### US-payments-15 — Receipt PDF generation
- **As** Finance Felix (B2B-2)
- **I want** every charge to produce a tax-compliant PDF receipt
- **so that** I can attach it to my expense report.
- **Acceptance criteria**:
  1. `GET /v1/charges/<id>/receipt.pdf` returns a PDF/A-3 compliant receipt within 2 seconds.
  2. PDF contains: tenant logo, charge details, tax breakdown, oyatie ID, PSP transaction ID, audit-chain reference.
  3. PDF passes PDF/UA accessibility check.
  4. Receipt URL signed; expires 90 days; re-issuable on request.
- **Accessibility AC**: PDF/UA conformant.
- **i18n AC**: receipt rendered in tenant locale; amounts in transaction currency.

### US-payments-16 — Subscription cancellation grace
- **As** Sofia (B2C-3)
- **I want** to cancel my subscription on day 25 of a 30-day cycle
- **so that** I retain access for the remaining 5 days but am not re-charged.
- **Acceptance criteria**:
  1. `DELETE /v1/subscriptions/<id>?cancel_at=period_end` succeeds.
  2. State machine: `active → cancelled_at_period_end → cancelled`.
  3. Cancellation confirmation email + push within 60 seconds.
  4. Re-subscribe flow available within the grace period without state corruption.
- **Accessibility AC**: cancellation flow non-coercive; one-click cancel; clear receipt of cancellation.
- **i18n AC**: pt-BR copy; BRT time zone in confirmation.

### US-payments-17 — Marketplace facilitator KYB verification
- **As** Marketplace Mai (B2B-1) onboarding a new vendor
- **I want** the vendor to complete KYB (Know Your Business) before receiving payouts
- **so that** I am compliant with sub-merchant rules.
- **Acceptance criteria**:
  1. New vendor enters business name, tax ID, address, beneficial owners.
  2. KYB submitted to Stripe / equivalent; status `pending_verification → approved` or `rejected_with_reasons`.
  3. Until approved, charges still route but payouts are held; on approval, hold released.
  4. Periodic re-KYB on PSP request (typically annually).
- **Accessibility AC**: KYB form accessibility-compliant; field labels explicit.
- **i18n AC**: KYB form localized per vendor jurisdiction.

### US-payments-18 — Marketplace dispute liability split
- **As** Marketplace Mai (B2B-1)
- **I want** chargebacks to debit the vendor's balance first (not my marketplace)
- **so that** my marketplace is not the chargeback liability holder.
- **Acceptance criteria**:
  1. Charge `transfer_data.destination=<vendor>` + `on_behalf_of=<vendor>` ensures dispute liability sits with vendor.
  2. Marketplace fee retained or reversed per `dispute_reversal_policy=keep|reverse_proportionally`.
  3. Vendor sees dispute in their dashboard; can upload evidence.
  4. Marketplace dashboard shows aggregate dispute metrics across vendors.
- **Accessibility AC**: dispute dashboard table-headers labeled.
- **i18n AC**: vendor locale for evidence UI.

### US-payments-19 — Tax registration onboarding
- **As** Finance Felix (B2B-2)
- **I want** to register my entity for US sales tax (Stripe Tax) + EU VAT + KR VAT
- **so that** every charge has correct tax computed at source.
- **Acceptance criteria**:
  1. Per-jurisdiction tax-registration entry with active dates, certificate of registration, nexus type.
  2. Tax engine queries registration → applies correct rate → emits tax-line on charge.
  3. Failure to register where required raises a warning (advisory) and an alert (post-threshold).
  4. Tax-filing CSV/EDI export available per jurisdiction.
- **Accessibility AC**: registration form fields well-labeled; date pickers accessible.
- **i18n AC**: forms localized per jurisdiction; legal-text in jurisdictional language.

### US-payments-20 — Payout schedule customization
- **As** Marcus (B2C-2)
- **I want** to switch from weekly to daily payouts
- **so that** I have faster cash flow.
- **Acceptance criteria**:
  1. `PATCH /v1/tenant/psp-account/payout-schedule` accepts `daily|weekly|monthly` with PSP-specific minimum thresholds.
  2. Change reflected within 1 PSP-business-day.
  3. Daily payouts incur PSP fee (informational; surfaced in UI).
  4. Audit-chain emits `PayoutScheduleChanged` event.
- **Accessibility AC**: payout-schedule UI accessibility-compliant.
- **i18n AC**: schedule terms localized.

### US-payments-21 — Idempotency on retry
- **As** a mobile client
- **I want** my tap-to-buy to be safe to retry on network failure
- **so that** I am never double-charged.
- **Acceptance criteria**:
  1. Client generates `Idempotency-Key: <uuid-v7>` per logical attempt.
  2. Server stores `(tenant_id, idempotency_key)` → response for 24h.
  3. Replay with same key returns identical response; no second PSP call.
  4. Different key + same amount + same metadata is treated as new charge.
- **Accessibility AC**: N/A (server semantics).
- **i18n AC**: error messages localized when retry semantics fail.

### US-payments-22 — Refund-on-failed-fulfillment auto
- **As** a creator (B2C-2)
- **I want** to mark a goods-delivery as failed
- **so that** the system auto-refunds the buyer.
- **Acceptance criteria**:
  1. `POST /v1/orders/<id>/mark-failed-fulfillment` triggers refund + state move.
  2. Audit-chain emits `FulfillmentFailed` + `RefundIssuedAutomatic`.
  3. Buyer notified within 60 seconds with apology + refund timeline.
- **Accessibility AC**: notification accessible.
- **i18n AC**: per locale.

### US-payments-23 — Recurring revenue analytics
- **As** Finance Felix (B2B-2)
- **I want** MRR / ARR / churn / cohort metrics for my tenant
- **so that** I can report to my board.
- **Acceptance criteria**:
  1. `GET /v1/analytics/mrr?tenant_id=<...>&from=2026-01&to=2026-05` returns time-series JSON.
  2. ARR = MRR × 12; gross + net (after refunds + churn).
  3. Cohort retention curve per signup-month.
  4. CSV + JSON + Excel export.
- **Accessibility AC**: dashboard accessibility AA; data tables labeled.
- **i18n AC**: amounts in tenant default currency.

### US-payments-24 — Failed-card retry ladder
- **As** Sofia (B2C-3)
- **I want** the system to retry my card on day 1, 3, 7 if a renewal fails
- **so that** transient failures don't immediately cancel me.
- **Acceptance criteria**:
  1. Retry ladder configurable per tenant: `[+0d, +3d, +7d]` default.
  2. On 3rd failure, subscription moves to `past_due → cancelled_for_nonpayment`.
  3. Notifications at each retry attempt.
  4. Card-update self-service link in notifications.
- **Accessibility AC**: notifications accessible.
- **i18n AC**: localized.

### US-payments-25 — In-flight currency conversion
- **As** Marcus (B2C-2)
- **I want** to receive payouts in EUR even though some viewers tip in USD
- **so that** I bank in my native currency.
- **Acceptance criteria**:
  1. Charge in USD; payout settlement in EUR per PSP FX rate at settlement time.
  2. FX rate + spread surfaced in payout statement.
  3. Settlement currency configurable per account.
- **Accessibility AC**: FX disclosure accessible.
- **i18n AC**: localized.

### US-payments-26 — Stripe → Adyen failover
- **As** ops-treasury (INT-1)
- **I want** charges to fail over to Adyen when Stripe US has an incident
- **so that** revenue continues during a single-PSP outage.
- **Acceptance criteria**:
  1. Health check + circuit breaker on each PSP; trip threshold = 3 consecutive 5xx in 60s.
  2. Routing decision uses `(tenant_pack, currency, payment_method, psp_health)`.
  3. Audit-chain emits `PspRouteFailover` event for each switch.
  4. Manual override via ops-dashboard for emergencies.
- **Accessibility AC**: ops dashboard accessibility AA.
- **i18n AC**: ops dashboard en-US (internal).

### US-payments-27 — Reconciliation report
- **As** Treasurer Tom (INT-1)
- **I want** a daily reconciliation report
- **so that** I can confirm PSP receipts match our ledger.
- **Acceptance criteria**:
  1. T+1 reconciliation: PSP settlement file ingested at 04:00 local; matched against ledger; mismatches flagged.
  2. Report contains `total_charged`, `total_refunded`, `total_disputed`, `psp_fees`, `platform_fees`, `net_to_us`.
  3. Mismatch >$1 flags an INC ticket auto-created.
- **Accessibility AC**: report PDF + CSV accessible.
- **i18n AC**: internal en-US.

### US-payments-28 — Subscription pause
- **As** a B2C subscriber going on vacation
- **I want** to pause my subscription for 30 days
- **so that** I am not charged but my account state persists.
- **Acceptance criteria**:
  1. `POST /v1/subscriptions/<id>/pause?resume_at=<ts>` succeeds.
  2. State machine: `active → paused → active (auto-resume)`.
  3. No charges during pause; access state per product policy.
  4. Resume reminder 24h before auto-resume.
- **Accessibility AC**: pause UI clear and non-misleading.
- **i18n AC**: localized.

### US-payments-29 — Bulk import of historical subscriptions
- **As** Finance Felix (B2B-2) migrating from Recurly to oyatie
- **I want** to import 500 active subscriptions with their renewal dates
- **so that** my customers are not double-billed.
- **Acceptance criteria**:
  1. `POST /v1/migrations/subscriptions:bulk-import` accepts CSV/JSON with `external_id`, `renewal_date`, `amount`, `currency`, `customer_id`.
  2. Dry-run mode shows the planned charges + dates without committing.
  3. Migration audit trail; rollback capability for 30 days.
- **Accessibility AC**: progress UI accessible.
- **i18n AC**: en-US.

### US-payments-30 — provider-BYOK rotation
- **As** ops-treasury
- **I want** to rotate tenant's Stripe restricted key on a quarterly schedule
- **so that** key compromise blast-radius is bounded.
- **Acceptance criteria**:
  1. SecretReference per ADR-0255 §D-4 surfaces the rotation policy.
  2. Rotation flow swaps key without service interruption (overlap window).
  3. Old key revoked at PSP within 24h of cutover.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-payments-31 — Webhook delivery + replay
- **As** a tenant integrator
- **I want** to subscribe to charge.* webhooks at my own URL
- **so that** my system reacts to events in real-time.
- **Acceptance criteria**:
  1. `POST /v1/webhook_endpoints` with URL + event types + HMAC secret returns a typed endpoint.
  2. Webhooks delivered with exponential backoff (`+1s, +5s, +30s, +2m, +10m, +1h, +6h, +24h`).
  3. Replay last 30 days from dashboard.
  4. Signature verification via HMAC-SHA256 with replay-attack window 5min.
- **Accessibility AC**: dashboard accessible.
- **i18n AC**: error messages localized.

### US-payments-32 — Subscription upgrade with proration credit
- **As** Finance Felix (B2B-2)
- **I want** to upgrade from $99/seat to $199/seat on day 10 of 30
- **so that** I get proration credit on the next invoice.
- **Acceptance criteria**:
  1. `POST /v1/subscriptions/<id>:upgrade` with new plan returns proration preview.
  2. Next invoice contains `proration_credit` + `proration_charge` lines.
  3. Tax recomputed on the new plan rate.
- **Accessibility AC**: preview UI accessibility.
- **i18n AC**: amounts localized.

### US-payments-33 — Refund window enforcement
- **As** Marketplace Mai (B2B-1)
- **I want** refunds to be allowed only within 30 days of charge
- **so that** my vendors' liability window is bounded.
- **Acceptance criteria**:
  1. Tenant policy `refund_window_days=30` enforced at refund time.
  2. Refunds beyond window require ops-supervisor override.
  3. Override is audit-chain logged with reason.
- **Accessibility AC**: refund UI clear about window.
- **i18n AC**: localized.

### US-payments-34 — Anti-fraud screening (Stripe Radar parity)
- **As** ops-fraud
- **I want** every charge >$500 to be screened by a risk engine
- **so that** high-risk charges are blocked or sent to manual review.
- **Acceptance criteria**:
  1. Risk engine: rule-based + ML; emits `risk_score 0-100`.
  2. Score ≥ 80 → block; score 60-79 → review; score <60 → allow.
  3. Manual review queue with reviewer UI.
  4. Review SLA: 1 business day; auto-decline after.
- **Accessibility AC**: review UI accessibility AA.
- **i18n AC**: review queue en-US (ops).

### US-payments-35 — 3DS step-up authentication
- **As** a EU consumer paying €120
- **I want** my bank to issue a 3DS challenge
- **so that** SCA (Strong Customer Authentication) is satisfied per PSD2.
- **Acceptance criteria**:
  1. 3DS challenge invoked when EU + amount > €30 (or risk-flagged).
  2. Challenge UI in iframe; fallback to redirect.
  3. On success: charge proceeds; on fail: charge declined.
- **Accessibility AC**: challenge iframe accessibility.
- **i18n AC**: bank UI in user locale.

### US-payments-36 — Sandbox + production environment isolation
- **As** a developer integrating payments
- **I want** sandbox mode with test cards
- **so that** I can develop without real money.
- **Acceptance criteria**:
  1. `X-Environment: sandbox|production` header distinguishes.
  2. Test cards per PSP (e.g., Stripe 4242…) available.
  3. Sandbox cannot reach production webhooks.
  4. Sandbox data purged after 90 days.
- **Accessibility AC**: docs accessibility.
- **i18n AC**: en-US (developer).

### US-payments-37 — Saved payment methods (vault)
- **As** Sofia (B2C-3)
- **I want** to save my PIX alias for future renewals
- **so that** I don't enter it monthly.
- **Acceptance criteria**:
  1. `POST /v1/payment_methods` with PSP token + metadata stores reference.
  2. Method visible in account UI; deletable.
  3. Default method per tenant + per subscription override.
- **Accessibility AC**: vault UI accessibility AA.
- **i18n AC**: localized.

### US-payments-38 — Per-tenant rate limit
- **As** ops-fraud
- **I want** to cap charge attempts at 100/min per tenant
- **so that** card-testing attacks are bounded.
- **Acceptance criteria**:
  1. Rate-limit per tenant via token-bucket.
  2. Over-limit returns 429 with `Retry-After`.
  3. Per-tenant override for legitimate high-volume tenants.
- **Accessibility AC**: N/A.
- **i18n AC**: error messages localized.

### US-payments-39 — Hosted-checkout page
- **As** a small-business B2B tenant
- **I want** to host my checkout page on `pay.acme.com` (a Stripe-Checkout-style hosted page)
- **so that** I don't need to build a payment UI.
- **Acceptance criteria**:
  1. `POST /v1/checkout/sessions` returns a hosted URL.
  2. Page is responsive, PCI-SAQ-A scope (no card data hits tenant's server).
  3. Branding configurable: logo, color, typography.
- **Accessibility AC**: hosted page WCAG 2.2 AA.
- **i18n AC**: per-locale.

### US-payments-40 — Cross-references to story banks
- See `docs/user-stories/b2c-consumer-surfaces.md#US-B2C-PAY-*` for non-payment-specific consumer payment stories.
- See `docs/user-stories/b2b-work-surfaces.md#US-B2B-FIN-*` for B2B finance flows.
- Stories US-payments-01..39 are payments-µservice-specific. The story bank stories cover product-surface integration (e.g., "buy a sticker pack" UX in messenger, "tip a creator" UX in shorts) and reference back to this PRD for the payment substrate.

### US-payments-41 — Subscription gifting
- **As** a B2C user
- **I want** to gift a 3-month subscription to a friend
- **so that** they receive an activation code without their billing info.
- **Acceptance criteria**:
  1. Gifter charged upfront; recipient receives an opaque activation code.
  2. On redemption, subscription activates without recipient's card.
  3. Auto-renew off by default for gifted subscriptions.
- **Accessibility AC**: gift flow accessible.
- **i18n AC**: localized.

### US-payments-42 — Multi-currency price book
- **As** Marketplace Mai (B2B-1)
- **I want** a per-region price book (USD $99, EUR €89, GBP £79, KRW ₩129,000, JPY ¥13,000)
- **so that** buyers see local prices without conversion friction.
- **Acceptance criteria**:
  1. `POST /v1/price_books` with per-currency prices succeeds.
  2. Buyer locale determines which price to display.
  3. Audit-chain emits `PriceShown` event for analytics.
- **Accessibility AC**: price display accessible.
- **i18n AC**: per-locale.

---

## D. Functional Requirements

### D.1 Core charge surface

| ID | Requirement |
|---|---|
| FR-P-01 | `POST /v1/charges` MUST accept `{tenant_id, amount, currency, payment_method, customer_id?, metadata, idempotency_key, purpose_class}`. |
| FR-P-02 | Idempotency-Key MUST be UUID v7 (or UUID v4); server MUST store key→response for 24h. |
| FR-P-03 | Currency MUST be ISO 4217; supported set day-one: USD, EUR, GBP, KRW, JPY, AUD, CAD, SGD, CHF, BRL, plus stretch HKD, THB, IDR. |
| FR-P-04 | Server MUST compute a `psp_route` from `(tenant_pack, currency, payment_method, psp_health, tenant_psp_preference)`. |
| FR-P-05 | Server MUST persist a `Charge` row before any PSP call (outbox pattern; ADR-0050). |
| FR-P-06 | On PSP success, server MUST persist state `succeeded` + receipt + emit `charge.succeeded` webhook + audit-chain seal. |
| FR-P-07 | On PSP failure, server MUST persist state `failed` + reason + emit `charge.failed` webhook. |
| FR-P-08 | Charges MUST be tenant-isolated; cross-tenant charge access is refused at Cedar gate AND Postgres RLS. |

### D.2 Subscription surface

| ID | Requirement |
|---|---|
| FR-P-10 | `POST /v1/subscriptions` MUST create a subscription with `{plan_id, customer_id, payment_method, trial_days?, billing_cycle_anchor?}`. |
| FR-P-11 | Subscription state machine: `pending → active → past_due → cancelled_at_period_end → cancelled` with `paused` side-branch. |
| FR-P-12 | Server MUST own the state machine (not PSP); PSP is transport only. |
| FR-P-13 | Renewals MUST retry per `payments::retry_ladder::standard` = `[+0d, +3d, +7d]` (configurable). |
| FR-P-14 | Proration MUST be computed on mid-cycle plan changes; preview endpoint MUST exist. |
| FR-P-15 | Pause + resume MUST be supported; max pause = 90 days. |

### D.3 Marketplace + surface

| ID | Requirement |
|---|---|
| FR-P-20 | `POST /v1/connect/accounts` MUST onboard a sub-merchant via Stripe Express / Adyen MarketPay / Toss Sub-Merchant. |
| FR-P-21 | KYB flow MUST collect business name, tax ID, beneficial owners, bank account. |
| FR-P-22 | Charges MAY include `transfer_data.destination=<connect_account>` + `application_fee=<cents>`. |
| FR-P-23 | Payouts MUST follow per-account schedule (`daily|weekly|monthly`) + jurisdictional minimum threshold. |
| FR-P-24 | 1099-K (US) / equivalent (other jurisdictions) MUST be generated annually per sub-merchant exceeding threshold. |

### D.4 Refund + dispute surface

| ID | Requirement |
|---|---|
| FR-P-30 | `POST /v1/refunds` MUST support full + partial refunds. |
| FR-P-31 | Refund MUST reverse tax proportionally. |
| FR-P-32 | Refund window enforcement per tenant policy (default 90 days; configurable). |
| FR-P-33 | Disputes MUST flow into a state machine: `needs_response → under_review → won|lost`. |
| FR-P-34 | Evidence submission MUST accept JSON + file attachments. |

### D.5 Tax surface

| ID | Requirement |
|---|---|
| FR-P-40 | Tax engine MUST compute tax per charge based on `(tenant_tax_registration, customer_billing_address, product_tax_class)`. |
| FR-P-41 | Supported tax regimes: US sales tax (Stripe Tax integration), EU VAT, UK VAT, KR VAT, JP consumption tax, AU GST, CA GST/HST/PST, BR ICMS/PIS/COFINS (stretch). |
| FR-P-42 | Tax-filing CSV/EDI export per jurisdiction MUST be available. |
| FR-P-43 | Tax-line MUST appear on charge receipt + invoice PDF. |

### D.6 Age + compliance surface

| ID | Requirement |
|---|---|
| FR-P-50 | Charges MUST go through Cedar `payments::age_assurance::*` gate. |
| FR-P-51 | <13 → refuse + parent-notification (COPPA). |
| FR-P-52 | 14-17 → KOSA tier with spend cap + parent notification. |
| FR-P-53 | ≥18 → allow per local age-of-consent. |
| FR-P-54 | EU member-state age-verification respected per `compliance_pack=pack-eu-{country}`. |

### D.7 provider-BYOK surface

| ID | Requirement |
|---|---|
| FR-P-60 | Tenant MAY connect their own Stripe / Adyen / Toss account via OAuth. |
| FR-P-61 | Tenant's PSP API key MUST be stored as SecretReference (ADR-0255 §D-4); oyatie MUST NOT see plaintext. |
| FR-P-62 | Rotation flow MUST be available; overlap window 24h. |
| FR-P-63 | Disconnect flow MUST support 30-day grace; old charges settle; new charges refused after disconnect. |

### D.8 Webhook surface

| ID | Requirement |
|---|---|
| FR-P-70 | `POST /v1/webhook_endpoints` MUST register a URL + event-type subscription. |
| FR-P-71 | Webhooks MUST be HMAC-SHA256 signed; replay window 5min. |
| FR-P-72 | Delivery MUST retry per exponential backoff ladder. |
| FR-P-73 | Replay last 30 days from dashboard. |

### D.9 Audit + DSAR surface

| ID | Requirement |
|---|---|
| FR-P-80 | Every charge + refund + dispute + payout MUST emit an audit-chain Ed25519 seal (ADR-0028). |
| FR-P-81 | DSAR right-to-erasure MUST cascade to payments; PII fields tombstoned within 30 days. |
| FR-P-82 | Financial-audit retention (e.g., 7 years) MUST be honored even after DSAR; subject-link broken via key rotation. |
| FR-P-83 | Regulator-pull endpoint MUST be available with 4-eye approval. |

### D.10 Currency conversion + settlement surface

| ID | Requirement |
|---|---|
| FR-P-90 | Charges MAY occur in one currency; payouts MAY settle in another. |
| FR-P-91 | FX rate + spread MUST be surfaced in payout statement. |
| FR-P-92 | Settlement currency configurable per account. |

---

## E. Non-functional Requirements

### E.1 Performance budgets

| Metric | P50 | P95 | P99 | Notes |
|---|---|---|---|---|
| `POST /v1/charges` (excluding PSP RTT) | ≤30 ms | ≤80 ms | ≤200 ms | server-side only; PSP RTT is reported separately |
| `POST /v1/charges` end-to-end (with PSP) | ≤300 ms | ≤800 ms | ≤2 s | depends on PSP; Stripe US ~450ms typical |
| `POST /v1/subscriptions` | ≤50 ms | ≤150 ms | ≤300 ms | server-side state-machine init |
| `POST /v1/refunds` end-to-end | ≤500 ms | ≤1.2 s | ≤3 s | PSP RTT included |
| Webhook delivery time-to-first-attempt | ≤500 ms | ≤2 s | ≤5 s | from event-emit to outbound POST |
| Tax computation per line | ≤5 ms | ≤15 ms | ≤30 ms | in-memory rates; per-jurisdiction nexus check |
| Idempotency cache lookup | ≤500 µs | ≤2 ms | ≤5 ms | Valkey hot |

(Evidence: modeling notes `docs/performance-budgets/payments-charge-budget.md` + `docs/performance-budgets/payments-webhook-budget.md` to be authored M02. Stripe public benchmark: P50 ~150ms charge, P99 ~600ms; Adyen P50 ~200ms.)

### E.2 Availability

| Surface | Target |
|---|---|
| `POST /v1/charges` | 99.99% monthly (≤4.4 min downtime/month) |
| `POST /v1/subscriptions` | 99.95% monthly |
| `POST /v1/refunds` | 99.95% monthly |
| Webhook delivery | 99.9% (events queued; eventually delivered) |
| Read-side analytics | 99.9% |

Charge surface is the highest-availability surface (revenue path). Subscription / refund are next. Webhooks are eventually-consistent with durable queue (Kafka outbox; ADR-0050).

### E.3 Scalability

- Per-cell baseline: 10,000 charges/s sustained; bursts to 100,000 charges/s.
- Per-tenant rate-limit: 100 charges/s default; per-tenant override.
- Postgres + Citus sharded by `tenant_id`.
- Outbox pattern (ADR-0050) for PSP calls + webhook dispatch.
- Read-replicas for analytics + dashboard reads.

### E.4 Subscription state machine

```
[pending] --activate--> [active]
              |
              +--pause--> [paused] --resume--> [active]
              |
              +--cancel_at_period_end--> [cancelled_at_period_end]
                  |
                  +--period_end_reached--> [cancelled]
              |
              +--retry_ladder_exhausted--> [past_due]
                  |
                  +--retry_succeed--> [active]
                  |
                  +--retry_fail_terminal--> [cancelled_for_nonpayment]
```

States are owned by oyatie payments, not by PSP. PSP state may diverge; payments substrate state is authoritative.

### E.5 Idempotency contract

- Client passes `Idempotency-Key` header on POST requests.
- Server stores `(tenant_id, idempotency_key, request_hash) → response` for 24h.
- Replay with same key + same request: returns identical response, no PSP call.
- Replay with same key + different request: returns 409 Conflict.

### E.6 Tax engine

- Per-tenant tax registrations: jurisdiction, certificate, nexus type, active dates.
- Per-product tax class (e.g., digital_goods, physical_goods, services, subscription_services).
- Per-charge tax computation: `(tenant_registration, customer_billing_address, product_tax_class) → tax_rate`.
- Tax-filing CSV/EDI export per jurisdiction.
- Stripe Tax integration as primary; Avalara as fallback; manual rates as backup for jurisdictions without integration.

### E.7 Security

- PCI-DSS L1 v4 compliance for the cardholder-data environment (CDE) — only the PSP touches PAN; we use tokenized payment_method references.
- All PSP API keys live as SecretReference in OpenBao (ADR-0117); HSM-backed in regulated packs.
- Webhook signatures verified HMAC-SHA256; replay window 5min.
- Per-tenant Postgres RLS enforces `tenant_id` isolation.
- Cedar gates every charge, refund, dispute, payout, regulator-pull.
- KR-FSS: pack-kr tenants get FSS-mandated event emission to `audit-chain-kr-fss-mirror`.

### E.8 Compliance posture per pack

| Pack | Standards |
|---|---|
| pack-us | PCI-DSS L1 v4; SSAE 18 SOC 2 Type II; state-specific privacy (CCPA, CDPA, etc.) |
| pack-us-healthcare | + HIPAA + BAA in place between oyatie ↔ PSP; PHI not in PSP payload |
| pack-eu | PCI-DSS L1 v4; PSD2 SCA + RTS; GDPR Art. 9 (special category) only with explicit consent; DSA |
| pack-kr | PCI-DSS L1 v4; KR-FSS Electronic Financial Transactions Act; PIPA; ISMS-P certification |
| pack-jp | PCI-DSS L1 v4; APPI; FSA payment-services act |
| pack-sg | PCI-DSS L1 v4; MAS Payment Services Act; PDPA |
| pack-au | PCI-DSS L1 v4; AUSTRAC AML/CTF Act; Privacy Act 1988 |
| pack-br | PCI-DSS L1 v4; LGPD; BCB PIX regulations |

### E.9 Audit retention

| Pack | Retention |
|---|---|
| pack-us | 7 years (IRS) |
| pack-us-healthcare | 6 years HIPAA + 7 years financial |
| pack-eu | 7 years (Member State commercial code typical; GDPR Art. 5(1)(e) bound by purpose) |
| pack-kr | 5 years KR-FSS; 7 years tax (NTS) |
| pack-jp | 7 years |
| pack-sg | 5 years |
| pack-au | 7 years |
| pack-br | 5 years LGPD; 5 years SPED (federal tax) |

### E.10 Cost budget

- Per-charge cost target: ≤$0.005 oyatie-internal compute + PSP fees passed through.
- Per-tenant infra cost: variable per volume; per-tenant cost dashboard exposed.

### E.11 DR posture (ADR-0343)

- Manifest target: `manifest.json` declares RTO p99 900 seconds, RPO p99 60 seconds, `multi_region_active_active: true`, `dr_tier: T1`, `replication_shape: active-active-multi-az-cross-region-warm`, and `failover_runbook: runbooks/psp-failover-cascade-execution.md`.
- RTO/RPO target: charge, refund, payout, dispute, subscription, KYC/KYB, settlement, and webhook paths use the manifest target of RTO p99 <= 15m and RPO p99 <= 60s.
- Compliance-pack floors: the manifest target exceeds PCI-DSS-L1-v4 24h/1h, HIPAA-2024 1h/5m, SOC2-T2 4h/15m, EU-AI-ACT-2024-HIGH-RISK 30m/5m, KR-CSAP 1h/15m, ISO27001 4h/1h, and KR-PIPA RRN 1h/5m.
- Multi-region posture: active-active is enabled for the finance-critical runtime; PSP failover remains governed by `microservices/payments/runbooks/psp-failover-cascade-execution.md`, while `microservices/payments/multi-region.md` provides legacy regional topology detail.
- WHY: tenants must know whether money movement is authorized, queued, declined, or under PSP failover, and must never see duplicate charges or silent payout loss during a cell event.

### E.12 Capacity model (ADR-0340)

- Manifest baseline: `capacity_model` declares 0.2 CPU per tenant, 768 MiB RAM per tenant, 24 GiB storage per tenant, and per-tenant connections of 8 Valkey, 6 Postgres, and 12 outbound HTTP.
- Capacity-model alignment: Year-3 sustained peak is about 56M charges/day, or 648/s average; flash peak is about 6480/s; Charge::Create critical-path budget is 242 ms p99 against a 500 ms SLO; CRDB aggregate write capacity is 450k/s sustained and 900k/s burst across 18 cells.
- Scaling dimension: manifest `scaling_dimension` is `per_request`; per-tenant token buckets remain separate for Charge::Create, Refund::Create, Payout::Schedule, SubMerchant::Onboard, and Dispute::SubmitEvidence.
- Cell placement class: manifest `cell_placement_class` is Tier-0 and `pod_runtime_tier` is 1; rationale is request-driven charge, payout, webhook, KYC/KYB, and settlement paths with high financial fan-out.
- Autoscaling boundary: autoscaling starts from the manifest baseline and expands by request pressure; webhook ingest still uses the companion 128->256 per-cell replica envelope during PSP retry storms.
- WHY: payments load is revenue-critical and PSP-limited per tenant, so capacity must protect idempotent charge paths and webhook absorbency before dashboard or analytics work.

### E.13 Sustainability and cost attribution (ADR-0344)

- Manifest status: `sustainability_emission_model` is currently absent; this section is the PRD adoption target that the next manifest pass must codify.
- Emission claim: every charge, refund, payout, dispute, subscription, KYC/KYB, AML, and audit-chain row includes `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with the rollup axes tenant, product, capability, provider, cell, and compliance_pack.
- Provider-routing affected by carbon: no for real-time charge authorization, fraud scoring, PCI-DSS real-time fraud detection, PSP failover, AML escalation, or HIPAA emergency incident response; yes for settlement exports, reconciliation, replay, monthly statements, and non-urgent regulator bundles.
- Tenant cost surface: tenants see PSP fees, Oyatie compute, audit-chain emission, storage, carbon, and marketplace-facilitator pass-through in payments dashboards and finops-portal.
- WHY: money movement needs transparent per-tenant chargeback while regulatory surfaces such as CSRD, SB-253, and SEC climate disclosure require carbon evidence that does not interfere with authorization latency or fraud controls.

### E.14 API versioning posture (ADR-0342)

- Public API version model: OpenAPI, AsyncAPI, and proto3 contracts carry the YYYY-MM-DD version triplet in `Oya-API-Version`, the URL prefix, and the proto3 version field.
- SDK semver model: generated payments SDKs use major.minor.patch, with breaking contract changes limited to major releases.
- Support window: the last 3 public API versions are supported for at least 180 days.
- Per-tenant pinning: supported for paid and regulated tenants; demo_trial tenants track the current stable version.
- Internal-mesh exemption: yes; direct gRPC between Oyatie services remains governed by ADR-0145 and does not require public carrier triplet routing.

---

## F. UX Flows

### F.1 One-shot purchase (B2C, KakaoPay)

```
[User taps Buy in messenger]
         |
         v
[POST /v1/charges {idempotency_key, amount, currency: KRW, ...}]
         |
         v
[Cedar age + jurisdiction gates] -- forbid --> [Refusal screen + audit]
         |
         v (permit)
[PSP route: KakaoPay primary]
         |
         v
[KakaoPay QR + push to phone]
         |
         v (user confirms in KakaoPay app)
[PSP webhook charge.succeeded]
         |
         v
[Audit chain seal + receipt PDF + UI updates with sticker pack delivered]
         |
         v
[Subscriber notification: "Your sticker pack is ready"]
```

### F.2 Subscribe (B2C, EU SEPA + SCA)

```
[User clicks Subscribe €5/mo]
         |
         v
[Locale: EU, amount × 12 = €60 → SCA required]
         |
         v
[POST /v1/subscriptions {plan_id, payment_method=sepa_debit}]
         |
         v
[State: pending → SCA challenge UI iframe]
         |
         v
[Bank SCA confirm]
         |
         v
[State: pending → active]
         |
         v
[First charge → succeed]
         |
         v
[Audit seal + welcome email + receipt PDF]
         |
         v
[Subsequent monthly renewals via retry ladder]
```

### F.3 Marketplace charge with platform fee

```
[Buyer pays $100 to vendor]
         |
         v
[POST /v1/charges {amount: 10000, application_fee: 1000, transfer_data.destination: vendor_acct}]
         |
         v
[Cedar gate: marketplace::charge_permitted]
         |
         v
[PSP: Stripe charge with transfer]
         |
         v
[Charge succeeds → vendor balance +$90; platform balance +$10]
         |
         v
[Weekly payout cron]
         |
         v
[Vendor: $90 to vendor's bank; Marketplace Mai: $10 to her bank]
         |
         v
[Year-end 1099-K to vendor]
```

### F.4 Refund flow

```
[Support agent clicks Refund $7.50]
         |
         v
[Cedar gate: refund::permit (agent role + reason code)]
         |
         v
[POST /v1/refunds {charge_id, amount: 750, reason: 'customer_request'}]
         |
         v
[Tax proportional reversal]
         |
         v
[PSP refund call]
         |
         v
[State: charge.partial_refunded]
         |
         v
[Webhook refund.created + audit seal]
         |
         v
[Customer notification with updated receipt]
```

### F.5 Dispute (chargeback) flow

```
[PSP notifies dispute]
         |
         v
[State: needs_response (created)]
         |
         v
[Vendor dashboard alert + email]
         |
         v
[Vendor uploads evidence]
         |
         v
[POST /v1/disputes/<id>/evidence]
         |
         v
[PSP submits to network]
         |
         v
[State: under_review]
         |
         v
[Network decision: won OR lost]
         |
         v (lost)
[Ledger reversal: vendor balance -$X]
         |
         v
[Audit chain emit dispute.lost]
```

### F.6 provider-BYOK Stripe OAuth onboarding

```
[Tenant clicks Stripe]
         |
         v
[GET /v1/tenant/psp-account/connect/init?provider=stripe]
         |
         v
[Redirect to Stripe OAuth Authorize URL]
         |
         v
[Tenant logs into their Stripe account; grants access]
         |
         v
[Stripe redirects back with auth code]
         |
         v
[POST /v1/tenant/psp-account/connect/callback {code}]
         |
         v
[Server exchanges code for restricted key]
         |
         v
[Store SecretReference in OpenBao]
         |
         v
[State: tenant.psp.connected]
         |
         v
[Tenant dashboard: "Stripe connected ✓"]
```

### F.7 KYB verification flow

```
[Vendor onboarding init]
         |
         v
[Collect: business name, tax ID, beneficial owners, address, bank]
         |
         v
[POST /v1/connect/accounts {vendor_payload}]
         |
         v
[Stripe Express → underwriting]
         |
         v
[State: kyb_pending]
         |
         v (approved)
[State: kyb_approved → payouts unblocked]
         |
         v
[Vendor receives email: "Your account is ready"]
```

### F.8 DSAR cascade

```
[DSAR submitted: subject email]
         |
         v
[Subject pseudonymized; cascade dispatched]
         |
         v
[Payments µservice receives DSAR cascade event]
         |
         v
[Identify charges + refunds + subscriptions linked to subject]
         |
         v
[Tombstone PII fields (name, email, address) within 30 days]
         |
         v
[PSP-side card-token revoked]
         |
         v
[Retain financial records 7y; break subject↔record link via key-rotation]
         |
         v
[Audit-chain emit DSARCompleted + report PDF]
```

---

## G. Success Metrics

### G.1 Latency (production target)

- P50 charge end-to-end (incl PSP): ≤300 ms.
- P99 charge end-to-end (incl PSP): ≤2 s.
- P50 charge server-only: ≤30 ms.
- P99 charge server-only: ≤200 ms.

### G.2 Throughput

- Sustained 10,000 charges/s per cell; bursts 100,000/s.
- Sustained 50,000 webhook deliveries/s.

### G.3 Conversion + retention

- Checkout-to-success conversion ≥ 92% (Stripe benchmark: ~95% for low-friction cards; oyatie target reflects multi-PSP routing improving on single-PSP baselines).
- Subscription month-1 retention ≥ 85%.
- Subscription month-12 retention ≥ 60%.
- Card-decline retry recovery ≥ 35% (Stripe Smart Retries parity).

### G.4 Reliability

- Charge success rate (excl. legitimate declines) ≥ 99.95%.
- Webhook delivery rate (excl. failed-endpoint) ≥ 99.99%.
- Idempotency duplicate-prevention ≥ 99.999%.
- Audit-seal latency P99 ≤ 1s.

### G.5 Support + business

- Tickets per 1k charges ≤ 0.5.
- Average time-to-resolution ≤ 2 business days.
- NPS (B2C) ≥ 60.
- NPS (B2B) ≥ 50.
- DAU/MAU on payment surface ≥ 0.5.
- ARR growth via payments substrate ≥ 30% YoY.

### G.6 Compliance

- PCI-DSS L1 v4 attested annually; no scope expansion incidents.
- DSAR-cascade SLA: 30 days from request to completion.
- KR-FSS audit-pull: ≤4 business hours.
- Zero PHI in PSP payloads (HIPAA scope hygiene): 100%.

---

## H. Compliance Impact

This µservice attaches the following compliance packs per tenant + cell per ADR-0251:

| Pack | Standards |
|---|---|
| pack-us | PCI-DSS L1 v4; SSAE 18 SOC 2 Type II; CCPA / CPRA; state privacy (CDPA, ConsumerPrivacyAct family) |
| pack-us-healthcare | + HIPAA + 45 CFR §164.308 §164.310 §164.312 §164.316; BAA with PSP |
| pack-eu | PCI-DSS L1 v4; PSD2 SCA + RTS; GDPR (Art. 5, 6, 9, 17, 25, 30, 32); DSA; eIDAS for digital signatures |
| pack-uk | PCI-DSS L1 v4; FCA Payment Services Regulations 2017; UK GDPR |
| pack-kr | PCI-DSS L1 v4; FSS Electronic Financial Transactions Act; PIPA; ISMS-P |
| pack-jp | PCI-DSS L1 v4; APPI; FSA Payment Services Act |
| pack-sg | PCI-DSS L1 v4; MAS Payment Services Act; PDPA |
| pack-au | PCI-DSS L1 v4; AUSTRAC AML/CTF; Privacy Act 1988 |
| pack-br | PCI-DSS L1 v4; LGPD; BCB PIX; SPED |
| pack-coppa-refuse | COPPA <13 refusal; no payment surface available |
| pack-kosa-minor | KOSA 14-17 tier; spend cap; parental notification |
| pack-eu-ai-act-annex-iii | If risk engine is classified as high-risk AI: Annex III §5(b) creditworthiness + §6 high-risk gates |

Compliance evidence emission:

- Per-charge audit-chain Ed25519 seal (ADR-0028).
- PCI-DSS quarterly ASV scan; annual on-site assessment (L1).
- HIPAA: per-disclosure record per §164.528 even for operational disclosures.
- KR-FSS: per-event emission to `audit-chain-kr-fss-mirror` topic for regulator subscription.
- DSAR: per-request completion report PDF + JSON.

---

## I. Open Questions

| # | Question | Owner | Target ADR / Date |
|---|---|---|---|
| 1 | Crypto-payment (BTC, ETH stablecoin) integration: in-scope for M03 or post-M06? | council-finance | M03 decision |
| 2 | Buy-now-pay-later (Klarna, Affirm, Afterpay): which packs? | axis-payments | M04 |
| 3 | Real-time payments (FedNow US, SEPA Instant): adopt as separate PSP or as PSP-feature flag? | axis-payments + ops-treasury | M03 |
| 4 | Per-tenant chargeback liability split policy: keep, reverse_proportionally, reverse_full — defaults? | council-finance | M02 |
| 5 | Fraud-ML model: train per-pack or global? | ops-fraud + axis-intelligence | M03 |
| 6 | Per-tenant payout currency vs settlement currency conversion fee: oyatie absorbs vs passes through? | council-finance | M02 |
| 7 | Tax-engine fallback when Stripe Tax is down: Avalara, manual rates, or refuse-with-retry? | axis-payments | M02 |
| 8 | Subscription gifting: gift-card balance vs direct charge? | axis-payments + axis-marketing | M03 |
| 9 | Cross-border B2B invoicing (e.g., US tenant billing EU buyer): VAT reverse-charge handling? | council-finance | M02 |
| 10 | PSP routing AI: rule-based or ML-optimized? | ops-finops + axis-intelligence | M04 |

---

## J. Out of Scope

The following are explicitly NOT in scope for this µservice:

1. **Insurance claim processing (837P/837I/835)** — out of scope; lives in the future `healthcare-claims` µservice. Payments substrate handles only patient-portion charges.
2. **Bank-account-to-bank-account peer transfers (e.g., Venmo-style)** — out of scope; lives in future `wallet` µservice.
3. **Loan origination + underwriting** — out of scope (highly regulated; separate µservice).
4. **Cryptocurrency custody** — out of scope.
5. **Buy-side credit cards (debit-card issuing for end-users)** — out of scope (Marqeta-class issuing); future µservice.
6. **In-store / point-of-sale terminal hardware** — out of scope (Stripe Terminal-class); future µservice.
7. **Tax-filing as-a-service** — we generate filing CSV/EDI but do NOT file on behalf of tenant.
8. **Anti-money-laundering case management** — we emit signals; case workflow is in `compliance` µservice.
9. **Mass-payouts to non-destinations** — out of scope (Stripe Treasury / Wise-equivalent); future.
10. **Currency exchange beyond PSP-provided FX** — we do not run an FX desk.

---

## K. Bounded Contexts (BC tree)

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api`, `adapter`, `rest`, `worker`, `sdk`, `app`. Backend-qualified adapters use `*-adapter-<backend>` per ADR-0105 Amendment 3.

| BC | Crate family | Purpose |
|---|---|---|
| `charge` | `payments-charge-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-stripe,adapter-adyen,adapter-toss,adapter-kakaopay,adapter-linepay,adapter-wechatpay,adapter-alipay,adapter-paypal,rest,worker,sdk,app}` | One-shot charge surface; PSP routing |
| `subscription` | `payments-subscription-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Subscription state machine; renewals; proration |
| `refund` | `payments-refund-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Refund flow + state machine |
| `dispute` | `payments-dispute-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Chargeback dispute flow + evidence |
| `payout` | `payments-payout-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Marketplace payout to accounts |
| `connect-account` | `payments-connect-account-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Sub-merchant onboarding + KYB |
| `tax-engine` | `payments-tax-engine-{kernel,domain,usecase,api,adapter,adapter-stripe-tax,adapter-avalara,sdk,app}` | Per-jurisdiction tax computation |
| `ledger` | `payments-ledger-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk,app}` | Double-entry ledger of all money movements |
| `webhook-dispatch` | `payments-webhook-dispatch-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Outbound webhook delivery + replay |
| `idempotency` | `payments-idempotency-{kernel,domain,usecase,api,adapter,adapter-valkey,sdk}` | Idempotency-Key store |
| `fraud-screening` | `payments-fraud-screening-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Risk engine; ML + rule-based |
| `psp-routing` | `payments-psp-routing-{kernel,domain,usecase,api,adapter,sdk}` | PSP-route decision logic |
| `audit-chain-bridge` | `payments-audit-chain-bridge-{kernel,domain,usecase,api,adapter,worker,sdk}` | Ed25519 seal emission per ADR-0028 |
| `regulator-pull` | `payments-regulator-pull-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | 4-eye regulator-audit pull endpoint |

(Crate count: ~80 across 14 BCs; scaffold incrementally via IP-001..IP-020 series.)

---

## L. Integration Surface

### L.1 Workflow events produced

| Event type | Trigger | Consumed by |
|---|---|---|
| `payment.charge.succeeded` | Charge success | messenger (sticker delivery), shorts (tip ack), audit-chain |
| `payment.charge.failed` | Charge fail | retry-logic, notification |
| `payment.subscription.created` | Subscription init | tenant onboarding flow |
| `payment.subscription.renewed` | Renewal success | retention analytics |
| `payment.subscription.cancelled` | Cancel | churn analytics |
| `payment.subscription.past_due` | Retry-ladder enter | notification, support |
| `payment.refund.issued` | Refund | accounting, notification |
| `payment.dispute.opened` | Dispute init | vendor alert |
| `payment.dispute.won` | Dispute win | accounting |
| `payment.dispute.lost` | Dispute loss | accounting, ledger reversal |
| `payment.payout.completed` | Payout settled | vendor notification, accounting |
| `payment.kyb.approved` | Sub-merchant KYB done | vendor onboarding complete |
| `payment.kyb.rejected` | Sub-merchant KYB fail | vendor onboarding retry |
| `payment.refused.coppa` | <13 refusal | parent notification |
| `payment.refused.kosa_cap` | KOSA cap hit | parent notification |
| `payment.psp.failover` | PSP route switch | ops alert |
| `payment.regulator.pull` | KR-FSS / regulator audit pull | ops alert |

### L.2 Workflow events consumed

| Event type | Produced by | Action |
|---|---|---|
| `tenant.onboarded` | tenancy | Initialize tenant payment config; default PSP route table |
| `tenant.activated` | tenancy | Enable charge surface |
| `tenant.suspended` | tenancy | Refuse new charges; hold payouts |
| `dsar.erasure.requested` | governance | Cascade DSAR; tombstone PII |
| `compliance.pack.attached` | compliance | Reload Cedar fragments + tax registrations |
| `messenger.purchase.intent` | messenger | Begin charge flow for sticker-pack purchase |
| `shorts.tip.intent` | shorts | Begin tip-charge flow |
| `plugin-app-store.purchase.intent` | plugin-app-store | Begin app-purchase flow |

### L.3 Ontology writes

| Object Type | Written by BC |
|---|---|
| `payments::Charge` | `charge` |
| `payments::Subscription` | `subscription` |
| `payments::Refund` | `refund` |
| `payments::Dispute` | `dispute` |
| `payments::Payout` | `payout` |
| `payments::ConnectAccount` | `connect-account` |
| `payments::TaxRegistration` | `tax-engine` |
| `payments::LedgerEntry` | `ledger` |

### L.4 Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `tenancy::Tenant` | `charge` | Tenant payment config + jurisdiction pack |
| `identity::User` | `charge` | Customer identification for risk + tax |
| `compliance::CompliancePack` | `charge`, `subscription` | Active packs for Cedar policy evaluation |

---

## M. Cross-references and acceptance criteria

| ID | Criterion | Verification |
|---|---|---|
| AC-P-01 | `POST /v1/charges` succeeds end-to-end within P99 ≤ 2s | k6 load test |
| AC-P-02 | Idempotency: same key + same body → identical response, no PSP call | nextest |
| AC-P-03 | Tenant isolation: tenant A cannot query tenant B charges | nextest + RLS test |
| AC-P-04 | Cedar gate: COPPA <13 refusal | nextest |
| AC-P-05 | KOSA spend cap enforced over rolling 30d window | nextest |
| AC-P-06 | DSAR cascade tombstones PII within 30d | e2e test |
| AC-P-07 | Financial audit retention 7y honored post-DSAR | nextest |
| AC-P-08 | PCI scope hygiene: no PAN in our DB | static analyzer |
| AC-P-09 | HIPAA: no PHI in PSP payload | static analyzer |
| AC-P-10 | KR-FSS audit-pull SLA ≤ 4h | e2e timing test |
| AC-P-11 | Webhook delivery ≥ 99.99% | observability SLO |
| AC-P-12 | Subscription state machine correctness | property-based test |
| AC-P-13 | Refund proportional tax reversal | nextest |
| AC-P-14 | provider-BYOK provider connect via OAuth | e2e |
| AC-P-15 | Multi-PSP failover ≤ 60s circuit-break | e2e |

---

## N. Related ADRs + memory references

| Reference | Relation |
|---|---|
| ADR-0244 | Tenant scoping primitive — every payments row carries tenant context |
| ADR-0251 | Compliance pack primitive — packs attach per tenant + cell |
| ADR-0255 §D-4 | provider-BYOK SecretReference model — provider-BYOK semantics |
| ADR-0243 | Cedar gate — every charge / refund / dispute / payout goes through a Cedar policy |
| ADR-0028 | Audit chain — Ed25519 + Merkle seal per money-movement |
| ADR-0049 | Outbox pattern — PSP calls + webhook dispatch reliability |
| ADR-0050 | Kafka outbox — event durability |
| ADR-0117 | OpenBao SecretReference — PSP keys + signing keys |
| ADR-0131 | Per-microservice flat layout — this µservice native under it |
| ADR-0145 | Inter-µservice communication — no direct PSP calls from product µservices |
| ADR-0148 | Layered service mesh — north-south + east-west posture |
| ADR-0179 | Sovereign cloud per pack — per-pack PSP routing |
| ADR-0249 | Multi-category marketplace — payments substrate enables it |
| `feedback_tenant_as_universal_scoping_primitive` | KS#3 |
| `feedback_cedar_as_universal_gate` | KS#2 |
| `feedback_byok_everywhere_credentials` | KS#10 |
| `feedback_build_ahead_of_certification` | KS#9 — build PCI-L1-shape day-one |
| `feedback_compliance_pack_primitive` | KS#8 |

---

## O. Cross-Slice References (to be added when sibling Slices land)

The following sections will be added when the corresponding Slice agents complete their work in the keystone-bundle 2026-05-20 documentation pass:

- **Slice ADR-author** — link to `ADR-payments-substrate.md` (if authored) for the formal decision record.
- **Slice runbook-author** — link to `microservices/payments/runbooks/charge-incident.md`, `subscription-renewal-incident.md`, `psp-failover.md`, `regulator-pull.md`.
- **Slice spec-author** — link to `/specs/microservices/payments.json` (JSON Schema for charge / subscription / refund payloads + tenant model overlay).
- **Slice user-story-bank** — extend `docs/user-stories/b2c-consumer-surfaces.md` with payment-product-surface stories; extend `b2b-work-surfaces.md` with B2B finance stories that REFERENCE this PRD.
- **Slice testing-strategy** — link to `microservices/payments/testing-strategy.md` for E2E test catalog, fuzz-test plan, property-based subscription state-machine test, idempotency replay test, PSP-mock harness.
- **Slice synthesis** — link to the keystone-bundle synthesis doc once it consolidates payment + identity + ontology + workflow PRDs.
- **Slice memory** — link to `feedback_payments_substrate_2026_05_20.md` (after capture by the memory Slice agent).

---

## P. PSP detail matrix

### P.1 Per-PSP capability mapping

| PSP | Region focus | Charges | Subscriptions | Refunds | Disputes | / Marketplace | 3DS / SCA | KR-FSS | provider-BYOK supported | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Stripe | Global; primary US/EU | Y | Y (Billing) | Y | Y | Y (Express/Standard) | Y | N (no native KR-FSS) | Y (OAuth Standard) | Reference PSP; canonical adapter shape |
| Adyen | Global; primary EU + APAC | Y | Y (Recurring) | Y | Y | Y (MarketPay) | Y | N | Y | Multi-acquirer routing inside Adyen |
| Braintree | Global; primary US | Y | Y | Y | Y | Y (Marketplace) | Y | N | Y | Owned by PayPal; useful for PayPal-balance payers |
| Checkout.com | Global; primary EU + UK | Y | Y | Y | Y | P | Y | N | Y | High-volume B2B |
| Toss Payments | KR | Y | Y | Y | Y | Y (KR sub-merchant) | KR-FSS | Y (native) | Y | KR-mandated routing for many tenant types |
| KakaoPay | KR | Y | P | Y | Y | N | KR-FSS | Y (native) | Y | KR consumer wallet |
| LINE Pay | JP, TW, TH | Y | P | Y | Y | N | JP/TW/TH equivalents | N | Y | JP-primary; TW/TH secondary |
| WeChat Pay | CN, HK, MO | Y | P | Y | Y | N | CN regulator | N | Y | Cross-border allowed with reporting |
| Alipay | CN, HK, SG | Y | P | Y | Y | N | CN regulator | N | Y | Cross-border allowed |
| Naver Pay | KR | Y | Y | Y | Y | N | KR-FSS | Y (native) | Y | KR consumer wallet (sibling of KakaoPay) |
| PayPal Business | Global | Y | Y | Y | Y | Y (Marketplaces) | Y | N | Y | High-trust consumer flow |
| Square | US, CA, JP, AU, UK, IE | Y | Y | Y | Y | Y | Y | N | Y | SMB POS-adjacent |
| Worldpay (FIS) | Global | Y | Y | Y | Y | Y | Y | N | Y | Enterprise volume |
| BCB PIX (BR) | BR | Y (via PSP adapter) | Y | Y | N (no chargeback) | N | N | N | Y | PSP wraps PIX rails |
| SEPA Direct Debit | EU | Y | Y | Y (R-transactions) | N (recall, not chargeback) | N | Y (SCA) | N | Y | EU recurring rail |
| ACH (US) | US | Y | Y | Y (R-codes) | N (NSF only) | N | N | N | Y | US recurring rail |
| GoCardless | EU, UK, AU, NZ | Y (DD only) | Y (Recurring) | P | P | N | Y (SCA) | N | Y | Direct-debit specialist |
| Razorpay | IN | Y | Y | Y | Y | Y | RBI guidelines | N | Y | IN expansion candidate |
| Mercado Pago | LATAM | Y | Y | Y | Y | Y | LATAM equivalents | N | Y | LATAM expansion |
| Klarna | EU + US BNPL | Y (BNPL) | N | Y | Y | N | SCA | N | Y | BNPL stretch |
| Afterpay | US, AU, UK | Y (BNPL) | N | Y | Y | N | N | N | Y | BNPL stretch |
| Affirm | US | Y (BNPL) | N | Y | Y | N | N | N | Y | BNPL stretch |

Day-one (M02) PSP adapter set: Stripe, Adyen, Toss, KakaoPay, LINE Pay, WeChat Pay, Alipay, PayPal Business. Naver Pay, Square, Razorpay, Mercado Pago in M03. BNPL family in M04.

### P.2 PSP routing decision table

```
INPUT: (tenant_pack, currency, payment_method_class, psp_health_map, tenant_psp_preference)
OUTPUT: ordered list of PSP candidates
```

| (pack, currency, method) | Primary | Fallback 1 | Fallback 2 |
|---|---|---|---|
| (pack-us, USD, card) | Stripe | Adyen | Braintree |
| (pack-us, USD, ach) | Stripe ACH | — | — |
| (pack-us-healthcare, USD, card) | Stripe (BAA) | — | — |
| (pack-eu, EUR, card) | Adyen | Stripe | Checkout.com |
| (pack-eu, EUR, sepa_debit) | GoCardless | Stripe SEPA | Adyen |
| (pack-uk, GBP, card) | Checkout.com | Stripe | Adyen |
| (pack-kr, KRW, card) | Toss | KG Inicis | Stripe (cross-border allowed) |
| (pack-kr, KRW, wallet:kakaopay) | KakaoPay | — | — |
| (pack-kr, KRW, wallet:naverpay) | NaverPay | — | — |
| (pack-kr, KRW, wallet:tosspay) | Toss | — | — |
| (pack-jp, JPY, card) | Stripe | Adyen | Square |
| (pack-jp, JPY, wallet:linepay) | LINE Pay | — | — |
| (pack-cn, CNY, wallet:wechatpay) | WeChat Pay | — | — |
| (pack-cn, CNY, wallet:alipay) | Alipay | — | — |
| (pack-sg, SGD, card) | Stripe | Adyen | — |
| (pack-au, AUD, card) | Stripe | Adyen | Square |
| (pack-br, BRL, pix) | BCB PIX via Stripe BR | Adyen BR | — |
| (pack-br, BRL, boleto) | Stripe Boleto | Adyen BR | — |

### P.3 PSP webhook normalization

Each PSP has its own webhook schema. The substrate normalizes to a canonical event shape:

```json
{
  "id": "evt_payments_<uuid>",
  "type": "payment.charge.succeeded",
  "tenant_id": "t_acme",
  "occurred_at": "2026-05-20T14:32:11.420Z",
  "data": {
    "object_type": "charge",
    "charge_id": "ch_<uuid>",
    "amount": 19900,
    "currency": "USD",
    "psp": "stripe",
    "psp_event_id": "evt_1ABC...",
    "metadata": { ... }
  },
  "_meta": {
    "psp_raw_signature_verified": true,
    "psp_received_at": "2026-05-20T14:32:11.000Z",
    "audit_chain_seal": "sha256:abc..."
  }
}
```

Adapters parse + normalize PSP-native shapes (Stripe Events API, Adyen NotificationItem, Toss notification, KakaoPay notification, LINE Pay confirmation, etc.) and emit the canonical shape onto the platform-internal event bus + tenant-subscribed webhook endpoints.

---

## Q. Ledger detail

### Q.1 Double-entry ledger model

Every money movement creates ≥2 ledger entries that sum to zero:

```
Charge of $100 from customer:
  customer_receivable (debit -$100) + revenue (credit +$100) = $0

Refund of $30:
  revenue (debit -$30) + customer_payable (credit +$30) = $0

Marketplace charge of $100 with $10 fee, $90 to vendor:
  customer_receivable (debit -$100)
    + vendor_payable (credit +$90)
    + platform_revenue (credit +$10)
  = $0

Payout of $90 to vendor:
  vendor_payable (debit -$90)
    + bank_account (credit +$90)
  = $0

Dispute lost (chargeback) of $100:
  revenue (debit -$100)
    + dispute_loss (credit +$100)
  = $0
```

### Q.2 Reconciliation cadence

- T+1 PSP settlement file ingestion at 04:00 local cell time.
- Per-PSP per-currency netting; matched against platform ledger.
- Mismatches > $1 raise INC-ticket auto.
- Mismatches > $100 also page on-call treasury.
- Daily, monthly, quarterly reports published to ops-dashboard.

### Q.3 Close-the-books

- Monthly close on the 5th business day of the following month.
- Adjustment journal entries (manual) reviewed + audit-chain sealed.
- Reports: P&L per tenant, P&L per PSP, P&L per region, refund-rate analysis, dispute-rate analysis.
- Export to NetSuite / SAP / QuickBooks via journal-export API.

---

## R. Fraud + risk detail

### R.1 Rule engine

- Per-tenant rule sets (e.g., "decline if velocity > 5 charges in 1 hour").
- Default rules: card-testing detection (rapid small charges, distinct cards from same IP), velocity (charges per minute per customer), geo-mismatch (billing country ≠ IP country > $500), card-BIN risk (anonymous-prepaid declined for amount > $200 unless tenant overrides).
- Rule output: `score 0-100` + reason codes.

### R.2 ML model

- Trained on historical charge outcomes (success, decline, dispute) + features (amount, currency, customer history, time-of-day, device fingerprint, BIN).
- Per-pack model (privacy: no cross-pack training); shared global model gated by ADR-0251 review.
- Outputs `risk_score 0-100` per charge.
- Monthly retrain; A/B test new model against current.

### R.3 Manual review

- Queue with reviewer UI: charge details, customer history, prior charges, risk-score breakdown.
- Reviewer decisions: allow, decline, escalate.
- SLA: 1 business day from queue entry; auto-decline after.

### R.4 EU AI Act note

If the risk engine is classified as a high-risk AI system per Annex III §5(b) (creditworthiness of natural persons), additional compliance lights up:

- High-risk AI conformity assessment (Annex IV).
- Risk-management system (Art. 9).
- Data governance (Art. 10).
- Technical documentation (Art. 11).
- Record-keeping (Art. 12).
- Transparency to deployers (Art. 13).
- Human oversight (Art. 14).
- Accuracy + robustness + cybersecurity (Art. 15).

Whether the risk engine triggers Annex III depends on the use case: charge-decline ≠ creditworthiness (no credit extended). The substrate refuses to host any high-risk-AI use case until the conformity assessment is complete (per ADR-0250 build-ahead-of-certification doctrine).

---

## S. Performance evidence

### S.1 Modeling notes

- `docs/performance-budgets/payments-charge-budget.md` (TBD M02) — decomposes the 200ms server-only budget into Cedar eval (5ms), idempotency lookup (2ms), tax compute (15ms), Postgres write (10ms), outbox enqueue (5ms), audit emit (10ms), response render (5ms), buffer (158ms).
- `docs/performance-budgets/payments-webhook-budget.md` (TBD M02) — decomposes webhook delivery time-to-first-attempt (500ms) into outbox poll (50ms), payload sign (5ms), DNS + TCP + TLS (200ms), HTTP send (245ms).

### S.2 Hyperscaler benchmark comparisons

- **Stripe**: P50 charge ~150ms, P99 ~600ms (public reports); Stripe Tax P50 ~30ms.
- **Adyen**: P50 ~200ms, P99 ~800ms.
- **PayPal**: P50 ~400ms (legacy), P99 ~1.5s.
- **oyatie target**: P50 ≤ 300ms (PSP-RTT bound), P99 ≤ 2s. Above Stripe baseline; below PayPal.

### S.3 Sensitivity analysis

- PSP RTT dominates P99 (~70%). Reducing P99 below 1s requires PSP cooperation OR pre-authorized payment-method tokenization (saves 1 PSP call).
- Per-pack region latency: pack-kr in OCI ap-seoul-1 → Toss in KR (≤20ms RTT typical). Cross-region (pack-us tenant charging KRW via Toss) adds 150-300ms.

---

## T. Migration + rollout

### T.1 M02 (Foundation) ship plan

- Day-0 to Week-2: PSP adapter scaffolds (Stripe, Adyen, Toss, KakaoPay, LINE Pay, WeChat Pay, Alipay, PayPal).
- Week-3 to Week-6: Charge BC + Idempotency BC + Webhook BC + Audit-chain bridge.
- Week-7 to Week-10: Subscription BC + Refund BC + Dispute BC.
- Week-11 to Week-14: Tax engine BC + Ledger BC + Connect-account BC + Payout BC.
- Week-15 to Week-18: Fraud-screening BC + Regulator-pull BC + PSP-routing BC.
- Week-19 to Week-22: E2E test + load test + chaos test; advisory CI lanes flip to BLOCKER.
- Week-23 to Week-26: M02 ship — pack-us + pack-eu + pack-kr; B2C + B2B + oyatie-internal-tenant.

### T.2 M03 expansion

- BNPL family (Klarna, Afterpay, Affirm).
- Naver Pay, Square, Razorpay (IN), Mercado Pago (LATAM).
- Real-time payments (FedNow, SEPA Instant).
- Healthcare patient-portion deep enhancement (Stripe Terminal optional).

### T.3 M04+ enhancements

- Cryptocurrency stablecoin (USDC, USDT) integration via Circle or equivalent.
- Wallet µservice integration for peer transfers.
- Advanced fraud ML (cross-pack with privacy budget).
- Loyalty + rewards integration.

### T.4 Sunset + deprecation policy

- PSP adapter deprecation: 18-month advance notice; per ADR `no_silent_regression` doctrine.
- Tenant configuration migration: automated for in-place compatible changes; manual migration with timelock for breaking changes.

---

## U. Change log

- **2026-05-20** — Initial publication as part of keystone-bundle 2026-05-20 foundational-doctrine documentation pass. Authored to close the gap identified in `feedback_autonomous_implementation_artifacts`: payments is a substrate but had no PRD; substrate µservices that touch money MUST be intern-buildable from the doc alone.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- ADR-0347 — every `governance-*` CI lane prefix in the Oyatie corpus RENAMES to `governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `payments` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `payments` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 6 module pin(s) across 3 context(s).
- Scaling input: `per_request` with cell placement `Tier-0` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
