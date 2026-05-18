# Runbook — DMARC policy tune

> ADR anchor: ADR-0201, IP-011.
> Severity: scheduled (post-warmup); SEV-3 for stuck alignment.

## When to use

- Tenant completes 14-day warm-up; promote DMARC from
  `p=quarantine` to `p=reject`.
- Persistent DMARC alignment failures need diagnosis.

## Prereqs

- Tenant from-domain in `active` state.
- DMARC RUA reports flowing into audit chain.

## Procedure

### Promote to p=reject

1. Confirm warm-up complete:
   `oya-cli comms-email from-domain-status --tenant {id} --from-domain {dom}` →
   `state = active` AND `warmup_completed_at` is at least 14 days ago.
2. Confirm DMARC RUA report alignment ≥ 99% over the last 7
   days:
   `oya-cli comms-email dmarc-summary --tenant {id}`.
3. Update DMARC record via OpenTofu DNS module:
   `oya-cli comms-email dmarc-set --tenant {id} --policy reject`.
4. The OpenTofu DNS module publishes the updated `_dmarc.{dom}`
   TXT record.
5. Audit-chain entry `dmarc.policy.tightened` emitted.
6. Monitor next 7 days; expect no alignment regression.

### Diagnose stuck alignment failures

1. Pull DMARC RUA reports from the audit chain for the past
   14 days.
2. Group by reporting org + result.
3. Common causes:
   - SPF record missing the active provider's IP pool — update
     via IP-011 onboarding flow.
   - DKIM signing domain mismatched with from_domain — verify
     SES / Postal / Mailgun identity binding.
   - Forwarding services breaking DKIM — engage receiver
     directly OR accept the small alignment loss.

## Validation

- After p=reject promotion: DMARC reports show alignment
  ≥ 99% sustained.
- After diagnosis: failure mode identified + fixed (or
  acknowledged as upstream forwarder issue).

## Rollback

- Promote-to-reject is reversible by setting back to
  `p=quarantine`. Audit-chain entry emitted.
- Customer-comms required if rollback triggered by deliverability
  regression.

## Anti-patterns

- Promoting to `p=reject` while alignment is below 99% (will
  block legitimate mail).
- Setting `p=none` (forbidden post-warmup per ADR-0201).

## References

- IP-011 per-tenant from-domain onboarding.
- ADR-0201 §"Per-tenant deliverability primitives".
- RFC 7489 DMARC.
