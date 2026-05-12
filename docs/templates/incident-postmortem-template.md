# Postmortem: Incident <id> — <one-line title>

> **Status:** draft / in-review / published
> **Severity:** Sev 1 | Sev 2 | Sev 3 | Sev 4
> **Detected:** YYYY-MM-DD HH:MM UTC
> **Resolved:** YYYY-MM-DD HH:MM UTC
> **Duration:** <hours>
> **Incident Manager:** <name / agent>
> **Affected surfaces:** <list per axis>
> **Affected tenants:** <count + class summary; never PII>
> **Affected regions / packs:** <list>
> **Regulatory notification window started:** <YYYY-MM-DD HH:MM>

> **This postmortem is BLAMELESS.** Names of individuals appear only in the IM/CM role record below; root-cause analysis names systems, processes, and contracts — never people.

---

## 1. Summary

One paragraph, 5-Whys-grade clarity. What broke + what customers experienced + how it was contained + how it was resolved.

## 2. Timeline (UTC)

| Time | Event | Source |
|---|---|---|
| HH:MM | (signal received) | (alert / customer / synthetic / security event) |
| HH:MM | Sev declared | IM |
| HH:MM | Bridge opened | IM |
| HH:MM | SME paged | IM |
| HH:MM | Comms manager paged | IM (Sev 1) |
| HH:MM | Customer notification sent | CM |
| HH:MM | Mitigation step 1: ... | SME |
| HH:MM | Mitigation step 2: ... | SME |
| HH:MM | Customer impact stopped | SME |
| HH:MM | Root cause identified | SME |
| HH:MM | Fix deployed | SME |
| HH:MM | SLO budget returned to within range | observability |
| HH:MM | Bridge closed | IM |

(Reconstructed from audit chain + on-call notes per ADR-0003.)

## 3. Impact

| Axis | Surface | Tenants affected | Data classes touched | Regulatory window |
|---|---|---|---|---|
| (axis) | (surface) | (count + class summary) | (per ADR-0008 class) | (per regulator) |

## 4. Root cause(s)

5-Whys / Causal-Tree:

1. (immediate technical cause)
2. (why that happened)
3. (why that)
4. (why that)
5. (root system / process / contract gap)

## 5. What went well

- Bullet list (e.g. detection time was X min; runbook had been drilled in last quarter; per-cell containment limited blast radius)

## 6. What went poorly

- Bullet list (e.g. on-call missed page for X min; runbook step Y was outdated; cross-axis contract review hadn't covered this case)

## 7. Action items (with owners + ETAs)

| # | Action | Owner team | Type | ETA |
|---|---|---|---|---|
| 1 | (mechanical prevention per [docs/standards/prevention-doctrine.md](../standards/prevention-doctrine.md)) | (team) | mechanical | <date or wave-gate> |
| 2 | (process improvement) | (team) | process | <date> |
| 3 | (runbook update / new runbook) | ops-sre-reliability | docs | <date> |
| 4 | (mistakes-and-fixes-ledger entry) | council-architecture | docs | <date> |

> Per [INCIDENT-MANAGEMENT.md §3.6](../INCIDENT-MANAGEMENT.md), prevention items must be **mechanical** (CI gate / hook / validator / test / config-as-code), not process-only. Sev 1 mechanical fix shipped within 30d; Sev 2 within 60d.

## 8. Trust portal

- ☐ Incident page live during incident at `trust.oyatie.com/incidents/<id>`
- ☐ Postmortem published within 30d (Sev 1) / 60d (Sev 2)
- ☐ Customer-facing summary excludes any tenant PII

## 9. Regulatory notification record

| Regulator | Notification deadline | Notification sent | Artifact |
|---|---|---|---|
| (e.g. KR PIPC) | (PIPA Art 34 / 72h) | YYYY-MM-DD HH:MM | (link / hash) |
| (e.g. EU SA) | (GDPR Art 33 / 72h) | YYYY-MM-DD HH:MM | (link / hash) |
| (e.g. HHS) | (HIPAA / 60d) | YYYY-MM-DD | (link / hash) |
| (e.g. PCI acquirer) | (per acquirer SLA) | YYYY-MM-DD | (link / hash) |

## 10. Audit-chain reference

- Per [ADR-0003 audit chain](../decisions/ADR-0003-audit-chain-and-evidence-emission.md): incident-class events emitted at every step
- Audit-chain shard for affected tenants: per-tenant per-cell shard
- Replay: `oya admin incident replay <id>`

## 11. Sign-off

- ☐ IM: <name / agent>
- ☐ Affected SMEs: <names>
- ☐ Privacy lead (if data-class touched): <name>
- ☐ Security lead (if security-class): <name>
- ☐ Founder (if Sev 1): <name>
- ☐ Council co-sign for trust-portal publish

## 12. Sources scanned
- Audit-chain replay
- Bridge transcript
- Per-runbook execution log
- Per-affected-surface SLO observability
- Per-regulator notification artifacts
