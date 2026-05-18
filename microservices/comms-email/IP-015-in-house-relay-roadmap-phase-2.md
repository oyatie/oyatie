# IP-015 — In-house relay roadmap (Phase 2 marker)

> ADR anchor: ADR-0201 §"In-house roadmap".
> Owner: `oya-substrate-comms`.
> Estimate: marker only (no implementation in Phase 1).

## Goal

Document the design + trigger conditions for the Phase-2
in-house Rust-native SMTP relay (`oya-comms-email-server`)
so that when either trigger fires, the substrate is ready to
land the implementation behind a fifth adapter
(`OyaCommsEmailServerAdapter`).

## Why this IP

ADR-0201 §"In-house roadmap" names the in-house relay as a
Phase-2 trigger-gated build. Recording the design intent
during Phase 1 ensures the substrate accumulates the right
abstractions (kernel trait, adapter shells, audit-chain
emission) so a future Phase-2 lands cleanly without
rewriting the adapter contract.

## Pre-conditions

- ADR-0201 ratified.
- `crates/oya-shared-email-comms-kernel` (delivered this
  batch) is the integration contract.

## Tasks

### 1. Design surface (recorded, not implemented)

The Phase-2 build adds a fifth adapter that implements
`EmailComms`:

- Rust-native SMTP server (lettre-server-side or a fork of
  `mail-server` / similar AGPL/Apache project — selection
  deferred until Phase 2 starts).
- Persistent queue backed by Postgres + S3-compatible blob
  store (ADR-0184 storage tier).
- Per-tenant DKIM signing using the kernel's existing key
  rotation pipeline (IP-005, unchanged).
- Webhook event emission identical in shape to IP-008
  (so audit-chain consumers don't break).
- Per-tenant rate ceiling enforcement in-process (existing
  pattern from IP-008).
- Per-region routing identical to IP-013 (so multi-region
  posture survives).

### 2. Trigger conditions

Either fires Phase 2:

1. **Parity-with-SES at scale**: when oyatie hits a
   throughput, cost-per-send, or audit-trail requirement
   that SES cannot meet.
2. **Sovereign Rust-native footprint**: when a sovereign /
   air-gapped deploy needs a Rust-native operator footprint
   and Postal (Ruby + RabbitMQ + MariaDB) is rejected by
   the customer's compliance posture.

### 3. Implementation entry path

When triggered:

1. Open ADR-0201 §"In-house roadmap" addendum with the
   trigger evidence.
2. Add `crates/oya-shared-email-comms-kernel-adapter-oya-comms-email-server`.
3. Wire the µservice as an additional Helm chart at
   `microservices/comms-email/iac/helm/oya-comms-email-server/`.
4. Manifest delta gains `provider = oya-comms-email-server`
   value.
5. Migration: per-tenant cutover with audit-chain
   provenance, identical to any other adapter swap.

### 4. Anti-goals

- Phase 2 is not a replacement for SES / Postal / Mailgun /
  SMTP — those adapters stay. Phase 2 adds a fifth, not
  removes the first four.
- Phase 2 is not a marketing-email platform. The substrate
  remains transactional-only.

## Failure modes

- Premature Phase-2 launch (no trigger satisfied): blocked
  by the ADR addendum requirement. Reviewer-agent enforces.
- Trigger evidence dispute: substrate authority arbitrates.

## Acceptance criteria

- This IP exists.
- ADR-0201 §"In-house roadmap" Phase-2 wording is consistent
  with this IP.
- No code lands in Phase 1 under the
  `oya-comms-email-server` namespace.

## References

- ADR-0201 §"In-house roadmap".
- ADR-0173 vendor lock-in avoidance.
- IP-001 .. IP-014 (the Phase-1 substrate this IP plans to
  extend).
