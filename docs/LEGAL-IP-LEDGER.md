---
purpose: Oyatie — Legal + IP Ledger
doc_status: published
---

# Oyatie — Legal + IP Ledger

> **Status:** Draft v0.1 skeleton — 2026-05-09.
> **Owner:** `gtm-partnerships` + Founder + external counsel.
> **Companion:** [VENDOR-PARTNER-LEDGER.md](VENDOR-PARTNER-LEDGER.md), [PRD.md §1 brand](PRD.md).

## 1. Trademark

| Mark | Class | Region | Status |
|---|---|---|---|
| Oyatie (wordmark) | 9, 35, 38, 41, 42 (software, business mgmt, telecom, training, scientific) | KR | TBD register |
| oYa (logo) | as above | KR | TBD register |
| Oyatie | as above | JP / US / EU / IN / BR / KSA / UAE / AU / SG | TBD per region |
| Oyatie (legacy) | (legacy) | KR | retain for forensic; no new use |

## 2. Domain

| Domain | Use | Owner | Renewal |
|---|---|---|---|
| oyatie.com | primary | TBD | annual |
| oyatie.kr / .co.kr | KR-locale | TBD | annual |
| oyatiemail.com (or per-tenant subdomain) | Workspace Mail | TBD | annual |
| dev.oyatie.com | dev surface per ADR-0025 | TBD | annual |
| docs.oyatie.com | public docs site | TBD | annual |
| trust.oyatie.com | trust portal | TBD | annual |
| status.oyatie.com | status page | TBD | annual |
| marketplace.oyatie.com | plugin marketplace | TBD | annual |
| api.oyatie.com | public API | TBD | annual |
| mcp.oyatie.com (per-tenant) | MCP gateway | TBD | annual |
| (legacy) oyatie.com / dev.oyatie.com / docs.oyatie.com | retain redirect | per ADR-0006 cookie/redirect | annual |

## 3. Patent strategy

- Defensive posture initially; offensive only if competitive necessity
- Areas of potential filing: cohesion-thesis cross-axis contract enforcement; per-tenant data-class lineage in agent runtime; multi-provider auth model; regional-pack architecture
- KR file first; PCT then per priority country

## 4. OSS license inventory

Per [VENDOR-PARTNER-LEDGER.md §2](VENDOR-PARTNER-LEDGER.md). License-policy ADR drafted at `decisions/ADR-0013-product-license-policy.md`.

## 5. Customer contract templates

| Template | Audience |
|---|---|
| MSA (Master Service Agreement) per region | Tenant onboarding |
| DPA (Data Processing Agreement) per region | Per regulator (GDPR / PIPA / HIPAA / etc.) |
| BAA (Business Associate Agreement) | Healthcare tenants (HIPAA) |
| SCC (Standard Contractual Clauses) | EU cross-border |
| Per-vertical addendum (clinical / fintech / education / public-sector) | Per regulated vertical |
| Plugin developer agreement | Marketplace |
| Cloud customer agreement | Cloud axis customers |
| Search advertiser agreement | Ads axis customers |

## 6. Open legal questions

1. Brand: trademark registration timing (pre-launch in each pack region)
2. ITAR exposure for any defense-adjacent vertical
3. Per-pack data-controller vs data-processor role per regulator
4. KR FSC PG license filing path
5. KR 인터넷전문은행 license path (if pursuing)
6. Per-region customer-contract template harmonization
7. Patent freedom-to-operate analysis per axis
8. Cosign-Rekor + Sigstore as legal evidence path

## 7. Sources
[PRD.md §1](PRD.md), ADR-0017 (brand rename), [VENDOR-PARTNER-LEDGER.md](VENDOR-PARTNER-LEDGER.md), [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md).
