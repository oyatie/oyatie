---
purpose: Oyatie — Go-to-Market Plan
---

# Oyatie — Go-to-Market Plan

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `gtm-sales-se` + `gtm-customer-success` + `gtm-marketing` + `gtm-partnerships`.
> **Companion:** [PRD.md §2-§3](PRD.md), per-product PRDs.

## 1. GTM thesis

Oyatie wins by selling **the cohesion** — one tenancy, one identity, one audit chain across SaaS / Workspace / Vertical / Foundry / Cloud / Search / Ads. Not by being best-in-class on any single axis. Every per-axis competitor (Microsoft 365, AWS, Google Workspace, Snowflake, Salesforce, Naver Cloud) sells one slice; Oyatie sells the integrated stack at lower integration tax.

## 2. Personas + buyer journey

| Persona | Buying motion | Channel |
|---|---|---|
| KR Group enterprise CIO | Multi-axis bundle (SaaS + Workspace + Cloud + Vertical-corp + Compliance) | Direct field sales + KR Big-4 SI partners |
| Mid-market vertical (KR + JP + SE-Asia) | Per-vertical pilot → SaaS + Workspace + Vertical | Solutions-engineering-led |
| ISV / Marketplace | Plugin authoring + Foundry capability publishing | Self-serve + dev-rel |
| Cloud customer (post W-Cloud-Stable) | IaaS consumption | Self-serve + enterprise field |
| Search advertiser (post W-Ads-Stable) | Auction self-serve | Self-serve + agency channel |
| Public sector | Per-region procurement (조달청 / 公共調達 / FedRAMP / GAIA-X) | Per-region public-sector specialist |

## 3. KR launch strategy (first commercial wave)

- **Lead vertical**: vertical-corporate (KR Group HR / payroll / mail / GL) — anchored by existing KR Group commitments (per ADR-0050 master plan)
- **Bundle**: SaaS + Workspace + Vertical-corp + per-tenant Foundry + KR-pack
- **Compliance proof**: K-ISMS-P + CSAP target + KCMVP-validated KMS + PIPA + 의료광고 / 금융광고 review evidence per applicable verticals
- **Naver / Kakao integration**: Day-1 connectors to maintain ecosystem hookups (per [DESIGN §12.1 KR pack](DESIGN.md))
- **Pricing**: per-seat for SaaS+Workspace; per-resource for Cloud; per-capability for Foundry

## 4. Pricing + packaging (skeleton; Founder + GTM finalize)

| Tier | Inclusions | Indicative pricing |
|---|---|---|
| **Starter** | SaaS + Workspace + 1 vertical kernel | per-seat |
| **Business** | + plugin marketplace + Foundry capabilities + multi-vertical | per-seat + per-call |
| **Enterprise** | + Cloud preview + dedicated cell + per-pack regulatory bundle + 24/7 support | per-seat + per-resource + commit |
| **Cloud Customer** (post W-Cloud-Stable) | IaaS / PaaS surfaces | per-resource (compute/storage/network) |
| **Search Advertiser** (post W-Ads-Stable) | Sponsored search + display | CPC / CPM auction |
| **Foundry-as-a-product** (post W-Foundry-Stable) | Direct MCP-discoverable capability access | per-call + monthly committed |

## 5. Sales org

- KR field: 5-10 field reps + 5 SEs + Big-4 SI partner alliance
- JP field: 3-5 field reps (post W-Region-Fan-Out wave 1)
- US/EU field: 5-10 each (W-Region-Fan-Out wave 2-3)
- Public-sector specialist per region

## 6. Customer success operating model

- Per-design-partner: dedicated CSM + named SE + monthly QBR
- Per-enterprise: shared CSM + per-vertical SE on-call
- Per-mid-market: pooled CSM + ticket-based support
- Per-self-serve: docs + community
- Trust portal (`trust.oyatie.com`) as compliance / status / postmortem / DSR self-service

## 7. Marketing

- Brand: Oyatie / oYa
- Korean-first content; English secondary; per-pack translations
- Trust portal + status page as marketing surfaces
- Per-vertical case studies (post pilot)
- Developer-rel for Foundry + Marketplace
- Naver Search SEO + KR-locale SEM
- Industry events (KR + JP + global)

## 8. Partnerships

- KR cloud partners: Naver Cloud / NHN Cloud / KT Cloud / Kakao Cloud (integration + co-sell)
- Hyperscalers: OCI + AWS (consume substrate per [DESIGN §3.0.4](DESIGN.md))
- KR Big-4 SI: Samsung SDS / LG CNS / SK C&C / POSCO ICT (delivery)
- KR adtech: KODA + 한국디지털광고협회 + Kakao Moment / Naver 검색광고 (interop)
- Per-region: per-pack partner roster

## 9. Open GTM questions
1. Per-axis cross-sell motion (do we sell axes separately or bundle-only?)
2. Foundry-as-a-product pricing (token-based or capability-based or both?)
3. Cloud-customer onboarding velocity (target invite-only pilot for first 6mo?)
4. Marketplace revenue share (industry-typical 30%? lower for design partners?)
5. Public-sector channel (in-house specialist vs SI partner?)

## 10. Sources
ADR-0017 (Bench + industry preset), ADR-0050 master plan, KR market research, per-product PRDs, [PRD.md §2-§3](PRD.md).
