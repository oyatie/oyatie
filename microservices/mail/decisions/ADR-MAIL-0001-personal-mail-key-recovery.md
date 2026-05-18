---
id: ADR-MAIL-0001
status: Accepted
date: 2026-05-17
microservice: mail
deciders: axis-mail, council-privacy, ops-security, council-architecture
owner: axis-mail + council-privacy
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

2. **Opt-in mode — Shamir-3-of-5 cryptographic trustee escrow, enforced as a tenant-tier feature.** Available only on tenant-tier `enterprise` or above; available to individual personal-pillar users on opt-in within an enterprise-tier tenant that has enabled the offering. The user's wrapping key is split into five Shamir Secret Sharing shares; recovery requires three shares + the user's surviving authentication factor (so escrow alone cannot decrypt without the user, and escrow holders alone cannot decrypt without each other). Trustee roles are bound to OIDC subjects with the `recovery_holder` entitlement; trustee actions are Ed25519-signed and audit-chained. The user picks five trustees from a per-tenant trustee directory (Personal contacts + tenant ombuds + council-privacy escrow service if the tenant subscribes to it).

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
- Rejected: insufficient adversarial resistance for an enterprise-tier feature.

### E. Threshold escrow with k=2/n=3 instead of k=3/n=5
- Pros: lower coordination cost; faster recovery.
- Cons: lower compromise resistance; two coerced trustees defeat the system. Industry precedent (Vault Shamir, Signal SVR, Apple iCloud Advanced Data Protection) consistently uses k≥3 with n≥5 for high-value secrets.
- Rejected: cohort precedent + threat-model evaluation favours k=3/n=5 as the floor.

## Consequences

### Positive

- Personal-pillar trust model preserved by default — most users get exactly what Bominal ADR-0208 promised: an inbox where org admins, oyatie operators, and even subpoena-served oyatie cannot decrypt.
- Enterprise-tier customers get an opt-in path that satisfies their inheritance/handover/regulated-substitution requirements without compromising users who don't need it.
- Shamir 3-of-5 + user-factor binding means even a full trustee compromise (3 out of 5 colluding) cannot decrypt without the user's separate authentication factor, raising the attack cost above subpoena-level coercion.
- Path E in `runbooks/e2e-encryption-key-recovery.md` becomes implementable rather than placeholder text; the runbook can move from "scheduled-for-distinct-tracked-work" to "operative."
- KR-pack tenants get a documented PIPA Art. 23-2 substitute-decision route through escrow without weakening default-user privacy.

### Negative

- Implementation surface includes Shamir Secret Sharing primitive (use the `vsss-rs` Rust crate with `gf256` GF field, audited; pinned LTS), trustee directory UX, signed-action ceremony, and a recovery-quorum-collection flow that's harder to test than a single-key flow.
- Two recovery modes increase test matrix: every recovery scenario (Path A/B/C/D/E) must be tested under both modes plus the mode-switch transition.
- Trustee-management UX is non-trivial: trustees need their own Ed25519 keys, must be reachable for recovery, and the user's choice of trustees has lasting consequences. We mitigate with a sane default trustee set (council-privacy escrow service if the tenant subscribes) and clear UX warnings.
- Tenant-tier gating (`enterprise` minimum) means starter / pro tier personal users get user-held-only with no escrow option, which is the correct trust posture but may be misread as a feature gap; the PRD competitive-parity row makes this explicit.

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
