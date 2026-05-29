# IP-012 — Audit-chain emission

> ADR anchor: ADR-0201, ADR-0145, ADR-0166.
> Owner: `oya-substrate-comms`.
> Estimate: 2 days.

## Goal

Every event the comms-email µservice generates becomes an
ADR-0145 audit-chain entry on a ADR-0166 schema-versioned
shape.

## Why this IP

Auditors must prove who sent what to whom when. Regulators
(GDPR, CAN-SPAM, HIPAA) require a tamper-evident trail. The
audit chain (ADR-0145) is that trail; this IP wires the
emission path.

## Pre-conditions

- ADR-0145 audit chain available.
- ADR-0166 schema registry available.
- `crates/oya-shared-audit-chain-client-kernel` integrated.

## Tasks

### 1. Event taxonomy

| Event                              | Schema id                                    |
| ---------------------------------- | -------------------------------------------- |
| send.preflight.accepted            | oya/comms-email/preflight-accepted/v1        |
| send.preflight.rejected            | oya/comms-email/preflight-rejected/v1        |
| send.provider.accepted             | oya/comms-email/provider-accepted/v1         |
| send.provider.error                | oya/comms-email/provider-error/v1            |
| delivery.event (sent / delivered / ...) | oya/comms-email/delivery-event/v1       |
| bounce.classified                  | oya/comms-email/bounce-classified/v1         |
| complaint.received                 | oya/comms-email/complaint-received/v1        |
| suppression.inserted               | oya/comms-email/suppression-inserted/v1      |
| suppression.removed                | oya/comms-email/suppression-removed/v1       |
| dkim.rotated                       | oya/comms-email/dkim-rotated/v1              |
| dkim.revoked                       | oya/comms-email/dkim-revoked/v1              |
| from-domain.onboarded              | oya/comms-email/from-domain-onboarded/v1     |
| from-domain.state-transition       | oya/comms-email/from-domain-state-changed/v1 |
| dns.publish.requested              | oya/comms-email/dns-publish-requested/v1     |

### 2. Schema registration

- All schemas land in the ADR-0166 schema registry as part
  of this IP.

### 3. Correlation

- Every event carries `audit_correlation_id` matching the
  upstream caller's correlation id when present.
- Internal events (DKIM rotation, suppression insertion) get
  a freshly-minted correlation id.

### 4. Tamper-evidence

- Per ADR-0145, the audit chain provides hash-linked entries.
  The comms-email µservice does not implement chaining itself
  — it emits through the kernel and the chain substrate seals.

### 5. PII handling

- Recipient address appears in events but is annotated as PII
  per ADR-0144 data class. The schema registry's PII tag
  drives downstream redaction.

### 6. Tests

- Unit tests for each schema emission.
- Integration test that runs a full send pipeline and asserts
  the expected audit chain entries land.

## Failure modes

- Audit chain unavailable: events buffer locally for ≤ 5 min;
  beyond that the µservice degrades to reject-new-sends so
  the trail does not develop holes. Runbook
  `webhook-replay.md` covers the chain failover.

## Acceptance criteria

- 100% of taxonomy events emit on the expected schema id.
- p99 emit-to-chain latency ≤ 5 s.
- Schema registry round-trip lints each schema at CI.

## Rollback

There is no rollback for audit emission — compliance posture
requires it. If the chain regresses, the µservice degrades to
reject-new-sends per the failure-mode plan.

## References

- ADR-0201, ADR-0145, ADR-0166, ADR-0144.
- `crates/oya-shared-audit-chain-client-kernel`.
