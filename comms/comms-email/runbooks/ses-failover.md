# Runbook — SES failover

> ADR anchor: ADR-0201, IP-013.
> Severity: SEV-2 (regional outage); SEV-1 if global SES outage.

## When to use

- SES regional outage detected (provider status page +
  internal 5xx rate spike).
- SES quota exhaustion sustained > 5 min.

## Prereqs

- Mailgun second-source healthy OR Postal regional capacity
  available.
- IP-013 multi-region routing config reachable.

## Procedure

### Regional outage with healthy sibling region

1. Confirm via SES status page + internal metrics.
2. Page on-call (SEV-2).
3. IP-013 routing auto-fails-over for tenants whose pack
   permits cross-region. Verify:
   `oya-cli comms-email routing-status --tenant {id}` shows
   `processing_region = {sibling}`.
4. Audit-chain entry `routing.failover` emitted automatically.
5. Hold cross-region routing until SES recovers.

### Quota exhaustion

1. Confirm via SES service quotas dashboard.
2. Reduce SES traffic share: flip tenant pack
   `provider_preference` to Mailgun.
3. Submit SES quota increase request via AWS support.
4. Monitor: `comms_email_ses_throttled_total` rate drops.
5. Restore SES traffic share once quota raised.

### Sovereign packs (no failover)

- KSA / UAE / KR packs do NOT use SES. No failover required.
- If sovereign-region Postal is unhealthy, see
  `postal-failover.md`.

## Validation

- Bound-region SES traffic resumes.
- p99 send latency returns to baseline.
- No customer-visible bounce surge.

## Rollback

- Routing is idempotent. Flip back when SES recovers.

## Anti-patterns

- Failing over sovereign packs to non-sovereign regions
  (forbidden by `policy/residency.md`).
- Increasing SES quota during an outage (won't help — the
  outage is upstream).

## References

- IP-013 multi-region routing.
- ADR-0201 §"Adapter set".
- `incident-response.md` SEV-2 §2.5.
