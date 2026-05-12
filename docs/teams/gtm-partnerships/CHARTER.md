# Team: GTM — Partnerships

## Mission
This team owns Oyatie's partner ecosystem: Naver / Kakao / NHN / KT / Amazon Korea / KR ISV relationships, global cloud-provider partnerships (AWS, GCP, Azure as resale/co-sell channels), regulatory-body relationships, and the legal/IP ledger that underpins all commercial relationships. It exists because the KR market requires deep ecosystem integration (Naver, Kakao, KT as infrastructure and identity partners), and global expansion requires cloud-provider co-sell agreements and regulator relationships in each jurisdiction. It does **not** own marketing co-campaigns (→ `gtm-marketing`) or enterprise sales execution (→ `gtm-sales-se`).

## Owned axes / surfaces / contracts
- **Axis(es):** GTM — Partnerships
- **Surfaces:**
  - Partner ledger (`VENDOR-PARTNER-LEDGER.md`) — all vendor and partner contracts, expiry tracking
  - Legal/IP ledger (`LEGAL-IP-LEDGER.md`) — patents, trademarks, contract templates
  - Partner integration specs: Naver Cloud API, Kakao i Cloud, KT Cloud, NHN Cloud, Amazon Korea — technical integration requirements to `axis-cloud` regional packs
  - Regulatory-body relationship management: KISA, MFDS, FSC, KCC, NIS, and foreign equivalents
  - ISV onboarding program: KR ISV onboarding to Oyatie marketplace
  - Global cloud-provider co-sell: AWS Partner Network, Google Cloud Partner Advantage, Azure Marketplace
- **Cross-axis contracts:** none owned (consumer of `axis-cloud` regional-pack seams for partner integrations)
- **Catalog records:** none
- **Runbooks:** `runbooks/partner-contract-renewal.md`, `runbooks/regulatory-relationship-escalation.md`
- **ADRs:** none (partner decisions are commercial; regulatory-change ADRs triggered through `ops-compliance`)

## In-scope work
- KR ecosystem partners: Naver / Kakao / NHN / KT partnership agreements, technical integration roadmap, go-to-market co-sell
- Amazon Korea: AWS co-sell for cloud axis, Marketplace listing for ISV channel
- Global cloud co-sell: AWS Partner Network, GCP Partner, Azure ISV co-sell program enrollment and maintenance
- KR ISV program: ISV onboarding to Oyatie plugin marketplace; revenue-share structure; ISV technical enablement
- Regulatory-body relationships: introductory meetings with KISA, MFDS, FSC, KCC, NIS; global equivalent relationships (FDA, EMA, FCA, etc.); briefings on Oyatie's regulatory posture
- Contract management: all partner contracts tracked in `VENDOR-PARTNER-LEDGER.md`; renewal alerts at 90 days
- IP ledger: patents (if any), trademarks (Oyatie / oYa registration in KR, US, EU, JP), contract template library
- Legal counsel coordination: external counsel for partner contract review; KR legal counsel for KR regulatory relationships

## Out-of-scope (anti-scope)
- Enterprise sales execution (→ `gtm-sales-se`)
- Marketing co-campaigns (→ `gtm-marketing` owns creative; partnerships provides the brief)
- Cloud infrastructure implementation (→ `axis-cloud` implements the technical integration; partnerships owns the commercial agreement)
- Regulatory compliance implementation (→ `ops-compliance` implements; partnerships owns the regulator relationship)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-cloud` | Regional-pack technical seam requirements for KR/JP ecosystem partner integrations | Per regional pack |
| `ops-compliance` | Regulatory posture briefing materials for regulatory-body meetings | Monthly |
| `gtm-marketing` | Co-brand templates, joint press release drafts | Per partner event |
| `gtm-sales-se` | Partner-sourced deal routing, co-sell pipeline | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-cloud` | Naver / Kakao / NHN / KT commercial agreements enabling regional-pack integration | Per regional pack |
| `gtm-sales-se` | Partner co-sell introductions, ISV referrals | Monthly |
| `gtm-marketing` | Partner co-marketing briefs, announcement timing | Per partner event |
| `ops-compliance` | Regulatory-body contact relationships for auditor introductions | Per audit |

## Success metrics
- **KR ecosystem partner agreements active (Naver, Kakao, NHN, KT):** ≥ 2 by W-Cloud-Preview
- **Amazon Korea / global cloud co-sell enrollment:** completed by W-Cloud-Preview
- **Partner contract renewal coverage (90-day alert):** 100%
- **Trademark registrations (KR, US, EU, JP for Oyatie/oYa):** 100% filed within 60 days of brand launch
- **Regulatory-body briefing cadence:** ≥ 1 per regulator per year for active regulatory relationships
- **ISV marketplace onboarding pipeline:** ≥ 5 ISVs in pipeline by W-SaaS-Preview

## Escalation path
- Internal: tech lead → team manager
- Cross-team: founder for partner terms outside delegated authority; `ops-compliance` for regulatory escalations
- Legal: external counsel for partner contract disputes
- Founder: as last resort (KR ecosystem partner relationship risk)

## Communication cadence
- Stand-up: async (partnerships is primarily an async function between meetings)
- Weekly: 30-min sync — partner pipeline, contract renewal tracker, regulatory-body engagement status
- Cross-team review: monthly GTM sync with all GTM teams; quarterly partner review with founder

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (partnership tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: N/A (commercial decisions are not ADRs)

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| KR ecosystem partner agreement delays regional-pack launch | High | 90-day renewal alert; founder escalation for partnership blockers |
| Trademark registration missed → brand infringement risk | Medium | 60-day trademark filing SLA; IP ledger tracking |
| Regulatory-body relationship gap → cold introduction at audit | Medium | Annual briefing cadence; relationship tracker in partner ledger |

## Sources scanned
PRD.md §12 (regional packs: KR ecosystem partners), DOC-CATALOG.md §2.3 (doc.vendor_partner_ledger owner = gtm-partnerships + ops-security; doc.legal_ip_ledger owner = gtm-partnerships + Founder).
