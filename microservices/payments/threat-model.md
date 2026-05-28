---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: payments
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-payments + ops-security + ops-fraud + council-privacy
deciders: council-architecture, ops-security, axis-payments, council-privacy, council-finance
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + PASTA (process for attack simulation) + OWASP Top 10 (2021) + OWASP ASVS L3 + PCI-DSS L1 v4 + NIST SP 800-154
related_adrs:
  - ADR-0028
  - ADR-0145
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0248
  - ADR-0251
  - ADR-0253
  - ADR-0263
  - ADR-0292
  - ADR-0294
  - ADR-0295
  - ADR-0296
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/compliance.md
  - microservices/payments/dpia.md
review_cadence: quarterly + on every Cedar fragment publish + on every new PSP adapter
hyperscaler_precedents:
  - Stripe public security whitepaper
  - Adyen security overview
  - Square PCI evidence library
  - PayPal Risk SDK threat-model
  - Cloudflare Magic Transit + WAF
enforced_frameworks:
  - "PCI-DSS L1 v4 (2024): Reqs 1-12"
  - "SOC 2 Type 2: CC6.1-CC6.8, CC7.1-CC7.5, CC8.1, CC9.1"
  - "ISO 27001:2022: A.5, A.6, A.8, A.9, A.10 (all controls applicable to payment processing)"
  - "GDPR Arts. 5, 6, 9, 22, 25, 28, 30, 32, 33, 35"
  - "KR-FSS Electronic Financial Transaction Act §6, §21-3"
  - "EU PSD2 (EU 2015/2366) Art. 95 (operational + security risks)"
  - "EU PSD2 RTS on SCA (EU 2018/389)"
suggested_frameworks_by_pack:
  pack-kr-fss: ["KR-EFTA §21-3 (보안)", "KR PIPA Arts. 23/24/29-2", "KR-FSS 전자금융감독규정"]
  pack-eu-psd2-sca: ["PSD2 Art. 95", "SCA-RTS Arts. 4-22 (dynamic linking + step-up)"]
  pack-us-state-mtl: ["per-US-state money-transmitter regs", "FinCEN Travel Rule", "OFAC sanctions screening"]
  pack-au-aml-ctf: ["AML / CTF Act 2006 Pt 7", "AUSTRAC reporting thresholds"]
  pack-br-lgpd-finance: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "BACEN Res. 4.893/2021"]
  pack-cn-pipl-2021: ["PIPL Arts. 38-43 (cross-border)", "PBoC payment regs"]
diataxis_quadrant: explanation
doc_status: published
---

# Threat Model — payments µservice

## Purpose

Identify, classify, and mitigate threats to the payments µservice's confidentiality, integrity, availability, and privacy posture. The payments substrate is the single load-bearing financial substrate that every oyatie monetisation surface depends on; a compromise here cascades to every product, every tenant, every consumer. This document is the canonical security artifact reviewed by:

- PCI QSA at first L1-v4 certification.
- KR-FSS during the Korean payments-aggregator licence audit.
- EU PSD2 RTS-on-SCA examiners.
- SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs at first-tenant onboarding.

## Scope

### In-scope

All components introduced by the payments substrate per [`ARCHITECTURE.md`](ARCHITECTURE.md). Specifically:

| Layer | Components |
|---|---|
| Public ingress | TLS 1.3 + ECH + PQC hybrid; OpenAPI 3.2.0 surface; AsyncAPI 3.1.0 event channel; HMAC-signed inbound PSP webhooks. |
| Application | `oya-payments-charge-*`, `oya-payments-refund-*`, `oya-payments-payout-*`, `oya-payments-dispute-*`, `oya-payments-subscription-*`, `oya-payments-sub-merchant-*` crates. |
| Adapter | Per-PSP adapter crates (Stripe / Adyen / Toss / KakaoPay / LINE Pay / WeChat Pay / Alipay). |
| Data | CRDB charges / refunds / payouts / sub-merchants tables; OpenBao PSP-credential storage; object-storage for dispute-evidence. |
| Worker | Webhook-handler workers; reconciliation CronJobs; dunning workers; KYB-document collection workers. |
| Policy | Cedar fragments at `policy/*.cedar`. |
| Audit | Merkle-sealed audit-chain emission per ADR-0028. |

### Out-of-scope

- Threats to the underlying Kubernetes cluster / hypervisor — owned by `cloud-k8s` threat model.
- Threats to OpenBao itself — owned by `cloud-secrets` threat model; payments inherits as upstream.
- Threats to the upstream PSP (Stripe / Adyen) — out-of-our-control; we monitor + degrade per [`runbooks/psp-outage.md`](runbooks/psp-outage.md).
- Threats to the on-cluster Cedar policy-engine itself — owned by `policy-engine` threat model.
- Threats to GitHub Actions runners — owned by `governance` threat model.
- Card-network threats (Visa / Mastercard / Amex card data) **above the PSP-tokenisation hop** — out of our PCI scope because PAN never touches oyatie systems; we operate as a SAQ-A merchant + L1-v4 facilitator above PSP tokenisation.

## Trust boundaries

```text
┌─ Internet ────────────────────────────────────────────────────────────────────┐
│                                                                               │
│  Consumer browser/app          Tenant operator         External PSP webhook   │
│  (B2C-personal)                (B2B-work)              (Stripe / Adyen / …)   │
│        │                              │                         │             │
│        │ HTTPS + HTTP/3 + ECH         │ HTTPS + mTLS-optional   │ HTTPS+HMAC  │
│        │ + PQC + WAF + Bot-Mgmt       │ + WAF + Bot-Mgmt        │             │
│        ▼                              ▼                         ▼             │
│  ┌─ Public ingress (Cilium L7 / Istio gateway / Cloudflare edge) ─────────┐   │
│  │  - TLS termination + ECH + PQC                                         │   │
│  │  - WAF (rate-limit + OWASP CRS + per-route burst caps)                 │   │
│  │  - Bot-Management (JA4+ fingerprint + ML score; X-Oya-Bot-Score header) │  │
│  │  - DDOS protection                                                     │   │
│  └────────────────────────────────────────────────────────────────────────┘   │
│                              │                                                │
└──────────────────────────────│────────────────────────────────────────────────┘
                               ▼
┌─ Payments cluster (Tier-1 / Tier-2 cells) ────────────────────────────────────┐
│                                                                               │
│  Trust boundary 1: External → ingress (TLS + WAF)                             │
│                                                                               │
│  ┌─ charge-rest ─┐  ┌─ refund-rest ─┐  ┌─ payout-rest ─┐  ┌─ dispute-rest ─┐  │
│  │ OIDC          │  │ OIDC          │  │ OIDC          │  │ OIDC           │  │
│  │ Cedar gate    │  │ Cedar gate    │  │ Cedar gate    │  │ Cedar gate     │  │
│  └───────────────┘  └───────────────┘  └───────────────┘  └────────────────┘  │
│             │                                                                 │
│  Trust boundary 2: gRPC mTLS service-to-service (SPIFFE SVID)                 │
│             │                                                                 │
│  ┌─ charge-usecase ─┐  ┌─ refund-usecase ─┐  ┌─ payout-usecase ─┐             │
│  │ Cedar gate again │  │ Cedar gate again │  │ Cedar gate again │             │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘             │
│             │                                                                 │
│  Trust boundary 3: PSP-adapter → external PSP (HTTPS + tenant-credential)     │
│             │                                                                 │
│  ┌─ stripe-adapter ─┐  ┌─ adyen-adapter ─┐  ┌─ toss-adapter ─┐                │
│  │ credential read  │  │ credential read │  │ credential read│                │
│  │ via OpenBao TTL  │  │ via OpenBao TTL │  │ via OpenBao TTL│                │
│  │ ≤60s sidecar     │  │ ≤60s sidecar    │  │ ≤60s sidecar   │                │
│  └──────────────────┘  └─────────────────┘  └────────────────┘                │
│             │                                                                 │
│  Trust boundary 4: data plane (CRDB + OpenBao + object-storage)               │
│  ┌─ CRDB (charges/refunds/payouts) ─┐  ┌─ OpenBao (psp credentials) ─┐        │
│  │ TLS + row-level tenant scoping   │  │ Sealed; KMS-backed unseal   │        │
│  │ Per-cell + RF-3 multi-AZ         │  │ Per-tenant + per-PSP path   │        │
│  └──────────────────────────────────┘  └─────────────────────────────┘        │
│  ┌─ Object-storage (dispute evidence) ─┐ ┌─ Audit-chain (Merkle) ────┐        │
│  │ SSE-KMS-tenant; immutable           │ │ Per ADR-0028; seal events │        │
│  └─────────────────────────────────────┘ └───────────────────────────┘        │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
```

## Threat catalogue

### T-S-01 — Spoofed PSP webhook

| Attribute | Value |
|---|---|
| STRIDE | Spoofing |
| Likelihood | High (PSP webhooks are public-internet-accessible) |
| Impact | Critical (forged refund / payout-state would corrupt ledger) |
| Vector | Adversary sends crafted HTTP POST to `/webhooks/<psp>/v1` claiming a successful charge / refund / dispute outcome. |
| Mitigation | HMAC-signature verification per PSP (Stripe signing-secret, Adyen HMAC, Toss webhook-key); idempotency-key dedup; replay-window ≤5 min; allow-list of known PSP IPs at edge (where available). |
| Detection | `oya.payments.webhook.replay-rejected` audit event; SLO alert on HMAC-failure-rate >0.1%. |
| Residual risk | Low. PSP-rotated signing-secret rotation discipline is the residual concern; rotation playbook in [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md). |

### T-S-02 — Spoofed tenant-operator API call

| Attribute | Value |
|---|---|
| STRIDE | Spoofing |
| Likelihood | Medium |
| Impact | High (cross-tenant data exposure) |
| Vector | Attacker steals tenant operator's OIDC token + tries to access other tenant. |
| Mitigation | Cedar tenant-scope FORBID overrides any erroneous permit (defence-in-depth); short OIDC token TTL (≤1h); step-up auth for sensitive actions per `docs/standards/step-up-auth-classes.md`. |
| Detection | `oya.payments.audit.read` with cross-tenant attempt → audit-chain seal alert. |
| Residual risk | Low. |

### T-S-03 — Spoofed sub-merchant onboarding

| Attribute | Value |
|---|---|
| STRIDE | Spoofing |
| Likelihood | High (KYC-fraud is a known attack pattern at scale) |
| Impact | Critical (money-laundering vector if approved) |
| Vector | Adversary submits forged KYB documents claiming to be a legitimate business. |
| Mitigation | PSP-native KYC stack (Stripe Identity / Adyen Verification API); cross-reference with sanctions list (OFAC + EU + KR-FSS); ML risk score; per-jurisdiction restricted-list; manual review on score >threshold. |
| Detection | `oya.payments.sub-merchant.restricted` event; periodic re-verification per ML re-score; AUSTRAC / FinCEN reporting thresholds. |
| Residual risk | Medium. We follow PSP-native KYC; ultimate residual lies in the PSP's KYC quality. |

### T-T-01 — Double-charge from PSP retry race

| Attribute | Value |
|---|---|
| STRIDE | Tampering |
| Likelihood | High (PSP idempotency systems are eventually-consistent) |
| Impact | High (customer chargeback + financial liability) |
| Vector | Stripe times out our call; we retry; the original call succeeds at Stripe → two charges. |
| Mitigation | Idempotency-key UNIQUE constraint on `(tenant_id, idempotency_key)`; key derived from `(intent_id + amount + currency + payment_method_id)`; 24h replay window. |
| Detection | `oya.payments.charge.double-charge-detected` audit event; daily reconciliation worker flags discrepancy. |
| Residual risk | Low. Covered by [`runbooks/double-charge-detected.md`](runbooks/double-charge-detected.md). |

### T-T-02 — Tampered audit-chain

| Attribute | Value |
|---|---|
| STRIDE | Tampering |
| Likelihood | Low |
| Impact | Critical (audit-trail integrity is foundational to PCI / SOC 2 / KR-FSS) |
| Vector | Adversary with write access modifies audit-event rows. |
| Mitigation | Merkle-sealed audit-chain per ADR-0028; per-µservice signing key in sidecar per ADR-0296; sealed chain on append-only object-storage; daily seal-verification CronJob. |
| Detection | Seal-verification CronJob alarms on chain-break; `oya.payments.audit.chain-break-detected` event. |
| Residual risk | Negligible (Merkle root is published to oyatie-internal `governance` µservice; tampering requires compromise of both the signing key + the seal-verifier). |

### T-T-03 — Tampered Cedar fragment (post-publish)

| Attribute | Value |
|---|---|
| STRIDE | Tampering |
| Likelihood | Low |
| Impact | Critical (could enable cross-tenant data access) |
| Vector | Adversary with cluster write access modifies a live Cedar fragment to add a malicious `permit`. |
| Mitigation | ADR-0294 60s soak window + signed publish per ADR-0293 meta-trust-root; Cedar fragments stored in Git + Kustomize, never live-mutated; PreCheck CronJob validates live state vs Git-of-record. |
| Detection | `oya.payments.policy.live-drift-detected` alarm. |
| Residual risk | Low. |

### T-R-01 — Repudiation: tenant operator denies authorising payout

| Attribute | Value |
|---|---|
| STRIDE | Repudiation |
| Likelihood | Medium (especially in dispute / fraud claims) |
| Impact | High |
| Vector | Tenant operator claims they never approved a >$10k payout. |
| Mitigation | Step-up authentication (passkey / WebAuthn) for payout > policy threshold; signed approval recorded in audit-chain with principal SVID + WebAuthn challenge-response. |
| Detection | Audit-chain replay shows full approval-trail. |
| Residual risk | Low. |

### T-I-01 — Information disclosure: PAN in logs / traces / audit-chain

| Attribute | Value |
|---|---|
| STRIDE | Information disclosure |
| Likelihood | Medium (without controls) |
| Impact | Catastrophic (PCI scope creep + criminal-charge risk + lifetime ban from card-networks) |
| Vector | A developer accidentally logs the raw card-number / CVV. |
| Mitigation | PSP-tokenised path **only** — PAN never reaches oyatie systems (Stripe Elements / Adyen Web Drop-in / Toss SDK collects PAN client-side and POSTs directly to PSP). Redaction-at-OTel-SDK lint that scans for 13-19 digit sequences in trace payloads. Cedar gate denies any log emission that fails the PAN-redaction lint. Per-µservice signing key never sees PAN. |
| Detection | OTel redaction-lint CI lane; daily grep audit on all log streams; QSA-validated quarterly. |
| Residual risk | Low. PCI scope: SAQ-A facilitator above PSP tokenisation. |

### T-I-02 — Information disclosure: cross-tenant payout-balance exposure

| Attribute | Value |
|---|---|
| STRIDE | Information disclosure |
| Likelihood | Low |
| Impact | High |
| Vector | A bug in the query layer returns another tenant's payout balance. |
| Mitigation | Row-level tenant_id scoping enforced by CRDB CHECK constraints; Cedar tenant-scope FORBID baseline; query-layer integration tests inject malicious tenant_id parameters and assert FORBID. |
| Detection | `oya.payments.cross-tenant-read.attempted` event; cross-tenant-query CI lane. |
| Residual risk | Low. |

### T-I-03 — Information disclosure: dispute-evidence exposure

| Attribute | Value |
|---|---|
| STRIDE | Information disclosure |
| Likelihood | Low |
| Impact | High (often contains PII + transaction context) |
| Vector | Object-storage misconfigured (public-read). |
| Mitigation | Object-storage with SSE-KMS-tenant; bucket-policy blocks public; Cedar gate on every read; IaC + scorecards verify private-by-default. |
| Detection | Cloud-config-drift CronJob; quarterly bucket-audit. |
| Residual risk | Low. |

### T-D-01 — DoS: charge-creation flood

| Attribute | Value |
|---|---|
| STRIDE | Denial of service |
| Likelihood | High |
| Impact | High (revenue loss + tenant SLO breach) |
| Vector | Adversary scripts many charge attempts to exhaust capacity or rate-limit-trip a tenant. |
| Mitigation | Edge rate-limit per-IP / per-fingerprint / per-tenant / per-route; Bot-Management ML score; CAPTCHA-on-suspicion; circuit-breaker on per-tenant PSP-quota breach. |
| Detection | `oya.payments.abuse-defence.denied` event; SLO alerts on burst-rate. |
| Residual risk | Medium. Covered by [`runbooks/fraud-spike-detected.md`](runbooks/fraud-spike-detected.md). |

### T-D-02 — DoS: PSP outage cascade

| Attribute | Value |
|---|---|
| STRIDE | Denial of service |
| Likelihood | High (Stripe + Adyen each have ≥1 publicly-disclosed outage per year) |
| Impact | High |
| Vector | Stripe US is down; we cannot process US charges. |
| Mitigation | Per-region PSP failover (Stripe US → Adyen US as fallback where tenant policy permits); circuit-breaker; degraded-mode queue (charges queued + processed on PSP recovery). |
| Detection | PSP-availability monitor; SLO alert on per-PSP success-rate <99%. |
| Residual risk | Medium (fallback PSP must be tenant-approved). Covered by [`runbooks/psp-outage.md`](runbooks/psp-outage.md). |

### T-D-03 — DoS: webhook-storm

| Attribute | Value |
|---|---|
| STRIDE | Denial of service |
| Likelihood | Medium |
| Impact | Medium (worker queue saturation) |
| Vector | PSP retries webhooks aggressively during their own incident → flooding worker queue. |
| Mitigation | Per-PSP rate-limit on webhook ingest; per-PSP queue-quota; dedup via idempotency-key. |
| Detection | Webhook-queue depth alarm. |
| Residual risk | Low. |

### T-E-01 — Elevation: sub-merchant accessing other sub-merchants

| Attribute | Value |
|---|---|
| STRIDE | Elevation of privilege |
| Likelihood | Low |
| Impact | Critical |
| Vector | Sub-merchant uses their token to query another sub-merchant's payouts. |
| Mitigation | Cedar sub-merchant-scope FORBID; sub-merchant.tenant_id binding in token; multi-tier defence per documentation-rigor.md §3.2.3. |
| Detection | Cross-sub-merchant-query audit event. |
| Residual risk | Low. |

### T-E-02 — Elevation: tenant-credential theft → PSP master account access

| Attribute | Value |
|---|---|
| STRIDE | Elevation of privilege |
| Likelihood | Low |
| Impact | Catastrophic |
| Vector | Adversary compromises OpenBao path for tenant's Stripe key → calls Stripe directly as the tenant. |
| Mitigation | OpenBao sealed; per-tenant + per-PSP path scoping; TTL ≤60s; per-µservice principal-bound read; never long-lived in-process. Provider-BYOK isolation (oyatie has no platform-master keys for tenant accounts). |
| Detection | OpenBao audit-log on every credential read; out-of-band PSP API-call-anomaly detection. |
| Residual risk | Low. |

### T-L-01 — Linkability: tenant analytics reveals cross-tenant volume comparison

| Attribute | Value |
|---|---|
| LINDDUN | Linkability |
| Likelihood | Medium |
| Impact | Medium (tenant could infer competitor volume) |
| Vector | Aggregated metrics exposed without DP-noise let one tenant infer another's volume. |
| Mitigation | DP-noise on cross-tenant aggregates per `dashboards/finops-cost-attribution.md` policy. |
| Detection | DP-noise check in dashboard build. |
| Residual risk | Low. |

### T-L-02 — Identifiability: dispute-representment bundle leaks PII

| Attribute | Value |
|---|---|
| LINDDUN | Identifiability |
| Likelihood | Low |
| Impact | High |
| Vector | Representment bundle sent to PSP includes more PII than necessary (e.g., raw IP, raw browser-fingerprint). |
| Mitigation | Per-pack PII-minimisation lint; bundle template per `docs/standards/dispute-representment-minimisation.md`. |
| Detection | Pre-send PII-lint. |
| Residual risk | Low. |

## STRIDE × data-class matrix

| Data class | S | T | R | I | D | E |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **Card data (PAN / PIN / track / CVV)** | T-S-01 | T-T-01 | T-R-01 | T-I-01 | T-D-01 | T-E-02 |
| **Payout / bank-account data** | T-S-02 | T-T-01 | T-R-01 | T-I-02 | T-D-02 | T-E-02 |
| **Sub-merchant KYB documents** | T-S-03 | T-T-02 | T-R-01 | T-I-03 | T-D-01 | T-E-01 |
| **Audit-chain seals** | — | T-T-02, T-T-03 | T-R-01 | — | — | — |
| **PSP credentials** | T-S-01 | T-T-03 | — | T-E-02 | — | T-E-02 |

## Defence-in-depth layers

1. **Edge** — TLS 1.3 + ECH + PQC + WAF + Bot-Management + DDOS + IP allowlist where available.
2. **Auth** — OIDC + step-up auth (passkey / WebAuthn) for high-value actions.
3. **Policy** — Cedar default-deny + cross-tenant FORBID + abuse-defence FORBID.
4. **Data** — Row-level tenant scoping (CHECK constraints) + SSE-KMS-tenant.
5. **Audit** — Merkle-sealed chain + per-µservice signing key + meta-trust-root attestation.
6. **Workload identity** — SPIFFE SVID per pod; mTLS service-to-service.
7. **Credential** — OpenBao sealed; TTL ≤60s; never in-process beyond request.
8. **Detection** — SLO alerts + audit-chain anomaly detection + reconciliation worker.

## Reviewer attestation

| Date | Reviewer | Outcome | Notes |
|---|---|---|---|
| 2026-05-20 | axis-payments + ops-security | Accepted (initial publication) | Standalone publication for Wave-3-A doc-set buildout; PCI-QSA scoping audit scheduled M02-foundation pre-cert. |

## References

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — substrate layering, Cedar gates, audit-event registry.
- [`dpia.md`](dpia.md) — DPIA per GDPR Art. 35 + KR-PIPA Art. 33.
- [`compliance.md`](compliance.md) — pack-overlay control mapping.
- [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md) — PCI DSS incident response.
- [`runbooks/psp-outage.md`](runbooks/psp-outage.md) — PSP-outage response.
- [`runbooks/double-charge-detected.md`](runbooks/double-charge-detected.md) — idempotency-violation response.
- [ADR-0028 — Merkle-sealed audit chain](../../docs/decisions/ADR-0028-audit-chain.md).
- [ADR-0263 — observability emission](../../docs/decisions/ADR-0263-observability-emission-contract.md).
- [ADR-0294 — Cedar fragment soak](../../docs/decisions/ADR-0294-cedar-fragment-soak.md).
- [ADR-0295 — bootstrap CI SPIFFE](../../docs/decisions/ADR-0295-bootstrap-ci-spiffe-killswitch.md).
- [ADR-0296 — library-first credential sidecar](../../docs/decisions/ADR-0296-library-first-credential-sidecar.md).
- Stripe public security whitepaper — `stripe.com/docs/security`.
- Adyen security overview — `adyen.com/legal/security`.
- PCI-DSS L1 v4 — `pcisecuritystandards.org`.
