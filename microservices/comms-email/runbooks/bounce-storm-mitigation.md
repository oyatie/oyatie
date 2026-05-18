# Runbook — Bounce storm mitigation

> ADR anchor: ADR-0201, IP-009.
> Severity: SEV-2 (auto-throttle kicked in); SEV-1 if cluster-
> wide reputation at risk.

## Trigger

Per IP-009 §3: hard-bounce rate > 5% per tenant per 1h window
fires `comms.email.bouncestorm.detected`. IP-009 auto-throttles
the tenant to 25%.

## Prereqs

- On-call paged.
- Tenant admin contactable.

## Procedure

1. Confirm auto-throttle applied:
   `oya-cli comms-email tenant-status --tenant {id}` shows
   throttle = 25%.
2. Identify root cause from the past hour's bounce events
   (audit-chain query):
   - Bulk recipient list with stale addresses.
   - Compromised account injecting bad recipients.
   - Provider-side delivery error (rare; verify provider status
     page).
3. If compromised account, engage Identity team to lock the
   tenant's user immediately.
4. Apply broad suppression to top hard-bounced addresses (this
   stops repeated attempts).
5. Engage tenant admin: confirm recipient list source, review
   acquisition channel, force list-cleaning.
6. Hold at 25% throttle until bounce rate drops < 1% for 1h.
7. Step up: 25% → 50% → 100% in 1h increments, monitoring at
   each step.
8. Audit-chain entries throughout.

## Validation

- Bounce rate < 1% sustained.
- No new escalation events from IP-009.
- Tenant dashboard reflects healthy deliverability.

## Rollback

- If symptoms recur after stepping up, return to 25% and
  re-engage tenant.

## Anti-patterns

- Bypassing the auto-throttle ("trust the tenant, they fixed
  it") — wait for measurable improvement.
- Removing suppression entries that landed during the storm —
  they protect future sends.

## References

- IP-009 bounce / complaint handler.
- IP-010 suppression list.
- `incident-response.md` SEV-2 §2.3.
