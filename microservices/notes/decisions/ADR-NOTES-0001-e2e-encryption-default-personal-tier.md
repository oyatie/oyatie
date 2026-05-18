---
id: ADR-NOTES-0001
status: Accepted
date: 2026-05-17
microservice: notes
deciders: council-privacy, axis-notes, ops-security, ops-legal
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-MSGR-0002
  - ADR-MAIL-0001
  - ADR-NOTES-0004
  - ADR-NOTES-0005
related_artifacts:
  - microservices/notes/PRD.md (NFR §Security; FR-21)
  - microservices/notes/policy/e2e-personal-tier-default.md
  - microservices/notes/policy/dual-context-isolation.md
  - microservices/notes/runbooks/e2e-key-rotation-and-recovery.md
  - microservices/notes/threat-model.md (A-01..A-03; T-S-01..T-D-01)
purpose: Establish the default-ON E2E encryption posture for Personal-tier notes and the default-OFF (admin-policy-override) posture for Professional-tier notes, distinct from messenger ADR-MSGR-0002 and sharper than docs.
---

# ADR-NOTES-0001: Personal-tier notes ship E2E ON by default; Professional-tier notes ship tenant-DEK envelope ON by default with admin-policy override

## Status

Accepted — 2026-05-17.

## Context

Notes are *first-thought capture*. The personal-pillar privacy posture must therefore be sharper than docs, which are deliberately collaborative long-form artifacts. Existing market posture establishes the baseline: Apple Notes ships a Lockable subset; Standard Notes ships E2E-by-default for everything; Obsidian Sync ships opt-in E2E; Notion ships server-side encryption only. The most privacy-respecting incumbent (Standard Notes) ships E2E ON by default at signup.

Two regulatory and ethical drivers reinforce this:

1. **GDPR Art. 9 (special categories)** — notes routinely carry health, sexuality, religion, political opinion, biometric data. Server-side processing of these categories without explicit consent violates Art. 9. Structurally impossible server-side processing is the cleanest mitigation.
2. **KR PIPA Art. 23 (sensitive information)** — sensitive-info handling requires elevated controls; E2E covers the requirement.

Professional-tier notes face the opposite legal force: SEC Rule 17a-4(f), HIPAA 45 CFR §164.312, KR PIPA Art. 29 require admin-disclosable + WORM-retainable records. A blanket E2E posture on Professional content makes regulated enterprises non-viable as customers (cf. messenger ADR-MSGR-0002).

Furthermore, notes-µservice introduces an AI surface (T1 summarize / tag-suggest / link-suggest) that, if not bounded, would create a structural backdoor — the AI provider would become the decryption oracle. This is unique to notes-µservice (the docs µservice has no AI surface at minimum-shippable-tier). Therefore the E2E posture must be bound to a structural AI-refusal invariant (ADR-NOTES-0005).

## Decision

oyatie notes adopts a **strict tier-split E2E posture**:

1. **Personal-tier notes (`context_kind=Personal`) ship E2E ON by default at note-creation time.**
   - Keys derived client-side via MLS RFC 9420 (`openmls 0.6`).
   - Server stores only `body_ciphertext` + per-device KeyPackage signing-cert + commit messages.
   - oyatie operators + tenant admins + foundry-runtime AI MUST NOT have plaintext access.
   - Recovery: paired hardware token + paper recovery seed (analogous to Apple iCloud Advanced Data Protection + Standard Notes recovery model).
   - Loss-of-all-devices-and-seed → cryptographic destruction of notes; this is the documented tradeoff per the user-onboarding double-confirmation flow.
   - The "E2E ON" is enforced as a *structural data-model property*, not a runtime flag: `oya-notes-note-store-kernel` `PersonalNoteRef` carries a zero-byte `body_client_only: ()` marker that the compiler refuses to coerce into a server-side plaintext type. Cedar `personal-note-scope.cedar` belt-and-suspenders forbids any `Action::read_plaintext` on Personal resources.

2. **Professional-tier notes (`context_kind=Professional`) ship tenant-DEK envelope encryption ON by default at note-creation time.**
   - Envelope keys per Bominal ADR-0111 (tenant-DEK + per-pack KMS).
   - Server can decrypt for legitimate operations: search-index emission (Cedar-scoped), version-history seal, four-eyes admin disclosure (Bominal ADR-0215 paired-principal pattern with audit-chain seal).
   - Tenant admins MAY opt the tenant into "Personal-tier E2E posture for Professional notes too" — but then attest the tenant accepts the eDiscovery / legal-hold consequences (analogous to messenger ADR-MSGR-0002 §2 final clause).

3. **Cross-tier context drift forbidden**: a Personal note cannot be promoted to Professional (or vice versa) at runtime. `policy/dual-context-isolation.md` DCI-07 enforces immutability of `context_kind`.

4. **MLS epoch rotation cadence**:
   - Personal-tier: client-driven, recommended monthly per RFC 9420 §11.6.
   - Professional-tier: not applicable; tenant-DEK rotation per Bominal ADR-0111 (90d).

5. **Compromise-driven rotation (Sev-1)** uniform for both tiers per `runbooks/e2e-key-rotation-and-recovery.md`.

6. **No platform-recovery middle ground**: oyatie operators NEVER hold Personal-tier keys. Tenant-admin recovery on Professional-tier is via four-eyes + Cedar `legal-hold` (same pattern as messenger ADR-MSGR-0002 Professional-channel tier).

7. **AI assist (T1 summarize / tag-suggest / link-suggest) is STRUCTURALLY REFUSED on Personal-tier notes** — ADR-NOTES-0005 establishes the invariant + CI lane + Cedar policy.

8. **FIPS 140-3 validated crypto modules required** for E2E key material on supported platforms (Apple CryptoKit FIPS-140-3 cert, Microsoft CNG FIPS-mode, openmls compiled with `fips` feature). Non-FIPS platforms supported but flagged in user-onboarding consent.

## Alternatives Considered

### A. Default OFF for Personal-tier; user opts in
- Pros: closest to current incumbent default (Apple Notes / OneNote); lowest UX friction.
- Cons: contradicts Personal-pillar privacy promise; equivalent to Notion's posture which fails GDPR Art. 9 best-practice; leaves AI-surface vulnerability open by default.
- Rejected: contradicts parallel ADR-0238 dual-context posture + Standard Notes precedent.

### B. Default ON for both tiers
- Pros: cleanest E2E posture across the board.
- Cons: makes Professional-tier eDiscovery + four-eyes admin disclosure + WORM retention structurally impossible (same trade-off rejected in messenger ADR-MSGR-0002 §B).
- Rejected: kills regulated-enterprise market viability for notes-µservice.

### C. Default ON for Personal; default OFF for Professional with tenant-admin opt-in to ON (this ADR's choice)
- Pros: honours Personal-pillar privacy promise + regulated-enterprise eDiscovery; tier-split maps to context boundary already enforced; matches messenger ADR-MSGR-0002 tier-split pattern; allows tenant-admin opt-in to Personal-tier-posture-on-Professional for low-regulated tenants who want it.
- Accepted.

### D. Default ON for both tiers with platform-recovery for Professional-tier
- Pros: balances some privacy with admin disclosure.
- Cons: makes oyatie a decryption oracle on Professional-tier; concentrates regulatory + adversarial risk; same trade-off rejected in messenger ADR-MSGR-0002 §E.
- Rejected.

### E. Default ON for Personal with per-note user-set toggle (some Personal notes E2E off, some on)
- Pros: granular user control.
- Cons: UX is fragile (users won't know to flip per-note); confused state where some "personal" notes are not actually private; defeats the structural-property goal.
- Rejected: per-tier is the right grain; per-note is too granular and confusing.

### F. Default ON for Personal with client-side encrypted backup to oyatie (recovery key sealed to oyatie HSM)
- Pros: lower data-loss risk on device-loss.
- Cons: oyatie becomes a decryption oracle if compelled by subpoena; defeats Personal-pillar trust model; same problem as platform-recovery.
- Rejected: paper seed + paired hardware token is the right model.

## Consequences

### Positive

- Personal-pillar privacy posture is a *structural* property (compile-time + LEAN-lane + Cedar default-deny), not a marketing claim.
- AI-surface bound: ADR-NOTES-0005 invariant + LEAN lane + Cedar forbid means AI cannot become a decryption oracle even if a future PM tries to wire it up; the structural impossibility is documented + tested.
- Regulated-enterprise viability preserved on Professional-tier: four-eyes + tenant-DEK + audit-chain serves SEC 17a-4(f), HIPAA, KR PIPA, GDPR.
- Tier-split parallels messenger ADR-MSGR-0002 → consistent mental model for tenant operators across notes + messenger.
- Search-architecture follows automatically: ADR-NOTES-0004 specifies client-side encrypted-inverted-index for Personal-tier (because server cannot index), and Meilisearch for Professional-tier.

### Negative

- Two code paths to maintain (E2E vs envelope). Mitigated by sharing the BC kernel + adapter shape via `oya-notes-note-store-kernel` ports.
- Personal-tier "total loss" scenario (user loses all devices + paper seed) → permanent data destruction. Documented as tradeoff at onboarding; matches Standard Notes + Apple iCloud Advanced Data Protection posture.
- Cross-tier "I posted in wrong context" cannot be remediated by re-encryption — user must recreate in correct context (DCI-07). Documented in UX.
- Personal-tier AI assist is refused; users wanting AI must mark notes Professional. UX must surface this cleanly.

### Operational

- Cargo workspace adds `oya-notes-e2e-key-management-{kernel,domain,usecase,api,adapter,adapter-mls,sdk,app}` (Personal-tier) + `oya-notes-note-store-adapter-postgres` (Professional-tier with envelope).
- IaC: per-tenant HSM provisioning for Professional-tier envelope keys; documented in `microservices/notes/iac/`.
- Cedar policy `microservices/notes/policy/e2e-personal-tier-default.md` (this ADR's binding artifact) encodes the structural posture.
- Runbook `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md` operative for both tiers.
- New CI lane `notes-tier-e2e-conformance` validates: (a) every Personal-tier write goes through `PersonalNoteRef` type; (b) every Professional-tier write goes through `ProfessionalNoteRef` type; (c) cross-tier promotion/demotion is unreachable code.

### Regulatory

- **GDPR Art. 9** — structural impossibility of server-side Personal-tier processing covers special-category requirement.
- **KR PIPA Art. 23** — sensitive-info handling satisfied by E2E.
- **HIPAA 45 CFR §164.312** — Professional-tier tenant-DEK + four-eyes + audit-chain satisfies.
- **SEC Rule 17a-4(f)** — Professional-tier WORM retention via tenant-DEK + audit-chain.
- **EU AI Act Art. 50** — transparency: AI assist on Professional-tier is labelled; Personal-tier AI is structurally refused.
- **ePrivacy Directive Art. 5** — communications confidentiality preserved by both tiers; Professional-tier escrow disclosed at onboarding.
- **NIST SP 800-57** — tier-shaped key lifecycle satisfies management-record requirements.

## Future (PQ migration)

MLS RFC 9420 currently uses ECDH (X25519) + Ed25519. Post-quantum MLS draft (`draft-ietf-mls-architecture-pq-mls`) tracked. Migration path: epoch-bump-on-new-cipher-suite per RFC 9420 §11.6. Target: PQ-ready upgrade path within 24 months of NIST-final-publication of selected PQ-KEM standard for MLS.

## References

- RFC 9420 — MLS.
- NIST SP 800-57 Rev. 5 — Key Management.
- GDPR Arts. 9, 25, 32.
- KR PIPA Arts. 23, 28, 29.
- HIPAA 45 CFR §164.312.
- SEC Rule 17a-4(f).
- EU AI Act Art. 50.
- ePrivacy Directive 2002/58/EC Art. 5.
- FIPS 140-3 (cryptographic module validation).
- Standard Notes E2E Whitepaper (publicly available).
- Apple iCloud Advanced Data Protection.
- Obsidian End-to-End Encryption Sync documentation.
- ADR-0135 — Connect dual-context (parallel).
- ADR-0131 — Per-microservice flat layout.
- ADR-0132 — Suite-and-bundle dissolution.
- ADR-MSGR-0002 — Messenger E2E tier-split (paired pattern).
- ADR-MAIL-0001 — Mail Personal-pillar key recovery (paired pattern).
- ADR-NOTES-0004 — Search architecture respecting E2E.
- ADR-NOTES-0005 — AI assist bounds + E2E invariant.
- `microservices/notes/policy/dual-context-isolation.md`.
- `microservices/notes/threat-model.md`.
