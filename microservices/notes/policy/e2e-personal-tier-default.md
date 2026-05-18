---
doc_class: PolicySpec
title: E2E Personal-Tier Default Posture
microservice: notes
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-notes + ops-security
deciders: council-privacy, ops-security, axis-notes, ops-legal
related_adrs: [ADR-NOTES-0001, ADR-NOTES-0004, ADR-NOTES-0005, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/notes/PRD.md (NFR §Security)
  - microservices/notes/policy/dual-context-isolation.md (DCI-03, DCI-08, DCI-09)
  - microservices/notes/policy/tenant-scope.cedar
  - microservices/notes/runbooks/e2e-key-rotation-and-recovery.md
  - microservices/notes/threat-model.md (A-01..A-03)
review_cadence: quarterly
doc_status: published
---

# E2E Personal-Tier Default Posture (notes µservice)

## Purpose

Bind the structural-property claim of ADR-NOTES-0001 to specific implementation invariants, runtime guards, and audit-chain primitives. This document is the contract auditors read when asked "*prove that oyatie cannot decrypt Personal-tier notes*" — and the engineering team's reference for what controls must remain in place.

## Scope

| Tier | Posture | Authority |
|---|---|---|
| Personal (`context_kind=Personal`) | E2E **default ON**; structurally impossible to disable for new notes | ADR-NOTES-0001 |
| Professional (`context_kind=Professional`) | tenant-DEK envelope encryption **default ON**; tenant-admin MAY opt the tenant into Personal-tier-style E2E for Professional notes (accepting eDiscovery / legal-hold tradeoff) | ADR-NOTES-0001 §2 |

## Personal-Tier Invariants

### Inv-E2E-01: Client-side key derivation only

- MLS RFC 9420 (`openmls 0.6`) for all key material.
- Server stores only:
  - `body_ciphertext` (opaque blob).
  - Per-device `KeyPackage` signing certs.
  - MLS commit messages (forwarded to group members; opaque to server).
- Server NEVER sees:
  - Plaintext body.
  - Plaintext tags.
  - Plaintext title (server stores only encrypted title; UI surfaces decrypted title via SDK).
  - Group key material.
  - Recovery seed.

### Inv-E2E-02: Type-system enforcement

```rust
// oya-notes-note-store-kernel — sealed; cannot be coerced.
pub struct PersonalNoteRef {
    note_id: NoteId,
    body_client_only: PhantomData<NeverServerPlaintext>,
}
pub struct ProfessionalNoteRef {
    note_id: NoteId,
}

// No From / Into impl between them.
// No method that returns NeverServerPlaintext exists.
```

### Inv-E2E-03: Cedar belt-and-suspenders

`policy/tenant-scope.cedar` carries unconditional `forbid` on:

- `Action::disclose_note_body` when `context_kind == "Personal"`.
- `Action::invoke_ai_assist` when `context_kind == "Personal"` (paired with ADR-NOTES-0005).
- `Action::start_collab_session` when `context_kind == "Personal"` (paired with ADR-NOTES-0003).

### Inv-E2E-04: CI-lane enforcement

- `oya-check-dual-context-isolation` validates type-system + Cedar (per DCI-01..09).
- `oya-check-e2e-ai-refusal` validates AI invariant (per ADR-NOTES-0005).
- `oya-check-notes-tier-conformance` (specific to notes) validates tier-shaped key-material types.
- All three are BLOCKER on `dev` and `staging`.

### Inv-E2E-05: Runtime guards

- `oya_notes_personal_decrypt_attempt_total` Prometheus counter; **expected = 0**; alarm at > 0 → Sev-1.
- `oya_notes_ai_call_blocked_e2e_total` Prometheus counter; alarm at > 0 → Sev-1.
- `oya_notes_collab_session_blocked_e2e_total` Prometheus counter; alarm at > 0 → Sev-1.

### Inv-E2E-06: Recovery model

- Personal-tier key recovery: paired hardware token (FIDO2-class) + paper recovery seed (24-word BIP39-style).
- Both presented at first-device-loss recovery.
- Loss of all devices + seed → **cryptographic destruction** of notes; oyatie cannot recover. Documented as Personal-pillar tradeoff in onboarding (double-confirmation UX).
- Analogous to Apple iCloud Advanced Data Protection + Signal account-loss + Standard Notes total-loss model.

### Inv-E2E-07: Epoch rotation

- Recommended cadence: monthly per RFC 9420 §11.6.
- Compromise-driven Sev-1 rotation per `runbooks/e2e-key-rotation-and-recovery.md`.
- Server-side enforcement: NONE (client-driven; oyatie can refuse stale KeyPackage but cannot force rotation).

### Inv-E2E-08: Personal-tier audit-chain

- Personal-tier emits audit-chain ONLY on sharing events (`ShareLinkCreated`, `ShareLinkRevoked`, `ShareLinkAccessed`).
- No `NoteCreated` / `NoteEdited` / `NoteDeleted` audit-chain entries for Personal-tier (per Inv-E2E-09 below).
- Sharing-event audit-chain seals omit body content (only metadata: token, scope, principal).

### Inv-E2E-09: Personal-tier Workflow event minimisation

- `NoteCreated` for Personal-tier carries: `note_id`, `tenant_id`, `user_id`, `context_kind`, `created_at`. Nothing else.
- No title, no tags, no body, no attachment refs.
- `NoteEdited` for Personal-tier carries the same minimal fields.
- LEAN lane `oya-check-personal-event-minimisation` verifies.

### Inv-E2E-10: FIPS 140-3 crypto modules

- On supported platforms:
  - Apple CryptoKit FIPS-140-3 cert (iOS/macOS).
  - Microsoft CNG FIPS-mode (Windows).
  - openmls compiled with `fips` feature (Linux server-side ciphertext blob handling).
  - RustCrypto FIPS-mode for share-link token generation.
- Non-FIPS platforms supported but flagged at onboarding.

## Professional-Tier Posture

### Default

- Tenant-DEK envelope per Bominal ADR-0111.
- Server can decrypt for legitimate Cedar-scoped operations.
- Search-index emission via decrypted body (server-side).
- AI assist via decrypted body (Cedar-scoped; tenant-admin opt-in per ADR-NOTES-0005).
- Loro collab via in-memory plaintext (broker-side; Cedar-scoped per ADR-NOTES-0003).
- Four-eyes admin disclosure per Bominal ADR-0215.

### Tenant-Admin Override (E2E on Professional)

- Tenant-admin MAY toggle tenant-wide setting: "Personal-tier-style E2E on Professional notes."
- Consequence: tenant loses eDiscovery + legal-hold capability for those notes.
- Tenant attests at toggle-time.
- All Professional notes created post-toggle are E2E; pre-existing remain envelope.
- Toggle is irreversible (tenant cannot un-E2E without per-note client-side re-encryption).

## Personal-Pillar UX Affordances

The product UX MUST:

- Surface "E2E" badge on every Personal note in the UI.
- Surface "AI assist not available — this is an E2E note" banner when user tries to invoke AI on Personal.
- Surface "Collab not available — this is an E2E note" banner when user tries to start collab on Personal.
- At onboarding, double-confirm recovery seed receipt: "I have stored the seed safely. I understand that loss = permanent destruction of my Personal-tier notes."

## Verification

- Unit + integration tests in `tests/regression/e2e-personal-tier/`.
- Quarterly chaos-test: synthetic decrypt-attempt + AI-call + collab-start on Personal notes; all must return 403 + emit metric + raise alarm.
- Annual external pen-test focused on Personal-tier-E2E breach scenarios.
- SOC 2 + ISO 27001 audit cycle includes this document.

## References

- ADR-NOTES-0001 (this document binds its claims).
- ADR-NOTES-0003 (Loro collab refused on Personal).
- ADR-NOTES-0004 (search architecture; client-side encrypted index for Personal).
- ADR-NOTES-0005 (AI refused on Personal).
- RFC 9420 (MLS).
- NIST SP 800-57 Rev. 5.
- Apple iCloud Advanced Data Protection.
- Standard Notes Threat Model Whitepaper.
- `microservices/notes/policy/dual-context-isolation.md`.
- `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`.
- `microservices/notes/threat-model.md`.
