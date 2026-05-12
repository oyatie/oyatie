# Oyatie — On-Call Standard

> **Owner:** `ops-sre-reliability`.
> **Companion:** [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [`checklists/incident-response.md`](../checklists/incident-response.md), [SLO-CATALOG.md](../SLO-CATALOG.md), [RUNBOOKS-INDEX.md](../RUNBOOKS-INDEX.md).

## 1. Rotation per axis

| Axis | Primary IM rotation | SME rotation per surface |
|---|---|---|
| SaaS | `axis-saas` lead + 1 senior | per-surface owners |
| Workspace | `axis-workspace` lead + 1 per-surface | per-surface (mail / doc / drive / meet) |
| Vertical (each) | per-vertical lead + 1 senior | per-vertical regulated-team member |
| Foundry | `axis-foundry` lead + 1 senior | per-capability domain owner |
| Cloud | `axis-cloud` lead + 1 senior | per-cell owner / per-region owner |
| Search | `axis-search` lead + 1 senior | per-component owner |
| Ads + Analytics | `axis-ads-analytics` lead + 1 senior | per-component owner |

## 2. Per-rotation duties

- Carry phone / pager 24×7 during rotation week
- Acknowledge alerts within 5min Sev 1; 15min Sev 2; 1h Sev 3
- Open bridge per [INCIDENT-MANAGEMENT.md §3](../INCIDENT-MANAGEMENT.md)
- Drive runbook per [RUNBOOKS-INDEX.md](../RUNBOOKS-INDEX.md)
- Decide Sev escalation
- Page CM (Comms Manager) for Sev 1
- Page Privacy Lead if data-class touched
- Page Security Lead if security-class
- Page Founder for Sev 1
- Bridge until resolved
- Postmortem ownership for closed incidents

## 3. Comms cadence (during incident)

- Sev 1: every 30min until resolved
- Sev 2: every 60min until resolved
- Sev 3: ad-hoc; daily summary
- Sev 4: ticket-only

## 4. Tools

- Pager: TBD (PagerDuty / Opsgenie / in-house Foundry capability)
- Bridge: voice + chat (e.g. Workspace Meet + Workspace Chat)
- Status page: `status.oyatie.com`
- Trust portal: `trust.oyatie.com/incidents/<id>`
- Runbooks: `oya ops runbook get <id>`
- Audit-chain replay: `oya admin incident replay <id>`

## 5. Burnout prevention

- Max 1 week per 4-week rotation per individual
- Compensatory time off after Sev 1
- Per-quarter rotation review by team lead
- Per-rotation handoff document

## 6. Drills

- Sev 1 game-day quarterly per axis
- Sev 1 game-day annually cross-axis (regulator-notification simulation)
- Region failover drill quarterly per region
- DR + tenant-restore drill quarterly per axis

## 7. Sources
Google SRE workbook (on-call chapters), [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), ADR-0040 + ADR-0042.
