---
doc_status: published
---

# Team: Vertical — Corporate (HR / Payroll / GL / Mail / Comms)

## Mission
This team owns the corporate vertical product: HR, payroll, general ledger, mail, and communications for global enterprise tenants with the KR Group anchor as the design-partner pilot. It exists because corporate business operations are the most common entry point for new enterprise tenants and provide the widest surface to prove Oyatie's cohesion thesis across identity, workflow, search, and compliance. It is the most likely candidate for W-Vertical-Pilot designation. It does **not** own the underlying SaaS workflow engine, cloud infrastructure, or cross-vertical regulatory machinery — it owns the corporate-domain entities, workflows, and regulatory compliance artifacts.

## Owned axes / surfaces / contracts
- **Axis(es):** Vertical industry cloud — Corporate (Axis 2 sub-axis)
- **Surfaces:**
  - `vertical-corporate-kernel` — `Employee`, `PayrollRun`, `GLEntry`, `OrgUnit`, `MailMessage`, `Comms`
  - `vertical-corporate-domain-*` — HR lifecycle, payroll calculation, GL journal, mail routing use-cases
  - `vertical-corporate-adapter-*` — KR 근로기준법 payroll adapter, K-IFRS GL adapter, 국민연금/건강보험 EDI
  - Per-region extensions: `pack-kr` → `KrPayrollExtension` (통상임금, 사대보험), JP → `JpPayrollExtension` (賞与, 社保), US → `UsPayrollExtension` (W-2/1099, FLSA)
  - Products owned: `products/vertical-corporate/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Object Graph property tier` (consumer) — Employee, OrgUnit nodes in OG
  - `Audit-chain event` (emitter) — HR lifecycle events, payroll runs, GL entries
  - `DSR / consent withdrawal cascade` (ack required) — employee PII in search index
- **Catalog records:** `crates/vertical-corporate-*`
- **Runbooks:** `runbooks/payroll-run-failure.md`, `runbooks/gl-reconciliation.md`, `runbooks/employee-dsr-cascade.md`
- **ADRs:** ADR-0033 (HR domain), ADR-0033 (payroll schema), ADR-0033 (GL schema), ADR-0050 (wave plan — corporate sections)

## In-scope work
- HR lifecycle: hire, transfer, promote, terminate, org-unit management, headcount reporting
- Payroll: KR 통상임금 calculation, 사대보험 (국민연금, 건강보험, 고용보험, 산재보험) deductions, tax withholding, pay stub generation
- General ledger: journal entry, chart of accounts (K-IFRS primary; US-GAAP/J-GAAP via regional pack), period close, trial balance, financial reporting
- Mail: internal corporate mail, domain-email routing, compliance archival
- Comms: internal announcements, notification routing, integration with external comms (Slack, Teams) via Connect
- KR 전자세금계산서 (e-tax invoice) generation and submission
- Design-partner pilot: first 3 KR Group tenant onboardings, feedback loop into PRD
- Regional pack seam implementations for KR, JP, US payroll/GL

## Out-of-scope (anti-scope)
- Clinical, financial-services, or industrial domain logic (→ respective vertical teams)
- SaaS workflow engine (→ `axis-saas` — corporate uses the engine)
- Cloud infrastructure (→ `axis-cloud`)
- Cross-vertical HR aggregation or benchmarking using inter-tenant data (→ Data Use Boundary prohibits)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `axis-saas` | Workflow engine, OG for Employee/OrgUnit nodes, plugin substrate | Per-release |
| `platform-tenancy-identity` | Tenant/identity kernel for employee identity | Per-release |
| `platform-privacy-dub` | Employee PII data class classification | ADR lifecycle |
| `platform-audit-evidence` | Audit chain emission for payroll runs, HR events | Per-release |
| `axis-foundry` | Foundry capability invocation for agent-assisted payroll, GL close | Wave gate |
| `axis-cloud` | Tenant residency + compute cells | Wave gate |
| `ops-compliance` | KR 근로기준법, 국민연금법 regulatory change watch | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `gtm-customer-success` | Design-partner health dashboard, payroll run metrics | Weekly during pilot |
| `gtm-sales-se` | Demo environment, payroll + GL demo workflows | Monthly |
| `council-architecture` | Pilot learnings to inform W-Vertical-Fan-Out sequencing | Post-pilot |

## Success metrics
- **KR Group payroll tenants live:** ≥ 3 design-partner groups (PRD §4.1)
- **Payroll run end-to-end (submit → pay-stub):** p99 < 30 min
- **GL period close agent-assisted automation rate:** ≥ 50% of close steps agent-executed (W-Vertical-Pilot target)
- **KR 사대보험 deduction accuracy:** 100% match to NHIS published tables
- **Employee DSR cascade completion:** 100% within 72 h
- **Audit chain completeness for payroll runs:** 100%

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for OG contract changes involving Employee/OrgUnit
- Compliance: `ops-compliance` for KR labor-law regulatory changes
- Founder: as last resort (KR Group relationship decisions)

## Communication cadence
- Stand-up: daily async; weekly during design-partner pilot sprint
- Weekly: 60-min sync during pilot; 30-min sync otherwise
- Cross-team review: monthly cross-axis review; quarterly KR regulatory watch with `ops-compliance`

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; KR statutory changes trigger immediate ADR

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| KR 통상임금 calculation bug causes payroll error | High | 100% accuracy gate against NHIS tables; audit chain on every payroll run |
| Design-partner pilot delays W-Vertical-Pilot gate | High | Weekly progress check; founder escalation path |
| Employee PII leaked into cross-tenant search index | Catastrophic | Data Use Boundary classification of Employee PII; fitness gate |

## Sources scanned
PRD.md §3.1 (W-Vertical-Pilot), §4.1 (design-partner metric), DESIGN.md §10, ADR-0033, ADR-0033, ADR-0033, ADR-0050, DOC-CATALOG.md §2.5, products/vertical-corporate/PRD.md (draft).
