# DPIA — `comms-email` µservice

> Data Protection Impact Assessment — GDPR Art. 35.
> Authored: 2026-05-18.
> ADR anchors: ADR-0201, ADR-0144, ADR-0145.

## 1. Description of processing

Transactional email sent on behalf of tenants. Each send
carries:

- Recipient email address (personal data; identifiable).
- Subject + body containing tenant-controlled personalization
  (name, account references, magic-link URLs, transaction
  references — varies per template).
- Locale (technical metadata).
- Lawful basis tag (Art. 6).
- Consent identifier (when basis = consent).
- Audit chain correlation id.

## 2. Purpose

Operate the canonical transactional-email substrate so every
oyatie µservice that needs to email a human can do so without
re-implementing DKIM / SPF / DMARC / suppression / rate-limit /
audit emission.

## 3. Lawful basis

Per-template, declared at the template registry:

- `consent` (Art. 6(1)(a)) — marketing-class templates
  (out-of-scope; substrate is transactional-only but the basis
  tag exists for future-proofing).
- `legitimate_interest` (Art. 6(1)(f)) — account-state
  notifications, security alerts.
- `contract` (Art. 6(1)(b)) — transaction confirmations,
  receipts.
- `legal_obligation` (Art. 6(1)(c)) — regulatory disclosures.

## 4. Categories of data subjects

- End-users of oyatie tenants (the recipient).
- Tenant employees (operator addresses, support inboxes).
- Tenant administrators (onboarding contacts).

## 5. Categories of personal data

| Category | Sensitivity | Notes |
| -------- | ----------- | ----- |
| Email address | Identifier | Hot path — every send. |
| Recipient name | Identifier | Frequently in templates. |
| Magic link / one-time token | Authenticator | Short-lived; not persisted in audit beyond hashed reference. |
| Account references | Pseudonymous identifier | Tenant-controlled. |
| Behavioral telemetry (open / click) | Behavioral | Persisted in audit chain. |
| PHI (us-healthcare pack only) | Special category (Art. 9) | Encrypted at rest; BAA path only. |

## 6. Recipients of personal data

- Provider (SES / Postal / Mailgun / SMTP relay).
- DNS resolver (DKIM verification by the receiving mailbox).
- ADR-0145 audit chain — internal.
- ADR-0166 schema registry — internal.

## 7. International transfers

- US-region tenants: data may transit through US-region SES.
- EU-region tenants: pinned to EU-region by IP-013; no
  transfers outside EU.
- Sovereign packs: locked to the sovereign region.

## 8. Retention

- Outbound message body: not retained by the substrate beyond
  the provider's own retention; oyatie does not store body
  content in Postgres or audit chain.
- Audit chain entries: retained per ADR-0145 (default 7 years
  for tamper-evident accountability).
- Suppression list: retained indefinitely for compliance
  (Art. 17 erasure requests are honored by inserting into
  suppression, not by erasing the suppression record itself).
- Webhook delivery events: 90 days at full resolution; older
  events down-sampled.

## 9. Risk assessment

| Risk | Likelihood | Impact | Net | Mitigation |
| ---- | ---------- | ------ | --- | ---------- |
| DKIM key compromise | Low | High | Med | IP-005 rotation + revocation path. |
| Cross-tenant data leak | Low | High | Med | Row-level security + tenant-bound DKIM. |
| Phishing from spoofed from-domain | Med | Med | Med | DKIM + DMARC reject. |
| Erroneous suppression (false positive) | Med | Low | Low | Operator-driven removal + audit trail. |
| Audit chain unavailability | Low | High | Med | Buffer ≤ 5min; degrade to reject-new on exceed. |
| Regional outage | Med | Med | Med | Multi-region + provider second-source. |

## 10. Consultation

- Substrate authority reviewed: yes (this ADR cadence).
- Data Protection Officer consultation: standing process
  per ADR-0201 ratification.

## 11. Sign-off

This DPIA is reviewed quarterly. Any new template class
(marketing, BIMI, inbound) triggers a fresh DPIA pass.
