---
id: ADR-0188
status: Superseded
deciders: council-architecture, axis-identity, ops-security
date: 2026-05-18
owner: axis-identity
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0145, ADR-0187, ADR-0189]
related_specs:
  - /specs/microservices/manifest-schema.json
  - /specs/regulatory-identity-kyc-policy-evidence-architecture.json
microservice: identity
versions_current_as_of: 2026-05-18
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0188 — Passkey / WebAuthn Level-3 substrate; phishing-resistant first-factor; TOTP fallback; SMS rejected

## Status

Accepted (2026-05-18). Mandates WebAuthn Level 3 (W3C Recommendation, Apr 2024) as the canonical strong-authentication substrate. Passkey (synced + device-bound) is the primary credential. TOTP (RFC 6238) is the only sanctioned fallback. SMS one-time-codes are forbidden per NIST SP 800-63B §5.1.3.[^1][^2]

## Context

OIDC password authentication is broken at scale: 81% of breaches involve weak or reused passwords (Verizon DBIR 2024), and phishing kits defeat traditional MFA (TOTP, SMS, push) by harvesting the second factor in real time. The hyperscaler bar has shifted: Apple, Google, Microsoft, Cloudflare, GitHub, and Stripe ship Passkey-first sign-in to consumers and Passkey-mandatory to enterprises in 2025-2026.

WebAuthn Level 3 promotes "Passkey" (multi-device synced credentials) and "cross-device sign-in" (caBLE: Cloud-Assisted BLE Endpoint) to first-class status, alongside the original device-bound model. Conditional UI (autofill, no explicit user-click) makes Passkey UX equivalent to password autofill on supported browsers.

This ADR sits inside `identity` µservice and is consumed by Zitadel (ADR-0187) as the relying-party WebAuthn implementation.
It also governs the REGID-002 planning artifact `specs/regulatory-identity-kyc-policy-evidence-architecture.json`, which maps passkey-first authentication into KYC/CDD/EDD policy-evidence architecture without adding a service, runtime handler, or CLI surface. WebAuthn Level 3 remains the normative implementation floor; the W3C WebAuthn Level 4 "First Published Working Draft" milestone is tracked as a non-blocking horizon for items such as immediate mediation UI, alternative error codes, sign extensions, CTAP2.3 virtual-authenticator versioning, and post-quantum/ML-DSA test-vector work, but no Oyatie policy or readiness claim may depend on Level 4 until the feature is standardized and implemented by supported clients/authenticators.

## Decision

**WebAuthn Level 3 is the canonical strong-authentication substrate. Passkeys are the primary credential. The Rust implementation is `webauthn-rs` v0.5+ (kanidm/webauthn-rs).**[^2] TOTP (RFC 6238) is the only sanctioned fallback when Passkey is unavailable. SMS OTP is forbidden.

### Credential ladder

| Tier | Credential | Acceptance | Notes |
|---|---|---|---|
| 1 (preferred) | Passkey (synced via iCloud / Google Password Manager / 1Password / Bitwarden / Dashlane) | universal | resident credential; conditional UI; cross-device caBLE |
| 1 (preferred) | Passkey (device-bound; Windows Hello / Touch ID / Android biometric) | universal | resident credential; AAGUID attested |
| 2 (fallback) | Hardware security key (YubiKey, Titan, NitroKey, SoloKey) | universal | non-resident OR resident; required for `acr=critical` per ADR-0189 |
| 3 (fallback only) | TOTP (RFC 6238 authenticator app) | when device lost or no WebAuthn | NEVER first-factor; only as recovery |
| forbidden | SMS OTP | n/a | NIST SP 800-63B §5.1.3 restricted; SIM-swap blast radius |
| forbidden | Email link as sole factor | n/a | acceptable only for account-recovery init, not steady-state auth |
| forbidden | Security questions | n/a | NIST SP 800-63B §5.1.1.2 restricted |
| forbidden | Push notifications without number-matching | n/a | MFA-fatigue attack vector |

### Browser support matrix

| Browser | Min version | WebAuthn L3 | Conditional UI | caBLE |
|---|---|---|---|---|
| Chrome / Chromium / Edge | 109+ | yes | yes | yes |
| Safari (macOS / iOS / iPadOS) | 16+ | yes | yes | yes |
| Firefox | 122+ | yes | yes | partial (Android only) |
| Samsung Internet | 23+ | yes | yes | yes |

Browsers older than the floor receive password+TOTP fallback (NOT the steady-state path).[^1]

### Credential storage

- Per-tenant Postgres table `webauthn_credentials` (tenant-scoped via RLS + Cedar policy).
- Columns: `credential_id` (binary, primary key), `tenant_id`, `user_id`, `public_key` (CBOR-encoded COSE_Key), `aaguid` (UUID), `transports` (jsonb), `attestation_format` (text), `attestation_object` (binary), `backup_eligible` (bool), `backup_state` (bool), `sign_count` (bigint), `last_used_at` (timestamptz), `created_at` (timestamptz).
- The PRIVATE key NEVER leaves the user's authenticator. The server stores only the public key + metadata.
- Credentials are revoked at logout-everywhere, on credential rotation, and on tenant deletion (cascade per ADR-0175).

### Conditional UI flow (Passkey autofill)

```
1. Browser GET /login
2. Server emits PublicKeyCredentialRequestOptions with `mediation: "conditional"`
3. Browser passively suggests Passkeys in username autofill
4. User selects Passkey → assertion sent to /webauthn/authenticate/finish
5. Server verifies signature, sign-count, AAGUID allowlist, RP-ID
6. On success → mint OIDC session per Zitadel (ADR-0187)
```

### Cross-device sign-in (caBLE)

```
1. Desktop browser shows QR code
2. Mobile authenticator scans, completes ceremony over BLE
3. Bluetooth handshake establishes ephemeral E2E channel
4. Mobile signs challenge; result returns to desktop browser
5. Desktop browser POSTs assertion to /webauthn/authenticate/finish
```

caBLE is required for headless desktops where the user's primary Passkey lives on a mobile device.

### Attestation policy

| Pack tier | Attestation requirement |
|---|---|
| sandbox / dev | `none` accepted (any Passkey) |
| pack-* standard | `direct` or `indirect` (AAGUID-attested authenticator) |
| pack-* regulated (kr, eu-pii, us-healthcare, jp, ksa, ae) | `direct` required + AAGUID allowlist (FIDO-MDS3 certified L1+ only) |
| `acr=critical` operations | `direct` + AAGUID on FIDO-MDS3 L2+ list (YubiKey 5+, Feitian K30+, etc.) |

The FIDO-MDS3 metadata blob is refreshed every 24h by the `webauthn-aaguid-refresher` worker; AAGUID allowlists are pack-policy data.

## Alternatives considered

### Build our own WebAuthn server

Rejected. The cryptographic protocol detail (CBOR, COSE_Key, AAGUID validation, attestation format zoo, conditional UI mediation negotiation) is undifferentiated heavy-lifting. `webauthn-rs` is hardened (used by kanidm production), passes FIDO conformance, and is Apache-2.0/MPL-2.0 dual-licensed.[^2]

### Use a SaaS WebAuthn provider (Hanko, Stytch, Auth0)

Rejected per ADR-0173-vendor-lock-in-avoidance: SaaS dependency for the strong-auth substrate would prevent air-gapped sovereign packs from operating.

### Stay with TOTP-only MFA

Rejected. TOTP is phishable in real time (evilginx2, Modlishka). The hyperscaler bar (Apple Touch ID, Microsoft Hello, Google Passkey-mandatory for Workspace admins) makes WebAuthn the floor.

### Allow SMS as steady-state second factor

Rejected per NIST SP 800-63B §5.1.3 (restricted) and per documented SIM-swap blast radius (Coinbase 2021, FTX 2022 incidents).

## Consequences

### Positive

- Phishing-resistant authentication is the default, not an opt-in.
- Conditional UI eliminates the friction objection to MFA.
- Per-tenant credential isolation aligns with multi-tenant blast-radius posture.
- Open-standard wire protocol preserves credential portability across IdPs (export/import via WebAuthn manage API).

### Negative

- Older devices and air-gapped enterprises without smartphones need YubiKey (hardware) at cost ~$50/user one-time.
- caBLE requires Bluetooth which IT departments sometimes block; document the fallback chain explicitly.
- WebAuthn attestation parsing is complex; we depend on `webauthn-rs` for correctness here.

### Neutral

- Account recovery (lost-all-Passkeys-and-TOTP) escalates to operator-mediated identity-proofing per `identity-account-recovery` runbook.

## Implementation

- `crates/oya-shared-webauthn-server-kernel` — `WebauthnServer` trait + `WebauthnRsServer` impl using `webauthn-rs` v0.5+.[^2]
- Per-tenant Postgres table; SchemaRegistry-versioned (ADR-0166).
- AAGUID allowlist + FIDO-MDS3 refresh worker (`oya-identity-webauthn-aaguid-refresher-worker`).
- HTTP handlers as `axum` routes under `/webauthn/register/{start,finish}`, `/webauthn/authenticate/{start,finish}`.
- Zitadel (ADR-0187) integrates via Zitadel's external IdP API or via embedded relying-party (decision deferred to the IP-013 spike).

## Verification

- `cargo test -p oya-shared-webauthn-server-kernel` — register + authenticate happy path + replay protection + sign-count rollback rejection.
- Lane `lean-a14-webauthn-aaguid-allowlist-policy` (advisory mode) verifies regulated packs declare allowlist.
- Browser conformance: WebAuthn.io test set + virtual authenticator (Chrome DevTools) in CI.

## In-house roadmap

Per user directive 2026-05-18 ("Wherever possible, we should support in-house tech stack — like AWS / Google / Microsoft / Oracle"), this ADR is evaluated under that policy:

- **Protocol**: WebAuthn Level 3 is a W3C **standard** + FIDO2 spec. We DO NOT replace standards; we conform to them. KEEP.
- **Server library**: `webauthn-rs` (kanidm) is OSS commodity (Apache-2.0 / MPL-2.0 dual). Implementing CBOR / COSE / AAGUID validation in-house is undifferentiated heavy-lifting — same posture as TLS or HTTP parsing. KEEP, but isolate the dependency behind `oya-shared-webauthn-server-kernel` trait so a future swap is mechanical.
- **Trigger to consider Phase 2**: ONLY if `webauthn-rs` becomes unmaintained (definition: no upstream commits in 12 months + 3 unanswered CVE reports). Until then, contribute upstream rather than fork.
- **AAGUID metadata source**: FIDO-MDS3 is FIDO Alliance authoritative — KEEP. The refresher worker is in-house code that consumes a public artifact.
- **Storage**: Postgres tables are in-house — KEEP. No vendor here.

Conclusion: this ADR introduces no vendor-replaceable substrate. Phase 0 = Phase 2 for WebAuthn (community standards + OSS commodity); no roadmap delta required.

## Cross-references

- W3C WebAuthn Level 3 (https://www.w3.org/TR/webauthn-3/, Apr 2024)
- FIDO2 spec (CTAP 2.1)
- NIST SP 800-63B (June 2017 base, December 2024 revision)
- RFC 6238 (TOTP)
- ADR-0187 canonical-oidc-idp-zitadel-primary
- ADR-0189 step-up-authentication-acr-classes
- ADR-0145 inter-microservice-communication-reform

[^1]: Versions current as of 2026-05-18. WebAuthn Level 3 W3C Recommendation: https://www.w3.org/TR/webauthn-3/
[^2]: Versions current as of 2026-05-18. webauthn-rs v0.5+ (kanidm/webauthn-rs); https://crates.io/crates/webauthn-rs ; https://github.com/kanidm/webauthn-rs
