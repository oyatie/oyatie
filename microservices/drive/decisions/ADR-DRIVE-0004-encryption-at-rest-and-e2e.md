---
id: ADR-DRIVE-0004
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, council-architecture, ops-security, cloud-secrets, council-privacy
owner: ops-security + axis-drive
supersedes: []
superseded_by: []
related: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131, ADR-0133, ADR-0140, ADR-DRIVE-0001, ADR-DRIVE-0003, ADR-DRIVE-0006]
related_artifacts:
  - microservices/drive/PRD.md (§Non-Functional Requirements security; §FR-19 E2E)
  - microservices/drive/threat-model.md (T-T-02 DEK substitution; T-I-06 DEK leak)
  - microservices/drive/policy/tenant-scope.cedar
purpose: |
  Pick the encryption-at-rest model + the optional client-side E2E model for
  the drive µservice. Match HIPAA + GDPR Art. 32 + KR PIPA Art. 29 + FIPS
  140-3 requirements; match Proton Drive / Tresorit / MEGA / Sync.com /
  Internxt zero-knowledge competitor parity for the Personal pillar
  (opt-in).
---

# ADR-DRIVE-0004: Encryption-at-rest via OpenBao Transit (FIPS 140-3 envelope) + optional client-side E2E via libsodium secretstream (Personal pillar opt-in)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-drive §Security mandates encryption-at-rest under per-tenant DEK envelope for Professional context, with optional client-side E2E for Personal context. Threat-model §T-T-02 + §T-I-06 establish DEK substitution + DEK leak as residual-M risks.

Industry precedent:

| Competitor | At-rest envelope | E2E option | KMS |
|---|---|---|---|
| Google Drive | yes (Google KMS) | no | Google KMS |
| Dropbox | yes (AWS KMS) | no | AWS KMS |
| OneDrive | yes | no | Microsoft KMS |
| Box | yes | yes (Box KeySafe) | Box KeySafe (BYOK) |
| iCloud Drive | yes | yes (Advanced Data Protection) | Apple HSM |
| Proton Drive | yes | yes (default; libsodium) | client-side |
| Tresorit | yes | yes (default; libsodium) | client-side |
| Sync.com | yes | yes (default) | client-side |
| MEGA | yes | yes (default; XSalsa20) | client-side |
| Internxt | yes | yes (default) | client-side |

Candidate envelope-encryption KMS:
- **AWS KMS** — proprietary; FIPS 140-3 Level 2 modules available; vendor-tied.
- **Google Cloud KMS** — proprietary; vendor-tied.
- **HashiCorp Vault Transit** — open-source; widely deployed.
- **OpenBao Transit** — IBM / Linux Foundation fork of Vault; OSS; FIPS 140-3 module available. ([openbao.org](https://openbao.org))

Candidate symmetric-encryption ciphers (at-rest):
- **AES-256-GCM** — NIST SP 800-38D; FIPS-approved; widely deployed.
- **ChaCha20-Poly1305** — RFC 8439; faster than AES on CPUs without AES-NI; not on the FIPS-approved list as of FIPS 140-3.
- **XSalsa20-Poly1305** — libsodium default; not FIPS-approved.

Candidate client-side E2E primitives:
- **libsodium secretstream** (RFC 7539 ChaCha20-Poly1305 streaming construction) — used by Proton Drive, Tresorit, Sync.com.
- **libsodium secretbox** (XSalsa20-Poly1305) — used by MEGA legacy.
- **PGP/OpenPGP** — broader interop but messier.

Per ADR-0133 axis-2 security-conformance + axis-4 industry-citation, the chosen KMS module must be FIPS 140-3 certified for pack-us-healthcare (HIPAA) + pack-eu (Schrems II supplementary measure).

## Decision

The drive µservice ships:

### Server-side envelope encryption (mandatory for all tenants)

- **KMS**: OpenBao Transit (FIPS 140-3 Level 2 module enabled for pack-us-healthcare + pack-eu; available as opt-in for other packs).
- **At-rest cipher**: AES-256-GCM (NIST SP 800-38D).
- **Key hierarchy**:
  - Master KEK per pack (rotated quarterly; managed by OpenBao).
  - Per-tenant DEK derived from KEK (rotated every 90 days per ADR-DRIVE-0003 alignment).
  - Per-file CEK (Content Encryption Key) derived from DEK + `(tenant_id, file_id, version)`; one CEK per file version.
- **Wrap**: every persisted ciphertext carries a binding tuple `(kek_id, dek_id, cek_derivation_path, ciphertext_iv, ciphertext_tag)`; integrity-protected against substitution.
- **Object-store integration**: ciphertext bytes uploaded directly to Garage/MinIO/SeaweedFS; key material never persisted alongside ciphertext.

### Client-side E2E (optional for Personal pillar)

- **Primitive**: libsodium `secretstream` (XChaCha20-Poly1305 streaming construction).
- **Key derivation**: client-side key derived from user passphrase via Argon2id; key never leaves client.
- **Wire**: ciphertext uploaded; server stores ciphertext + key-wrapping metadata; server cannot decrypt.
- **Search**: E2E files indexed client-side only; server-side search returns metadata only (filename, mime, size).
- **Preview**: E2E files preview client-side only; server-side preview returns "client-only" placeholder.
- **Mode**: per-tenant Personal-pillar opt-in; OFF by default; tenant warning at activation that recovery is client-key-bound (key loss = data loss).
- **Sync**: delta-sync (per ADR-DRIVE-0002) still works at chunk level since chunks are content-addressed by their pre-encryption hash; sync delta protocol unchanged.

### Key rotation

- **KEK**: quarterly; pack-level event; re-derives all DEKs.
- **DEK**: 90 days; per-tenant; rotation event re-encrypts active records via OpenBao Transit `rewrap`.
- **CEK**: derived per-file-version; never rotated (versioning replaces).

### Key storage

- All KMS keys stored in OpenBao Transit.
- HSM backing for pack-us-healthcare + pack-eu (FIPS 140-3 Level 2 HSM via cloud provider; nShield Connect XC equivalent).
- Per-tenant `Secret<T>` wrapper type in Rust (stripped `Debug` impl + no `Serialize` impl; LEAN check `oya-check-secret-no-log` refuses any path that could log a key).

## Alternatives Considered

### A. AWS KMS (vendor-managed)

- **Pros**:
  - FIPS 140-3 Level 2 modules available.
  - Managed key lifecycle.
  - SaaS audit logs.
- **Cons**:
  - Vendor-tied; per-pack residency requires AWS region selection.
  - Cross-cloud portability lost.
  - Inconsistent with `cloud-iac` self-hosted posture per ADR-0117.
- **Rejected** as primary; retained as a tenant-choice alternative for tier-3 tenants with their own AWS DPA.

### B. HashiCorp Vault Transit (the original)

- **Pros**:
  - Mature.
  - Wide deployment.
  - FIPS 140-3 module available.
- **Cons**:
  - Licence change in 2023 (BSL); not FOSS for many use cases.
  - OpenBao fork covers oyatie's open-source posture.
- **Rejected** in favour of OpenBao Transit (the Linux Foundation OSS fork).

### C. Per-file E2E mandatory (no server-side envelope)

- **Pros**:
  - Strongest privacy posture by default.
  - Matches Proton Drive / Tresorit / Sync.com defaults.
- **Cons**:
  - Server cannot full-text search, preview, or scan E2E files; loses competitive features for the bulk of (non-zero-knowledge) tenants.
  - Recovery: client-key loss → permanent data loss; not acceptable for Professional pillar where workflow continuity is required.
- **Rejected** as default; **accepted as opt-in for Personal pillar**.

### D. PGP / OpenPGP for E2E

- **Pros**:
  - Standard.
  - Broad interop with email pipelines.
- **Cons**:
  - Operationally messy at scale.
  - Less ergonomic than libsodium secretstream.
- **Rejected** in favour of libsodium secretstream.

### E. OpenBao Transit envelope (mandatory) + libsodium secretstream E2E (Personal opt-in)  ← **CHOSEN**

- **Pros**:
  - Defaults: every byte at-rest under tenant-DEK envelope; meets HIPAA + GDPR + KR PIPA at-rest requirements.
  - Personal-pillar opt-in: matches Proton Drive / Tresorit / Sync.com zero-knowledge for tenants who want it; competitive parity with the E2E-first cohort.
  - OpenBao Transit FIPS 140-3 module unblocks pack-us-healthcare + pack-eu Schrems II supplementary measure posture.
  - Library choice (libsodium) is battle-tested at billion-user scale (Signal, Proton, Tresorit, Wire).
- **Cons**:
  - Two key-handling code paths (server envelope + client E2E); doubled audit surface.
  - E2E files lose server-side search / preview / scan; tenant comms on activation must clarify.
- **Accepted** because the dual posture is the only way to satisfy both Professional-pillar workflow requirements and Personal-pillar zero-knowledge preferences.

## Consequences

### Positive

- **HIPAA 45 CFR §164.312(a)(2)(iv)** + **GDPR Art. 32(1)(a)** + **KR PIPA Art. 29** technical safeguards met.
- **FIPS 140-3 Level 2** posture for pack-us-healthcare + pack-eu via OpenBao Transit HSM-backed mode.
- **Personal-pillar zero-knowledge** — matches Proton Drive / Tresorit / Sync.com defaults.
- **Per-file CEK** — fine-grained key isolation; per-file revocation possible via CEK destruction (file becomes unrecoverable).
- **DEK rotation** — `rewrap` operation in OpenBao Transit rotates DEK without re-uploading bytes.

### Negative

- **Operator complexity** — OpenBao Transit HSM setup + per-pack KEK lifecycle + per-tenant DEK lifecycle + per-file CEK derivation = three layers of key lifecycle to manage. Mitigation: runbook + automation.
- **E2E feature gaps** — server-side search / preview / scan unavailable for E2E files; tenant comms must clarify at activation.
- **OpenBao operational maturity** — newer than HashiCorp Vault upstream. Mitigation: tracked CVE feed + fallback plan to HashiCorp Vault if OpenBao upstream cools.

### Hyrum's Law

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law":
- **Ciphertext binding tuple shape**: legacy `oya-connect-drive-domain` used a different ciphertext-binding format; new format includes `(kek_id, dek_id, cek_derivation_path, ciphertext_iv, ciphertext_tag)`. Consumers that pattern-matched on the legacy binding see different bytes; ciphertext-at-rest is internal-only. No external Hyrum surface.

### Operational

- **New CI lane**: `oya-governance-encryption-at-rest-coverage` (BLOCKER) — refuses any file-write path that doesn't pass through the envelope-encryption pipeline.
- **New CI lane**: `oya-check-dek-binding-integrity` (BLOCKER) — validates ciphertext binding-tuple integrity.
- **New CI lane**: `oya-check-secret-no-log` (BLOCKER) — refuses any code path that could log a key.
- **OpenBao Transit deployment**: shared per-pack with `cloud-secrets` µservice; not a drive-µservice-internal resource.

### Regulatory

- **HIPAA 45 CFR §164.312(a)(2)(iv)** — encryption controls satisfied.
- **GDPR Art. 32(1)(a)** — pseudonymisation + encryption satisfied.
- **KR PIPA Art. 29** — technical safeguards satisfied.
- **FIPS 140-3** — Level 2 module via OpenBao Transit + HSM backing.
- **NIST SP 800-57** — key management lifecycle (generate / distribute / use / revoke / archive / destroy) implemented.
- **EU Schrems II supplementary measure** — encryption at the processor level (oyatie) satisfies the supplementary-measure requirement for non-adequate-country transfer (when SCCs apply).

## Verification

- [ ] AES-256-GCM KAT test vectors — `cargo nextest run -p oya-drive-file-store-domain -- aes_256_gcm_kat`.
- [ ] Ciphertext binding integrity — `cargo nextest run -p oya-drive-file-store-domain -- ciphertext_binding_integrity`.
- [ ] DEK rotation `rewrap` — `cargo nextest run -p oya-drive-file-store-domain -- dek_rotation_rewrap`.
- [ ] libsodium secretstream KAT — `cargo nextest run -p oya-drive-file-store-domain -- libsodium_kat`.
- [ ] E2E end-to-end flow — `cargo nextest run --test e2e_e2e_personal_pillar`.
- [ ] FIPS 140-3 conformance verification — `cargo run -p oya-dev-cli -- gate validate fips-140-3 --microservice drive`.

## References

- NIST SP 800-38D — AES-GCM.
- NIST SP 800-57 — Recommendation for Key Management.
- FIPS 140-3 — Security Requirements for Cryptographic Modules.
- RFC 8439 — ChaCha20-Poly1305.
- RFC 9106 — Argon2 (used for client-side passphrase derivation).
- libsodium upstream — `libsodium.gitbook.io`.
- OpenBao upstream — `openbao.org`.
- Proton Drive whitepaper — libsodium secretstream reference.
- Tresorit security whitepaper.
- ADR-0028 (Bominal) — audit chain.
- ADR-0117 — cloud-native infrastructure / data residency.
- ADR-0140 — Cedar policy enforcement.
- ADR-DRIVE-0001 — object-storage substrate (ciphertext bytes land on Garage/MinIO/SeaweedFS).
- ADR-DRIVE-0003 — share-link signing (Ed25519 + Argon2id; shared OpenBao Transit dependency).
- ADR-DRIVE-0006 — WORM (envelope ciphertext is what's WORM'd).
- `microservices/drive/PRD.md` §Security; §FR-19 E2E.
- `microservices/drive/threat-model.md` T-T-02 + T-I-06.
- `microservices/drive/policy/tenant-scope.cedar`.
