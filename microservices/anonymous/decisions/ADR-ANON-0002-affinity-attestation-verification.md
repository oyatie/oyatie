---
id: ADR-ANON-0002
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, council-architecture, council-privacy, ops-security
owner: axis-anonymous + ops-security
supersedes: []
superseded_by: []
related:
  - ADR-ANON-0001
  - ADR-ANON-0007
  - ADR-0056
  - ADR-0131
related_artifacts:
  - microservices/anonymous/PRD.md (I2, FR-16, FR-22, FR-23)
  - microservices/anonymous/policy/affinity-attestation-verification.md
  - microservices/anonymous/runbooks/affinity-attestation-key-rotation.md
  - microservices/anonymous/IP-006-affinity-attestation-bc.md
purpose: |
  Select the canonical flow by which the platform verifies a user's affinity
  (employer / education / geographic / workspace) WITHOUT learning the user's
  identity. Anchors PRD invariant I2.
---

# ADR-ANON-0002: Affinity-attestation verification — BBS+ selective-disclosure (primary) + OIDC + blinding-proxy (fallback for legacy IdPs)

## Status

Accepted — 2026-05-17.

## Context

PRD I2 asserts that affinity attestation reveals the *affinity* (employer / education / geographic region / workspace) but **never** the identity. The decision is how to verify a user's affinity claim cryptographically without the platform learning who the user is.

The realistic universe of issuer types splits into two camps:

- **BBS+-native IdPs** (Microsoft Entra Verified ID, Hyperledger Aries Cloud Agent, MATTR Pi, IBM-Indy-based university IdPs in 2026): issue W3C VC 2.0 BBS+ credentials directly; user's wallet generates selective-disclosure proofs; platform verifies without learning identity.
- **Legacy enterprise IdPs** (Okta, Auth0, Microsoft Entra ID classic, on-prem ADFS, SAML-only IdPs): issue OIDC ID tokens or SAML assertions containing a subject claim; do NOT support BBS+ credential issuance natively.

The decision:

1. **Primary path (BBS+-native)**: user → IdP → user holds BBS+ VC → user generates selective-disclosure proof revealing only affinity attributes → platform verifies. No platform-side identity exposure ever.
2. **Fallback path (legacy IdPs)**: user → legacy IdP → OIDC ID token / SAML assertion → platform's **blinding-proxy** consumes the ID token → blinding-proxy issues BBS+ credential to user → discards OIDC subject claim → emits audit-chain event documenting the discard.

The fallback path is a structural weakness (the blinding-proxy briefly sees the OIDC subject); we mitigate with short-lived in-memory handling, audit-chain seal of discard, and a documented transparency annotation in the user's signup flow.

## Decision

The anonymous µservice adopts a **two-path verification model**:

1. **Primary (preferred) path**: BBS+ selective-disclosure per ADR-ANON-0001. Platform verifies the BBS+ proof against the registered issuer's public key. Identity attributes are NEVER disclosed.

2. **Fallback path (only for IdPs not BBS+-capable)**: OIDC + blinding-proxy. The blinding-proxy:
   - is the only platform component that ever sees the OIDC `sub` claim;
   - is implemented under FIPS 140-3 Level 3 air-gapped enclave constraints (no logging of `sub`; no persistence beyond proxy memory);
   - immediately issues a fresh BBS+ credential to the user under the platform's affinity-issuer signing key;
   - immediately emits an `OIDCSubjectDiscarded` audit-chain seal;
   - retains zero state about the OIDC subject after issuance.

3. **k-anonymity floor enforcement** is at the verification step per ADR-ANON-0007 (k=50 geo / k=20 employer / k=10 small-employer fallback).

4. **Identity-attribute discard**: even on the BBS+ primary path, if the verifier receives identity attributes (e.g., a misconfigured SDK reveals more than intended), the platform discards the attribute at the verifier layer and emits an `IdentityAttributeReceivedAndDiscarded` audit-chain event. This is defence-in-depth, not the primary mechanism.

5. **Issuer registry**: registered issuers (employer / edu / geo) are recorded in the `AffinityIssuerRegistry` Postgres table. Registration is an out-of-band onboarding flow owned by `ops-security + general-counsel`. Key rotation per `runbooks/affinity-attestation-key-rotation.md`.

## Alternatives Considered

### A. BBS+ only (refuse legacy IdPs)

- **Pros**: Cleanest cryptographic posture; no blinding-proxy weakness; structural I2 guarantee in all cases.
- **Cons**: Excludes the ~95% of enterprise IdPs that are not BBS+-native in 2026 (Okta, on-prem ADFS, Auth0, Microsoft Entra ID classic, etc.). Effectively unusable for the Blind-class employer-affinity market.
- **Rejected because**: Product viability requires supporting legacy IdPs at scale. Pure BBS+ is a 5-year roadmap goal.

### B. OIDC only (no BBS+)

- **Pros**: Compatibility with every enterprise IdP.
- **Cons**: Platform sees the OIDC subject claim at every authentication; defeats I2 structurally; Blind-precedent failure mode.
- **Rejected because**: Defeats the µservice's defining invariant.

### C. SAML-only (without blinding-proxy)

- **Pros**: Some enterprise IdPs only support SAML.
- **Cons**: SAML assertions include `NameID` (identity); same I2 failure as plain OIDC.
- **Rejected because**: Same as B.

### D. BBS+ primary + plain OIDC fallback (no blinding-proxy)

- **Pros**: Simpler than blinding-proxy.
- **Cons**: Platform retains OIDC subject claim per session; I2 weakened; user-visible privacy claim materially weaker.
- **Rejected because**: We want the I2 claim to hold across all paths.

### E. BBS+ primary + token-binding fallback (RFC 8471)

- **Pros**: Conceptually elegant token-binding-based separation.
- **Cons**: RFC 8471 is effectively abandoned (Chrome removed support in 2018); poor ecosystem support; would be a maintenance dead-end.
- **Rejected because**: Industry has moved away from token-binding.

## Consequences

### Positive

- **I2 invariant preserved on the primary path** (BBS+ selective-disclosure: platform never sees identity).
- **I2 invariant approximately preserved on the fallback path** (blinding-proxy sees subject for sub-second window; discards; audit-chain seal).
- **Coverage**: ~100% of enterprise IdPs supported (BBS+-native + legacy OIDC + SAML).
- **Standards alignment**: W3C VC 2.0 + OIDC Core 1.0 + SAML 2.0.
- **k-anonymity floor enforced** per ADR-ANON-0007 at verification.

### Negative

- **Fallback path is a structural weakness**. Mitigated: blinding-proxy is documented; in-memory only; audit-chain seal of discard; transparency to user at signup; FIPS 140-3 Level 3 enclave constraints.
- **Issuer onboarding is manual** (ops-security + general-counsel verification of issuer identity). Cannot be self-serve until issuer trust framework matures (W3C did:web or similar).
- **Two implementation paths** = more maintenance burden. Mitigated: paths share the verifier core; only the issuance frontend differs.

### Operational

- IP-006 (`IP-006-affinity-attestation-bc.md`) implements both paths.
- `runbooks/affinity-attestation-key-rotation.md` covers issuer-key rotation.
- `runbooks/employer-affinity-employer-domain-takeover.md` covers issuer change-of-ownership.
- LEAN lane `oya-check-identity-attribute-discard` enforces the audit-chain seal on every received-and-discarded attribute.

### Regulatory

- **GDPR Art. 11 + Recital 26 pseudonymisation**: BBS+ primary path is the canonical implementation; fallback path with documented blinding-proxy is acceptable under Recital 26 with risk-reduction measures.
- **KR PIPA Art. 24-2**: alternative-pseudonymous-processing satisfied.
- **APPI Art. 18 (purpose-limitation)**: affinity-attestation purpose disclosed at issuer-registration.
- **EU AI Act**: not triggered (no AI in this BC).

### Invariant Preservation

This decision flows **I2** through both paths. The fallback path is a deliberate weakening with documented mitigation; future ADR may strengthen it as the BBS+-native IdP ecosystem matures.

## References

- W3C Verifiable Credentials Data Model 2.0 — `https://www.w3.org/TR/vc-data-model-2.0/`
- OpenID Connect Core 1.0 — `https://openid.net/specs/openid-connect-core-1_0.html`
- SAML 2.0 Core — `https://docs.oasis-open.org/security/saml/v2.0/`
- IRTF CFRG `draft-irtf-cfrg-bbs-signatures`
- Microsoft Entra Verified ID (deployer)
- Hyperledger Aries Cloud Agent (deployer)
- ADR-ANON-0001 (cryptographic-blinding protocol — the BBS+ foundation)
- ADR-ANON-0007 (affinity-cluster k-anonymity floor)
- RFC 6749 (OAuth 2.0); RFC 7519 (JWT); SAML 2.0
- GDPR Recital 26 (pseudonymisation definition)
- KR PIPA Art. 24-2 (alternative-pseudonymous-processing)
