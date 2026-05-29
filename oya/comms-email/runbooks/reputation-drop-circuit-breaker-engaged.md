---
doc_class: Runbook
shape: How-to
related_adrs: [ADR-0201, ADR-0255, ADR-0263]
companion_docs:
  - microservices/comms-email/IP-020-reputation-monitor-worker.md
  - microservices/comms-email/dashboards/reputation-monitoring.json
---

# Runbook — reputation-drop circuit-breaker engaged

## A. Trigger conditions

- Tenant reputation score < 70.
- Marketing send forbidden by `policy/abuse-defence.cedar`.
- Alert page `CommsEmailReputationDropCircuitBreaker`.

## B. Pre-checks

1. `oya comms-email reputation show --tenant=$TID` — confirm score + sources.
2. Identify primary signal: Gmail Postmaster / SNDS / Sender Score / Talos.
3. Check recent bounce + complaint rates.

## C. Procedure

1. Notify tenant via in-app + admin email (transactional NOT affected, only marketing).
2. Pause tenant's marketing automations (`oya comms-email automation pause --tenant=$TID`).
3. Investigate root cause: spam-trap hit / list-hygiene / content-issue / authentication-fail.
4. If DKIM/SPF/DMARC misalignment → rotate keys + verify DNS (`runbook dkim-key-rotation.md`).
5. If list-hygiene issue → recommend re-engagement campaign + scrub inactives.
6. If content-flagged → review template; check phishing-classifier output.
7. Apply remediations + wait 24-48h.
8. `oya comms-email reputation refresh --tenant=$TID` to pull updated signals.
9. If score crosses 70, circuit-breaker resets automatically.
10. Document root cause in `evidence/comms-email-reputation-postmortems/$TID-$DATE.md`.

## D. Verification

- Reputation score ≥70 sustained for 24h.
- Marketing send Cedar gate passes again (synthetic test).

## E. Rollback

- No rollback — reputation recovery is forward-only.

## F. Post-incident

- Tune reputation alert thresholds per `dashboards/reputation-monitoring.json`.
- File ADR if structural issue identified.

## G. References

- ADR-0201 — comms-email substrate
- IP-020 — reputation-monitor worker
- Runbook `dkim-key-rotation.md`
