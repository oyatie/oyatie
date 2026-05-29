---
doc_class: CompetitorParity
title: "Competitor parity matrix"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Competitor parity matrix


| Capability | Apple App Store | VS Code Marketplace | AWS Marketplace | Shopify App Store | Stripe | Salesforce AppExchange | Oyatie |
|---|---|---|---|---|---|---|---|
| Per-app permissions | Yes (since iOS 6) | Yes (Permissions API) | Limited | Yes (scopes) | n/a | Yes | **Yes** |
| Signed artifacts | Yes (notarization) | Yes (VSIX signing) | Limited | n/a | n/a | Yes | **Yes (Cosign)** |
| Sandbox runtime | Yes (iOS sandbox) | Limited | n/a | n/a | n/a | n/a | **Yes (Wasmtime)** |
| Revenue share | Yes (30/15%) | Yes | Yes | Yes | Yes | Yes | **Yes (planned)** |
| Auto vetting pipeline | Yes | Limited | Manual | Yes | n/a | Yes | **Yes (8-stage deterministic)** |
| Per-app SLO | n/a | n/a | Limited | n/a | n/a | n/a | **Yes (OpenSLO published)** |
| Per-app rate limit | Limited | n/a | Yes (per-acct) | Yes (per-app) | Yes | n/a | **Yes (100 req/s default; admin override)** |
| Auditable per-action trail | Limited | n/a | CloudTrail | Limited | Yes (radar) | Yes | **Yes (audit-chain) |
| Cross-pack regulatory overlay | n/a | n/a | n/a | n/a | n/a | n/a | **Yes (per ADR-0064)** |
| In-house portal substrate | n/a | n/a | n/a | n/a | n/a | n/a | **Yes (Backstage)** |
| In-house payout | n/a | n/a | n/a | n/a | n/a | n/a | **Yes (ACH/SEPA/KFTC direct)** |

## Differentiation

- Oyatie's auditable per-action trail + per-app SLO publishing is novel vs. App Store / VS Code Marketplace.
- Oyatie's in-house payout substrate (no Stripe dependency) is the Stripe parity ADR-0211 commits to.
- Oyatie's cross-pack regulatory overlay (kr / eu / us-healthcare / us-financial / us-public-sector) is unique to oyatie's product surface.

