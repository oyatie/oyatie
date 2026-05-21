---
doc_class: Tutorial
microservice: identity
persona: identity-engineer + iam-engineer
related_adrs: [ADR-ID-001, ADR-identity-002, ADR-identity-004]
date: 2026-05-20
doc_status: published
---

# Tutorial — Register a hardware passkey + create a recovery envelope + exercise recovery

You will: provision a workforce user, register a hardware-backed YubiKey 5C passkey via WebAuthn Level 3, create an operator-undecryptable recovery envelope, exercise the full recovery ceremony with passphrase + new device, verify session rotation on recovery, and audit-chain-verify the lifecycle. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant on paid with per_seat billing_component tenant_class (`ADR-0330 and ADR-0331 tenant_class model`).
- `oya-dev-cli` ≥ 1.42.0.
- A YubiKey 5C (or compatible FIDO2 authenticator with valid attestation).
- A tenant principal in the `identity_admin` Cedar role.
- OpenBao with FIPS 140-3 L2 HSM auto-unseal.

## Step 1 — Provision a workforce user (≤ 5 min)

```sh
oya identity user create \
    --tenant acme-corp \
    --principal-id u-alice@acme-corp.com \
    --email alice@acme-corp.com \
    --display-name "Alice Anderson" \
    --audience-type workforce \
    --session-class workforce_standard \
    --auth-policy passkey-primary \
    --require-hardware-passkey-for-roles "drive_admin,messenger_admin,compliance_admin"
# Output:
#   user_id: u-alice@acme-corp.com
#   onboarding_token: oyatie-onboard-7c4a2b8e9f...
#   onboarding_url: https://identity.acme-corp.oyatie.local/enroll?token=...
#   audit_event_id: ae_id_user_created_001
```

The onboarding token is single-use + 24-hour TTL. Send it to Alice via secure channel (typically corporate email).

## Step 2 — Generate WebAuthn registration challenge (≤ 5 min)

Alice clicks the onboarding URL, lands on the enrollment page. Her browser POSTs:

```sh
oya identity webauthn registration-options \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --onboarding-token oyatie-onboard-7c4a2b8e9f... \
    --required-authenticator-class hardware-backed
# Cedar evaluates:
#   - identity::passkey::register ✓
#   - onboarding-token valid ✓
#   - required-authenticator-class allowed by tenant policy ✓
# Output:
#   challenge_id: ch_acme_001
#   challenge_b64: <128-bit random>
#   rp_id: acme-corp.oyatie.local
#   rp_name: "ACME Corp"
#   user_handle_b64: <base64 user_id>
#   pub_key_cred_params: [
#     {alg: -8, type: "public-key"},  # Ed25519 (RFC 8037 + ALG-edDSA)
#     {alg: -7, type: "public-key"}   # ES256
#   ]
#   timeout: 60000
#   exclude_credentials: []
#   authenticator_selection: {
#     required_resident_key: true,
#     user_verification: "required",
#     authenticator_attachment: "cross-platform"  # USB security key
#   }
#   attestation: "direct"  # require attestation for hardware-backed
```

Browser invokes `navigator.credentials.create({publicKey: ...})`. Alice touches her YubiKey.

## Step 3 — Verify registration server-side (≤ 5 min)

Browser POSTs the result to:

```sh
oya identity webauthn registration-verify \
    --tenant acme-corp \
    --challenge-id ch_acme_001 \
    --user-id u-alice@acme-corp.com \
    --client-data-json-b64 <client_data_json> \
    --attestation-object-b64 <attestation_object>
# Server validates:
#   1. Challenge matches ch_acme_001 + not expired ✓
#   2. Origin in clientDataJSON matches RP origin ✓
#   3. clientDataJSON.type == "webauthn.create" ✓
#   4. Attestation signature verifies ✓
#   5. AAGUID is in trust catalog (YubiKey 5C NFC ee882879-721c-4913-9775-3dfcce97072a) ✓
#   6. Attestation class is "hardware-backed" per FIDO Metadata Service ✓
#   7. User verification flag in authenticatorData is set ✓
#   8. Resident key flag is set ✓
# Output:
#   credential_id_hash: blake3:7c4a2b8e9f...
#   aaguid: ee882879-721c-4913-9775-3dfcce97072a
#   attestation_class: hardware-backed
#   attestation_format: packed
#   credential_epoch: 1
#   public_key_alg: ES256
#   sign_count: 0
#   audit_event_id: ae_id_passkey_registered_001
```

Verify the binding:

```sh
oya identity credential list --tenant acme-corp --user u-alice@acme-corp.com
# Output:
#   credentials:
#     - credential_id_hash: blake3:7c4a2b8e9f...
#       aaguid: ee882879-721c-4913-9775-3dfcce97072a
#       device_label: "YubiKey 5C NFC"  # from FIDO MDS
#       attestation_class: hardware-backed
#       state: active
#       credential_epoch: 1
#       registered_at: 2026-05-20T14:32:17Z
```

## Step 4 — Test authentication (≤ 5 min)

Alice signs in to the test app. Browser POSTs:

```sh
oya identity webauthn authentication-options \
    --tenant acme-corp \
    --user u-alice@acme-corp.com
# Output:
#   challenge_id: ch_auth_acme_001
#   challenge_b64: <128-bit random>
#   rp_id: acme-corp.oyatie.local
#   allow_credentials: [
#     {type: "public-key", id_b64: <credential_id>}
#   ]
#   user_verification: "required"
#   timeout: 60000
```

Browser invokes `navigator.credentials.get({publicKey: ...})`. Alice touches YubiKey + provides PIN.

Server verifies:

```sh
oya identity webauthn authentication-verify \
    --tenant acme-corp \
    --challenge-id ch_auth_acme_001 \
    --client-data-json-b64 ... \
    --authenticator-data-b64 ... \
    --signature-b64 ...
# Output:
#   session_id: s_acme_alice_001
#   acr: aal3_hardware_bound  # hardware-backed credential
#   amr: ["hwk", "mfa", "uv"]
#   tenant_id: acme-corp
#   principal_id: u-alice@acme-corp.com
#   audience_type: workforce
#   home_cell: prod-us-east-1
#   credential_epoch: 1
#   recovery_epoch: 0
#   sign_count: 1
#   audit_event_id: ae_id_authentication_completed_001
```

## Step 5 — Create recovery envelope (≤ 10 min)

Alice creates a recovery envelope BEFORE losing her device (proactive). Server prompts for passphrase + emits one-time recovery code:

```sh
oya identity recovery envelope create \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --session s_acme_alice_001 \
    --passphrase-policy strong \
    --display-recovery-code-once true
# Cedar evaluates:
#   - identity::recovery::envelope::create ✓
#   - session has aal3_hardware_bound ACR ✓ (required for recovery creation per ADR-ID-001)
# Server prompts Alice via secure channel for passphrase:
#   "correct horse battery staple cellphone walrus iceberg" (24-character; passes Argon2id derive)
# Server derives wrapping key (Argon2id; 2 GiB memory; 4 iter):
#   wrapping_key = Argon2id(passphrase, salt=tenant+user+epoch, ...)
# Server generates recovery secret + wraps it:
#   recovery_envelope_ciphertext = AEAD-Encrypt(wrapping_key, recovery_secret, AAD=tenant_user)
# Server stores in OpenBao:
#   secret/acme-corp/identity/recovery/u-alice/1
# Output (one-time):
#   recovery_epoch: 1
#   envelope_handle: re_acme_alice_001
#   recovery_code: "blossom-tulip-arctic-mountain-zenith-velvet-thunder-lighthouse-jaguar-saffron-meadow-iceberg"  # BIP-39 mnemonic
#   verifier_hash: blake3:7c4a2b8e9f...
#   audit_event_id: ae_id_recovery_envelope_created_001
#   IMPORTANT: This recovery code will not be shown again. Save it physically (printed) in a secure location.
```

Alice prints the recovery code + stores it physically (e.g., in a safe).

## Step 6 — Simulate device loss + recover (≤ 15 min)

Alice's YubiKey is lost. She has a new device + her recovery code + her passphrase memorized.

```sh
# 1. From new device, initiate recovery
oya identity recovery initiate \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --recovery-code "blossom-tulip-arctic-mountain-zenith-velvet-thunder-lighthouse-jaguar-saffron-meadow-iceberg"
# Cedar evaluates:
#   - identity::recovery::initiate ✓
#   - recovery-code matches verifier_hash ✓ (BLAKE3 verification)
#   - no active recovery freeze ✓
# Output:
#   recovery_grant_id: rg_acme_alice_001
#   ceremony_state: awaiting_passphrase_and_new_credential
#   expires_at: 2026-05-20T15:00:00Z  # 30-minute window

# 2. Generate a registration challenge for the new device's passkey
oya identity webauthn registration-options \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --recovery-grant-id rg_acme_alice_001
# Output: challenge_id=ch_recovery_001, ... (similar to Step 2)

# 3. Alice registers new YubiKey
# ... (WebAuthn ceremony with new device)

# 4. Complete recovery
oya identity recovery complete \
    --tenant acme-corp \
    --recovery-grant-id rg_acme_alice_001 \
    --passphrase "correct horse battery staple cellphone walrus iceberg" \
    --new-credential-attestation <attestation_object_b64>
# Server:
#   - Derives wrapping key from passphrase + Argon2id (matches verifier_hash ✓)
#   - Unwraps recovery secret from OpenBao envelope ✓
#   - Validates new device's WebAuthn attestation ✓
#   - Binds new device credential
#   - Rotates all active sessions (per ADR-ID-001 Constraint ID-C14)
#   - Increments recovery_epoch
#   - Revokes delegated grants (per ADR-ID-001 Constraint ID-C14 unless preserved)
# Output:
#   new_session_id: s_acme_alice_002
#   acr: aal3_recovery_ceremony
#   credential_epoch: 2
#   recovery_epoch: 2
#   old_credential_id_hash: blake3:7c4a2b8e9f...  # marked revoked
#   new_credential_id_hash: blake3:9e8d7c6b5a...
#   revoked_sessions: 1
#   revoked_delegated_grants: 0
#   audit_event_id: ae_id_recovery_completed_001
```

Verify the old credential + old sessions are revoked:

```sh
oya identity credential list --tenant acme-corp --user u-alice@acme-corp.com
# Output:
#   - credential_id_hash: blake3:7c4a2b8e9f...
#     state: revoked  # old YubiKey
#     revoked_reason: recovery_ceremony
#   - credential_id_hash: blake3:9e8d7c6b5a...
#     state: active   # new YubiKey
#     credential_epoch: 2

oya identity session show --session s_acme_alice_001
# Output: state=revoked, revoked_reason=recovery_ceremony
```

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "identity.*" --since 60m
```

Expected events for our flow:

- `identity.user.created.v1`
- `identity.passkey.registered.v1` (× 2; original + recovery)
- `identity.authentication.completed.v1` (× ≥ 2; original sign-in + post-recovery sign-in)
- `identity.recovery.envelope.created.v1`
- `identity.recovery.initiated.v1`
- `identity.recovery.completed.v1`
- `identity.session.revoked.v1` (× 1; old session)
- `identity.token.issued.v1` (× ≥ 1)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 8 — Best-practice followup (≤ 5 min)

Per ADR-ID-001 § Implementation Notes, after recovery:

- Rotate recovery envelope (new passphrase recommended).
- Review delegated grants (some may need re-issuance).
- If recovery was due to suspected compromise: check audit-chain for unauthorized actions in 30-day window before recovery.

```sh
oya identity recovery envelope rotate \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --new-passphrase-prompt true
# Output:
#   new_recovery_epoch: 3
#   new_recovery_code: "..." (one-time)
#   audit_event_id: ae_id_recovery_envelope_rotated_001
```

## What you've learned

- WebAuthn Level 3 registration ceremony with hardware passkey + attestation validation.
- WebAuthn authentication ceremony.
- OIDC token claims (acr, amr, tenant_id, principal_id, audience_type, home_cell, credential_epoch, recovery_epoch).
- Recovery envelope creation (operator-undecryptable).
- Recovery initiation + completion ceremony.
- Session rotation + credential revocation on recovery.
- Audit-chain verification of full identity lifecycle.

Next tutorial: `tutorials/configure-external-idp-federation-okta.md` — set up incoming OIDC federation from Okta to oyatie identity (paid with per_seat billing_component tenant_class).
