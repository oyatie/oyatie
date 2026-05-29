# api-gateway — Incident response

**Authority:** ADR-0157 + ops-sre-reliability charter.

## A — Severity classification

| SEV | Definition | Examples | Page-out |
|---|---|---|---|
| SEV-0 | Total or near-total outage | Anycast withdrawal global; TLS cert chain compromise; mass deny-storm | Page everyone; war-room |
| SEV-1 | Major degradation | Single-region outage; bot-storm; circuit-breaker mass trip | Page on-call + axis lead |
| SEV-2 | Localised degradation | Single-cell evac; per-tenant DDoS; cert near-expiry | Page on-call |
| SEV-3 | Functional impact, no degradation | Audit-emit lag; rate-limit cross-cell drift | Ticket; respond next business day |

## B — On-call rotation

- Primary: axis-network on-call (weekly rotation).
- Secondary: ops-security on-call (weekly rotation).
- Escalation: axis-network lead → CTO → board (for SEV-0 only).
- PagerDuty: `api-gateway-oncall`.
- Slack: `#api-gateway-incidents`.
- War-room: `#api-gateway-warroom` (SEV-0 only).

## C — Response playbook

1. **Acknowledge** in PagerDuty within 5 min.
2. **Triage:** classify SEV; declare incident in `#incidents` channel.
3. **Stabilise:** apply mitigation per runbook (see `runbooks/`).
4. **Communicate:** status page update every 15 min during SEV-0/1.
5. **Mitigate:** root-cause analysis OR rollback to known-good state.
6. **Recover:** verify SLO recovery; un-evacuate cells.
7. **Postmortem:** within 5 business days; blameless; published to `docs/postmortems/`.

## D — Common runbook map

| Symptom | Runbook |
|---|---|
| Latency p99 > target | `runbooks/circuit-breaker-engaged.md` |
| 5xx burst | `runbooks/circuit-breaker-engaged.md` |
| 429 storm | `runbooks/rate-limit-saturation.md` |
| TLS handshake fail spike | `runbooks/tls-cert-rotation.md` |
| Bot-score storm | `runbooks/bot-storm.md` |
| DDoS detected | `runbooks/ddos-mitigation.md` |
| Edge cache poisoning suspected | `runbooks/edge-cache-poisoning.md` |
| Blue/green release degraded | `runbooks/blue-green-rollback.md` |
| Cell evac needed | `runbooks/cell-evac.md` |
| h3 negotiation rate dropped | `runbooks/h3-fallback-verification.md` |
| ECH config rotation needed | `runbooks/ech-key-rotation.md` |
| PQC cert rotation needed | `runbooks/pqc-cert-rotation.md` |
| Edge admission regression | `runbooks/edge-admission-regression.md` |

## E — Status page

- Public status page: `https://status.oyatie.com/`.
- Update within 15 min of SEV-0/1; within 1h of SEV-2.
- Per-region + per-tenant-tier status indicators.
- Postmortem published to `status.oyatie.com/incidents/<id>` within 5 business days.

## F — Customer communication

- SEV-0/1: email + in-app notification within 30 min of detection; updates every 1h until resolution.
- SEV-2: email within 4h.
- SEV-3: tracked in support ticket only.
- Enterprise tenants: dedicated TAM escalation channel per contract.

## G — Postmortem template

Per `docs/templates/postmortem-template.md`. Sections:

1. Summary (≤5 lines).
2. Impact (who, when, what).
3. Timeline (UTC).
4. Root cause (the actual cause; not "human error").
5. Trigger (what changed).
6. Detection (how we found out + lag).
7. Response (what we did).
8. Lessons (what we learned).
9. Action items (assignees + due dates).
10. References (ADRs, runbooks, dashboards).

## H — References

- `microservices/api-gateway/runbooks/`
- `microservices/api-gateway/failure-modes.md`
- `docs/templates/postmortem-template.md`
- Google SRE Workbook ch. 14 (Managing Incidents)
