---
doc_status: published
---

# Oyatie — Regional Packs

> Per [ADR-0010 regional-pack architecture](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md), every per-locale concern lives in a swappable pack that plugs into canonical seams. Korea, Japan, US, EU, India, Brazil, KSA, UAE, AU, SG can all onboard in parallel.

## 1. Active + planned packs

| Pack | Region | Status | First-onboarding wave |
|---|---|---|---|
| [`oya-pack-kr`](oya-pack-kr/PACK.md) | South Korea | draft (full content) | W-Cloud-Preview (anchor pack) |
| oya-pack-jp | Japan | skeleton (planned) | W-Cloud-Preview (parallel) |
| oya-pack-us | United States | skeleton (planned) | W-Cloud-Preview (parallel) |
| oya-pack-eu | European Union (DE first; FR/SE/NL/IE follow) | skeleton (planned) | W-Cloud-Preview (parallel) |
| oya-pack-in | India | skeleton (planned) | W-Region-Fan-Out wave 1 |
| oya-pack-br | Brazil | skeleton (planned) | W-Region-Fan-Out wave 1 |
| oya-pack-ksa | Saudi Arabia | skeleton (planned) | W-Region-Fan-Out wave 2 |
| oya-pack-ae | UAE | skeleton (planned) | W-Region-Fan-Out wave 2 |
| oya-pack-au | Australia + NZ | skeleton (planned) | W-Region-Fan-Out wave 2 |
| oya-pack-sg | Singapore | skeleton (planned) | W-Region-Fan-Out wave 2 |
| (later) | MX / ID / PH / VN / TH / TR / NG / ZA / CL / CO / ... | planned | W-Region-Fan-Out wave 3+ |

## 2. Authoring a new pack

Per [`checklists/regional-pack-onboarding.md`](../checklists/regional-pack-onboarding.md):
1. Council ratification per ADR-0012 axis admission protocol (per-region pack inherits axis 3 vertical-cloud + cross-cutting)
2. Pack scaffold via `oya pack new <region>` (Foundry skill `oya-regional-pack-author`)
3. Fill 14 sections per [`_TEMPLATE.md`](_TEMPLATE.md): regulatory, compliance packs, i18n, currency, calendar, tax, identity providers, payment rails, address book, ecosystem partners, content safety, ad policy, industry data models, vendor partners
4. Author per-seam impls for `RegulatoryPack` / `Tokenizer` / `TaxInvoiceFormatter` / `IdentityProvider` / `PaymentRail` / `AddressValidator` / `LocalAdPolicy` / `ContentSafetyRules` / `LocaleFormatter`
5. CI lane `oya-governance-regional-pack` validates seam-impl coverage
6. Per-region regulator filing (CSAP / ISMAP / FedRAMP / GAIA-X / MeitY / LGPD / etc.)
7. First tenant onboarded with new pack (design partner)
8. Pack promoted to `preview`

## 3. Per-pack maintainer team

`regional-packs` umbrella under [`teams/regional-packs/`](../teams/regional-packs/) with per-pack sub-team. Per-pack maintainer responsibilities:
- Per-pack CI lane health
- Per-pack regulator-watch lane (per-pack regulator feed)
- Per-pack semver discipline
- Per-pack tenant-onboarding support
- Per-pack tax / invoice / payment / identity / KYC adapter currency
- Per-pack on-call escalation

## 4. Pack release cadence

- Minor (impl-only): weekly / per merge
- Major (regulator semantic shift): quarterly or per-event
- Cosign-signed per release per ADR-0039

## 5. Pack-overlap policy

A tenant may bind to multiple packs (rare; e.g. EU subsidiary of KR HQ).
- Primary pack = tenant's home jurisdiction
- Secondary packs = per-resource override
- Cross-pack conflicts resolved per most-restrictive class per [PRIVACY-PROGRAM §2.2.7](../PRIVACY-PROGRAM.md)

## 6. Sources

- [ADR-0010 regional-pack architecture](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md)
- [DESIGN.md §12 Regional Pack Architecture](../DESIGN.md)
- [INTERNATIONALIZATION.md](../INTERNATIONALIZATION.md)
- [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md)
- Per-pack maintainer's regulator + industry feeds
