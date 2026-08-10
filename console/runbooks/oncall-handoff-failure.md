---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0248
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/incident-response.md
  - console/policy/cedar/on-call-handoff-authorization.cedar
  - console/runbooks/admin-action-rollback.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: On-Call Handoff Failure

## A — Trigger conditions

- `OnCallHandoffCreated` event NOT emitted within 5 min of scheduled handoff window.
- Incoming on-call engineer unable to access handoff panel (`GET /ops/v1/oncall/handoffs/current` returns `404` or `503`).
- On-call rotation schedule diverges from system state (PagerDuty shows different responder than ops-dashboard).
- EMERGENCY_SERVICES page received but automated handoff failed to create.
- Cell outage: ops-dashboard-control-center unavailable in home-region cell.

## B — Pre-checks

1. **[≤30s]** Check ops-dashboard availability: `GET /ops/v1/health` → expect `200 OK`.
   If `503`: control plane is down — proceed to Static Fallback (step C-Emergency).
2. **[≤30s]** Check current on-call handoff state: `GET /ops/v1/oncall/handoffs/current`.
3. **[≤30s]** Check PagerDuty/OpsGenie primary on-call schedule for current window.
4. **[≤30s]** Verify outgoing on-call engineer is reachable via direct channel.

## C — Procedure

### Normal path — handoff panel available

1. **[≤2min]** Outgoing engineer creates handoff manually:
   ```
   POST /ops/v1/oncall/handoffs
   Headers: X-Idempotency-Key: <uuid4>  X-Step-Up-Token: <T2_token>
   Body: {
     "incoming_operator_id": "<id>",
     "open_incidents": [...],
     "pending_deployments": [...],
     "notes": "<free text>",
     "severity_context": "GREEN|YELLOW|RED"
   }
   ```
2. **[≤2min]** Incoming engineer acknowledges:
   ```
   POST /ops/v1/oncall/handoffs/{id}/acknowledge
   Headers: X-Step-Up-Token: <T2_token>
   ```
3. **[≤30s]** Verify PagerDuty/OpsGenie responder updated: `GET /ops/v1/oncall/rotation/current`.
4. **[≤30s]** Verify `OnCallHandoffCreated` + `OnCallHandoffAcknowledged` events in audit chain.

### Static fallback path — ops-dashboard unavailable

1. **[≤1min]** Contact incoming on-call directly via phone (primary) or backup Slack channel `#oncall-handoff-fallback`.
2. **[≤5min]** Transfer context verbally + share incident list via read-only Grafana dashboard (direct link: `dashboards/ops-overview.json` — accessible without ops-dashboard).
3. **[≤5min]** Update PagerDuty/OpsGenie responder DIRECTLY (not via ops-dashboard).
4. **[≤5min]** Create manual handoff record in Incident management system (fallback: GitHub issue in `ops-incidents` repo) to preserve audit trail.
5. Once ops-dashboard recovers: backfill handoff record via `POST /ops/v1/oncall/handoffs/backfill` with `source: MANUAL_FALLBACK`.

### EMERGENCY_SERVICES page during handoff failure

1. EMERGENCY_SERVICES pages route directly to PagerDuty — ops-dashboard is NOT on the critical path for emergency pages.
2. Emergency escalation to outgoing on-call is via PagerDuty direct page (ops-dashboard not involved).
3. Document the bypass in the handoff record once ops-dashboard recovers.

## D — Verification

- `GET /ops/v1/oncall/handoffs/current` → `state: ACKNOWLEDGED`, `incoming_operator_id` correct.
- `GET /ops/v1/oncall/rotation/current` → matches PagerDuty/OpsGenie.
- Audit chain: `OnCallHandoffCreated` + `OnCallHandoffAcknowledged` both sealed.

## E — Rollback

If wrong incoming engineer was acknowledged:
1. `POST /ops/v1/oncall/handoffs/{id}/reassign` with correct engineer (T2 step-up required).
2. Audit chain records reassignment event.

## F — Post-incident

- Was the handoff failure caused by cell outage? Check `slos/command-availability.openslo.yaml` burn rate.
- Was it caused by step-up auth expiry? Check operator session logs.
- Was static fallback exercised? Document in quarterly chaos-drill report.

## G — References

- `policy/cedar/on-call-handoff-authorization.cedar`
- `ARCHITECTURE.md §critical-path-edge-cases row 1` (emergency services)
- `multi-region.md §3 Failure modes`
- `runbooks/dashboard-perf-degradation.md`
