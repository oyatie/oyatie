---
id: ADR-MSGR-0002
status: Accepted
date: 2026-05-17
microservice: messenger
deciders: council-privacy, axis-messenger, ops-security, ops-legal
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-MAIL-0001
related_artifacts:
  - microservices/messenger/PRD.md (Open Question 5 — E2E personal-DM key escrow)
  - microservices/messenger/policy/personal-dm-scope.cedar
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/runbooks/e2e-encryption-key-rotation.md
  - microservices/messenger/threat-model.md (T-I-04, T-S-03)
purpose: Close PRD-messenger Open Question 5 — establish key-escrow posture for Personal-DM tier vs Professional-channel tier under MLS RFC 9420.
---

# ADR-MSGR-0002: E2E key escrow tier-split — Personal-DM has no admin escrow ever; Professional-channel supports tenant-admin escrow under Cedar legal-hold policy

## Status

Accepted — 2026-05-17.

## Context

PRD-messenger Open Question 5 asks: what is the key-escrow policy for E2E personal-DM (RFC 9420 MLS) — none, platform-recovery-only, or admin-escrow-allowed? The question parallels (but is distinct from) ADR-MAIL-0001 because mail's personal-pillar is a single-user surface, whereas messenger has both personal DMs (two users sharing a private channel) AND professional channels (tenant-owned multi-user channels where eDiscovery + legal hold MUST function).

The dual-context invariant (parallel ADR-0238 + Bominal ADR-0208 + DCI-03/06 in `policy/dual-context-isolation.md`) forbids tenant-admin disclosure of personal DMs. `policy/personal-dm-scope.cedar` already encodes this as an unconditional `forbid` of `Action::disclose_dm_body`, `Action::admin_decrypt_dm`, `Action::read_dm_plaintext` actions. The runbook `runbooks/e2e-encryption-key-rotation.md` describes MLS epoch advance as a client-driven ceremony with server-no-plaintext invariant, but explicitly defers the escrow question to "council-privacy ADR pending — PRD Open Question 5."

At the same time, Professional channels carry tenant data subject to:
- **SEC Rule 17a-4(f)** — broker-dealer record retention (3-7 years; tamper-evident WORM).
- **KR PIPA Art. 29** — security obligations for personal information processor.
- **HIPAA 45 CFR §164.312** — technical safeguards including encryption/decryption controls.
- **eDiscovery (US FRCP Rules 26 + 34)** — preservation and production of electronically stored information.
- **Tenant-internal legal-hold workflows** — when employee disputes, regulatory inquiry, or litigation hold engages, Professional-channel messages MUST be preservable + producible.

A blanket "no escrow on any DM or channel" makes Professional-channel legal-hold structurally impossible — a deal-breaker for regulated enterprise tenants. A blanket "admin escrow on every DM and channel" violates the Personal pillar's trust model. The answer must be tier-shaped.

MLS (RFC 9420) is well-suited to either posture: MLS supports group-key derivation with optional admin-controlled "external commit" patterns that can be wired to escrow when the group's policy permits, and supports pure user-to-user posture when policy forbids escrow.

## Decision

oyatie messenger adopts a **strict tier-split** for MLS key escrow:

1. **Personal-DM tier (DirectConversation between two users in active_context=Personal): NO ADMIN ESCROW EVER.**
   - MLS group epoch keys are derived solely on client devices; oyatie servers see only ciphertext + KeyPackage signing certs + commit messages.
   - Recovery is user-held-only (paired hardware tokens + paper recovery seed per ADR-MAIL-0001's Personal-pillar pattern; messenger reuses the Personal-pillar key-recovery primitives).
   - Tenant admin, oyatie operator, legal-hold engine, and eDiscovery export are STRUCTURALLY UNABLE to decrypt — Cedar `personal-dm-scope.cedar` already encodes this as unconditional `forbid`.
   - Loss of all participant devices + all paper backups → message history is cryptographically destroyed; oyatie cannot recover it. This is the design tradeoff.
   - This is identical in spirit to the Signal / WhatsApp E2E posture for private chats, except the Cedar policy + audit-chain make the unconditional forbid auditable rather than implicit.

2. **Professional-channel tier (Channel where active_context=Professional, or DirectConversation between users in Professional context): PER-TENANT ADMIN-CONTROLLED ESCROW under Cedar `legal-hold` policy.**
   - MLS group epoch keys are escrowed to a per-tenant Hardware Security Module (cloud-secrets/OpenBao with KMS-wrapped key path `secret/messenger/<tenant>/mls-escrow/<group_id>/<epoch>`).
   - Escrow release requires Cedar `legal-hold` policy evaluation: four-eyes approval (two distinct principals with `legal_hold_approver` entitlement), explicit hold scope, reason code, and audit-chain seal — same pattern as Bominal ADR-0215 four-eyes admin disclosure.
   - eDiscovery export reads through the escrow path; export bundle includes Ed25519 chain-of-custody seal; verifier re-derives digest from source blocks.
   - SEC 17a-4(f) WORM retention + HIPAA + KR PIPA + GDPR all served via the escrow + audit-chain combination.
   - Tenant admins MAY configure "escrow off" for their tenant's professional channels (e.g., starter-tier non-regulated tenant who wants Personal-tier privacy on professional channels too) — but then the tenant attests that legal-hold + eDiscovery cannot be served and accepts the consequence.

3. **Cross-tier context drift forbidden**: a Personal DM cannot be promoted to a Professional channel (or vice versa) at runtime — context binding is set at conversation creation and immutable. `personal-dm-scope.cedar` already enforces context-drift forbid.

4. **MLS epoch rotation cadence**:
   - Personal-DM tier: client-driven, recommended monthly per RFC 9420 §11.6 (no server-side enforcement).
   - Professional-channel tier: client-driven monthly + server-side enforced ≤ 90 days (escrow records rotate with the epoch; old epoch escrow retained per tenant retention floor).

5. **Compromise-driven rotation (Sev-1)** per runbook applies uniformly to both tiers; the runbook is already written.

6. **No "platform-recovery" middle ground**: oyatie operators NEVER hold keys for any tier. Escrow on the Professional tier is to a per-tenant HSM under the tenant's KMS region (per pack residency); oyatie's role is administrator-of-the-administrator at most, never decryption-oracle.

## Alternatives Considered

### A. Blanket admin escrow on every group (Personal-DM + Professional-channel)
- Pros: uniform behaviour; trivial implementation; legal-hold always available.
- Cons: violates the Personal pillar (Invariant DCI-03); makes the dual-context posture a marketing claim rather than a structural property; tenant-admin can read personal DMs on demand, eviscerating the personal/professional distinction.
- Rejected: contradicts ADR-0135.

### B. Blanket no-escrow (Personal-DM + Professional-channel both user-held-only)
- Pros: cleanest E2E posture; matches Signal/WhatsApp purity.
- Cons: defeats Professional-channel legal-hold + eDiscovery + SEC 17a-4(f) WORM retention; enterprise tenants subject to broker-dealer / HIPAA / KR-FSS rules cannot deploy oyatie messenger; effectively concedes the regulated enterprise market.
- Rejected: kills regulated-enterprise viability.

### C. Tier-split (this ADR's choice — Personal-DM no escrow; Professional-channel tenant-admin escrow)
- Pros: honours both the Personal-pillar trust model AND the regulated-enterprise eDiscovery requirement; service class boundary maps cleanly to context boundary; Cedar policy + audit-chain make the boundary auditable.
- Accepted.

### D. Per-conversation user-set escrow (each conversation creator picks escrow on/off)
- Pros: maximum granularity.
- Cons: legal-hold requires predictability per channel; a user-creator choosing "no escrow" on a Professional channel would defeat the tenant's legal-hold posture; creates a coordination problem (who chooses, when, with what consent from other group members).
- Rejected: granularity mis-aligned with the legal-hold predictability requirement; tenant-level control is the right grain.

### E. Platform-recovery-only (oyatie holds keys for recovery; admin doesn't)
- Pros: technically simple; mirrors Apple iCloud non-Advanced-Data-Protection mode.
- Cons: oyatie becomes a decryption oracle; subpoena to oyatie becomes an effective decryption path; defeats the dual-context posture; concentrates risk at the provider.
- Rejected: contradicts the entire E2E posture; concentrates regulatory + adversarial risk.

### F. Tenant-admin escrow on Personal-DM only when tenant opts in at the user level
- Pros: gives enterprise tenants the option of "this user's personal DMs are also retained for compliance" (e.g., regulated employees who agreed to it at onboarding).
- Cons: blurs the Personal-pillar invariant — a "personal" DM that's actually escrowed isn't really personal; the user-consent UX is fragile (renewals, scope-creep, regulatory-induced consent pressure).
- Rejected: the cleaner answer is to make those users' DMs explicitly Professional-context (different conversation kind) rather than escrowed-Personal-context.

## Consequences

### Positive

- Personal-pillar trust model preserved as a structural property — Cedar `personal-dm-scope.cedar` already encodes the unconditional forbid; this ADR makes the posture canonical.
- Regulated-enterprise eDiscovery + legal-hold viable — Professional-channel escrow + four-eyes disclosure + Ed25519 chain-of-custody seal serves SEC 17a-4(f), HIPAA, KR PIPA, GDPR.
- Tier boundary maps to context boundary already enforced everywhere else (Cedar, audit-chain, retention policy, dual-context-isolation CI lane); zero additional coordination.
- MLS RFC 9420 is well-suited to both postures; no protocol fork required.
- Runbook `e2e-encryption-key-rotation.md` Path E ("escrow recovery") becomes operative for Professional channels with this ADR.

### Negative

- Two key-management code paths to maintain (escrow vs no-escrow); mitigated by sharing the MLS core + adapter-shape via the `oya-messenger-channel-store-kernel` MLS port traits.
- Tenant-admin who toggles "escrow off" for Professional channels (per Decision §2 final clause) creates a regulatory liability they may not fully understand; mitigated by UX warnings + tenant attestation requirement.
- Personal-DM "total loss" scenario (user loses all devices + all backups) means message history is cryptographically destroyed; documented as a Personal-pillar tradeoff in the runbook; consistent with Signal/WhatsApp E2E.
- Cross-tier "I posted this in the wrong context" cannot be remediated by re-keying — the only path is to repost in the correct context, accepting the original is orphaned. Documented in UX warning + the dual-context-isolation policy.

### Operational

- Cargo workspace adds `oya-messenger-channel-store-adapter-mls-escrow` (Professional-channel tier) and `oya-messenger-channel-store-adapter-mls-no-escrow` (Personal-DM tier); MLS core via shared `oya-messenger-channel-store-domain` library.
- IaC: per-tenant HSM provisioning (cloud-secrets/OpenBao mount per tenant) for Professional-channel tier; documented in `microservices/messenger/iac/`.
- Cedar policy `microservices/messenger/policy/professional-channel-legal-hold.cedar` (NEW) encodes the four-eyes legal-hold release pattern (paired with existing `personal-dm-scope.cedar`).
- Runbook `microservices/messenger/runbooks/e2e-encryption-key-rotation.md` updates to reference this ADR; Path E becomes operative.
- New CI lane `messenger-tier-escrow-conformance` validates: (a) Personal-DM conversations have no escrow record, (b) Professional-channel conversations have escrow record per epoch, (c) cross-tier promotion/demotion forbidden at the data-model layer.

### Regulatory

- **RFC 9420** (Messaging Layer Security): both tiers conform; client-side key derivation + server-side ciphertext routing.
- **NIST SP 800-57** (Key Management): tier-shaped key lifecycle satisfies the management-record requirements.
- **KR PIPA Art. 29** (security obligations) — Professional-channel escrow + audit-chain seal + 6-year retention satisfies; Personal-DM tier honoured under Art. 23 individual-rights protection.
- **HIPAA 45 CFR §164.312** — Professional-channel escrow + WORM-tamper-evident storage satisfies "encryption + decryption controls" requirement; Personal-DM tier out of HIPAA scope (Personal pillar is not part of the covered entity's workforce communications).
- **SEC Rule 17a-4(f)** — Professional-channel escrow + audit-chain seal + retention floor satisfies WORM requirement.
- **GDPR Art. 32** — appropriate technical measures satisfied by both tiers; Personal-pillar Art. 6(1)(a) consent basis; Professional-channel Art. 6(1)(f) legitimate-interest basis.
- **ePrivacy Directive 2002/58/EC Art. 5** — communications confidentiality preserved by both tiers; the existence of escrow on the Professional tier is disclosed at tenant onboarding.

## References

- RFC 9420 — Messaging Layer Security (MLS)
- NIST SP 800-57 Part 1 Rev. 5 — Key Management
- SEC Rule 17a-4(f) — Broker-Dealer record retention
- HIPAA 45 CFR §§164.312, 164.316 — technical safeguards + retention
- KR PIPA Arts. 23, 28, 29 — individual rights + security obligations
- GDPR Arts. 6, 32 — lawful basis + security of processing
- ePrivacy Directive 2002/58/EC Art. 5 — confidentiality of communications
- US FRCP Rules 26, 34 — eDiscovery
- IETF MLS WG — `https://datatracker.ietf.org/wg/mls/`
- Signal Double-Ratchet precedent (Personal-DM trust model)
- Apple iCloud Advanced Data Protection (tier-split precedent at consumer scale)
- ADR-0135 — Connect full social network super-app (dual-context source)
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- ADR-MAIL-0001 — Personal-pillar mail E2E key recovery (companion privacy posture)
- Bominal ADR-0215 — four-eyes professional-context disclosure pattern (inherited)
- `microservices/messenger/PRD.md` Open Question 5
- `microservices/messenger/policy/personal-dm-scope.cedar`
- `microservices/messenger/policy/dual-context-isolation.md` DCI-03, DCI-06
- `microservices/messenger/runbooks/e2e-encryption-key-rotation.md` Path E
- `microservices/messenger/threat-model.md` T-I-04, T-S-03
