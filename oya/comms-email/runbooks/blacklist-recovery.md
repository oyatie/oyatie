# Runbook — Blacklist recovery

> ADR anchor: ADR-0201.
> Severity: SEV-1 if a customer-visible bounce surge results;
> SEV-2 for ordinary blocklist listing.

## When to use

A tenant's IP or domain is listed on an upstream blocklist
(Spamhaus SBL/CSS/XBL, SURBL, Barracuda BRBL, UCEPROTECT).

## Prereqs

- On-call accepts SEV.
- Access to provider console (SES, Postal admin, Mailgun) for
  the affected adapter.
- Blocklist provider delisting portal credentials in OpenBao.

## Procedure

1. Confirm the listing via authoritative source:
   - Spamhaus: `dig +short {ip}.zen.spamhaus.org`.
   - SURBL: web check.
   - Vendor-specific portal where applicable.
2. Identify root cause:
   - Recent template change introducing spam-class content.
   - Compromised tenant account sending high volume.
   - Recipient list of poor quality (high invalid-address rate).
3. Throttle tenant send rate to 10%:
   `oya-cli comms-email tenant-throttle --tenant {id} --pct 10`.
4. Apply suppression to top hard-bounced addresses.
5. Submit delisting request via blocklist provider portal.
6. Implement countermeasure (engage tenant to clean list,
   force password reset on compromised account, roll back
   template).
7. Wait 24h after sustained < 1% bounce rate.
8. Restore full send rate:
   `oya-cli comms-email tenant-throttle --tenant {id} --pct 100`.
9. Audit-chain entries throughout.

## Validation

- Re-check blocklist authoritative source — listing removed.
- 24h post-restoration bounce rate < 0.5%.
- Tenant deliverability dashboard returns to baseline.

## Rollback

- If symptoms recur within 7 days, repeat at higher throttle
  (5%) and engage provider deliverability team.

## Anti-patterns

- Spamming the blocklist delisting portal (gets banned).
- Resuming full rate without root-cause fix.

## References

- ADR-0201.
- IP-009 bounce / complaint handler.
- IP-010 suppression list.
- `incident-response.md` SEV-1 §2.2.
