---
doc_status: published
---

# Team: GTM — Sales & Solutions Engineering

## Mission
This team owns the outbound commercial motion and technical sales cycle: enterprise sales, solutions engineering (SE) for design-partner pilots and enterprise RFPs, and the GTM plan that maps Oyatie's axis/vertical surface to customer segments. It exists to convert Oyatie's cohesion thesis from an architectural argument into a commercial reality — starting with KR Group anchor and the first three design-partner groups. It does **not** own product pricing strategy (→ founder), customer success post-close (→ `gtm-customer-success`), or partner-ecosystem development (→ `gtm-partnerships`).

## Owned axes / surfaces / contracts
- **Axis(es):** GTM — Sales
- **Surfaces:**
  - GTM plan (`GTM-PLAN.md`) — pricing coverage, segment targeting, wave-gate commercial readiness
  - Sales pipeline (CRM tooling — Salesforce or equivalent)
  - Demo environment (maintained as a shared resource across Sales + SE + CS)
  - RFP / RFI response library
  - SE technical playbooks: per-vertical, per-axis technical evaluation guides
  - Commercial pricing calculator (cost input from `ops-finops`)
- **Cross-axis contracts:** none owned (consumer of all axes' demo environments)
- **Catalog records:** none (GTM tooling is SaaS; no product crates)
- **Runbooks:** `runbooks/design-partner-onboarding.md`, `runbooks/demo-environment-reset.md`
- **ADRs:** none (commercial decisions via founder; ADRs are engineering decisions)

## In-scope work
- Enterprise sales: outbound prospecting, qualification, proposal, commercial negotiation for KR Group anchor + design-partner groups (≥ 3 KR Group payroll tenants per PRD §4.1)
- Solutions engineering: technical evaluation support, PoC scoping, RFP response, architecture review for prospects
- Demo environment: maintain multi-axis demo tenant; reset cadence; per-vertical demo workflows
- Pilot scoping: scope W-Vertical-Pilot design-partner engagements; coordinate with `vertical-corporate` on pilot tenant onboarding
- GTM plan authorship: pricing coverage, segment definitions, wave-gate commercial readiness gates
- RFP / RFI library: per-vertical, per-axis technical response templates
- Revenue pipeline reporting to founder
- Commercial pricing calculator (works with `ops-finops` unit-economics data)
- KR market development: KR Group relationship stewardship, KR enterprise prospect outreach

## Out-of-scope (anti-scope)
- Product roadmap decisions (→ founder + council-architecture)
- Customer success post-close (→ `gtm-customer-success`)
- Partner ecosystem (→ `gtm-partnerships`)
- Marketing and brand (→ `gtm-marketing`)
- Pricing strategy decisions (→ founder; Sales provides market input)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `vertical-corporate` | Design-partner pilot readiness (first 3 KR Group tenants) | Per pilot sprint |
| `ops-finops` | Per-tenant unit economics for commercial pricing calculator | Monthly |
| `ops-compliance` | Compliance posture summary for enterprise sales (RFP responses) | Per RFP |
| `gtm-marketing` | Brand materials, trust portal link, product collateral | Monthly |
| `gtm-customer-success` | Customer health data for expansion conversations | Monthly |
| `platform-api-sdk` | Developer documentation for SE technical evaluations | Per evaluation |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-customer-success` | Closed deals and pilot commitments for CS handover | Per close |
| `council-architecture` | Commercial signal input for vertical sequencing decisions | Quarterly |
| `gtm-marketing` | Win/loss insights for content strategy | Monthly |

## Success metrics
- **KR Group payroll design-partner tenants closed:** ≥ 3 (PRD §4.1 target)
- **Sales cycle length (enterprise SaaS):** tracked; target ≤ 90 days for KR SME, ≤ 180 days for enterprise
- **SE technical evaluation win rate:** ≥ 60%
- **Demo environment uptime:** ≥ 99.5%
- **RFP response turnaround:** ≤ 5 business days
- **Pipeline coverage ratio:** ≥ 3× quarterly revenue target

## Escalation path
- Internal: tech lead → team manager
- Cross-team: founder for commercial terms outside delegated authority
- Product: architecture council for product-gap discovery in sales cycle
- Legal: `gtm-partnerships` + counsel for contract terms

## Communication cadence
- Stand-up: daily async (pipeline review)
- Weekly: 60-min sync — pipeline review, pilot status, demo environment health
- Cross-team review: monthly GTM sync with `gtm-marketing`, `gtm-customer-success`, `gtm-partnerships`; quarterly commercial review with founder

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (SE tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: N/A (commercial decisions are not ADRs)

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Design-partner pilot delayed → PRD §4.1 metric missed | High | Weekly pilot status check; founder escalation |
| Demo environment outage during prospect evaluation | Medium | ≥ 99.5% uptime target; monitored by `ops-sre-reliability` |
| RFP win on features not yet built | High | SE must validate against current wave gate before committing in RFP |

## Sources scanned
PRD.md §2 (user classes: tenant operator, external developer), §4.1 (design-partner target), DOC-CATALOG.md §2.1 (doc.gtm_plan owner = gtm-sales-se).
