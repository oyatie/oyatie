# Incident response — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0145, ADR-0180.

## 1. Severity ladder

- **SEV-1**: deliverability compromise, DKIM key leak, or full
  send outage across all providers in a region.
- **SEV-2**: provider outage with healthy fallback; per-tenant
  bounce storm; webhook ingest backlog > 10k.
- **SEV-3**: degraded SLOs (p99 latency widening, single-pack
  template render failure); single-tenant SLO breach.
- **SEV-4**: cosmetic / informational.

## 2. Sev-1 incident scenarios

### 2.1 Leaked DKIM private key

Trigger: external report of forged mail using a tenant's
DKIM selector; OR OpenBao audit log shows unexpected read.

Response:

1. Page on-call (≤ 2 min).
2. Invoke `runbooks/dkim-key-rotation.md` revocation path.
3. New selector + DNS record published within 5 min.
4. Old selector DNS removed within 5 min.
5. Suppression list flush for in-flight retries.
6. Audit chain entries for revocation + rotation.
7. Post-mortem within 5 business days.

### 2.2 Deliverability blacklist

Trigger: receiver block-rate spikes > 5%; OR upstream
blocklist provider (Spamhaus, etc.) lists a tenant IP / domain.

Response:

1. Page on-call (≤ 5 min).
2. Invoke `runbooks/blacklist-recovery.md`.
3. Throttle tenant send rate to 10% to slow the spread.
4. Identify root cause (template, recipient list quality,
   complaint rate).
5. Engage upstream blocklist remediation process.
6. Resume full rate after blocklist removal + 24h cool-down.

### 2.3 Bounce storm

Trigger: hard-bounce rate > 5% per tenant per hour.

Response:

1. IP-009 escalation auto-throttles tenant to 25%.
2. Page on-call (≤ 10 min).
3. Invoke `runbooks/bounce-storm-mitigation.md`.
4. Identify root cause (corrupt recipient list, compromised
   tenant account).
5. Tenant-side action (clean list, force password reset).
6. Resume full rate after sustained < 1% bounce rate for 1h.

### 2.4 Webhook delivery failure backlog

Trigger: DLQ depth > 10k entries.

Response:

1. Page on-call.
2. Invoke `runbooks/webhook-replay.md`.
3. Identify root cause (audit chain outage, schema mismatch,
   credential drift).
4. Replay from DLQ in chronological order.
5. Audit chain entries for the replay batch.

### 2.5 SES quota exhaustion

Trigger: SES throttling rate > 1% sustained over 5 min.

Response:

1. IP-013 fallback routes new sends to Mailgun + Postal.
2. Page on-call (≤ 5 min).
3. Invoke `runbooks/ses-failover.md`.
4. Request SES quota increase OR shift sustained load to
   Postal.
5. Audit chain entries for the failover.

## 3. Post-incident artifacts

- Post-mortem doc per `docs/templates/post-mortem-template.md`.
- Audit chain entry for the incident lifecycle.
- Updated runbook if the response procedure changes.

## 4. Communication

- Internal Slack channel `#incident-comms-email` (auto-created
  on SEV-1).
- Customer notification per ADR-0180 customer-comms SLA.

## 5. Drill cadence

- Quarterly: DKIM key revocation drill.
- Quarterly: SES failover drill.
- Semi-annual: full-region failover drill.

## 6. Anti-goals

- Silent recovery without audit entry.
- Customer notification skip on SEV-1.
- Single-person on-call (always pair on SEV-1).
