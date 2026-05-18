# Failure modes — `comms-email` µservice

> ADR anchors: ADR-0201, ADR-0145, ADR-0180.

## 1. Adapter-side failures

### Provider outage (regional)

- Symptom: provider returns 5xx persistently.
- Detection: 5xx rate > 1% over 30s.
- Response: IP-013 multi-region routing fails over to sibling
  region (if pack permits).
- Recovery SLA: 5 min mean-time-to-failover.

### Provider rate limiting

- Symptom: provider returns 429.
- Detection: 429 rate > 0.1% over 30s.
- Response: back-pressure to caller via `RateCeilingExceeded`;
  reroute new sends to second-source provider.

### TLS handshake failure (SMTP)

- Symptom: handshake error on submission.
- Detection: SMTP transport error rate spike.
- Response: surface to caller as non-retryable
  `ProviderError`. Operator investigates relay.

## 2. Deliverability-side failures

### DKIM verification fail at receiver

- Symptom: receiver bounces with DKIM verification error.
- Detection: bounce reason includes `DKIM=fail`.
- Response: page on-call; invoke `dkim-key-rotation.md`
  diagnosis steps; verify DNS published correctly.

### SPF authorization fail at receiver

- Symptom: receiver bounces with SPF=fail.
- Detection: bounce reason analysis.
- Response: verify SPF record published per IP-011; verify
  active provider's IP pool is in the SPF include.

### DMARC alignment fail

- Symptom: DMARC report shows alignment failures.
- Detection: ingested DMARC RUA reports.
- Response: investigate from-domain vs DKIM domain mismatch;
  most common is a misconfigured Reply-To.

## 3. Substrate-side failures

### OpenBao unavailable

- Symptom: DKIM key read fails.
- Detection: preflight rejection rate spikes with
  `DkimBindingMissing`.
- Response: page on-call; runbook references ADR-0173
  secrets storage failover.

### Postgres unavailable

- Symptom: suppression lookup fails; idempotency store fails.
- Detection: DB connection error rate.
- Response: comms-email degrades to read-only; new sends
  reject; runbook `blacklist-recovery.md` covers PG failover.

### Audit chain unavailable

- Symptom: emission lag > buffer window.
- Detection: `audit-chain-emit-lag` SLO breach.
- Response: buffer ≤ 5min; beyond that reject-new-sends to
  prevent silent gaps.

### Schema registry unavailable

- Symptom: emission rejects with schema-not-found.
- Detection: emit error rate.
- Response: cached schemas continue to work; new schemas
  block; alert.

## 4. Tenant-side failures

### Misconfigured from-domain

- Symptom: preflight rejects with from-domain mismatch.
- Detection: per-tenant rejection rate.
- Response: alert tenant; runbook
  `per-tenant-from-domain-onboard.md`.

### Bounce storm

- See `incident-response.md` SEV-2 scenarios.

### Complaint surge

- Symptom: complaint rate > 0.1%.
- Detection: IP-009 escalation.
- Response: throttle to 25%; alert tenant; runbook
  `dmarc-policy-tune.md`.

## 5. Cross-cutting failures

### Schema-version mismatch during rollout

- Symptom: emission rejects after schema update.
- Detection: schema-mismatch metric.
- Response: parent rolls back schema change; ADR-0166 sunset
  policy.

### Multi-region split-brain

- Symptom: same tenant routed to two regions simultaneously.
- Detection: cross-region tenant pinning audit.
- Response: parent reconciles routing table; emit audit
  event for the inconsistency.

## 6. Catastrophic failure

### Full DKIM key compromise across all tenants

- Symptom: external report of mass forgery.
- Detection: external + internal monitoring.
- Response: invoke per-tenant revocation in parallel; pause
  all sends in affected packs; emit ADR-0145 incident
  entries for each tenant; engage compliance + legal.

### Audit chain global outage > 5 min

- Symptom: emission lag exceeds buffer.
- Detection: buffer-exhausted alert.
- Response: reject-new-sends globally; preserve audit
  integrity; restore audit chain per ADR-0145 runbook.

## 7. Detection summary

Every failure mode above has a Prometheus alert and a runbook.
No detection-without-runbook is allowed (lane discipline:
`failure-modes ↔ runbooks ↔ alerts` parity check).
