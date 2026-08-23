---
doc_status: published
---

# Team: regional-packs

## Mission

Own the per-locale regional packs that plug into Oyatie's canonical architecture per ADR-0010 regional-pack-architecture. Each pack supplies regulatory binding, i18n / morphology, currency / calendar, tax-invoice, identity providers, payment rails, address book, ecosystem partners, content safety, ad-policy gates, industry data-model extensions, and vendor partners. Do NOT own the canonical seams (those live in `platform-*` kernels); pack maintainers ship the impls.

## Owned axes / surfaces / contracts

- **Axis(es):** cross-cutting; pack architecture per ADR-0010
- **Per-product PRDs:** none directly (packs are infrastructure); per-pack content at [`regional-packs/<pack-id>/PACK.md`](../../regional-packs/)
- **Cross-axis contracts owned/co-owned:** per-pack regulatory binding, per-pack tax / payment / identity adapters, per-pack content + ad-policy gates
- **Catalog records:** `registry/catalog/regional-packs/pack-<region>.yaml`
- **Runbooks owned:** `cross-axis/regional-pack-regulator-update.md`
- **ADRs authored / co-authored:** ADR-0010 (regional-pack-architecture), ADR-0048 (Korean morphology + multilingual tokenization), ADR-0049 (cross-region replication + residency)

## In-scope work

- Per-pack regulatory binding + control mapping
- Per-pack `RegulatoryPack` / `Tokenizer` / `TaxInvoiceFormatter` / `IdentityProvider` / `PaymentRail` / `AddressValidator` / `LocalAdPolicy` / `ContentSafetyRules` / `LocaleFormatter` impls
- Per-pack regulator-watch lane + per-quarter regulator-binding refresh
- Per-pack tenant onboarding support (in concert with per-vertical teams)
- Per-pack regulatory evidence pack regeneration
- Per-pack semver discipline + Cosign signing per ADR-0039
- Per-pack on-call escalation for regulator-impact events

## Out-of-scope (anti-scope)

- Authoring vertical-axis kernels (those live with per-vertical teams; per ADR-0033)
- Authoring axis-foundry code (those live with per-axis teams)
- Authoring cross-axis canonical seams (those live in `platform-*`; this team consumes via trait impls)
- Per-customer integration (handled by `gtm-customer-success`)

## Key dependencies on other teams

| Depends on | What we need | Cadence |
|---|---|---|
| `council-architecture` | Pack architecture ratification + new-pack sign-off | per pack |
| `platform-tenancy-identity` | Tenant kernel + identity seam | continuous |
| `platform-eventing-og` | Object Graph property-tier seam | continuous |
| `axis-cloud` | Region/AZ/cell + KMS + per-region IaC profile | continuous |
| `axis-foundry` | Capability registry + MCP gateway for per-pack capabilities | per pack |
| `council-privacy` | Per-pack residency policy + DSR cascade integration | per pack |
| `ops-compliance` | Per-regulator evidence-pack template + audit cadence | continuous |
| `gtm-partnerships` | Per-region partner contracts (per pack) | per pack |

## Teams that depend on us

| Consumer | What they need | Cadence |
|---|---|---|
| All vertical teams | Per-pack regulatory + identity + payment binding | per vertical wave |
| `axis-saas` | Per-tenant pack binding | continuous |
| `axis-workspace` | Per-pack mail-security + DLP rules + content moderation | continuous |
| `axis-cloud` | Per-pack tax-invoice format + per-region partner | continuous |
| `axis-search` | Per-pack tokenizer + per-pack content-rights | continuous |
| `axis-ads-analytics` | Per-pack ad-policy gate + per-region adtech compliance | continuous |
| `vertical-fintech` | Per-region payment-rails + KYC + AML | per vertical wave |
| `vertical-healthcare` | Per-region clinical + 본인확인서비스 + medical-ad review | per vertical wave |
| `vertical-public-sector` | Per-region 조달청 / public-procurement adapter | per public-sector tenant |

## Success metrics

| Metric | Target |
|---|---|
| Per-pack onboarding cycle (new pack to preview) | ≤ 4 weeks once seam impls authored |
| Per-pack regulator-watch alert response | ≤ 5 business days |
| Per-pack quarterly regulatory refresh | 100% on schedule |
| Per-pack `RegulatoryPack` seam-impl coverage | 100% |
| Per-pack Cosign signing per release | 100% |
| Cross-pack contract violations | 0 per quarter |

Org-level metrics this rolls up to:
- `governance-regional-pack` lane pass rate
- Per-region tenant onboarding velocity
- Per-region regulator-incident count

## Escalation path

- **Internal:** per-pack maintainer → `regional-packs` lead → `council-architecture`
- **Cross-team:** affected per-axis or per-vertical team
- **Privacy / data-class:** `council-privacy`
- **Security:** `ops-security`
- **Founder:** for new pack ratification + brand-naming impacts

## Communication cadence

- Stand-up: per-pack daily + cross-pack weekly
- Per-pack regulator-watch report: weekly
- Cross-pack review: weekly
- Per-quarter pack refresh meeting: each pack reviews its bindings
- Per-quarter cross-pack architecture review with `council-architecture`

## Bandwidth + hiring

- Current FTE: TBD (per-pack maintainer + central architect)
- Target FTE: 2-3 per active pack + 2-3 central architects coordinating cross-pack
- Initial KR pack: 2-3 FTE; JP / US / EU each 2-3 FTE at W-Cloud-Preview onboarding

## Operating norms

- Per-pack maintainer is the regulator-relations primary contact
- Per-pack ADR amendments authored by pack maintainer + reviewed by `council-privacy` + `council-architecture`
- Per-pack regulator-watch lane fires on regulatory feed updates (e.g. KISA / PIPC announcements)
- Per-PR `governance-regional-pack` lane validates seam-impl coverage
- Cosign signing required per release per ADR-0039

## Slice of risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Per-pack regulatory drift between updates | high | quarterly regulator-watch + per-pack refresh cadence |
| Cross-pack contract violations | high | central architect coordinates seam evolution |
| Per-pack maintainer single-point-of-failure | medium | per-pack 2-FTE minimum + cross-training cadence |
| Per-region regulator audit cycle delays pack onboarding | medium | start regulator engagement at month 0 of pack scope |
| Korean morphology library license drift | medium | per ADR-0048 — khaiii Apache-2 fallback; in-house Rust port long-horizon |

## Sources scanned

- ADR-0010 (regional-pack architecture), ADR-0048, ADR-0049
- [`docs/regional-packs/_TEMPLATE.md`](../../regional-packs/_TEMPLATE.md)
- [`docs/regional-packs/pack-kr/PACK.md`](../../regional-packs/pack-kr/PACK.md)
- [DESIGN.md §12](../../DESIGN.md)
- [INTERNATIONALIZATION.md](../../INTERNATIONALIZATION.md)
- [COMPLIANCE-MATRIX.md](../../COMPLIANCE-MATRIX.md)
- Per-pack regulator + industry sources
