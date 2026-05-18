---
doc_class: PolicyDocument
template_id: TPL-POLICY-MD
title: Affinity Attestation Verification — Cryptographic Protocol Detail
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-security + council-privacy
related_adrs: [ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0007]
related_artifacts:
  - microservices/anonymous/PRD.md §"Design Invariants" (I2)
  - microservices/anonymous/policy/tenant-scope.cedar
doc_status: published
---

# Affinity Attestation Verification — Cryptographic Protocol Detail

This document is the canonical operational specification for how the `anonymous` µservice verifies that a user is bound to an affinity (employer, university, geographic region, workspace) WITHOUT learning the user's identity. It implements **PRD invariant I2** under the protocol selected in **ADR-ANON-0002**.

## Protocol Summary

The platform uses **BBS+ signatures with selective disclosure** (W3C Verifiable Credentials 2.0; IRTF CFRG `draft-irtf-cfrg-bbs-signatures`). The flow is:

```
Affinity Issuer (employer IdP / university IdP / geo issuer)
       │
       │  1. (out-of-band) Issuer registers signing public key with platform
       │     and provides issuer-policy (e.g., "I attest @bominal.com email holders")
       │
       │  2. End-user authenticates with issuer (corporate SSO / SAML / OIDC)
       │
       │  3. Issuer issues a BBS+ Verifiable Credential to end-user with attributes:
       │       - affinity_kind   (employer | edu | geo | workspace)
       │       - affinity_scope  (e.g., "bominal.com")
       │       - issued_at
       │       - expires_at
       │       - cluster_id      (which affinity cluster on platform)
       │     (Identity attributes — name, email — are present in the credential
       │      but the user controls disclosure.)
       │
       ▼
End-user (client SDK)
       │
       │  4. End-user generates a BBS+ selective-disclosure proof revealing
       │     ONLY the affinity_* + cluster_id attributes; identity attributes
       │     are NOT disclosed.
       │
       │  5. End-user submits the proof to the platform's
       │     `POST /v1/affinity-attestations:verify` endpoint.
       │
       ▼
Platform (oya-anonymous-affinity-attestation-usecase)
       │
       │  6. Platform verifies the proof under the issuer's public key. Verify
       │     succeeds iff:
       │       - signature is valid;
       │       - revealed attributes match the issuer's declared schema;
       │       - credential is not expired;
       │       - cluster_id resolves to an existing affinity cluster on platform;
       │       - k-anonymity floor for that cluster is met (ADR-ANON-0007).
       │
       │  7. Platform records an AffinityAttestationBinding ontology entity:
       │       binding_id, affinity_ref (cluster_id), blinded_commitment,
       │       established_at, expires_at.
       │     The blinded_commitment is a fresh cryptographic commitment from
       │     ADR-ANON-0001 — it is structurally NOT linked to the credential.
       │
       │  8. Platform emits an AffinityAttestationBound event to audit-chain.
       │
       ▼
End-user now has an affinity-bound blinded credential to use for posting.
```

## Cryptographic Building Blocks

| Component | Algorithm | Library (pinned) | FIPS 140-3 boundary |
|---|---|---|---|
| Affinity credential signature | BBS+ over BLS12-381 | `rust-bls 0.5.x` (per ADR-ANON-0001) | Yes (in FIPS-validated module path) |
| Issuer key registration | Ed25519 | `ring 0.17` | Yes |
| Blinded commitment | Pedersen / Camenisch-Lysyanskaya (per ADR-ANON-0001) | `rust-bls 0.5.x` | Yes |
| Issuer-to-platform transport | mTLS 1.3 | `rustls 0.23` | Yes |
| Selective-disclosure proof | BBS+ ZK-proof | `rust-bls 0.5.x` | Yes |

## Verification API

```
POST /v1/affinity-attestations:verify
Content-Type: application/json

{
  "issuer_key_id": "<issuer-public-key-fingerprint>",
  "credential_format": "bbs-plus-vc-2.0",
  "selective_disclosure_proof": "<base64-bytes>",
  "revealed_attributes": {
    "affinity_kind": "employer",
    "affinity_scope": "bominal.com",
    "cluster_id": "<opaque-cluster-id>",
    "issued_at": "<rfc3339>",
    "expires_at": "<rfc3339>"
  }
}
```

Response (success):

```json
{
  "binding_id": "<opaque>",
  "blinded_commitment": "<bbs-plus-commitment-bytes>",
  "affinity_cluster_id": "<opaque-cluster-id>",
  "expires_at": "<rfc3339>",
  "audit_chain_seal_hash": "<merkle-root>"
}
```

Response (failure):

```json
{
  "error": "verification_failed",
  "code": "BBS_VERIFY_FAILED | EXPIRED | UNKNOWN_ISSUER | K_ANONYMITY_FLOOR_NOT_MET | CLUSTER_NOT_FOUND",
  "diagnostic": "<safe-message-with-no-credential-bytes>"
}
```

## Performance budget

| Step | p50 | p95 | p99 |
|---|---|---|---|
| BBS+ verify (cryptographic) | ≤ 80 ms | ≤ 200 ms | ≤ 350 ms |
| Issuer-key lookup (cached) | ≤ 5 ms | ≤ 20 ms | ≤ 50 ms |
| k-anonymity floor query | ≤ 10 ms | ≤ 30 ms | ≤ 60 ms |
| Audit-chain seal | ≤ 30 ms | ≤ 80 ms | ≤ 200 ms |
| **End-to-end attestation-verify** | **≤ 150 ms** | **≤ 500 ms** | **≤ 1 s** |

(Matches PRD §"Performance" affinity-attestation-verify-latency row.)

## Issuer Registration

Issuers (employer IdP, university IdP, geo-issuer service) register their signing public keys with the platform through an **out-of-band onboarding flow** owned by `ops-security + general-counsel`:

1. Issuer submits public key + attestation policy (which affinity_kind + affinity_scope they may attest) signed under a one-time-bootstrap key.
2. `ops-security` verifies the issuer's identity through a documented manual flow (notarised company-domain attestation OR ICANN registrar verification OR education-issuer national registry).
3. Issuer is recorded in `AffinityIssuerRegistry` (Postgres) with audit-chain seal.
4. Issuer-key rotation: see `runbooks/affinity-attestation-key-rotation.md`.

## Invariant Enforcement

- **I2 (affinity-not-identity)**: the verification endpoint NEVER stores the credential's identity attributes (name, email). The SDK is required to call BBS+ selective-disclosure with identity attributes hidden; if the SDK reveals them, the platform discards them at the verifier layer and seals an `IdentityAttributeReceivedAndDiscarded` audit-chain event for compliance evidence.
- **I7 (legal-process-only correlation)**: the binding (`binding_id, affinity_ref, blinded_commitment`) is structurally NOT correlatable to any `user_id`. Only the legal-process disclosure workflow (ADR-ANON-0003) under court order can perform that correlation.
- **k-anonymity floor (ADR-ANON-0007)**: a binding into a cluster with cardinality below the configured k-floor (k=50 geo / k=20 employer / k=10 small-employer fallback) is **refused**. The cluster must grow before bindings are accepted; the verifier surfaces this error explicitly.

## OIDC + Blinding (alternative path; PRD open question 2)

For corporate IdPs that do NOT issue BBS+ credentials (most enterprise SAML / OIDC IdPs), an alternative path is supported (per PRD §"Open Questions" Q2): the user authenticates via OIDC to the corporate IdP, and a platform-internal **blinding proxy** issues a BBS+ credential after consuming the OIDC ID token. The blinding proxy is the only platform component that ever sees the OIDC subject claim; it discards the subject claim after issuance and emits an `OIDCSubjectDiscarded` audit-chain event.

This path is documented in ADR-ANON-0002 §"OIDC + Blinding Proxy". The cryptographic guarantee is weaker than the pure-BBS+ path because the platform's blinding proxy briefly sees the OIDC subject — mitigated by short-lived in-memory handling, no logging, and audit-chain seal of the discard.

## References

- ADR-ANON-0001 (cryptographic-blinding protocol)
- ADR-ANON-0002 (affinity-attestation verification)
- ADR-ANON-0007 (affinity-cluster k-anonymity floor)
- W3C Verifiable Credentials Data Model 2.0 — https://www.w3.org/TR/vc-data-model-2.0/
- IRTF CFRG `draft-irtf-cfrg-bbs-signatures`
- NIST SP 800-186 (elliptic-curve parameters)
- FIPS 140-3 (Federal Information Processing Standard for cryptographic modules)
- BLS12-381 — IETF `draft-irtf-cfrg-pairing-friendly-curves`
