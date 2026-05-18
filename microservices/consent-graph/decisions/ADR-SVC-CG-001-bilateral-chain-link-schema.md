# ADR-SVC-CG-001: Bilateral chain link cross-pointer schema

- Status: Accepted
- Scope: service
- Date: 2026-05-18
- Authority: ADR-0214 §2.6, ADR-0003 (audit-chain), IP-012, IP-013.

## Context

Bilateral chain entries (grantor + grantee) need a tamper-evident cross-pointer schema. Without an
explicit schema, divergence detection (IP-013) is unstructured + non-portable across audit-chain
schema versions.

## Decision

Define `CrossPointer` as:
```rust
struct CrossPointer {
    grantor: ChainLink { chain_id, seq, sealed_at, merkle_root },
    grantee: ChainLink { chain_id, seq, sealed_at, merkle_root },
    paired_hmac: [u8; 32],   // HMAC-SHA256
}
```
Plus a **pair-confirmation entry** emitted on both chains *after* the primary entry's seals, that
captures the cross-pointer itself (recursive seal). Without the confirmation entry, an attacker who
compromised the cross-pointer table could go undetected; with it, the pairing fact is itself sealed.

`paired_hmac = HMAC_SHA256(per-pair-key, grantor.merkle_root || grantee.merkle_root || agreement_id ||
event_id)`.

The per-pair-key lives in OpenBao at `secret/consent-graph/pair-hmac/{grantor_short}-{grantee_short}`
with 1y rotation.

## Alternatives

- Single-side chain with grantee-signed receipt (rejected: grantee can deny receipt).
- Blockchain consensus (rejected: too slow per ADR-0214 §3.5).
- No cross-pointer, infer from event_id correlation (rejected: no tamper evidence).

## Consequences

- Tamper requires three compromises: grantor HSM key, grantee HSM key, and per-pair HMAC key. Each is
  region-isolated + audit-chained.
- Pair-confirmation entry doubles audit-chain volume on consent-graph events. Mitigated by Permit-event
  sampling (0.1%); Deny + lifecycle events 100% (these are the ones that matter forensically).

## Verification

- IP-013 reconciler validates HMAC + cross-pointer existence on every active pair daily.
- Tampering one of the three required compromises causes immediate P0 alert.
