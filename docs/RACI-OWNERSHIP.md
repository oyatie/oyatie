---
purpose: Ownership and accountability matrix for teams, decision classes, code ownership, and review responsibilities.
doc_status: published
---

# Oyatie — RACI + Ownership Matrix

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `council-architecture`.
> **Companion:** [teams/](teams/) (39 charters), `.github/CODEOWNERS` (per-file owners).
> **Source of truth for per-file owners:** `.github/CODEOWNERS`. This doc is for *roles* across the project.
>
> **`.github/CODEOWNERS` routes ZERO reviewers today — it is a declaration, not a routing mechanism.**
> Measured 2026-08-02: `gh api repos/:owner/:repo/codeowners/errors` returns **111 `Unknown owner`
> errors against 111 owner references** — every one. Cause is structural, not a typo: the repo is
> `owner.type: User`, so `gh api repos/:owner/:repo/teams` returns `[]` and no `@teams/*` handle can
> ever resolve. It also has no force: `required_pull_request_reviews.require_code_owner_reviews` is
> unset on `dev`. Corroborating outcome probe — the eight most recent PRs (#1498–#1507) carry zero
> review requests and zero reviews. Treat the owner column below as the *intended* map, sourced from
> `docs/teams/*/CHARTER.md` and the 110 per-directory `OWNERS` files; do not treat a merged PR as
> having been seen by the listed owner. Disposition (convert to individual handles vs. delete)
> is a founder call — see the open registries task.

## 1. RACI key

- **R** = Responsible (does the work)
- **A** = Accountable (signs off / blocked-by)
- **C** = Consulted (input)
- **I** = Informed (notified)

## 2. Cross-axis ownership matrix (illustrative slice; full table is generated)

| Decision | Council-architecture | Council-privacy | Axis-foundry | Axis-cloud | Axis-saas | Axis-search | Axis-ads-analytics | Axis-workspace | Vertical-* | Regional-packs | Ops-security | Ops-sre | Ops-compliance | GTM | Founder |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| New ADR proposed | C | C | C | C | C | C | C | C | C | C | C | C | C | C | I |
| ADR ratified | A | C | I | I | I | I | I | I | I | I | I | I | I | I | (R for vision-class) |
| New cross-axis contract | A | C | R/C | R/C | R/C | R/C | R/C | R/C | C | I | C | I | I | I | I |
| Data Use Boundary change | C | A+R | C | I | I | I | C | C | C | C | C | I | C | I | I |
| License policy change | A | I | C | I | I | I | I | I | I | I | A | I | I | I | I |
| New regional pack | C | C | I | C | C | C | C | C | C | A+R | I | I | C | C | I |
| New vertical | A | C | I | I | C | C | C | C | A+R | C | I | I | C | C | I |
| New axis | A+R | C | C | C | C | C | C | C | I | I | I | I | I | C | C |
| Wave gate pass | A | C | C | C | C | C | C | C | C | C | C | A+R | A+R | I | I |
| Sev 1 incident | I | C (if data-class) | C (if Foundry) | C (if Cloud) | C (if SaaS) | C (if Search) | C (if Ads) | C (if Workspace) | C (if vertical) | C (if pack-affected) | A+R (if security) | A+R | A+R (regulator notif) | C | I (cross-tenant Sev 1) |
| Foundry capability publish | I | C | A+R | I | I | I | I | I | I | I | I | I | I | I | I |
| New tenant onboarding | I | C | I | I | C | I | I | C | A+R (per vertical) | I | I | I | I | A+R | I |
| Public pricing change | I | I | I | I | C | I | I | I | I | I | I | I | I | A+R | A |
| Brand decision | A | I | I | I | I | I | I | I | I | C | I | I | I | C | A |
| Hire team lead | A (cross-axis) | C | A (foundry) | A (cloud) | A (saas) | A (search) | A (ads) | A (workspace) | A (per vertical) | A (regional packs) | A (security) | A (sre) | A (compliance) | A (gtm) | A |

## 3. Per-surface ownership

Pointer: `.github/CODEOWNERS` is the per-file truth. Sample mapping:

```
crates/platform-tenant-*/        @teams/platform-tenancy-identity
crates/platform-identity-*/      @teams/platform-tenancy-identity
crates/platform-audit-chain-*/   @teams/platform-audit-evidence
crates/platform-eventing-*/      @teams/platform-eventing-og
crates/platform-object-graph-*/  @teams/platform-eventing-og
crates/foundry-*/                @teams/axis-foundry
crates/cloud-*/                  @teams/axis-cloud
crates/saas-*/                   @teams/axis-saas
crates/workspace-*/              @teams/axis-workspace
crates/search-*/                 @teams/axis-search
crates/ads-*/                    @teams/axis-ads-analytics
crates/analytics-*/              @teams/axis-ads-analytics
crates/vertical-healthcare-*/    @teams/vertical-healthcare
crates/vertical-corporate-*/     @teams/vertical-corporate
... (per vertical + per regional pack)

decisions/                           @teams/crew-adr-promotion @teams/council-architecture
docs/PRD.md             @teams/council-architecture
docs/PRIVACY-PROGRAM.md @teams/council-privacy
docs/security-program/security-program.json @teams/ops-security
docs/COMPLIANCE-MATRIX.md @teams/ops-compliance
docs/products/<id>/PRD.md @teams/<owning-team>
docs/teams/<id>/CHARTER.md @teams/<id>
regional-packs/<pack>/                @teams/regional-packs
```

### 3.1 Team charter coverage

This table is generated from `docs/teams/*/CHARTER.md` and validated by `presubmit` (retired CLI `gate validate raci-team-coverage`). Every team charter MUST have a RACI row and a direct CODEOWNERS owner handle.

| team_id | charter | owning_handle | CODEOWNERS requirement |
|---|---|---|---|
| `axis-ads-analytics` | `docs/teams/axis-ads-analytics/CHARTER.md` | `@teams/axis-ads-analytics` | `docs/teams/axis-ads-analytics/CHARTER.md @teams/axis-ads-analytics` |
| `axis-cloud` | `docs/teams/axis-cloud/CHARTER.md` | `@teams/axis-cloud` | `docs/teams/axis-cloud/CHARTER.md @teams/axis-cloud` |
| `axis-foundry` | `docs/teams/axis-foundry/CHARTER.md` | `@teams/axis-foundry` | `docs/teams/axis-foundry/CHARTER.md @teams/axis-foundry` |
| `axis-saas` | `docs/teams/axis-saas/CHARTER.md` | `@teams/axis-saas` | `docs/teams/axis-saas/CHARTER.md @teams/axis-saas` |
| `axis-search` | `docs/teams/axis-search/CHARTER.md` | `@teams/axis-search` | `docs/teams/axis-search/CHARTER.md @teams/axis-search` |
| `axis-workspace` | `docs/teams/axis-workspace/CHARTER.md` | `@teams/axis-workspace` | `docs/teams/axis-workspace/CHARTER.md @teams/axis-workspace` |
| `council-architecture` | `docs/teams/council-architecture/CHARTER.md` | `@teams/council-architecture` | `docs/teams/council-architecture/CHARTER.md @teams/council-architecture` |
| `council-privacy` | `docs/teams/council-privacy/CHARTER.md` | `@teams/council-privacy` | `docs/teams/council-privacy/CHARTER.md @teams/council-privacy` |
| `crew-adr-promotion` | `docs/teams/crew-adr-promotion/CHARTER.md` | `@teams/crew-adr-promotion` | `docs/teams/crew-adr-promotion/CHARTER.md @teams/crew-adr-promotion` |
| `gtm-customer-success` | `docs/teams/gtm-customer-success/CHARTER.md` | `@teams/gtm-customer-success` | `docs/teams/gtm-customer-success/CHARTER.md @teams/gtm-customer-success` |
| `gtm-marketing` | `docs/teams/gtm-marketing/CHARTER.md` | `@teams/gtm-marketing` | `docs/teams/gtm-marketing/CHARTER.md @teams/gtm-marketing` |
| `gtm-partnerships` | `docs/teams/gtm-partnerships/CHARTER.md` | `@teams/gtm-partnerships` | `docs/teams/gtm-partnerships/CHARTER.md @teams/gtm-partnerships` |
| `gtm-sales-se` | `docs/teams/gtm-sales-se/CHARTER.md` | `@teams/gtm-sales-se` | `docs/teams/gtm-sales-se/CHARTER.md @teams/gtm-sales-se` |
| `ops-compliance` | `docs/teams/ops-compliance/CHARTER.md` | `@teams/ops-compliance` | `docs/teams/ops-compliance/CHARTER.md @teams/ops-compliance` |
| `ops-dr-capacity` | `docs/teams/ops-dr-capacity/CHARTER.md` | `@teams/ops-dr-capacity` | `docs/teams/ops-dr-capacity/CHARTER.md @teams/ops-dr-capacity` |
| `ops-finops` | `docs/teams/ops-finops/CHARTER.md` | `@teams/ops-finops` | `docs/teams/ops-finops/CHARTER.md @teams/ops-finops` |
| `ops-security` | `docs/teams/ops-security/CHARTER.md` | `@teams/ops-security` | `docs/teams/ops-security/CHARTER.md @teams/ops-security` |
| `ops-sre-reliability` | `docs/teams/ops-sre-reliability/CHARTER.md` | `@teams/ops-sre-reliability` | `docs/teams/ops-sre-reliability/CHARTER.md @teams/ops-sre-reliability` |
| `platform-api-sdk` | `docs/teams/platform-api-sdk/CHARTER.md` | `@teams/platform-api-sdk` | `docs/teams/platform-api-sdk/CHARTER.md @teams/platform-api-sdk` |
| `platform-audit-evidence` | `docs/teams/platform-audit-evidence/CHARTER.md` | `@teams/platform-audit-evidence` | `docs/teams/platform-audit-evidence/CHARTER.md @teams/platform-audit-evidence` |
| `platform-eventing-og` | `docs/teams/platform-eventing-og/CHARTER.md` | `@teams/platform-eventing-og` | `docs/teams/platform-eventing-og/CHARTER.md @teams/platform-eventing-og` |
| `platform-privacy-dub` | `docs/teams/platform-privacy-dub/CHARTER.md` | `@teams/platform-privacy-dub` | `docs/teams/platform-privacy-dub/CHARTER.md @teams/platform-privacy-dub` |
| `platform-tenancy-identity` | `docs/teams/platform-tenancy-identity/CHARTER.md` | `@teams/platform-tenancy-identity` | `docs/teams/platform-tenancy-identity/CHARTER.md @teams/platform-tenancy-identity` |
| `regional-packs` | `docs/teams/regional-packs/CHARTER.md` | `@teams/regional-packs` | `docs/teams/regional-packs/CHARTER.md @teams/regional-packs` |
| `tactical-first-vertical-pilot` | `docs/teams/tactical-first-vertical-pilot/CHARTER.md` | `@teams/tactical-first-vertical-pilot` | `docs/teams/tactical-first-vertical-pilot/CHARTER.md @teams/tactical-first-vertical-pilot` |
| `vertical-agriculture` | `docs/teams/vertical-agriculture/CHARTER.md` | `@teams/vertical-agriculture` | `docs/teams/vertical-agriculture/CHARTER.md @teams/vertical-agriculture` |
| `vertical-construction` | `docs/teams/vertical-construction/CHARTER.md` | `@teams/vertical-construction` | `docs/teams/vertical-construction/CHARTER.md @teams/vertical-construction` |
| `vertical-corporate` | `docs/teams/vertical-corporate/CHARTER.md` | `@teams/vertical-corporate` | `docs/teams/vertical-corporate/CHARTER.md @teams/vertical-corporate` |
| `vertical-education` | `docs/teams/vertical-education/CHARTER.md` | `@teams/vertical-education` | `docs/teams/vertical-education/CHARTER.md @teams/vertical-education` |
| `vertical-fintech` | `docs/teams/vertical-fintech/CHARTER.md` | `@teams/vertical-fintech` | `docs/teams/vertical-fintech/CHARTER.md @teams/vertical-fintech` |
| `vertical-food` | `docs/teams/vertical-food/CHARTER.md` | `@teams/vertical-food` | `docs/teams/vertical-food/CHARTER.md @teams/vertical-food` |
| `vertical-healthcare` | `docs/teams/vertical-healthcare/CHARTER.md` | `@teams/vertical-healthcare` | `docs/teams/vertical-healthcare/CHARTER.md @teams/vertical-healthcare` |
| `vertical-hospitality` | `docs/teams/vertical-hospitality/CHARTER.md` | `@teams/vertical-hospitality` | `docs/teams/vertical-hospitality/CHARTER.md @teams/vertical-hospitality` |
| `vertical-industrial` | `docs/teams/vertical-industrial/CHARTER.md` | `@teams/vertical-industrial` | `docs/teams/vertical-industrial/CHARTER.md @teams/vertical-industrial` |
| `vertical-legal` | `docs/teams/vertical-legal/CHARTER.md` | `@teams/vertical-legal` | `docs/teams/vertical-legal/CHARTER.md @teams/vertical-legal` |
| `vertical-logistics` | `docs/teams/vertical-logistics/CHARTER.md` | `@teams/vertical-logistics` | `docs/teams/vertical-logistics/CHARTER.md @teams/vertical-logistics` |
| `vertical-public-sector` | `docs/teams/vertical-public-sector/CHARTER.md` | `@teams/vertical-public-sector` | `docs/teams/vertical-public-sector/CHARTER.md @teams/vertical-public-sector` |
| `vertical-real-estate` | `docs/teams/vertical-real-estate/CHARTER.md` | `@teams/vertical-real-estate` | `docs/teams/vertical-real-estate/CHARTER.md @teams/vertical-real-estate` |
| `vertical-retail` | `docs/teams/vertical-retail/CHARTER.md` | `@teams/vertical-retail` | `docs/teams/vertical-retail/CHARTER.md @teams/vertical-retail` |

## 4. Decision rights matrix

| Decision class | Who decides | Veto |
|---|---|---|
| Cross-axis contract change | Architecture council | Founder |
| Data Use Boundary change | Privacy council | Founder + legal |
| License policy change | Architecture council + Security | Founder + legal |
| New axis | Founder + Architecture council |  |
| New vertical | Architecture council + GTM | Founder |
| New regional pack | Architecture council + GTM + per-pack regulator-relations | Founder |
| Brand change | Founder | (none) |
| Sev 1 customer/regulator notification | IM + Comms Manager + Privacy lead + Founder | (none) |
| Hiring per team lead | Per-team manager + skip-level + Founder | (none) |
| Pricing | GTM lead + Founder |  |

## 4.1 Cutover human-orchestrator row

| Role / row id | Responsible | Accountable | Consulted | Informed | Authorized cutover actions | Required pre-execution evidence |
|---|---|---|---|---|---|---|

This row does not grant agents direct `git` or `gh` authority. Agents prepare
manifests, verification evidence, and halt rows; the human orchestrator performs
only the three named one-time carve-outs from ADR-0053.

## 5. Sources
`teams/`, `.github/CODEOWNERS`, [DOC-CATALOG.md](DOC-CATALOG.md), [PRD.md](PRD.md), `CLAUDE.md`.
