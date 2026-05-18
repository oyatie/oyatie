# Runbook — Per-tenant from-domain onboard

> ADR anchor: ADR-0201, IP-011.
> Severity: scheduled (new tenant); SEV-3 if onboarding hangs.

## When to use

- New tenant signs up and configures a from-domain.
- Existing tenant adds a second from-domain.

## Prereqs

- Tenant has DNS control over the from-domain.
- OpenBao access for DKIM key generation.
- OpenTofu DNS module reachable.

## Procedure

1. Receive onboarding request via API:
   `POST /v1/tenants/{id}/from-domains` with body
   `{from_domain, locale_pack, support_inbox,
   provider_preference}`.
2. The µservice transitions to `requested`.
3. Generate DKIM key pair (Ed25519 + RSA-2048) per IP-005:
   private into OpenBao, public into the onboarding response.
4. Generate canonical SPF + DKIM + DMARC record strings.
5. Emit `dns.publish.requested` audit-chain event; OpenTofu
   DNS module picks up and publishes.
6. Transition to `dns-pending`.
7. Wait for DNS propagation: poll resolver every 5 min for up
   to 24h.
8. Transition to `dns-published` once all three records
   visible.
9. Bind provider identity:
   - SES: `aws-sdk-sesv2 CreateEmailIdentity` + DKIM attrs.
   - Postal: per-domain identity via Postal API.
   - Mailgun: per-domain via Mailgun API.
10. Transition to `provider-binding` → `provider-bound`.
11. Begin warm-up: 14 days of low-rate sends.
12. Transition to `warm-up` → after T+14d, `active`.
13. (Operator action) Run `dmarc-policy-tune.md` to promote
    DMARC to `p=reject`.

## Stuck-state recovery

| Stuck state | Likely cause | Action |
| ----------- | ------------ | ------ |
| `requested` | API request malformed | Reject + ask tenant to retry |
| `dns-pending` > 24h | DNS not published by tenant | Re-emit `dns.publish.requested`; if still failing, surface to tenant admin |
| `provider-binding` | Provider API error | Check provider status; retry; if persistent, fail over to alternate provider per tenant pack |
| `warm-up` | Send rate exceeded ramp schedule | Throttle to schedule |

## Validation

- External tester (`mail-tester.com`) scores DKIM = pass,
  SPF = pass, DMARC = pass.
- First test send arrives in recipient inbox (not spam).
- Audit chain shows full state-transition trail.

## Rollback

- If onboarding cannot complete after 7 days, revoke partial
  state:
  - Remove provider identity binding.
  - Remove published DNS records.
  - Mark tenant from-domain as `revoked`.
  - Audit-chain entry for the revocation.

## Anti-patterns

- Manual `bao kv put` of DKIM keys (bypasses audit emission).
- Manually publishing DNS records outside the OpenTofu pipeline
  (violates ADR-0202 Tier-B discipline).

## References

- IP-005 DKIM key rotation pipeline.
- IP-011 per-tenant from-domain onboarding.
- ADR-0202 OpenTofu DNS module.
- ADR-0201 §"Per-tenant deliverability primitives".
