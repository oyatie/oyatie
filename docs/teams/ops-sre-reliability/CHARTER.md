---
doc_status: published
---

# Team: Ops — SRE & Reliability

## Mission
This team owns site reliability engineering across all Oyatie axes: SLO catalog authorship and burn-rate gating, runbook registry, on-call rotation management, incident management (Sev-1/2 lifecycle), and the release management lane. It exists to ensure that every axis surface has a declared SLO, a tested runbook, and an on-call owner — and that no release ships without meeting burn-rate gates. It does **not** own per-axis business logic, the audit chain, or security program; it owns the reliability posture and the incident-response scaffolding that every axis team plugs into.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting operations
- **Surfaces:**
  - SLO catalog (`SLO-CATALOG.md`) — every surface must declare an SLO here
  - Runbook registry (`RUNBOOKS-INDEX.md`) — every runbook registered and discoverable
  - Incident management process (`INCIDENT-MANAGEMENT.md`) — Sev-1/2 lifecycle, postmortem trigger
  - Release management lane (`RELEASE-MANAGEMENT.md`) — burn-rate gate, rollout strategy, CI lane integration
  - On-call rotation schedules (all axes)
  - Error-budget dashboards (per-surface)
  - PagerDuty / alerting configuration
- **Cross-axis contracts (DESIGN §10):** (consumer of all surfaces' SLO declarations; no contract ownership)
- **Catalog records:** SRE tooling crates (under `governance-*` in coordination with `axis-foundry`)
- **Runbooks:** `runbooks/sev1-incident-response.md`, `runbooks/error-budget-exhaustion.md`, `runbooks/on-call-handover.md`, `runbooks/release-rollback.md`
- **ADRs:** ADR-0050 (release management), ADR-0040 (launch readiness)

## In-scope work
- SLO authorship: work with each axis team to define and publish SLOs (availability, latency, error-rate, throughput)
- Error-budget burn-rate gates: block releases when error budget is exhausted; define burn-rate alert thresholds
- Runbook registry: every runbook must be indexed, linked from SLO catalog, tested quarterly
- On-call rotations: maintain schedules for all axes; primary + secondary for every surface
- Incident management: Sev-1/2 declaration, war-room facilitation, postmortem coordination (trigger only — postmortems owned by prevention-doctrine, not written as long-form docs per CLAUDE.md)
- Release management: coordinate multi-axis releases, rollout strategy (canary/ring), rollback gate
- DR drill coordination: quarterly non-prod failover drills per axis (execution owned by `ops-dr-capacity`)
- Availability reporting: weekly reliability report to leadership
- Chaos engineering (steady-state hypothesis + game-day facilitation)
- Capacity-alerting thresholds (worked with `ops-dr-capacity`)

## Out-of-scope (anti-scope)
- Per-axis architecture decisions (→ per-axis team + council-architecture)
- Security program (→ `ops-security`)
- Compliance matrix (→ `ops-compliance`)
- FinOps unit economics (→ `ops-finops`)
- DR planning and capacity modelling (→ `ops-dr-capacity`)
- Postmortem authorship (per prevention doctrine: fix the system, not write a postmortem)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| All axis teams | SLO declarations, runbook authorship, on-call participation | Continuous |
| `axis-foundry` | CI lane integration for release gates, repoctl `release-verify` | Per-release |
| `axis-cloud` | Cloud observability metrics (dashboards, tracing) | Per-release |
| `ops-dr-capacity` | DR drill schedule, capacity headroom alerts | Quarterly |
| `platform-audit-evidence` | Incident audit chain records | Per Sev-1/2 |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All axis + vertical teams | SLO targets, on-call coverage, runbook links, release gate | Continuous |
| `council-architecture` | Reliability posture summary for wave-gate readiness | Per wave |
| `ops-compliance` | Incident records for regulatory evidence | Per Sev-1/2 |
| `gtm-customer-success` | Tenant-visible uptime and reliability data | Monthly |

## Success metrics
- **SLO coverage of all production surfaces:** 100%
- **Runbook coverage of all SLO surfaces:** 100%
- **On-call gap (surface with no on-call owner):** 0
- **Sev-1 mean time to acknowledge (MTTA):** < 5 min
- **Sev-1 mean time to resolve (MTTR):** < 2 h
- **Release rollbacks caused by missing burn-rate gate:** 0
- **DR drill success rate:** 100% quarterly in non-prod (PRD §3.1 W-Cloud-Preview)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council for cross-axis reliability contract disputes
- Security: `ops-security` for security-class incidents
- Founder: as last resort (Sev-1 impacting KR Group design partner)

## Communication cadence
- Stand-up: daily async + on-call handover
- Weekly: 60-min sync — incident review, error-budget status, runbook audit
- Cross-team review: monthly SLO review with all axis leads; quarterly DR drill debrief

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; ADR-0040 (launch readiness) amendments at wave gates

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Surface goes to production without SLO declaration | High | SLO coverage fitness function; release gate blocks no-SLO surfaces |
| On-call gap leaves a surface unmonitored | High | Weekly on-call schedule audit; PagerDuty gap alerting |
| Release ship during error-budget exhaustion | High | Burn-rate gate blocks; SRE lead manual override required with audit record |
| Postmortem not actioned → repeat incident | Medium | Prevention doctrine: fix the system (CI gate / hook / validator); no long-form postmortem |

## Sources scanned
PRD.md §4.1 (uptime metrics), §4.2 (MTTR), DESIGN.md §9 (horizontal scale: region failover drill), DOC-CATALOG.md §2.2 (doc.runbooks_index, doc.slo_catalog, doc.incident_management owners), ADR-0050, ADR-0040.
