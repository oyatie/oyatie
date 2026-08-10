---
doc_class: Runbook
title: E2E encryption key rotation (MLS RFC 9420)
microservice: messenger
severity: "Sev-3 (planned) / Sev-1 (compromise-driven)"
status: Accepted
owner_team: council-privacy + ops-security + axis-messenger
date: 2026-05-17
related_artifacts:
  - comms/messenger/policy/personal-dm-scope.cedar
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/threat-model.md (T-I-04 + T-S-03)
  - microservices/messenger/sdk-plan.md
doc_status: published
---

# Runbook: E2E encryption key rotation (MLS RFC 9420 client-derived keys)

## When

Two triggers:

1. **Planned rotation** — scheduled MLS epoch advance per RFC 9420 §11.6
   (recommended cadence: monthly per active conversation).
2. **Compromise-driven** — a participant device suspected compromised;
   immediate epoch advance + member removal.

## Severity

- Planned: Sev-3 (no user-facing impact if SDK handles transparently).
- Compromise-driven: Sev-1 (potential PII / PHI exfiltration window).

## Preconditions

- Server has NO plaintext access (per dual-context-isolation.md DCI-03).
- Rotation is initiated by a client (group admin or self-driven).
- Server's role is signing-key-bundle distribution + ciphertext routing only.

## Planned Rotation Procedure

| Step | Action | Owner |
|---|---|---|
| 1 | Group admin client invokes `mls.advance_epoch()` via SDK | client |
| 2 | SDK generates new commit message containing new epoch keying material | client |
| 3 | Commit broadcast to all group members via messenger WebSocket | gateway |
| 4 | Each member's SDK derives new epoch keys; ratchets forward | client |
| 5 | Old epoch keys discarded; in-flight messages on old epoch routed via key-package fallback | client |
| 6 | Audit-chain seal: `MlsEpochAdvanced` event emitted (epoch hash + group_id; never key material) | server |

## Compromise-Driven Rotation Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | ops-security confirms compromise; identifies affected user/device | ops-security | ≤ 30 min |
| 2 | Force-revoke compromised KeyPackage in server-side directory | server | ≤ 5 min |
| 3 | Notify affected group admins (Sev-1 banner in client UI) | gateway | ≤ 5 min |
| 4 | Group admins remove compromised member from groups (MLS Remove proposal) | client | ≤ 1h |
| 5 | Group commits new epoch; compromised device cannot derive new key | client | ≤ 1h |
| 6 | Audit forensics: examine event-replay for any ciphertext the compromised device might have decrypted | ops-security | ≤ 24h |
| 7 | Post-mortem within 5 business days | council-privacy + ops-security | |

## Verification

- `MlsEpochAdvanced` audit-chain event emitted per group per rotation.
- All members confirm new epoch within 24h (epoch convergence metric).
- Search index for affected groups re-emits with new epoch's content_hash
  bound; old encrypted snippets unchanged (cannot be decrypted without
  client key).

## Failure Modes

| Failure | Recovery |
|---|---|
| Group split-brain (some members on old epoch, some on new) | SDK falls back to KeyPackage-bundle dual-encryption for catch-up window (24h); split-brain alert fires |
| Compromised KeyPackage signing key | server-side revocation list; affected groups warned; users re-onboard with fresh KeyPackages |
| Audit-chain seal failure on epoch event | retry; alert; if persistent, halt new epoch advancement until audit-chain restored |

## Server-Side Boundaries (what server DOES NOT do)

- Server NEVER sees plaintext.
- Server NEVER signs MLS commits (clients sign).
- Server NEVER decides epoch timing (clients decide).
- Server NEVER stores private keys (clients hold; optional escrow per
  council-privacy ADR pending — PRD Open Question 5).
- Server CAN distribute public KeyPackages (one per device per user).
- Server CAN route ciphertext commit messages to group members.
- Server CAN refuse stale or revoked KeyPackages.

## Pack Overlays

| Pack | Variation |
|---|---|
| pack-eu | per GDPR Art. 32 — rotation evidence retained per audit-chain |
| pack-us-healthcare | HIPAA 45 CFR §164.312(a)(2)(iv) — encryption + decryption controls; rotation logged |
| pack-kr | KR PIPA Art. 29-2 §"인증, 식별, 권한관리" — rotation cadence in DPIA |
| pack-us-financial | SEC 17a-4(f) — key-rotation events retained alongside message archive |

## References

- RFC 9420 (Messaging Layer Security).
- IETF MLS WG `datatracker.ietf.org/wg/mls/`.
- NIST SP 800-57 (Key Management).
- `microservices/messenger/policy/dual-context-isolation.md` DCI-03.
- `microservices/messenger/threat-model.md` T-I-04, T-S-03.
- Signal Double-Ratchet precedent + MLS-vs-Signal trade-off analysis.
