---
id: ADR-identity-002
scope: microservice
microservice: identity
status: Accepted
date: 2026-05-18
owner: axis-identity + ops-security
related: [ADR-0188]
---

# ADR-identity-002 — Passkey attestation policy per pack tier

## Context

WebAuthn attestation conveyance (`none` / `indirect` / `direct`) signals how much the relying party trusts the authenticator's manufacturer claims. Stricter = better identity guarantees + more friction.

## Decision

| Pack tier | Attestation | AAGUID allowlist |
|---|---|---|
| sandbox / dev | None | not enforced |
| Pack-standard (us, jp, sg, au, in, br) | Indirect | not enforced |
| Pack-regulated (kr, eu, us-healthcare, ae, ksa) | Direct | enforced (FIDO-MDS3 L1+) |
| `acr=critical` operations | Direct | enforced (FIDO-MDS3 L2+) |

## Consequences

- Regulated packs refuse novel/unattested authenticators.
- AAGUID allowlist refreshed every 24h via FIDO-MDS3 worker.
- Users in regulated packs may need to choose from a specific authenticator allowlist.
