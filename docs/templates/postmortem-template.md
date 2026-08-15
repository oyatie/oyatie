---
doc_class: Template
template_id: TPL-PM
status: Accepted
date: 2026-05-12
purpose: |
  Google-SRE blameless postmortem. Mandatory for every Sev-1/Sev-2. Names systems, processes, and contracts; never people in root-cause analysis. Includes regulator notification matrix and machine-readable action-item table.
supersedes: docs/templates/incident-postmortem-template.md
enforcing_fitness_lane: oya-governance-postmortem-shape
owner_team: ops-sre-reliability
related:
  - docs/INCIDENT-MANAGEMENT.md
  - docs/MISTAKES-LEDGER.md
  - docs/RISK-REGISTER.md
  - docs/standards/prevention-doctrine.md
  - docs/templates/runbook-template-v2.md
  - docs/templates/mistakes-ledger-row-template.md
adrs_cited:
  - ADR-0052  # inventory / audit chain (replay command)
  - ADR-0053  # sanctioned primitives (agent path in timeline)
doc_status: published
---

```yaml
# Required frontmatter
---
doc_class: Postmortem
template_id: TPL-PM
incident_id: INC-YYYY-NNNN
title: "<imperative one-line>"
status: draft | in-review | published
severity: Sev-1 | Sev-2 | Sev-3 | Sev-4
detected_at: YYYY-MM-DDTHH:MMZ
resolved_at: YYYY-MM-DDTHH:MMZ
duration_minutes: NNN
blameless: true
incident_manager: <role | agent-id>
affected_axes: [axis-..., ...]
affected_regions: [<region/pack>]
regulator_notification_window_started_at: YYYY-MM-DDTHH:MMZ | null
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# Postmortem INC-YYYY-NNNN — <one-line title>

> **This postmortem is BLAMELESS.** Names of individuals appear only in the IM/CM role record below. Root-cause analysis names systems, processes, and contracts — never people. (Google-SRE doctrine; CONSTITUTION §Decision principles Do.8.)

## 1. Summary

One paragraph, 5-Whys-grade clarity. What broke + what customers experienced + how it was contained + how it was resolved.

## 2. Timeline (UTC)

| Time | Event | Source |
|---|---|---|
| HH:MM | Signal received | alert / customer / synthetic / security event |
| HH:MM | Sev declared | IM |
| HH:MM | Bridge opened | IM |
| HH:MM | SME paged | IM |
| HH:MM | Comms manager paged | IM (Sev-1) |
| HH:MM | Customer notification sent | CM |
| HH:MM | Mitigation step 1: ... | SME |
| HH:MM | Mitigation step 2: ... | SME |
| HH:MM | Customer impact stopped | SME |
| HH:MM | Root cause identified | SME |
| HH:MM | Fix deployed | SME |
| HH:MM | SLO returned to within budget | observability |
| HH:MM | Bridge closed | IM |

Reconstructed from audit chain (ADR-0003) + on-call notes.

## 3. Impact

| Axis | Surface | Tenants affected (count + class) | Data classes touched | Regulatory window started |
|---|---|---|---|---|

## 4. Root cause analysis (5-Whys / Causal Tree)

1. Immediate technical cause.
2. Why that happened.
3. Why that.
4. Why that.
5. Root system / process / contract gap.

## 5. Contributing factors

- Bullet list. Each row names a *system*, *process*, or *contract*. Names of individuals are forbidden here.

## 6. What went well

- Bullet list (e.g., detection time was X min; runbook had been drilled last quarter; per-cell containment limited blast radius).

## 7. What went poorly

- Bullet list (e.g., on-call missed page for X min; runbook step Y was outdated; cross-axis contract review hadn't covered this case).

## 8. Action items

| # | Action | Owner team | Type | Due | Tracking |
|---|---|---|---|---|---|
| 1 | Authoritative mechanical prevention (blocking CI lane / validator / runtime gate; optional local hook only with that backstop) per `docs/standards/prevention-doctrine.md` | <team> | mechanical | <date or wave-gate> | PR# / IP-NNN |
| 2 | Process improvement (training / runbook drill / on-call rota) | <team> | process | <date> | issue# |
| 3 | Runbook update / new runbook authored | ops-sre-reliability | docs | <date> | runbook path |
| 4 | `docs/MISTAKES-LEDGER.md` row (`MFL-NNNN`) | council-architecture | docs | <date> | MFL-NNNN |
| 5 | `docs/RISK-REGISTER.md` row update | council-architecture | docs | <date> | RM-NN |

> Prevention items **MUST** be authoritative mechanical enforcement (blocking CI gate / validator / runtime test / config-as-code), not process-only. An optional local hook alone does not qualify. Sev-1 mechanical fix ships within 30 days; Sev-2 within 60 days (per `docs/INCIDENT-MANAGEMENT.md §3.6`).

## 9. Trust portal

- [ ] Incident page live during incident at `trust.oyatie.com/incidents/<id>`.
- [ ] Postmortem published within 30d (Sev-1) / 60d (Sev-2).
- [ ] Customer-facing summary excludes any tenant PII.

## 10. Regulatory notification record

| Regulator | Obligation | Notification deadline | Notification sent | Artifact |
|---|---|---|---|---|
| KR-PIPC | PIPA Art 34 (72h) | YYYY-MM-DDTHH:MMZ | YYYY-MM-DDTHH:MMZ | <link / hash> |
| EU-SA | GDPR Art 33 (72h) | YYYY-MM-DDTHH:MMZ | YYYY-MM-DDTHH:MMZ | <link / hash> |
| US-HHS | HIPAA (60d) | YYYY-MM-DD | YYYY-MM-DD | <link / hash> |
| PCI-acquirer | per acquirer SLA | YYYY-MM-DD | YYYY-MM-DD | <link / hash> |

## 11. Audit-chain reference

- Per ADR-0003: incident-class events emitted at every timeline row.
- Audit-chain shard for affected tenants: per-tenant per-cell shard. Inventory entry per ADR-0052.
- Replay command: `oya admin incident replay <id>` → produces an evidence bundle conforming to `docs/templates/evidence-bundle-template.json`.

## 12. Sign-off

- [ ] IM: <role / agent-id>
- [ ] Affected SMEs: <roles>
- [ ] Privacy lead (if data-class touched): <role>
- [ ] Security lead (if security-class): <role>
- [ ] Founder (if Sev-1): <role>
- [ ] Council co-sign for trust-portal publish.

## 13. Sources scanned

- Audit-chain replay.
- Bridge transcript.
- Per-runbook execution log.
- Per-affected-surface SLO observability dashboards.
- Per-regulator notification artifacts.
