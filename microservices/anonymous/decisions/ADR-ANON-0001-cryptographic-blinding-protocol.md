---
id: ADR-ANON-0001
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, council-architecture, council-privacy, ops-security
owner: axis-anonymous + ops-security
supersedes: []
superseded_by: []
related:
  - ADR-0028
  - ADR-0056
  - ADR-0105
  - ADR-0126
  - ADR-0131
related_artifacts:
  - microservices/anonymous/PRD.md (I1, FR-01, FR-02)
  - microservices/anonymous/policy/affinity-attestation-verification.md
  - microservices/anonymous/runbooks/blind-signature-key-ceremony.md
  - microservices/anonymous/IP-007-blind-signatures-crypto.md
  - microservices/anonymous/catalog/oya-anonymous-blind-signatures-kernel.yaml
purpose: |
  Select the cryptographic blinding protocol that anchors PRD invariant I1
  ("the platform cannot correlate user_id ↔ post_id outside the legal-process
  workflow"). Without this decision, no other crate in the µservice can ship.
---

# ADR-ANON-0001: Cryptographic-blinding protocol — BBS+ over BLS12-381 with Camenisch-Lysyanskaya commitments under FIPS 140-3 Level 3 air-gapped HSM ceremony

## Status

Accepted — 2026-05-17.

## Context

PRD I1 asserts that the platform **cannot** structurally correlate `user_id ↔ post_id` outside the legal-process workflow. This is the load-bearing invariant of the µservice; without a cryptographic mechanism, "anonymous" is a marketing claim rather than a structural property (as Whisper, Blind, and YikYak have learned to their regulatory cost).

The decision space:

1. **Anonymous credentials protocol** — what signs the user's per-session credential?
2. **Commitment scheme** — what scheme is used at the platform side to bind a post to a blinded commitment without learning the user's identifier?
3. **Cryptographic curve** — what underlies the signature + commitment math?
4. **Key-ceremony posture** — how are signing keys generated and rotated?
5. **FIPS validation** — what FIPS 140-3 boundary applies?

Four competing protocols were evaluated:

- **RSA-PSS-blind** (Chaum 1982, IRTF CFRG `draft-irtf-cfrg-rsa-blind-signatures`): mature, conceptually simple, but signature size large, no selective disclosure
- **Schnorr-blind** (Schnorr 1991; revisited Hauck et al. 2019): smaller signatures than RSA-PSS; conceptually elegant; selective disclosure not native
- **Camenisch-Lysyanskaya (CL) anonymous credentials** (CL 2002; refined Idemix at IBM): rich selective-disclosure semantics; integrates with W3C VC 2.0; computationally heavier
- **BBS+ signatures** (Au et al. 2006; refined IRTF CFRG `draft-irtf-cfrg-bbs-signatures`): pairing-based; supports selective disclosure of credential attributes; standardised by W3C VC 2.0; growing adoption (Microsoft Entra Verified ID, Hyperledger Indy, MATTR)

Competing forces:

- **Selective-disclosure semantics** — BBS+ and CL natively support revealing only some attributes (e.g., "affinity = bominal-employer" without revealing identity attributes). RSA-PSS-blind and Schnorr-blind do not.
- **Curve choice** — BLS12-381 is widely deployed (Ethereum, Zcash, IETF `pairing-friendly-curves`); FIPS 140-3 Level 3 modules are available (Thales Luna HSM 7.x, Entrust nShield); known plaintext + ciphertext security at 128-bit level.
- **Performance** — BBS+ verify is ~80ms on commodity hardware; CL is ~150ms; RSA-PSS-blind is ~30ms; Schnorr-blind is ~20ms. The performance gap is acceptable given PRD affinity-attestation-verify p95 budget of 500ms.
- **Standards adoption** — W3C VC 2.0 standardises BBS+ as the selective-disclosure credential format; aligning with VC 2.0 lets oyatie inherit a deep tooling ecosystem.
- **FIPS-validated implementations** — rust-bls 0.5 is in active FIPS validation track; ring 0.17 already FIPS-validated for Ed25519 (which we use for issuer-key registration but not for the blind-signature itself).

## Decision

The anonymous µservice adopts:

1. **Protocol**: BBS+ signatures over BLS12-381 for the per-session blind credentials.
2. **Commitment scheme**: Camenisch-Lysyanskaya commitments at the platform side. The platform sees only `Commit(post, blinding_nonce)`, never the post-author identifier.
3. **Curve**: BLS12-381 (IETF `draft-irtf-cfrg-pairing-friendly-curves`).
4. **Library**: `rust-bls 0.5` (default; FIPS 140-3 Level 3 in air-gapped HSM ceremony path) + `ring 0.17` (Ed25519 issuer-key registration; FIPS 140-3 Level 1). Two adapters behind feature flags.
5. **Key ceremony**: Air-gapped HSM (Thales Luna 7.x or Entrust nShield) ceremony; Shamir 3-of-5 secret-sharing; per-pack signing-key set; 24-month planned rotation (per ADR-ANON-0001 §"Rotation"); per `runbooks/blind-signature-key-ceremony.md`.
6. **FIPS boundary**: Level 3 (tamper-evident + role-based authentication) for production signing-key set; Level 1 for issuer-key registration (Ed25519 only).
7. **Standards alignment**: W3C Verifiable Credentials Data Model 2.0; IRTF CFRG `draft-irtf-cfrg-bbs-signatures`.

The decision is canonical across all 12 regulatory packs. Pack-specific overlays affect deployment (region) but never the protocol.

## Alternatives Considered

### A. RSA-PSS-blind (IRTF CFRG `draft-irtf-cfrg-rsa-blind-signatures`)

- **Pros**: Mature (Chaum 1982); widely understood; FIPS-validated implementations available (OpenSSL FIPS); fastest verify (~30ms).
- **Cons**: No native selective disclosure (would require ad hoc wrapper protocol that hasn't been peer-reviewed); large signature size (256-512 bytes); no W3C VC 2.0 alignment.
- **Rejected because**: The selective-disclosure feature is load-bearing for I2 (affinity-attestation reveals only affinity, not identity). Adding it as an ad hoc wrapper on top of RSA-PSS would create an unreviewable surface; we'd be the only deployer of such a wrapper, and the cryptographic-research community could not vet it.

### B. Schnorr-blind (Schnorr 1991; Hauck et al. 2019)

- **Pros**: Smaller signatures than RSA-PSS (64 bytes); conceptually elegant; faster verify (~20ms); no patent encumbrance.
- **Cons**: Same selective-disclosure gap as RSA-PSS; less library maturity for selective-disclosure extensions; not aligned with W3C VC 2.0.
- **Rejected because**: Same reason as A — the selective-disclosure gap is structural.

### C. Camenisch-Lysyanskaya (CL) anonymous credentials (CL 2002; Idemix at IBM)

- **Pros**: Rich selective-disclosure semantics; mature theoretical foundation (Camenisch-Lysyanskaya signature scheme is foundational in anonymous-credentials literature); Hyperledger Indy ecosystem uses CL; integrates with W3C VC 2.0.
- **Cons**: Computationally heavier than BBS+ (~150ms verify); less performant; fewer FIPS-validated implementation tracks; Hyperledger Indy is the primary deployer and is shrinking in favour of BBS+.
- **Rejected because**: BBS+ is the strict performance + ecosystem winner among selective-disclosure-capable schemes. CL would be a fine choice in 2018; in 2026, BBS+ has caught up + overtaken.

### D. Schnorr-blind + a custom selective-disclosure ZK-proof wrapper

- **Pros**: Could in principle achieve BBS+-like semantics with smaller signature size.
- **Cons**: Custom unreviewed protocol; long-term maintenance burden; auditor friction (the wrapper would need its own security proof + cryptographic review).
- **Rejected because**: Engineering team cannot publish + maintain its own cryptographic protocol at audit-grade quality.

### E. CL-only with BLS12-381 (skip BBS+)

- **Pros**: Single protocol; simpler key-ceremony.
- **Cons**: Worse performance; smaller library ecosystem; lower W3C VC 2.0 momentum.
- **Rejected because**: BBS+ is now the de facto W3C VC 2.0 selective-disclosure credential format; using CL alone would be against the standardisation grain.

## Consequences

### Positive

- **I1 invariant structurally satisfied.** The Postgres `posts` table contains `blinded_commitment` bytes; nothing in the platform's state correlates a commitment to a user identifier. Whisper-precedent breach is structurally impossible.
- **I2 invariant flowing.** BBS+ selective-disclosure lets the platform learn `affinity_kind` + `affinity_scope` + `cluster_id` while NOT learning identity attributes (name, email).
- **Standards alignment.** W3C VC 2.0 + IRTF CFRG alignment lets oyatie inherit ecosystem tooling (SDKs, validators, conformance suites).
- **FIPS 140-3 Level 3 path.** Air-gapped HSM ceremony + Shamir 3-of-5 share-splitting is industry-precedent (CA root-key ceremony pattern); auditors recognise the posture.
- **Audit-grade key rotation.** 24-month planned rotation + compromise-driven rotation procedures documented in `runbooks/blind-signature-key-ceremony.md`.

### Negative

- **Higher computational cost** than RSA-PSS-blind (~80ms vs ~30ms). Mitigated: still well inside PRD affinity-attestation-verify p95 budget (≤500ms).
- **Pairing-based cryptography is younger** than RSA. Mitigated: BLS12-381 is deployed at scale by Ethereum + Zcash; cryptanalytic community attention is substantial; no known structural weakness.
- **rust-bls 0.5 is still in active FIPS-validation track.** Mitigated: until fully validated, FIPS 140-3 Level 3 is achieved through air-gapped HSM (Thales/Entrust); the rust-bls library is used outside the FIPS boundary only for verify (which does not require validated implementation per FIPS rules).

### Operational

- Key-ceremony runbook (`runbooks/blind-signature-key-ceremony.md`) authored.
- HSM procurement: Thales Luna 7.x or Entrust nShield per pack.
- Per-pack signing-key set; 24-month rotation cadence; compromise-rotation playbook.
- IP-007 (`IP-007-blind-signatures-crypto.md`) implements the crate stack.
- LEAN lane `oya-check-blinding-column-isolation` enforces `posts` table has no `user_id` column.

### Regulatory

- **GDPR Art. 11 + Recital 26 pseudonymisation**: BBS+ + CL commitment IS the canonical pseudonymisation; the platform cannot identify data subjects without the legal-process workflow.
- **KR PIPA Art. 24-2 alternative-pseudonymous-processing**: BBS+ commitment is the canonical alternative-identifier (대체수단).
- **FIPS 140-3 Level 3**: required for federal-procurement-track tenants.
- **NIST SP 800-186 + SP 800-57**: elliptic-curve parameter selection and key-management aligned.
- **W3C VC 2.0 + IRTF CFRG**: conformance tests available; competitive tooling.

### Invariant Preservation Summary

This decision is the structural anchor for **I1** and (with ADR-ANON-0002) **I2**. A change to this protocol would require a superseding ADR + Council Privacy approval + 12-month migration plan.

## References

- IRTF CFRG `draft-irtf-cfrg-bbs-signatures` — `https://datatracker.ietf.org/doc/draft-irtf-cfrg-bbs-signatures/`
- W3C Verifiable Credentials Data Model 2.0 — `https://www.w3.org/TR/vc-data-model-2.0/`
- Au, M. H., Susilo, W., Mu, Y. (2006). "Constant-Size Dynamic k-TAA". SCN 2006.
- Camenisch, J., Lysyanskaya, A. (2002). "A Signature Scheme with Efficient Protocols". SCN 2002.
- Chaum, D. (1982). "Blind Signatures for Untraceable Payments". CRYPTO 1982.
- Hauck, E., Kiltz, E., Loss, J. (2019). "A Modular Treatment of Blind Signatures from Identification Schemes". EUROCRYPT 2019.
- Schnorr, C. P. (1991). "Efficient Signature Generation by Smart Cards". J. Cryptology.
- NIST SP 800-186 (Recommendations for Discrete Logarithm-Based Cryptography: Elliptic-Curve Domain Parameters)
- NIST SP 800-57 Part 1 (Key Management)
- FIPS 140-3 (Security Requirements for Cryptographic Modules)
- IETF `draft-irtf-cfrg-pairing-friendly-curves` (BLS12-381)
- Microsoft Entra Verified ID (BBS+ deployer reference)
- Hyperledger Indy + Aries (CL + BBS+ deployer reference)
- MATTR (BBS+ tooling vendor reference)
