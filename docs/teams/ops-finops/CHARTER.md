---
doc_status: published
---

# Team: Ops — FinOps

## Mission
This team owns Oyatie's FinOps practice: per-tenant unit economics, cost-anomaly detection, cloud spend optimization, and the FinOps plan that bridges engineering decisions to financial outcomes. It exists because PRD §3.1 mandates "FinOps unit economics from day one" and because the cohesion thesis — one cloud, one billing trail — creates the opportunity to deliver per-tenant, per-axis, per-resource cost visibility that fragmented multi-vendor stacks cannot match. It does **not** own cloud billing infrastructure (→ `axis-cloud`), SaaS metering (→ `axis-saas`), or the DR capacity model (→ `ops-dr-capacity`).

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting operations
- **Surfaces:**
  - FinOps plan (`FINOPS-PLAN.md`) — margin coverage, unit economics, spend optimization
  - Per-tenant unit economics dashboards (built on `axis-cloud` billing events + `axis-saas` metering events)
  - Cost-anomaly detection pipeline: per-tenant, per-axis, per-resource anomaly alerts
  - Cloud spend optimization recommendations: rightsizing, reserved-instance coverage, spot usage
  - Per-product gross margin model: tracks contribution margin per axis surface
- **Cross-axis contracts (DESIGN §10):**
  - `Billing event` (consumer — reads cloud billing events and SaaS metering events to compute unit economics)
- **Catalog records:** FinOps tooling scripts (no product crates)
- **Runbooks:** `runbooks/cost-anomaly-response.md`, `runbooks/finops-monthly-close.md`
- **ADRs:** none owned directly; FinOps plan is a Tier-3 doc per DOC-CATALOG.md

## In-scope work
- Per-tenant unit economics: COGS per tenant, gross margin per tenant, per-axis attribution
- Per-axis cost attribution: cloud compute, storage, network, Kafka, search index, Foundry agent runs — each attributed to the consuming axis and tenant
- Cost-anomaly detection: statistical anomaly detection on per-tenant spend; alert when spend deviates > 2σ from 30-day baseline
- Cloud spend optimization: rightsizing recommendations, reserved-instance coverage analysis, spot-usage opportunities, idle-resource detection
- Budget vs actual tracking: monthly variance report; escalate > 10% drift (DOC-CATALOG.md `doc.finops_plan` trigger)
- FinOps plan authorship and monthly maintenance (`FINOPS-PLAN.md`)
- Showback / chargeback tooling: per-tenant cost report exportable for design-partner billing verification
- Coordination with `ops-dr-capacity` on capacity scaling cost impact

## Out-of-scope (anti-scope)
- Cloud billing infrastructure (→ `axis-cloud` owns `cloud-billing-kernel`)
- SaaS metering event emission (→ `axis-saas` owns `platform-metering-kernel`)
- Product pricing decisions (→ founder + GTM; FinOps provides cost input)
- Financial accounting / GAAP reporting (→ finance function outside engineering)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-cloud` | Cloud billing events from `cloud-billing-kernel` | Daily |
| `axis-saas` | SaaS metering events from `platform-metering-kernel` | Daily |
| `axis-ads-analytics` | Ad spend analytics data for FinOps attribution | Monthly |
| `ops-dr-capacity` | Capacity scaling cost impact for headroom recommendations | Monthly |
| `ops-sre-reliability` | SLO context for cost-vs-reliability tradeoff decisions | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-cloud` | Rightsizing and reserved-instance recommendations | Monthly |
| `gtm-sales-se` | Per-tenant unit economics for commercial proposals | Per deal |
| `gtm-customer-success` | Per-tenant cost health for design-partner reporting | Monthly |
| `council-architecture` | Gross margin model for wave-gate financial readiness | Per wave |
| All axis teams | Per-axis cost attribution for engineering budget decisions | Monthly |

## Success metrics
- **FinOps plan freshness:** updated within 30 days of any ≥ 10% cost drift (DOC-CATALOG.md trigger)
- **Per-tenant unit economics coverage:** 100% of production tenants
- **Cost-anomaly alert false-positive rate:** < 5% per month
- **Cloud spend vs budget variance:** < 10% monthly (escalation trigger at > 10%)
- **Reserved-instance coverage:** ≥ 70% of baseline compute spend
- **Idle-resource elimination:** 100% of detected idle resources actioned within 7 days

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for cost-vs-architecture tradeoff disputes
- Founder: as last resort (budget overrun events)

## Communication cadence
- Stand-up: async (FinOps is primarily a reporting function; no daily stand-up required)
- Weekly: 30-min sync — cost-anomaly review, optimization pipeline
- Cross-team review: monthly FinOps report to all axis leads + founder

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules (FinOps tooling PRs)
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Per-tenant cost attribution gaps → incorrect unit economics | Medium | 100% metering event coverage gate; monthly reconciliation |
| Cost anomaly undetected → budget overrun | Medium | 2σ anomaly alert; PagerDuty escalation on > 3σ |
| FinOps plan not updated after ≥ 10% cost drift | Medium | DOC-CATALOG.md automated trigger; monthly close process |

## Sources scanned
PRD.md §3.1 (optimization built in, FinOps unit economics from day one), DESIGN.md §10 (billing event row), DOC-CATALOG.md §2.3 (doc.finops_plan owner = ops-finops).
