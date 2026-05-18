# Runbook: partner-onboarding (audit-officer Verified→Active approval)

- Severity: routine (P3 if pending > 7d)
- Trigger: partner record in `Verified` state awaiting audit-officer review.
- Authority: IP-014, partnership-onboarding.md.

## Steps

1. **Check the queue**: `oya consent-graph partner queue --awaiting-approval` lists partners in `Verified`.
2. For each:
   a. Open partner record: `oya consent-graph partner show <peer>`.
   b. Verify handshake evidence:
      - Audit-chain entry `partner-handshake-completed` exists on both sides.
      - X.509 cert chain matches `peer_x509_spki` stored.
      - Pulsar JWT issuer pub-key SPKI matches.
      - audit-chain Merkle root proof verifies via `oya audit-chain verify-root <peer-root>`.
      - Schema version compatibility check passes.
   c. Verify out-of-band legal identity:
      - Confirm legal entity name + jurisdiction with peer's CISO (or designate).
      - If peer is in a high-risk jurisdiction, defer to legal review (≤14d).
   d. **Approve**:
      `oya consent-graph partner approve <peer> --reviewer <audit-officer-id> --notes "<...>"`.
      State transitions Verified → Active; audit-chain emission.
   e. **OR reject**:
      `oya consent-graph partner reject <peer> --reason "<...>"`.
      State rolls back to ∅ (handshake invalidated; cert + Merkle anchors purged).

## Time bounds

- Initial review: ≤7 days. Pending > 7d triggers P3 alert.
- Legal review (high-risk jurisdiction): ≤14 days.

## Verification

- Test agreement draft + offer + accept after approval; full lifecycle works.
- Bilateral chain entries flow on first event.

## Audit evidence

- Approval action sealed: actor + timestamp + notes.
- Rejection action sealed: actor + timestamp + reason.

## Re-handshake

If peer's audit-chain HSM key rotates or X.509 cert renews (preserves SPKI: no action; rotated: re-
handshake):
- Auto-detected on first failed bilateral emission.
- Auto-triggered re-handshake; falls back to `Verified` (pending re-approval if material change).
