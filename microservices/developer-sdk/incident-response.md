---
doc_class: IncidentResponse
title: "Incident Response Playbook"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Incident Response Playbook


## Severity definitions

| Sev | Definition | Page target |
|---|---|---|
| Sev-1 | Cross-tenant data leak; financial loss > $10k; regulatory exposure | On-call + council-security + axis-ecosystem lead, ≤ 5 min |
| Sev-2 | Single-tenant impact; vetting backlog > 24h; budget burn > 50% | On-call, ≤ 15 min |
| Sev-3 | Single-budget breach; no tenant impact | Slack #axis-ecosystem-ops, ≤ 1h |

## On-call rota

- Primary: axis-ecosystem lead.
- Secondary: ops-sre-reliability.
- Tertiary: council-security (Sev-1 only).

## Communication

- Customer-facing: status.oyatie.dev banner within 5 min for Sev-1, 15 min for Sev-2.
- Developer-facing: developer portal banner within 5 min.
- Council: out-of-band Telegram + email.

## Post-incident

1. Post-mortem doc within 5 business days.
2. Action items tracked in IP backlog.
3. SLO review if budget overshoot ≥ 50%.
4. Runbook update.

