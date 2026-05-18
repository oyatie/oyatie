# Threat model — `comms-email` µservice

> Authored: 2026-05-18
> ADR anchors: ADR-0201, ADR-0145, ADR-0173, ADR-0184.
> Framework: STRIDE per asset.

## 1. Trust boundaries

1. Caller µservice ↔ comms-email API.
2. comms-email ↔ provider (SES / Postal / Mailgun / SMTP relay).
3. comms-email ↔ OpenBao (DKIM private key + provider credentials).
4. comms-email ↔ Postgres (suppression list + idempotency store).
5. comms-email ↔ audit chain (ADR-0145).
6. comms-email ↔ DNS (DKIM TXT, SPF, DMARC).

## 2. Asset inventory

- DKIM private keys (most critical — forged-mail risk).
- Per-tenant provider API credentials.
- Suppression list state.
- Audit chain entries.
- Templates + per-locale translation strings.
- Provider webhook secrets.

## 3. Threats per asset (STRIDE)

### DKIM private keys

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Key exfiltration via log emission | I | Key material never in logs — only key reference. CI lint forbids `key_material` substring in logs. |
| Key exfiltration via audit chain emission | I | Schema explicitly excludes key material; CI schema check. |
| Key exfiltration via crash dump | I | OpenBao mount uses tmpfs (memory only); crash dump excludes `/run/secrets/`. |
| Key tampering at rest | T | OpenBao integrity check on read. |
| Key rotation skipped | E | Rotation pipeline (IP-005) emits an audit event; the absence of the event past T+12mo fires a critical alert. |
| Tenant cross-spoof (key used for wrong tenant) | S | Kernel preflight binds key to `from_domain` and rejects when mismatched. |

### Per-tenant provider API credentials

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Credential exfiltration | I | Same OpenBao tmpfs pattern as DKIM. |
| Credential reuse across tenants | E | Per-tenant scoping; the kernel rejects sends where the credential's tenant ≠ the `from_domain`'s tenant. |
| Credential rotation drift | I | Rotation tracked in audit chain; alerts on stale credentials. |

### Suppression list

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Tampering (silent removal of a regulatory opt-out) | T | Every removal emits an ADR-0145 audit event with operator identity. |
| Cross-tenant leakage of suppression entries | I | Postgres row-level security per `tenant_id`. |
| DoS via massive bounce storm | D | Rate limiter on bounce intake; IP-009 bounce-storm escalation. |

### Audit chain entries

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Tamper of historical events | T | Chain seal (ADR-0145). |
| Delayed emission masking abuse | T | SLO `audit-chain-emit-lag p99 ≤ 5s`; alert on breach. |

### Templates + locale strings

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Phishing template injection (operator) | T | Template changes require ADR cadence + two-person review. |
| Liquid SSRF / RCE | E | Forbidden Liquid constructs disabled (IP-007 §6). |

### Webhook secrets

| Threat | STRIDE | Mitigation |
| ------ | ------ | ---------- |
| Webhook forgery | S | HMAC signature verification per provider (IP-008 §1). |
| Replay | T | Webhook fingerprint dedup (IP-008 §4). |

## 4. Cross-cutting threats

### Phishing via from-domain spoof

A malicious tenant tries to send mail as another tenant's
domain.

- Mitigation 1: Per-tenant DKIM key bound to from_domain.
- Mitigation 2: Kernel preflight rejects `from.domain() !=
  binding.from_domain`.
- Mitigation 3: Provider-side identity binding (SES
  `CreateEmailIdentity`, Postal per-domain, Mailgun per-domain)
  rejects the spoofed domain at provider layer too.

### Cross-tenant data leakage in shared provider account

If two tenants share an SES account (cost optimization), one
tenant must never see another tenant's send history.

- Mitigation: Per-tenant configuration sets + per-tenant
  metric pools.
- Mitigation: Audit chain row-level security per `tenant_id`.

### DKIM downgrade attack

A man-in-the-middle strips DKIM headers before delivery.

- Mitigation: DMARC `p=reject` post-warm-up — receivers reject
  any mail claiming to be from the tenant's domain that fails
  DKIM verification.
- Mitigation: SES / Postal / Mailgun use TLS-on-submit; SMTP
  fallback enforces STARTTLS or port 465.

### Bounce-storm-driven reputation attack

An attacker triggers a tenant to send to many invalid addresses
to torch deliverability reputation.

- Mitigation: IP-009 bounce-storm escalation throttles the
  tenant.
- Mitigation: Suppression list (IP-010) prevents repeated
  attempts.

### SES quota exhaustion

A surge of legitimate traffic exhausts the SES regional quota.

- Mitigation: Multi-region routing (IP-013) + Mailgun
  second-source. Runbook `ses-failover.md`.

## 5. Out-of-scope threats (acknowledged)

- Compromise of the OpenBao substrate itself (covered by
  ADR-0173 secrets storage threat model).
- Compromise of the audit chain substrate (ADR-0145 threat
  model).
- Physical compromise of a sovereign-tier datacenter (customer-
  scope per ADR-0180 DR / BC).

## 6. Review cadence

- Quarterly walkthrough with substrate authority.
- ADR-cadence review on any new adapter (IP-015 Phase-2 in-house
  relay).
