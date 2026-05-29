---
doc_class: Dashboard-Reference
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/runbooks/oncall-handoff-failure.md
  - microservices/ops-dashboard-control-center/policy/cedar/on-call-handoff-authorization.cedar
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# On-Call Handoff Panel — Reference

The on-call handoff panel is a purpose-built sub-panel within the ops-overview dashboard (`dashboards/ops-overview.json`), accessible via keyboard shortcut **G→H** (handoff).

## Panel contents

| Widget | Data source | Refresh |
|---|---|---|
| Current on-call engineer (incoming + outgoing) | `oya_ops_control_center_oncall_rotation_current` | 30s |
| Open incidents count + severity | `oya_ops_control_center_incidents_active_total` by severity | 15s |
| Pending deployments requiring approval | `oya_ops_control_center_deployments_pending_approval_total` | 30s |
| Last handoff acknowledged (time since) | `oya_ops_control_center_oncall_handoff_ack_age_seconds` | 30s |
| Handoff notes (last 3 entries) | Audit stream query | On-demand |
| Severity context (GREEN/YELLOW/RED) | `oya_ops_control_center_oncall_severity_context` | 15s |

## UX design

- **Glanceable header row**: severity context (RED/YELLOW/GREEN badge), current on-call name, open incident count — visible without scrolling.
- **Keyboard shortcut**: from any ops-dashboard panel, press **H** to jump to handoff view.
- **Dark-mode default**: high-contrast severity badges (red #FF4040, yellow #FFB800, green #52C41A on dark #1A1A2E background).
- **WCAG 2.2 AA**: severity badges use both colour AND icon (●/▲/✓) to avoid colour-only signalling.
- **Static fallback**: panel embeds direct Grafana link (no ops-dashboard auth required) for use when ops-dashboard is unavailable — per `runbooks/oncall-handoff-failure.md §static-fallback`.

## Prometheus queries

```promql
# Last handoff ack age
oya_ops_control_center_oncall_handoff_ack_age_seconds

# Open incidents by severity
sum by (severity) (oya_ops_control_center_incidents_active_total)

# Pending deployment approvals
oya_ops_control_center_deployments_pending_approval_total
```

## Audit events emitted

- `OnCallHandoffCreated` — on new handoff record.
- `OnCallHandoffAcknowledged` — on ack.
- `OnCallHandoffEscalatedToIncident` — on escalation.

All sealed per ADR-0263 + ADR-0028 Merkle chain.
