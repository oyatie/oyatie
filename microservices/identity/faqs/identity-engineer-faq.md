---
doc_class: FAQ
microservice: identity
persona: identity-engineer + iam-engineer + webauthn-engineer
related_adrs: [ADR-ID-001, ADR-identity-001, ADR-identity-002, ADR-identity-003, ADR-identity-004, ADR-identity-005]
date: 2026-05-20
doc_status: published
---

# Identity Engineer FAQ — identity

## Why passkey-primary instead of password-primary with optional passkey?

Per ADR-ID-001 § Alternatives Considered. Password-primary makes phishing + credential stuffing the leading account-takeover vector. Passkeys are:

1. **Phishing-resistant by construction** (origin-bound; cannot be replayed against attacker-controlled sites).
2. **Hardware-backed where it counts** (YubiKey, Apple Secure Enclave, Android StrongBox).
3. **Standardized** (WebAuthn Level 3 + FIDO2; broad library support).
4. **Mature in 2026** (Apple, Google, Microsoft all support synced passkeys; broad consumer awareness).

For high-risk tenants (HIPAA, FedRAMP, SOX), hardware-bound passkeys + AAGUID attestation prove device class. For consumer + low-risk workforce, synced platform passkeys give Phishing-resistant + UX-friendly authentication.

Passwords remain available ONLY as a tenant-explicit fallback for migration scenarios; never as a default.

## Why 4 ACR values (aal1_observed, aal2_passkey_uv, aal3_hardware_bound, aal3_recovery_ceremony)?

Per ADR-ID-001 § Decision + ADR-identity-004. NIST 800-63B defines AAL1/AAL2/AAL3. oyatie maps:

- **aal1_observed**: rare; only for low-risk consumer flows where user has no passkey yet (e.g., social-login flow). NOT acceptable for tenant actions.
- **aal2_passkey_uv**: synced passkey + user-verification (TouchID, FaceID, biometric, PIN). Default for workforce.
- **aal3_hardware_bound**: hardware-backed credential (YubiKey, Secure Enclave with platform-bound). Required for admin + eDiscovery + break-glass + recovery-authority.
- **aal3_recovery_ceremony**: a session issued through the recovery ceremony. Cedar can require additional step-up before high-risk actions even at this ACR.

The distinction between the two `aal3_*` values matters because a recovery-ceremony session may need additional auditing or limits (e.g., "Just recovered; can read data but not change admin settings until 24 h have passed").

## What's the AAGUID trust catalog and why does it matter?

Per ADR-ID-001 § Decision + IP-006-aaguid-refresh-worker. AAGUID = Authenticator Attestation GUID, a 128-bit identifier embedded in WebAuthn attestation. Each authenticator vendor has assigned AAGUIDs per model.

Trust catalog:

- **Allowed**: vendor passes FIDO Alliance metadata + has positive security posture.
- **Allowed for low-risk only**: synced passkeys (Apple, Google, 1Password, etc.) — strong privacy + cryptographic posture but harder to attest to device-class.
- **Denied**: known-compromised vendor batches (e.g., a hypothetical Yubico recall) OR consumer-grade devices where pack policy requires enterprise attestation.

The worker pulls FIDO Metadata Service every 6 h. Tenants can also maintain their own allowlist/denylist (per ADR-identity-002 attestation policy).

## How does recovery work without operator decryption?

Per ADR-ID-001 § Decision + Constraint ID-C7. Three phases:

1. **Proof collection**: user provides recovery code (BIP-39 mnemonic given to them at envelope creation). Server verifies the code matches `verifier_hash`.
2. **Recovery grant issuance**: server creates a `RecoveryGrant` row + sends it via Cedar-evaluated channel (typically the verified email-on-file).
3. **Session rebinding**: user provides recovery passphrase + new device passkey. Server derives wrapping key from passphrase (Argon2id) + unwraps recovery secret + binds new credential + rotates all existing sessions.

The recovery passphrase NEVER leaves the user. The server's wrapping key requires the passphrase to derive. Operator can request the encrypted envelope but cannot decrypt without the passphrase.

If the user loses BOTH their devices AND their recovery passphrase, the account is unrecoverable. This is by design — per ADR-ID-001 § Decision: "Reject any server-side plaintext recovery path".

## Why does Cedar gate session step-up?

Per ADR-ID-001 § Decision. Cedar `identity::session::step_up` requires:

- The action context (what is being attempted).
- The required ACR.
- The currently-acceptable authenticator class.

Example: a tenant admin attempting `drive::cmk::cryptoshred` triggers Cedar evaluation:

```cedar
permit (
    principal,
    action == Action::"identity::session::step_up",
    resource
) when {
    context.action_being_attempted == "drive::cmk::cryptoshred" &&
    context.required_acr == "aal3_hardware_bound" &&
    principal.tenant_id == resource.tenant_id
};
```

The step-up is initiated by the calling µservice (drive) on detecting that the current session's ACR is insufficient. User completes step-up; new ACR injected into the session; the original action retries.

## How does external IdP federation work?

Per IP-011-external-idp-federation + ADR-ID-001 § Decision. Two modes:

1. **OIDC federation (inbound)**: oyatie accepts OIDC ID tokens from configured external IdPs (Okta, Entra ID, Google Workspace, OneLogin, Ping, JumpCloud). The tenant has a `TenantExternalIdpFederation` record specifying IdP discovery URL + client credentials.
2. **OIDC federation (outbound)**: oyatie acts as an upstream IdP for SaaS apps; SaaS apps redirect to oyatie OIDC issuer; receive token.

For inbound: the external IdP claim is treated as identity proof + audience-type binding, NOT as passkey-equivalent assurance (per ADR-ID-001 Constraint ID-C11). High-risk actions still require local passkey step-up.

## What about SCIM bulk provisioning at scale?

Per ADR-identity-003 + IP-007-scim-server-kernel + IP-008-scim-adapter-zitadel. SCIM 2.0 RFC 7644 endpoints:

- Single-user POST/PATCH/DELETE: rate limit 100 ops/sec/tenant (demo_trial), 5 000 (paid with per_usage billing_component).
- Bulk endpoint `/v2/Bulk`: up to 1 000 operations per request.
- Batch streams via gRPC `IdentityScim.BulkStream` for ≥ 100k ops.

Rate limit per ADR-identity-003 protects against IdP misconfiguration (e.g., Workday sending full org weekly).

## What is the dual-context principal resolver?

Per IP-017-multi-context-principal-resolver + ADR-ID-001 § Decision. A human can have multiple principals:

- `u-alice@acme-corp.com` (audience_type=workforce; tenant=acme-corp).
- `u-alice@personal-tenant` (audience_type=personal; tenant=alice-personal).
- `u-alice@acme-corp-recruiting-portal` (audience_type=external-applicant; tenant=acme-corp).

The same passkey credential can be bound to multiple principals (with explicit dual-context grants). When Alice authenticates, the resolver picks the appropriate principal based on the requesting service's audience_type expectation.

Critical: personal-tenant data is NEVER admin-recoverable through the workforce tenant (per ADR-ID-001 Constraint ID-C8/C9).

## What's the JIT IT approval protocol (ADR-identity-005)?

Per ADR-identity-005. JIT = Just-in-Time. For privileged actions (admin role, eDiscovery export, key rotation):

1. User initiates action.
2. Cedar evaluates: requires JIT approval per pack policy.
3. Request enters approval workflow via `workflow-engine` µservice.
4. Approvers receive notification (mail + push).
5. After approval (single or dual), session step-up to higher ACR + action allowed for a bounded window.

Example: Alice requests admin role to revoke a credential. Bob (security admin) approves. Alice's session gets `acr=aal3_hardware_bound` + `jit_approval_id=ja_001` + `valid_until=now+1h`. After 1 h, Alice's session ACR drops back.

## How does behavioral biometrics work without EU AI Act issues?

Per ADR-ID-001 + IP-014-continuous-risk-scoring. Behavioral biometrics (typing cadence, mouse movement, swipe patterns) are processed:

- **Default tenants**: behavioral signals feed risk score → adaptive step-up.
- **EU-GDPR + Annex-III-restricted packs**: behavioral biometrics DISABLED (per ADR-MAIL-0004 EU AI Act doctrine).
- **HIPAA + FedRAMP-High packs**: behavioral signals limited to risk-trigger; not stored or modeled longitudinally.

The substrate models behavioral data as transient signals (not stored beyond 24 h) to avoid Annex III "high-risk AI" classification.

## What's the session class tenant_class model (ADR-identity-004)?

Per ADR-identity-004. Four session classes:

- **consumer_low_risk**: B2C; default tenant_class; `aal2_passkey_uv`; passwords accepted as fallback.
- **workforce_standard**: B2B default; `aal2_passkey_uv`; password fallback denied; SCIM-provisioned.
- **workforce_elevated**: admin + audit; `aal3_hardware_bound` mandatory; JIT approval for privileged.
- **regulated_high_assurance**: HIPAA + FedRAMP + legal; `aal3_hardware_bound` always; dual-control for sensitive ops.

Session class is set at authentication time based on the requesting application + tenant pack. Cedar enforces actions against the session class.

## How are JWKs rotated (ADR-identity-001)?

Per ADR-identity-001. Issuer signing keys (Ed25519) rotate every 30 d under normal operation. Process:

1. Day -7: new key generated; published in JWKs (clients can verify with old OR new).
2. Day 0: signing operations switch to new key; old key remains in JWKs for verification.
3. Day +7: old key removed from JWKs.

Token verification: `kid` in JWT header tells the verifier which key to use. JWKs endpoint cached for ≤ 24 h.

Critical key-compromise scenario: 7-d emergency rotation; old key marked as revoked; all sessions issued with old key forced to re-authenticate.

## How does the issuer signing key interact with FIPS 140-3 L3 HSM at paid with compliance_pack gating?

Per ADR-ID-001 sovereign path. Issuer signing keys live in OpenBao transit (non-exportable) or external HSM (Thales Luna). Signing ops never expose plaintext key. Per-pack residency:

- KR-PIPA: keys in KR-cells; cross-region replication denied.
- FedRAMP-High: keys in US-Gov cells.
- CN-PIPL: keys in CN cells.

This is achieved via per-pack issuer instance (one Zitadel deployment per pack) + per-pack OpenBao path scoping.

## What about migration from Okta / Auth0 / Entra ID?

See `migration-playbooks/from-okta.md` for the detailed playbook. Short version:

1. SCIM-export users from existing IdP.
2. Run `oya identity migrate import-okta` (creates oyatie principals).
3. **No password import** — passkey-bootstrap mandatory on first login.
4. Configure external IdP federation for transition period (oyatie can accept Okta tokens during bridge).
5. Phased cutover: app-by-app SAML/OIDC client re-points to oyatie.
6. After all apps migrated, retire external IdP.
