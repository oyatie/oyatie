# Runbook — DKIM key rotation

> ADR anchor: ADR-0201, IP-005.
> Severity: scheduled (annual) OR SEV-1 (on revocation).

## When to use

- **Scheduled**: every 12 months per tenant from-domain, driven
  by IP-005.
- **On-demand**: tenant requests rotation (compliance audit,
  ownership change).
- **Emergency revocation**: SEV-1 — DKIM private key leak.

## Prereqs

- OpenBao access for the comms-email service account.
- Audit-chain emission path healthy.
- DNS provider IaC pipeline (OpenTofu DNS module) reachable.

## Procedure — scheduled

1. Confirm next selector is precomputed (e.g. `oya202605` for
   May 2026). The selector follows IP-005 §3 schedule.
2. Verify OpenBao key generation completed at T-14d:
   `kv list /oya/comms-email/dkim/{tenant}/` shows both
   selectors.
3. Verify DNS publish for the new selector at T-14d via
   external resolver (`dig +short oya202605._domainkey.{from_domain} TXT`).
4. Switch active selector at T-0:
   `oya-cli comms-email dkim activate --tenant {id} --selector oya202605`.
5. Audit-chain entry `dkim.rotated` emitted automatically.
6. Verify next send signs with the new selector via header
   inspection.
7. Wait 14 days, then remove old DNS record:
   `oya-cli comms-email dkim retire --tenant {id} --selector oya202504`.
8. Audit-chain entry for the retire.

## Procedure — emergency revocation

1. Page substrate authority (SEV-1).
2. Run `oya-cli comms-email dkim revoke --tenant {id} --selector {compromised}`.
3. New selector generated immediately; DNS publish requested.
4. Old selector DNS record removed within ≤ 5 min.
5. Suppression list flush for in-flight retries against the
   compromised key.
6. Audit-chain entries for `dkim.revoked` + `dkim.rotated`.
7. Post-mortem within 5 business days.

## Validation

After rotation:

- Send a test message to `mail-tester.com`; expect DKIM `pass`
  with the new selector.
- Inspect SES / Postal / Mailgun provider state — new key
  reflected.
- DMARC RUA reports over the next 48h show alignment ≥ 99%.

## Rollback

If new selector fails DKIM verification at receivers, roll back
to the prior selector:

1. `oya-cli comms-email dkim activate --tenant {id} --selector {prior}`.
2. Investigate the new selector's DNS record vs OpenBao key
   pair.
3. Re-attempt after fix.

## Anti-patterns

- Skipping the 14-day overlap window — receivers may still
  have the old key cached.
- Manual `bao kv put` — bypasses audit-chain emission.

## References

- IP-005 DKIM key rotation pipeline.
- RFC 6376 DKIM signatures.
- ADR-0201 §"Per-tenant deliverability primitives".
