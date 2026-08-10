---
doc_class: README
template_id: TPL-README
microservice: payments
status: Accepted
owner_team: axis-payments + council-finance + ops-fraud + ops-treasury
date: 2026-05-20
related_adrs: [ADR-0145, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0246, ADR-0248, ADR-0251, ADR-0253, ADR-0254, ADR-0255, ADR-0263, ADR-0273, ADR-0292]
companion_docs:
  - app/payments/PRD.md
  - app/payments/ARCHITECTURE.md
  - app/payments/compliance.md
  - app/payments/threat-model.md
diataxis_quadrant: explanation
doc_status: published
---

# Payments µservice — README

## What this µservice does

The `payments` µservice is the **shared hero-substrate** for every oyatie monetisation surface. Every B2C purchase (sticker packs, creator tipping, premium handles), every B2B subscription (per-seat SaaS, usage-metered cloud-billing), every marketplace-facilitator flow (plugin-app-store checkout, shorts creator tipping, community super-chats), and every payout to a tenant or sub-merchant flows through this µservice. It mirrors the **Stripe platform-facilitator** shape (the canonical hyperscaler precedent), augmented with per-region PSP routing (Adyen EU, Toss / KakaoPay KR, LINE Pay JP, WeChat Pay / Alipay CN-PIPL) and a **per-tenant provider-BYOK** model (the tenant's own Stripe / Adyen account, never oyatie's master account).

## Why it exists

| Goal | How payments delivers |
|---|---|
| **Tenant-scoped money** | Every charge / refund / payout / payment-method / sub-merchant row carries `tenant_id` (ADR-0244). No row exists without tenant context. |
| **Compliance-pack-aware** | PCI-DSS L1 v4, KR-FSS, EU PSD2 / SCA, US state money-transmitter, CCPA / CPRA, AU-AML, BR-LGPD, IN-RBI, CN-PIPL — each attaches per tenant + cell (ADR-0251). |
| **Provider-BYOK** | Every tenant can plug their own Stripe / Adyen / Toss account; oyatie holds **zero PSP credentials** (ADR-0255 §D-4). |
| **Multi-PSP** | One charge-orchestrator routes to Stripe / Adyen / Toss / KakaoPay / LINE Pay / WeChat Pay / Alipay per (region, currency, payment-method, tenant-policy) per ADR-0145 direct-gRPC invariants. |
| **Self-monitoring** | Every charge / refund / payout emits an audit event in the ADR-0263 registry; SLO-gated promotion via ADR-0139. |

## Quick links

- **PRD** → [`PRD.md`](PRD.md) — 1612 lines; problem, personas, ≥40 user stories, NFRs, compliance impact.
- **Architecture** → [`ARCHITECTURE.md`](ARCHITECTURE.md) — substrate layering, BC roster, Cedar gate roster, audit-event registry.
- **Threat model** → [`threat-model.md`](threat-model.md) — STRIDE per data class (PII + PAN + PIN + payout).
- **Compliance** → [`compliance.md`](compliance.md) — per-pack control mapping (PCI / KR-FSS / EU-PSD2 / etc.).
- **Phase plan** → [`PHASE-01-PAYMENTS-MVP.md`](PHASE-01-PAYMENTS-MVP.md) — MVP delivery sequence.
- **Runbooks** → [`runbooks/`](runbooks/) — 8 operational playbooks.
- **Contracts** → [`contracts/`](contracts/) — OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3.
- **Manifest** → [`manifest.json`](manifest.json) — tenant_class posture, BC roster, layer enum, compliance packs.

## Tenant-class model

Payments adopts ADR-0330's canonical model:

- `demo_trial`: $0 evaluation posture for sandbox money movement, OCI Always
  Free defaults, and explicit non-production caps.
- `paid`: production money movement with composable `billing_components`
  (`revenue_share`, `per_seat`, `per_usage`).

Payment capability access is uniform at the product-quality bar. Production
processing, provider-BYOK, compliance packs, marketplace settlement, and
regulated payout flows are gated by `tenant_class == paid`, active
`billing_components`, and the relevant `compliance_pack`, not by a customer
capability ladder.

## Bounded contexts

| BC | Purpose | Key aggregates |
|---|---|---|
| `charge` | Authorise + capture a charge against a payment-method | `Charge`, `PaymentMethod`, `CardFingerprint`, `ChargeAttempt` |
| `refund` | Issue full / partial refund against an original charge | `Refund`, `RefundReason`, `RefundEvidence` |
| `payout` | Move funds from oyatie balance / tenant balance / sub-merchant balance to bank | `Payout`, `BankAccount`, `PayoutSchedule`, `CoolingPeriod` |
| `settlement` | Reconcile per-PSP settlement reports against internal ledger | `SettlementBatch`, `Reconciliation`, `Discrepancy` |
| `kyc-kyb` | Onboard sub-merchants (Stripe / Adyen Marketplace pattern) | `SubMerchant`, `KycKybDocument`, `Verification`, `RestrictedReason` |
| `dispute` | Manage chargeback / dispute lifecycle | `Dispute`, `Evidence`, `RepresentmentBundle` |
| `subscription-lifecycle` | Recurring billing: schedule, dunning, trial, upgrade, cancel | `Subscription`, `BillingCycle`, `DunningStep`, `UsageRecord` |

## Layer roster (per ADR-0105 13-layer)

Per BC × layer, the crates are: `oya-payments-<bc>-{domain,kernel,usecase,adapter,rest,grpc,worker,api,app,sdk}`.

## Audience

- **Internal SRE / on-call** → [`runbooks/`](runbooks/).
- **Tenant operator** → [`contracts/openapi-v1.yaml`](contracts/openapi-v1.yaml) + SDK plan.
- **External auditor** (PCI QSA, KR-FSS) → [`policy/auditor-scope.cedar`](policy/auditor-scope.cedar), [`compliance.md`](compliance.md), [`AUDIT-FINDINGS-2026-05-20.json`](AUDIT-FINDINGS-2026-05-20.json).
- **Intern / new engineer** → start at [`PRD.md`](PRD.md), then [`ARCHITECTURE.md`](ARCHITECTURE.md), then [`PHASE-01-PAYMENTS-MVP.md`](PHASE-01-PAYMENTS-MVP.md).

## Compliance pack roster

- `pack-pci-dss-l1-v4` — card-data scope (PAN / PIN / track data) — **mandatory**.
- `pack-kr-fss` — Korean Financial Supervisory Service oversight.
- `pack-eu-psd2-sca` — Strong Customer Authentication + dynamic linking.
- `pack-us-state-mtl` — per-state money-transmitter licences (US).
- `pack-ccpa-cpra-2023` — California Consumer Privacy Act + 2023 rights expansion.
- `pack-au-aml-ctf` — AML / CTF (Australia).
- `pack-br-lgpd-finance` — LGPD + BACEN.
- `pack-in-rbi` — RBI payment-aggregator licensing.
- `pack-cn-pipl-2021` — China data-residency for WeChat Pay / Alipay.
- `pack-coppa-minor-refusal` — refuse purchases by <13s per ADR-0292.

## Hyperscaler precedents

- **Stripe platform-facilitator** — the canonical sub-merchant onboarding pattern.
- **Adyen MarketPay** — the EU-native marketplace-payments shape.
- **Square Cash for Business** — payout cooling-period + KYB.
- **PayPal Adaptive Payments** — multi-party charge / split / payout.
- **AWS Marketplace** — usage-metered B2B billing.
- **Apple Pay / Google Pay** — wallet-tokenisation flows.

## Status

- **Tenant class posture**: shared substrate for `demo_trial` sandbox flows and `paid` production flows.
- **Maturity**: Proposed → MVP land in M02-foundation (Q3 2026).
- **First non-trivial consumer**: messenger sticker store (B2C MVP), cloud-billing (B2B MVP).
- **SLO tier**: tier-0 (critical revenue path).
- **Cell tier eligibility**: Tier-1 (regulated finance) / Tier-2 (default product) per ADR-0248.

## References

- [`PRD.md`](PRD.md) — full requirements.
- [`docs/decisions/ADR-0702-identity-authz-live-apex.md`](../../docs/decisions/ADR-0702-identity-authz-live-apex.md) — tenant scoping rules.
- [`docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md) — compliance-pack model.
- [`docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md) §D-4 — provider-BYOK.
- [`docs/decisions/ADR-0706-observability-live-apex.md`](../../docs/decisions/ADR-0706-observability-live-apex.md) — audit-event emission.

## Change log

- 2026-05-20: Initial publication. Full doc-set buildout to PR-143 baseline + ≥100 artifact operating bar.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
