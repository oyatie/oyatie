# Team: GTM — Customer Success

## Mission
This team owns post-close customer success: design-partner program management, CS playbook, tenant health monitoring, expansion plays, and churn prevention. It exists because the design-partner program is not a sales motion — it is an active product co-development relationship, and the lessons from the first three KR Group tenants directly shape which wave surfaces ship next. It does **not** own commercial sales (→ `gtm-sales-se`), marketing (→ `gtm-marketing`), or product roadmap decisions (→ founder + council-architecture).

## Owned axes / surfaces / contracts
- **Axis(es):** GTM — Customer Success
- **Surfaces:**
  - CS playbook: onboarding, adoption, health scoring, expansion, renewal, churn-prevention
  - Design-partner program: structured feedback cadence, product co-development sessions, feature prioritization input
  - Tenant health dashboards (built on `axis-ads-analytics` analytics plane + per-vertical metrics)
  - NPS / CSAT survey program
  - Escalation management: customer-facing Sev-1/2 coordination with `ops-sre-reliability`
- **Cross-axis contracts:** none owned (consumer of all axes' tenant health metrics)
- **Catalog records:** none
- **Runbooks:** `runbooks/tenant-escalation-management.md`, `runbooks/design-partner-feedback-session.md`
- **ADRs:** none

## In-scope work
- Design-partner program: weekly cadence with each pilot tenant; structured feedback sessions; feature prioritization input to council-architecture
- Tenant onboarding: post-sales handover from `gtm-sales-se`; onboarding checklist; first-value milestone tracking
- Health scoring: composite score per tenant (adoption, SLO health, NPS, support ticket volume)
- Expansion: identify upsell/cross-sell opportunities within existing tenant base; hand to `gtm-sales-se` for commercial execution
- Churn prevention: early-warning triggers; intervention playbook; escalation to founder for design-partner churn risk
- Compliance trust conversations: provide compliance posture summaries (from `ops-compliance`) to enterprise tenant procurement/security teams
- Support tier definition: tiered support model (design-partner / enterprise / standard); escalation SLAs per tier
- Feedback loop to product: monthly synthesis of tenant feedback → structured input to council-architecture

## Out-of-scope (anti-scope)
- Commercial negotiations and pricing (→ `gtm-sales-se` + founder)
- Product roadmap ownership (→ founder + council-architecture; CS provides input)
- Technical implementation support (→ SE in `gtm-sales-se` for pre-close; `axis-*` teams for post-close technical issues)
- Marketing content (→ `gtm-marketing`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `gtm-sales-se` | Closed deal handover, pilot commitments | Per close |
| `ops-sre-reliability` | Tenant SLO health data, incident status during escalations | Continuous |
| All vertical teams | Per-vertical tenant health metrics (payroll runs, workflow completion, etc.) | Monthly |
| `ops-compliance` | Compliance posture summaries for tenant trust conversations | Per request |
| `axis-ads-analytics` | Analytics plane data for tenant health dashboards | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `council-architecture` | Design-partner feedback synthesis for product decisions | Monthly |
| `gtm-sales-se` | Expansion signals and health data for renewal conversations | Monthly |
| `gtm-marketing` | Customer references, case study candidates | Quarterly |

## Success metrics
- **Design-partner NPS:** ≥ 50 (target; measured quarterly)
- **Tenant first-value milestone:** ≤ 30 days post-onboarding (first workflow live)
- **Design-partner churn:** 0 (any churn is a product signal requiring council review)
- **Customer escalation response SLA (design-partner Sev-1):** < 30 min to CS acknowledgment
- **Monthly feedback synthesis delivered to council-architecture:** 100%
- **Tenant health score coverage:** 100% of production tenants scored monthly

## Escalation path
- Internal: tech lead → team manager
- Cross-team: `ops-sre-reliability` for technical escalations; `ops-compliance` for regulatory queries
- Founder: design-partner churn risk, relationship-level escalations
- Legal: `gtm-partnerships` + counsel for contract disputes

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — health dashboard review, design-partner cadence, escalation queue
- Cross-team review: monthly GTM sync with `gtm-sales-se`, `gtm-marketing`, `gtm-partnerships`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (CS tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: N/A

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Design-partner churns before W-Vertical-Pilot gate | High | Weekly health check; churn-risk escalation to founder |
| Feedback synthesis not delivered to council → product drift | Medium | Monthly synthesis is a team OKR; council-architecture consumes it |
| Tenant escalation mis-routed → SLA miss | Medium | Escalation runbook with clear routing; CS on-call for design partners |

## Sources scanned
PRD.md §4.1 (design-partner target), §2 (tenant operator persona), DOC-CATALOG.md §2.1 (doc.gtm_plan).
