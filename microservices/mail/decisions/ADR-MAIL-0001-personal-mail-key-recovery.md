---
id: ADR-MAIL-0001
title: "Personal mail key recovery uses user-held default and opt-in Shamir escrow"
status: Accepted
date: 2026-05-17
microservice: mail
related_oyatie_adrs:
  - ADR-0003
  - ADR-0131
  - ADR-0135
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
deciders: axis-mail, council-privacy, ops-security, council-architecture
owner: axis-mail + council-privacy
decision_owner: axis-mail + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-0208 (Bominal — personal-pillar policy inherited)
related_artifacts:
  - microservices/mail/PRD.md (Open Question 4)
  - microservices/mail/runbooks/e2e-encryption-key-recovery.md (Path A/B/C/D/E)
  - microservices/mail/policy/dual-context-isolation.md (Invariant DCI-03)
  - microservices/mail/threat-model.md (T-T-03, T-S-05)
  - microservices/mail/dpia.md (R-06)
purpose: Close PRD-mail Open Question 4 — what is the default and opt-in posture for Personal-pillar E2E mailbox key recovery at M03 launch.
---

# ADR-MAIL-0001: Personal-pillar mail E2E key recovery — user-held-only default + opt-in Shamir 3-of-5 trustee escrow

## Status

Accepted — 2026-05-17.

## Context

The `mail` µservice ships at M03 with native dual-context isolation per ADR-0135 + Bominal ADR-0208. Personal-pillar mailboxes are encrypted under a user-derived DEK whose wrapping key is derived from the user's passphrase + per-user salt (PBKDF2-HMAC-SHA256, 600 000 iterations; Argon2id migration tracked separately). Per Invariant DCI-03 in `policy/dual-context-isolation.md`, the org admin cannot decrypt a user's Personal mailbox under any path — neither legal-hold, eDiscovery, four-eyes disclosure, nor operator override applies. This invariant is what makes the Personal pillar a genuine Personal pillar rather than an HR-visible "personal folder."

PRD-mail Open Question 4 surfaces the unresolved trade-off: user-held-only keys honour the Personal-pillar trust model perfectly but produce **irrecoverable loss** when the user loses both their device AND their paper backup (Path B in `runbooks/e2e-encryption-key-recovery.md`). Real-world surfaces include user death and family inheritance, prolonged user incapacitation, corporate transitions where an employee opted into a Personal account they later need to hand over, and high-stakes regulated workflows (KR-pack tenants subject to PIPA Art. 23-2 substitute-decision rules) that expect a recoverable path.

This ADR has no direct Bominal predecessor; Bominal ADR-0208 mandates personal-pillar confidentiality but defers the recovery posture to the implementing product. oyatie chooses now, with this ADR, before the M03 launch window closes.

The decision must be authoritative enough that the runbook's Path E ("escrow recovery — future opt-in mode; not M03 default") becomes implementable rather than placeholder text.

## Decision

oyatie mail adopts a **two-mode recovery posture** for Personal-pillar mailboxes, set per user at mailbox creation and changeable by the user thereafter via an authenticated rotation flow:

1. **Default mode — user-held-only with QR paper backup.** The Personal mailbox DEK wrap is derived from the user's passphrase, optionally augmented by a hardware token (YubiKey / Nitrokey via WebAuthn L3 + OpenPGP card). At onboarding, the SDK MUST generate an Ed25519 recovery seed, render it as a QR code, prompt the user to print it (or save to a hardware-isolated medium), and refuse to mark onboarding complete until the user confirms backup possession. Org admin decrypt is structurally impossible (Invariant DCI-03; CI lane `personal-pillar-kms-scope`).

2. **Opt-in mode — Shamir-3-of-5 cryptographic trustee escrow, enforced as a paid tenant_class feature.** Available only when `tenant_class=paid` and the applicable compliance/customer contract enables it; available to individual personal-pillar users on opt-in within a paid tenant that has enabled the offering. The user's wrapping key is split into five Shamir Secret Sharing shares; recovery requires three shares + the user's surviving authentication factor (so escrow alone cannot decrypt without the user, and escrow holders alone cannot decrypt without each other). Trustee roles are bound to OIDC subjects with the `recovery_holder` entitlement; trustee actions are Ed25519-signed and audit-chained. The user picks five trustees from a per-tenant trustee directory (Personal contacts + tenant ombuds + council-privacy escrow service if the tenant subscribes to it).

3. **Recovery via Path E** (escrow) becomes operative when this ADR ships. Until a user opts in, Path E is inapplicable to that user and Path A/B/C/D from the runbook apply.

Both modes share three invariants:

- **Server has no plaintext, ever.** The KMS-wrapped DEK is unwrapped client-side; servers see only wrapped-DEK bytes plus audit events with no key material.
- **All recovery events Ed25519-sealed.** `PersonalDekRecovered`, `PersonalDekEscrowReleased`, `PersonalDekCompromiseRotation` events emit to the audit chain; users can view their own recovery history.
- **No silent escrow enablement.** Switching a user from default to escrow mode requires the user's authenticated action + a fresh QR backup re-generation; tenant admins MAY recommend it but MUST NOT toggle it on behalf of a user.

## Alternatives Considered

### A. Mandatory escrow (no user-only mode)
- Pros: zero irrecoverable-loss path; predictable inheritance/handover story; matches the corporate IT mental model that "data should be recoverable."
- Cons: violates the Personal-pillar trust model — the user no longer has sole custody, and the existence of an escrow path means an attacker who compromises trustees gains a route to decryption. Recreates the "personal folder" anti-pattern oyatie deliberately rejects.
- Rejected: incompatible with Invariant DCI-03 as a personal-pillar posture; obliterates the structural distinction between Personal and Professional mailboxes.

### B. User-held only forever (no escrow option at all)
- Pros: simplest model; absolute trust guarantee; lowest implementation surface.
- Cons: irrecoverable loss for legitimate inheritance/handover scenarios; no answer for KR-PIPA Art. 23-2 substitute-decision cases or estate handover under GDPR Recital 27 + Art. 6(1)(c) "legal obligation"; effectively cedes the enterprise-personal market segment to Proton Mail / Tutanota who offer recovery options.
- Rejected: blocks legitimate use cases that enterprise customers will refuse to onboard without.

### C. Cloud key (oyatie-held wrap key on the server)
- Pros: trivial recovery; standard pattern for consumer mail (Gmail, Outlook personal).
- Cons: defeats the E2E posture — oyatie operators become a decryption oracle; org admins via subpoena to oyatie become an effective decryption path; violates the entire premise of Personal-pillar mail.
- Rejected: contradicts ADR-0135 + Bominal ADR-0208 dual-context isolation by construction.

### D. Single-party escrow (one trustee, no Shamir split)
- Pros: simpler implementation than Shamir.
- Cons: single point of compromise; trustee coercion → full key recovery; no defence in depth.
- Rejected: insufficient adversarial resistance for a paid tenant_class feature.

### E. Threshold escrow with k=2/n=3 instead of k=3/n=5
- Pros: lower coordination cost; faster recovery.
- Cons: lower compromise resistance; two coerced trustees defeat the system. Industry precedent (Vault Shamir, Signal SVR, Apple iCloud Advanced Data Protection) consistently uses k≥3 with n≥5 for high-value secrets.
- Rejected: cohort precedent + threat-model evaluation favours k=3/n=5 as the floor.

## Consequences

### Positive

- Personal-pillar trust model preserved by default — most users get exactly what Bominal ADR-0208 promised: an inbox where org admins, oyatie operators, and even subpoena-served oyatie cannot decrypt.
- Paid tenant_class customers get an opt-in path that satisfies their inheritance/handover/regulated-substitution requirements without compromising users who don't need it.
- Shamir 3-of-5 + user-factor binding means even a full trustee compromise (3 out of 5 colluding) cannot decrypt without the user's separate authentication factor, raising the attack cost above subpoena-level coercion.
- Path E in `runbooks/e2e-encryption-key-recovery.md` becomes implementable rather than placeholder text; the runbook can move from "scheduled-for-distinct-tracked-work" to "operative."
- KR-pack tenants get a documented PIPA Art. 23-2 substitute-decision route through escrow without weakening default-user privacy.

### Negative

- Implementation surface includes Shamir Secret Sharing primitive (use the `vsss-rs` Rust crate with `gf256` GF field, audited; pinned LTS), trustee directory UX, signed-action ceremony, and a recovery-quorum-collection flow that's harder to test than a single-key flow.
- Two recovery modes increase test matrix: every recovery scenario (Path A/B/C/D/E) must be tested under both modes plus the mode-switch transition.
- Trustee-management UX is non-trivial: trustees need their own Ed25519 keys, must be reachable for recovery, and the user's choice of trustees has lasting consequences. We mitigate with a sane default trustee set (council-privacy escrow service if the tenant subscribes) and clear UX warnings.
- Tenant_class gating (`paid` required) means demo_trial personal users get user-held-only with no escrow option, which is the correct trust posture but may be misread as a feature gap; the PRD competitive-parity row makes this explicit.

### Operational

- New CI lane `personal-pillar-shamir-escrow-conformance` validates: (a) escrow-mode mailboxes have exactly 5 distinct trustee public keys recorded, (b) recovery requires ≥3 valid Ed25519 signatures + 1 user-factor proof, (c) `data_class` annotation on escrow records is `SECRET` + `SENSITIVE_PIPA_ART23`.
- Mailbox creation flow gains a "recovery mode" step; default selection is user-held-only with QR backup; opt-in escrow path requires acknowledgement of escrow-quorum implications.
- Audit-chain events `PersonalDekRecoveryModeChanged` emitted on mode switch; both old and new wrapped DEKs retained for 14 days during transition window, then old wrap revoked.
- Pen-test annual scope expands to include trustee-coercion attack simulation; the test must fail to decrypt without quorum + user factor.

### Regulatory

- **GDPR Art. 32** (security of processing): both modes pass the "appropriate technical measures" bar; the escrow path's quorum gate is documented in DPIA R-06.
- **KR PIPA Art. 23-2** (substitute decisions for the absent / incapacitated data subject): escrow mode provides the documented substitute path; user-held mode documents irrecoverability as the user's informed choice.
- **EU AI Act**: out of scope (no automated decision-making in the recovery path).
- **HIPAA 45 CFR §164.312(a)(2)(iv)** (encryption/decryption controls): per-tenant DEKs for Professional mailboxes unaffected; Personal mailboxes are personal-pillar so HIPAA scope does not attach.
- **NIST SP 800-57** (key management lifecycle): Shamir-3-of-5 + Ed25519 trustee signatures + audit-chained events satisfy the key-management-record requirements; the runbook documents the lifecycle.

## Implementation Notes

- Data shape `PersonalMailboxRecoveryModeV1` contains `mailbox_id`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `user_id`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `tenant_id`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `recovery_mode`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `dek_wrap_version`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `trustee_set_id`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `hardware_token_required`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `paper_seed_confirmed_at`.
- Data shape `PersonalMailboxRecoveryModeV1` contains `mode_changed_at`.
- Data shape `TrusteeEscrowShareV1` contains `share_id`.
- Data shape `TrusteeEscrowShareV1` contains `trustee_subject`.
- Data shape `TrusteeEscrowShareV1` contains `trustee_public_key`.
- Data shape `TrusteeEscrowShareV1` contains `share_ciphertext`.
- Data shape `TrusteeEscrowShareV1` contains `share_signature`.
- Data shape `TrusteeEscrowShareV1` contains `share_version`.
- Data shape `RecoveryCeremonyV1` contains `ceremony_id`.
- Data shape `RecoveryCeremonyV1` contains `mailbox_id`.
- Data shape `RecoveryCeremonyV1` contains `requester_user_id`.
- Data shape `RecoveryCeremonyV1` contains `trustee_signatures`.
- Data shape `RecoveryCeremonyV1` contains `user_factor_proof`.
- Data shape `RecoveryCeremonyV1` contains `audit_event_id`.
- API endpoint `POST /v1/personal-mailboxes/{mailbox_id}/recovery-mode` changes recovery mode.
- API endpoint `GET /v1/personal-mailboxes/{mailbox_id}/recovery-mode` returns current mode.
- API endpoint `POST /v1/personal-mailboxes/{mailbox_id}/escrow/trustees` stores trustee set.
- API endpoint `POST /v1/personal-mailboxes/{mailbox_id}/recovery-ceremonies` starts recovery.
- API endpoint `POST /v1/personal-mailboxes/{mailbox_id}/recovery-ceremonies/{ceremony_id}/trustee-signatures` adds trustee signature.
- API endpoint `POST /v1/personal-mailboxes/{mailbox_id}/compromise-rotation` rotates after suspected compromise.
- API endpoint `GET /v1/personal-mailboxes/{mailbox_id}/recovery-history` returns user-visible audit history.
- Default mode enum is `user_held_only`.
- Escrow mode enum is `shamir_3_of_5_user_factor_bound`.
- Shamir implementation is `vsss-rs` pinned to audited 5.x line with `gf256` backend.
- Trustee signatures use Ed25519 per RFC 8032.
- User factor proof uses WebAuthn Level 3 passkey assertion.
- Wrapped DEK bytes are never logged.
- Trustee share ciphertext is envelope-encrypted under the trustee public key.
- OpenBao transit path for escrow metadata signatures is `transit/mail/personal-recovery/{tenant_id}`.
- Paper seed QR encodes recovery seed version and mailbox id.
- QR backup confirmation emits `PersonalDekPaperSeedConfirmed`.
- Escrow enablement emits `PersonalDekRecoveryModeChanged`.
- Escrow release emits `PersonalDekEscrowReleased`.
- Recovery completion emits `PersonalDekRecovered`.
- Compromise rotation emits `PersonalDekCompromiseRotation`.
- Cedar principal for mode change is `Oyatie::Principal::User("{user_id}")`.
- Cedar principal for trustee release is `Oyatie::Principal::User("{trustee_subject}")`.
- Cedar principal for recovery orchestration is `Oyatie::Principal::Service("mail-personal-recovery-api")`.
- Cedar resource is `Mail::PersonalMailboxRecovery`.
- Cedar action `mail.personal_recovery.mode_change` gates mode changes.
- Cedar action `mail.personal_recovery.trustee_release` gates trustee signatures.
- Cedar action `mail.personal_recovery.complete` gates final unwrap.
- Cedar action `mail.personal_recovery.history_read` gates audit-history read.
- Example permit: principal `User("usr_01HY")`, action `mail.personal_recovery.mode_change`, resource `Mail::PersonalMailboxRecovery::"mbx_01HY"`, context `{mailbox_owner:"usr_01HY", requested_mode:"shamir_3_of_5_user_factor_bound", tenant_class:"paid"}`.
- Example permit: principal `User("trustee_01")`, action `mail.personal_recovery.trustee_release`, resource `Mail::PersonalMailboxRecovery::"mbx_01HY"`, context `{ceremony_id:"rec_01HY", trustee_in_set:true, ceremony_state:"open"}`.
- Example forbid: principal `User("tenant_admin_01")`, action `mail.personal_recovery.mode_change`, resource `Mail::PersonalMailboxRecovery::"mbx_01HY"`, context `{mailbox_owner:"usr_01HY", admin_override:true}`.
- Example forbid: final recovery with context `{valid_trustee_signatures:2, required_trustee_signatures:3}`.
- SLO `mail-personal-recovery-mode-change.openslo.yaml` sets mode-change p99 <= 2 seconds.
- SLO `mail-personal-recovery-ceremony.openslo.yaml` sets escrow ceremony p95 <= 30 minutes after quorum is present.
- SLO `mail-personal-recovery-audit-emission.openslo.yaml` sets audit event p99 <= 1 second.
- Failure mode `lost_all_devices_and_seed` is documented irreversible data loss in user-held mode.
- Failure mode `trustee_quorum_unavailable` leaves mailbox unrecovered.
- Failure mode `trustee_signature_invalid` rejects share release.
- Failure mode `user_factor_missing` rejects final recovery.
- Failure mode `admin_attempted_override` emits security event and denies.

## Verification

- Test `personal_recovery_default_user_held_only` verifies onboarding default.
- Test `personal_recovery_requires_paper_seed_confirmation` verifies QR backup gate.
- Test `escrow_available_only_paid_tenant_class` verifies the tenant_class gate.
- Test `escrow_requires_exactly_five_distinct_trustees` verifies trustee set.
- Test `escrow_recovery_requires_three_trustee_signatures` verifies Shamir quorum.
- Test `escrow_recovery_requires_user_factor` verifies user-factor binding.
- Test `tenant_admin_cannot_enable_escrow_for_user` verifies no silent escrow.
- Test `server_never_logs_wrapped_dek_plaintext` verifies log redaction.
- Test `compromise_rotation_revokes_old_wrap_after_fourteen_days` verifies transition.
- Test `recovery_history_visible_to_user` verifies audit read.
- Test `trustee_signature_ed25519_required` verifies signature type.
- Test `personal_recovery_cedar_admin_override_forbidden` verifies Cedar forbid.
- Metric `oya_mail_personal_recovery_mode_change_ms` must meet p99 <= 2 seconds.
- Metric `oya_mail_personal_recovery_quorum_fail_total` tracks unavailable quorum.
- Metric `oya_mail_personal_recovery_admin_override_denied_total` must remain zero outside tests.
- Metric `oya_mail_personal_recovery_audit_emission_ms` must meet p99 <= 1 second.
- Dashboard `mail-personal-recovery.json` shows mode mix, recovery attempts, and failures.
- Dashboard `mail-personal-recovery-security.json` shows invalid trustee signatures and admin override attempts.
- Dashboard `mail-personal-recovery-key-age.json` shows escrow share age and rotation due dates.
- CI check `personal-pillar-kms-scope` verifies no server plaintext path.
- CI check `personal-pillar-shamir-escrow-conformance` verifies k=3/n=5.
- CI check `mail-personal-recovery-cedar` validates permits and forbids.
- CI check `mail-personal-recovery-audit-events` validates event names and payloads.
- Chaos test removes two trustees and expects no recovery.
- Security test compromises tenant admin and expects no Personal mailbox decrypt.
- Recovery drill runs semiannually with synthetic trustee identities.

## References

- RFC 9580 — OpenPGP (cryptographic envelope for signing certs)
- RFC 4880 — OpenPGP predecessor (still cited by many tooling chains)
- RFC 8551 — S/MIME 4.0 (alternative signing chain at user preference)
- Shamir, A. — "How to share a secret" (CACM 1979) + modern implementations (`vsss-rs`, OpenBao Shamir, Signal Secure Value Recovery v3)
- NIST SP 800-57 Part 1 Rev. 5 — Recommendation for Key Management
- KR PIPA (Personal Information Protection Act) Art. 23-2 — substitute decision provisions
- GDPR Art. 32 — security of processing; Recital 27 — data of the deceased
- HIPAA 45 CFR §164.312(a)(2)(iv) — encryption controls
- ePrivacy Directive 2002/58/EC Art. 5 — confidentiality of communications
- ProtonMail recovery model — `https://proton.me/support/recover-encrypted-messages-files` (industry precedent — user-held + recovery file + emergency access)
- Apple iCloud Advanced Data Protection — recovery contact + recovery key (Shamir-style precedent at consumer scale)
- ADR-0135 — Connect full social network super-app (parallel dual-context source)
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- Bominal ADR-0208 — Connect dual-context unified channel hub
- `microservices/mail/PRD.md` Open Question 4
- `microservices/mail/policy/dual-context-isolation.md` Invariant DCI-03
- `microservices/mail/runbooks/e2e-encryption-key-recovery.md` Paths A-E
- `microservices/mail/threat-model.md` T-T-03, T-S-05
- `microservices/mail/dpia.md` R-06
