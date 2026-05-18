---
adr_id: finops-portal-007
authored: 2026-05-18
status: accepted
authority_chain: ADR-0162
microservice: finops-portal
---

# ADR finops-portal-007 — Ed25519 quarterly signing key

## Context

Quarterly regulator-evidence envelopes (IP-015) require a signing
scheme that is:

- Cryptographically strong (modern, not broken).
- Verifiable by external auditors with no special tooling.
- Rotatable on a regular cadence without invalidating prior
  signatures.
- Suitable for HSM custody.

## Decision

Sign quarterly envelopes with **Ed25519**. One key pair per
quarter; key pairs published to audit-chain at quarter-start
(class `FinOpsQuarterlyKeyPublished`); private key held in the
per-pack HSM.

## Rationale

1. Ed25519 is widely supported (every modern crypto library, every
   modern CLI tool); auditors can verify with `openssl`,
   `python-cryptography`, or any other standard tool.
2. Ed25519 signatures are 64 bytes (compact for envelope storage).
3. Per-quarter key rotation limits blast radius if a key is
   compromised; old signatures remain verifiable against the
   published public key.
4. HSM-custodied private key prevents accidental leakage.

## Consequences

- The HSM provisioning is a dependency on the secrets µservice.
- Per-pack HSMs (KR, EU, US-healthcare, US-financial, US-public-
  sector) each hold their own key for residency reasons.
- Audit-chain class `FinOpsQuarterlyKeyPublished` is part of the
  manifest seal_events list.
- Verification procedure documented in `compliance-matrix.md`.

## Alternatives considered

- **RSA-4096**: rejected because signatures are 512 bytes (larger);
  Ed25519 is also faster.
- **ECDSA-P256**: acceptable, but Ed25519's deterministic signature
  property simplifies replay-verification.
- **HMAC**: rejected — symmetric; auditor cannot independently
  verify without holding the secret.

## References

- ADR-0162 audit-log integrity.
- IP-015 quarterly emit.
- `compliance-matrix.md`.
- `threat-model.md`.
