# Competitor parity matrix — `comms-email` µservice

> Authored: 2026-05-18
> Scope: feature parity check vs commercial + OSS providers.

## 1. Comparison set

- **AWS SES** (commercial, cloud-native; oyatie's default for
  AWS clusters).
- **SendGrid** (Twilio, commercial-only; rejected as canonical
  per ADR-0173).
- **Mailgun** (commercial; second-source).
- **Postmark** (commercial; rejected as canonical).
- **Postal** (AGPL OSS; sovereign-tier default).

## 2. Feature matrix

| Capability | oyatie comms-email | SES | SendGrid | Mailgun | Postmark | Postal |
| ---------- | ------------------ | --- | -------- | ------- | -------- | ------ |
| Transactional API | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| DKIM signing | ✓ enforced | ✓ | ✓ | ✓ | ✓ | ✓ |
| SPF posture check | ✓ enforced at preflight | partial | partial | partial | partial | partial |
| DMARC alignment | ✓ enforced | ✓ | ✓ | ✓ | ✓ | ✓ |
| Per-tenant from-domain | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Multi-tenant isolation | ✓ row-level | ✓ config-set | ✓ subuser | ✓ domain | ✓ server | ✓ server |
| Suppression list | ✓ canonical cross-provider | ✓ per-account | ✓ per-account | ✓ per-account | ✓ per-account | ✓ |
| Webhook events | ✓ normalized | ✓ via SNS | ✓ | ✓ | ✓ | ✓ |
| Multi-region | ✓ per-pack | ✓ multi-region | ✓ regional | regional | regional | self-host |
| Self-hosted | ✓ via Postal | ✗ | ✗ | ✗ | ✗ | ✓ |
| Air-gapped | ✓ via Postal | ✗ | ✗ | ✗ | ✗ | ✓ |
| MJML templating | ✓ kernel-side | ✗ (BYO) | partial | ✗ (BYO) | partial | ✗ (BYO) |
| Liquid substitution | ✓ kernel-side | ✗ | ✓ (Handlebars) | ✗ | ✓ (Mustache) | ✗ |
| Vendor-neutral adapter | ✓ 4 adapters | N/A | N/A | N/A | N/A | N/A |
| AGPL/OSS license | ✓ kernel + Postal | proprietary | proprietary | proprietary | proprietary | ✓ AGPL |
| Per-locale templates | ✓ ADR-0064 packs | ✗ | ✗ | ✗ | ✗ | ✗ |
| Audit-chain emission | ✓ ADR-0145 | ✗ | ✗ | ✗ | ✗ | ✗ |
| Idempotency key (ADR-0149) | ✓ | partial | ✗ | partial | ✗ | partial |
| Per-tenant DKIM rotation | ✓ pipeline | manual | manual | manual | manual | manual |

## 3. Where oyatie is ahead

- Vendor-neutral adapter pattern (no single-provider lock-in).
- Self-hosted parity via Postal (SES + SendGrid + Mailgun +
  Postmark are all SaaS-only).
- Canonical cross-provider suppression list.
- Audit-chain emission on a tamper-evident shape.
- Per-locale templates via ADR-0064 packs.
- Idempotency-key enforcement at the kernel (ADR-0149).

## 4. Where oyatie is at parity

- DKIM / SPF / DMARC enforcement.
- Multi-region routing.
- Webhook event taxonomy.
- Per-tenant from-domain support.

## 5. Where oyatie is behind (deliberate)

- Marketing-class campaign management: not in scope (would
  require a separate µservice).
- HTML editor / WYSIWYG: not in scope (tenants supply MJML).
- BIMI logos: deferred to follow-up ADR.
- Inbound email: deferred to follow-up ADR.

## 6. Conclusion

For transactional substrate posture — DKIM enforcement, vendor
neutrality, self-host capability, audit-chain emission, per-pack
localization — oyatie comms-email is at least at parity with
every commercial provider and ahead on lock-in posture +
sovereign-tier readiness.
