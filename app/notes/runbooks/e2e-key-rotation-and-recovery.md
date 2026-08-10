---
doc_class: Runbook
title: E2E key rotation + recovery (MLS RFC 9420)
microservice: notes
severity: "Sev-3 (planned) / Sev-1 (compromise-driven)"
status: Accepted
owner_team: council-privacy + ops-security + axis-notes
date: 2026-05-17
related_artifacts:
  - microservices/notes/policy/e2e-personal-tier-default.md
  - microservices/notes/policy/dual-context-isolation.md
  - microservices/notes/policy/tenant-scope.cedar
  - microservices/notes/threat-model.md (A-02, A-03; T-S-02, T-T-02, T-I-02)
  - microservices/notes/decisions/ADR-NOTES-0001-e2e-encryption-default-personal-tier.md
doc_status: published
---

# Runbook: E2E key rotation + recovery (MLS RFC 9420 client-derived keys)

## When

Three triggers:

1. **Planned rotation** — scheduled MLS epoch advance per RFC 9420 §11.6 (recommended cadence: monthly per active Personal-tier user).
2. **Compromise-driven** — a user device suspected compromised; immediate epoch advance + device removal.
3. **Recovery** — user has lost all devices but retains paper recovery seed.

## Severity

- Planned: Sev-3 (no user-facing impact if SDK handles transparently).
- Compromise-driven: Sev-1 (potential plaintext exfiltration window).
- Recovery: Sev-3 (user-initiated; no incident).

## Preconditions

- Server has NO plaintext access (per `policy/dual-context-isolation.md` DCI-03 + `policy/e2e-personal-tier-default.md` Inv-E2E-01).
- Rotation is client-initiated; server's role is KeyPackage distribution + ciphertext routing.

## Planned Rotation Procedure

| Step | Action | Owner |
|---|---|---|
| 1 | Client invokes `mls.advance_epoch()` via SDK | client |
| 2 | SDK generates new commit message containing new epoch keying material | client |
| 3 | Commit broadcast to user's device fleet via WebSocket | gateway |
| 4 | Each device's SDK derives new epoch keys; ratchets forward | client |
| 5 | Old epoch keys discarded; in-flight content on old epoch routed via KeyPackage fallback | client |
| 6 | Server stores new KeyPackage; verifies signature; emits `MlsEpochAdvanced` for audit-chain seal (epoch hash + user_id; never key material) | server |

## Compromise-Driven Rotation Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | ops-security confirms compromise; identifies affected user/device | ops-security | ≤ 30 min |
| 2 | Force-revoke compromised KeyPackage in server-side directory | server | ≤ 5 min |
| 3 | Notify affected user (Sev-1 banner in client UI on their other devices) | gateway | ≤ 5 min |
| 4 | User removes compromised device from their fleet (MLS Remove proposal) | client | ≤ 1h |
| 5 | Fleet commits new epoch; compromised device cannot derive new key | client | ≤ 1h |
| 6 | Audit forensics: examine event-replay for any ciphertext the compromised device might have decrypted | ops-security | ≤ 24h |
| 7 | Post-mortem within 5 business days | council-privacy + ops-security | |

## Recovery Procedure (User Lost All Devices but Retains Paper Seed)

| Step | Action | Owner |
|---|---|---|
| 1 | User contacts support; verifies identity (out-of-band; tenant-controlled for Professional; user-side OOB for Personal) | support |
| 2 | User initiates recovery on new device | client (user) |
| 3 | New device prompts for 24-word paper seed (BIP39-style) | client |
| 4 | SDK derives root key from seed; generates new device-bound MLS KeyPackage | client |
| 5 | New KeyPackage signed under root key; pushed to server | client → server |
| 6 | Server accepts; populates user's fleet with new device | server |
| 7 | User starts decrypting historical ciphertext (server pushes ciphertext blobs; client decrypts locally) | client |
| 8 | `MlsRecoveryExecuted` audit-chain event written (no key material) | server |

## Loss of All Devices + Seed (No Recovery Possible)

This is the documented Personal-pillar tradeoff. The user's Personal-tier notes are cryptographically destroyed; oyatie cannot recover them.

| Step | Action |
|---|---|
| 1 | User contacts support; declares loss; identity verified |
| 2 | Support documents loss event in audit-chain `MlsTotalLossDeclared` |
| 3 | User offered: (a) start fresh, or (b) recover non-E2E notes (none for Personal-tier; perhaps shared Professional copies) |
| 4 | User-comms: "I understand my Personal-tier notes are destroyed. I accept this tradeoff." (consistent with onboarding double-confirmation) |
| 5 | Tombstone unread ciphertext after 30d (storage reclamation) |

This procedure matches Apple iCloud Advanced Data Protection + Standard Notes + Signal account-recovery posture.

## Verification

- `MlsEpochAdvanced` audit-chain event emitted per rotation per user.
- All user devices confirm new epoch within 24h (epoch convergence metric `oya_notes_mls_epoch_converged_within_24h_ratio`).
- Search index (client-side encrypted; Personal-tier) regenerates with new epoch's content_hash bound; old encrypted snippets unchanged (cannot be decrypted without new key).

## Failure Modes

| Failure | Recovery |
|---|---|
| User device fleet split-brain (some on old epoch, some on new) | SDK falls back to KeyPackage-bundle dual-encryption for catch-up window (24h); split-brain alert fires |
| Compromised KeyPackage signing key | server-side revocation list; affected fleet warned; user re-onboards with fresh KeyPackages |
| Audit-chain seal failure on epoch event | retry; alert; if persistent, halt new epoch advancement until audit-chain restored |
| Paper seed lost; only one device left | encourage user to print fresh seed under that device; document; tradeoff communicated |

## Server-Side Boundaries (what server DOES NOT do)

- Server NEVER sees plaintext.
- Server NEVER signs MLS commits (clients sign).
- Server NEVER decides epoch timing (clients decide).
- Server NEVER stores private keys (clients hold; oyatie holds no escrow on Personal-tier).
- Server CAN distribute public KeyPackages (one per device per user).
- Server CAN route ciphertext commit messages to fleet.
- Server CAN refuse stale or revoked KeyPackages.

## Pack Overlays

| Pack | Variation |
|---|---|
| pack-eu | per GDPR Art. 32 — rotation evidence retained per audit-chain |
| pack-us-healthcare | HIPAA 45 CFR §164.312(a)(2)(iv) — encryption + decryption controls; rotation logged (Professional-tier only since Personal-tier typically not in HIPAA scope) |
| pack-kr | KR PIPA Art. 29-2 §"인증, 식별, 권한관리" — rotation cadence in DPIA |

## Metrics

- `oya_notes_mls_epoch_advance_total{tier=personal}` — rotation count.
- `oya_notes_mls_epoch_converged_within_24h_ratio` — convergence health.
- `oya_notes_mls_recovery_total` — recovery count; alert at spike.
- `oya_notes_personal_decrypt_attempt_total` — expected = 0 server-side; alert if > 0.

## References

- RFC 9420 (MLS).
- NIST SP 800-57 Rev. 5.
- ADR-NOTES-0001.
- `microservices/notes/policy/e2e-personal-tier-default.md`.
- `microservices/notes/policy/dual-context-isolation.md`.
- Apple iCloud Advanced Data Protection.
- Standard Notes recovery model.
- Signal account-recovery posture.
