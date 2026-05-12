# Team: GTM — Marketing

## Mission
This team owns Oyatie's brand and marketing: the Oyatie / oYa brand identity, trust portal content, product marketing, content strategy, partner marketing, and demand generation. It exists to make the cohesion thesis legible to enterprise buyers, regulators, and developer communities — the product is genuinely differentiated, and the marketing team's job is to close the gap between product reality and market perception. It does **not** own product roadmap, partnership contracts (→ `gtm-partnerships`), or customer success (→ `gtm-customer-success`).

## Owned axes / surfaces / contracts
- **Axis(es):** GTM — Marketing
- **Surfaces:**
  - Brand: Oyatie / oYa identity system, logo, color, typography, voice, tone
  - Trust portal (content owned here; infrastructure hosted on `axis-search`): compliance posture, audit evidence summaries, SLO uptime history
  - Product marketing: positioning, messaging, competitive differentiation, ICP definition
  - Content: blog, developer docs (marketing layer), case studies, white papers, webinars
  - Demand generation: SEO, SEM, events, KR tech community (DEVIEW, PyCon KR, GDG Korea)
  - Partner marketing: co-marketing with Naver, Kakao, NHN, KT, Amazon Korea
  - Developer community: developer relations, open-source community engagement
- **Cross-axis contracts:** none owned
- **Catalog records:** none
- **Runbooks:** none (marketing is not an on-call function)
- **ADRs:** none

## In-scope work
- Brand identity: Oyatie (full name) / oYa (logo mark); brand guidelines for all product surfaces, marketing materials, partner co-brand
- Trust portal content: compliance evidence summaries, uptime history, certifications earned (CSAP, K-ISMS-P, SOC-2 when applicable) — sourced from `ops-compliance` and `ops-sre-reliability`
- Product marketing: per-axis and per-vertical positioning; competitive landscape; ICP (ideal customer profile) definitions per vertical
- Content marketing: technical blog (developer-targeted), thought leadership (regulatory, AI governance, cohesion thesis), case studies (from `gtm-customer-success` references)
- Demand generation: KR-market SEO/SEM, developer event sponsorship, analyst relations (Gartner, IDC Korea)
- Internationalization: brand localization per regional pack (KR: 오야티, JP: オヤティ, etc.) — worked with `council-architecture` per `INTERNATIONALIZATION.md`
- Developer relations: developer advocacy, community programs, open-source engagement, hackathons
- Partner co-marketing: joint press releases, co-branded collateral with Naver / Kakao / NHN / KT / Amazon Korea

## Out-of-scope (anti-scope)
- Partnership contract negotiation (→ `gtm-partnerships`)
- Product roadmap (→ founder + council-architecture)
- Customer success (→ `gtm-customer-success`)
- Advertising platform operations (→ `axis-ads-analytics` — marketing may *buy* ads on external platforms; it doesn't operate Oyatie's own ads platform)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `ops-compliance` | Compliance certifications, evidence summaries for trust portal | Monthly |
| `ops-sre-reliability` | SLO uptime history for trust portal | Monthly |
| `gtm-customer-success` | Customer references, case study candidates, NPS data | Quarterly |
| `gtm-sales-se` | Win/loss insights, competitive intelligence | Monthly |
| `gtm-partnerships` | Partner co-marketing briefs, joint announcement timing | Per partnership event |
| `council-architecture` | Brand localization guidance, INTERNATIONALIZATION.md | Per new locale |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-sales-se` | Sales collateral, competitive battlecards, product marketing materials | Monthly |
| `gtm-partnerships` | Co-brand templates, partner marketing briefs | Per partner |
| `gtm-customer-success` | Onboarding materials, product update communications | Per release |

## Success metrics
- **Trust portal uptime and freshness:** updated within 24 h of new compliance certification or SLO report
- **Developer community NPS:** ≥ 40 (measured quarterly)
- **Inbound pipeline contribution from marketing:** tracked; ≥ 30% of pipeline from inbound by W-Cloud-Preview
- **Brand recognition in KR enterprise tech segment:** baseline survey at W-SaaS-Preview; track quarterly
- **Content publication cadence:** ≥ 2 technical blog posts/month; ≥ 1 white paper/quarter

## Escalation path
- Internal: tech lead → team manager
- Cross-team: founder for brand guideline exceptions; `ops-compliance` for trust portal content disputes
- Legal: `gtm-partnerships` + counsel for press release or partner announcement legal review
- Founder: as last resort (brand crisis)

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — content calendar, trust portal updates, demand generation metrics
- Cross-team review: monthly GTM sync with all GTM teams; quarterly brand review with founder

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (marketing tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: N/A

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Brand consolidation to Oyatie inconsistency in external materials | Medium | Brand guidelines gate; all external content reviewed against `ADR-0017` brand rename batch |
| Trust portal content outdated → enterprise buyer trust gap | Medium | Monthly refresh cycle; automated trigger from `ops-compliance` on new certification |
| Partner co-marketing commitment exceeds product capability | High | SE in `gtm-sales-se` must validate product readiness before any partner announcement |

## Sources scanned
PRD.md §1 (brand: Oyatie / oYa), §2 (user classes), DOC-CATALOG.md §2.3 (doc.internationalization co-owner = gtm-marketing), ADR-0017 (brand rename).
