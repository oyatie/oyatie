---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-NOTES-0001, ADR-0244]
companion_docs: [microservices/notes/policy/share-link-scope.cedar]
inbound_citations: [microservices/notes/ARCHITECTURE.md]
---

# Runbook: Notes share-link revocation

## A. Trigger conditions

- User-initiated revocation via UI / API.
- Detected harvesting of a leaked share-URL (anti-scrape signal).
- GDPR Art. 17 erasure that touches a shared note.

## B. Pre-checks

1. Verify operator (or note owner) Cedar permit `oya.notes.share-link-revoke`.
2. Look up share-token: `oya notes share-link list --owner <id>`.
3. Verify auth-class WEBAUTHN_PASSKEY for B2C_PERSONAL_E2E tier.

## C. Procedure

1. **Mark revoked.** `oya notes share-link revoke --token <id>`; emits `oya.notes.share-link-revoke`. Timing ≤2s.
2. **Invalidate sync.** Push revocation event to all clients holding the note + the recipient's clients via WebSocket; clients delete the local cache.
3. **MLS group rotation.** For E2E shares, rotate the MLS group key (Remove member); emit `oya.notes.e2e-key-rotate`.
4. **Audit redemption history.** Pull `oya.notes.share-link-redeem` events; if redemptions occurred outside scope, file security finding.
5. **Verify revocation.** Synthetic redemption returns Cedar FORBID + audit `oya.notes.share-link-redeem-refused`.
6. **GDPR Art. 17.** If erasure, also invoke `oya notes note-erase --tenant <id> --note <id>` to remove blobs from all replicas.
7. **Notify recipients.** `oya.notes.recipient-notify` with `notify_class=SHARE_LINK_REVOKED`.
8. **Emit closure.** `oya.notes.share-link-revoke-complete`.

## D. Verification

- `oya notes share-link get <id>` returns `status=REVOKED`.
- Synthetic redemption blocked.
- Recipient client UI shows access removed.

## E. Rollback

Revocation is permanent for the token ID; re-issuance creates a new token.

## F. Post-incident

If revocation tied to a leak / scrape: file postmortem.

## G. References

- `policy/share-link-scope.cedar`
- ADR-NOTES-0001
- ADR-0276 portability
