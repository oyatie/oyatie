---
doc_class: Onboarding
microservice: identity
persona: identity-engineer + iam-engineer + webauthn-engineer
related_adrs: [ADR-ID-001, ADR-identity-001, ADR-identity-002, ADR-identity-003, ADR-identity-004, ADR-identity-005]
date: 2026-05-20
doc_status: published
---

# Identity Engineer onboarding — first 5 working days on `identity`

Audience: a new identity engineer, IAM engineer, or WebAuthn engineer joining the `identity` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, registered a passkey via WebAuthn Level 3, issued OIDC tokens with all required claims, created a recovery envelope, exercised recovery + session rebinding, walked the AAGUID-revocation runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the five-vendor displacement + passkey-primary doctrine.
2. Read `ARCHITECTURE.md` § oidc-issuer + § webauthn-relying-party + § recovery-envelope + § multi-context-principal-resolver + § external-idp-federation (∼ 90 min).
3. Read `decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md` end-to-end (∼ 60 min). The binding architecture.
4. Read `decisions/ADR-identity-001..005` (JWKS rotation, passkey attestation policy, SCIM rate limits, session class tiers, JIT IT approval) (∼ 45 min total).
5. Read WebAuthn Level 3 spec § 7-9 (registration ceremony, authentication ceremony, credential management) (∼ 60 min — critical).
6. Read RFC 8259 (JSON), RFC 8628 (OIDC Device Code), RFC 7644 (SCIM 2.0) overviews (∼ 45 min).
7. Open the Grafana folder `identity`. core boards: `identity-webauthn-verify-latency`, `identity-recovery-completion-total`, `identity-unknown-aaguid-login-total`, `identity-high-risk-stepup-success-ratio`, `identity-jwks-rotation-status`, `identity-scim-throughput`.
8. Walk `runbooks/README.md`. The on-call runbooks: `passkey-reset.md`, `passkey-cross-device-debug.md`, `jwks-rotation.md`, `brute-force-mitigation.md`, `idp-failover-drill.md`, `aaguid-revocation.md`, `recovery-ceremony-stuck.md`, `dual-context-leak.md`, `scim-rate-limit-storm.md`.
9. Sit in on the Wednesday identity-substrate handoff.

Acceptance: you can sketch the WebAuthn registration ceremony: client → `/v1/identity/webauthn/registration/options` (server returns challenge + RP-ID + user info) → client invokes navigator.credentials.create() → user uses authenticator → client POSTs attestation to `/v1/identity/webauthn/registration/verify` → server validates attestation (AAGUID trust, attestation_class) → `CredentialBinding` row inserted → audit-chain `EVT-ID-PASSKEY-REGISTERED`. And the recovery: user lost device → POST `/v1/identity/recovery/ceremonies` with recovery code → Cedar validates → server creates recovery grant → user provides new device passkey + recovery passphrase → server unwraps recovery envelope → new session issued + old sessions revoked.

## Day 2 — demo_trial cell bootstrap + first passkey registration

```sh
cargo run -p oya-dev-cli -- identity bootstrap \
    --profile demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/identity \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --kafka-endpoint kafka://drill-kafka-syd-1:9092 \
    --openbao-endpoint https://drill-openbao-syd-1:8200 \
    --openbao-mount identity \
    --zitadel-endpoint https://drill-zitadel-syd-1:443 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 18 min. Verify:

```sh
oya identity health --cell drill-syd-1
# Expected:
#   postgres.identity_credentials: up (lag_ms=14)
#   valkey.webauthn-challenge-cache: up
#   kafka.identity-events: connected
#   openbao.issuer-signing-key: up (key_age_days=12)
#   openbao.recovery-envelope-wrapping-key: up
#   zitadel: up (oidc_issuer_url=https://drill-zitadel-syd-1/oidc/v1)
#   aaguid_catalog_freshness: ok (last_refresh=4h ago)
#   audit-chain.emit: up
```

Create a tenant + first user:

```sh
oya identity tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Identity" \
    --rp-id drill-acme.drill.test \
    --auth-policy passkey-primary

oya identity user create \
    --tenant drill-acme \
    --principal-id u-alice \
    --email alice@drill-acme.com \
    --display-name "Alice Test"
# Output:
#   user_id: u-alice
#   tenant_id: drill-acme
#   onboarding_token: oyatie-onboard-7c4a2b8e9f...
#   audit_event_id: ae_id_user_created_001
```

Now register a passkey. In production, this is driven by the WebAuthn client. For the drill, use the harness:

```sh
# 1. Server generates the registration challenge
oya identity webauthn registration-options \
    --tenant drill-acme \
    --user u-alice \
    --onboarding-token oyatie-onboard-7c4a2b8e9f...
# Output:
#   challenge_id: ch_drill_001
#   rp_id: drill-acme.drill.test
#   rp_name: "ACME Identity"
#   user_id: u-alice
#   user_name: alice@drill-acme.com
#   pub_key_cred_params: [{alg=-7, type=public-key}]  # ES256
#   timeout: 60000
#   exclude_credentials: []
#   authenticator_selection: {required_resident_key: true, user_verification: required}

# 2. Client (via harness) creates credential with simulated YubiKey
oya identity webauthn registration-simulate \
    --challenge-id ch_drill_001 \
    --simulated-authenticator yubikey-5c-nfc \
    --simulated-aaguid ee882879-721c-4913-9775-3dfcce97072a
# Output:
#   client_data_json_b64: ...
#   attestation_object_b64: ...

# 3. Server verifies + binds credential
oya identity webauthn registration-verify \
    --challenge-id ch_drill_001 \
    --client-data-json-b64 ... \
    --attestation-object-b64 ...
# Server:
#   - Validates challenge ✓
#   - Validates attestation (AAGUID trust ✓)
#   - Creates CredentialBinding row
# Output:
#   credential_id_hash: blake3:7c4a2b8e9f...
#   aaguid: ee882879-721c-4913-9775-3dfcce97072a
#   attestation_class: hardware-backed
#   credential_epoch: 1
#   audit_event_id: ae_id_passkey_registered_001
```

Acceptance: cell bootstrap + tenant + user + passkey registered.

## Day 3 — Authentication ceremony + OIDC token issuance

Authenticate with the registered passkey:

```sh
# 1. Server generates authentication challenge
oya identity webauthn authentication-options \
    --tenant drill-acme \
    --user u-alice
# Output:
#   challenge_id: ch_auth_drill_001
#   rp_id: drill-acme.drill.test
#   allow_credentials: [{type=public-key, id=...}]
#   user_verification: required
#   timeout: 60000

# 2. Client (harness) generates assertion
oya identity webauthn authentication-simulate \
    --challenge-id ch_auth_drill_001 \
    --principal u-alice

# 3. Server verifies + creates session
oya identity webauthn authentication-verify \
    --challenge-id ch_auth_drill_001 \
    --client-data-json-b64 ... \
    --authenticator-data-b64 ... \
    --signature-b64 ...
# Output:
#   session_id: s_drill_001
#   acr: aal2_passkey_uv  # synced passkey would give aal2; hardware gives aal3
#   amr: [hwk, mfa]
#   tenant_id: drill-acme
#   principal_id: u-alice
#   audience_type: workforce
#   home_cell: drill-syd-1
#   credential_epoch: 1
#   recovery_epoch: 0
#   audit_event_id: ae_id_authentication_completed_001
```

Issue an OIDC token (per ADR-ID-001 § Decision claims):

```sh
oya identity oidc token-issue \
    --session s_drill_001 \
    --audience drive-api \
    --scopes "drive:files:read drive:files:write"
# Output:
#   access_token: eyJhbGciOiJFZERTQSIsImtpZCI6ImlzcnVlcl8yMDI2XzA1XzIwIiwidHlwIjoiSldUIn0.eyJpc3MiOiJodHRwczovL2RyaWxsLXppdGFkZWwtc3lkLTEvb2lkYy92MSIsInN1YiI6InUtYWxpY2UiLCJ0ZW5hbnRfaWQiOiJkcmlsbC1hY21lIiwiYXVkaWVuY2VfdHlwZSI6Indvcmtmb3JjZSIsImhvbWVfY2VsbCI6ImRyaWxsLXN5ZC0xIiwiY3JlZGVudGlhbF9lcG9jaCI6MSwicmVjb3ZlcnlfZXBvY2giOjAsImFjciI6ImFhbDJfcGFzc2tleV91diIsImFtciI6WyJod2siLCJtZmEiXSwiYXVkIjoiZHJpdmUtYXBpIiwic2NvcGVzIjoiZHJpdmU6ZmlsZXM6cmVhZCBkcml2ZTpmaWxlczp3cml0ZSIsImlhdCI6MTcxNjI4MDcxMSwiZXhwIjoxNzE2Mjg0MzExfQ.Ed25519_signature_here
#   token_type: Bearer
#   expires_in: 3600
#   audit_event_id: ae_id_token_issued_001
```

Decode the claims (the JWT is human-readable):

```sh
echo "$ACCESS_TOKEN" | jq -R 'split(".") | .[1] | @base64d | fromjson'
# Output:
#   {
#     "iss": "https://drill-zitadel-syd-1/oidc/v1",
#     "sub": "u-alice",
#     "tenant_id": "drill-acme",
#     "audience_type": "workforce",
#     "home_cell": "drill-syd-1",
#     "credential_epoch": 1,
#     "recovery_epoch": 0,
#     "acr": "aal2_passkey_uv",
#     "amr": ["hwk", "mfa"],
#     "aud": "drive-api",
#     "scopes": "drive:files:read drive:files:write",
#     "iat": 1716280711,
#     "exp": 1716284311
#   }
```

Per ADR-ID-001 § Decision: every claim required is present.

Acceptance: authentication + OIDC token issuance + claims verified.

## Day 4 — Recovery envelope creation + redemption

Create a recovery envelope for Alice (per ADR-ID-001 § Decision):

```sh
# Alice provides a strong recovery passphrase (24+ chars, dictionary-checked)
oya identity recovery envelope create \
    --tenant drill-acme \
    --user u-alice \
    --session s_drill_001 \
    --passphrase-policy strong
# Server prompts for passphrase via secure channel; user provides "correct horse battery staple cellphone walrus iceberg"
# Server derives wrapping key via Argon2id (2 GiB, 4 iter); wraps recovery secret; stores in OpenBao
# Output:
#   recovery_epoch: 1
#   envelope_handle: re_drill_001
#   verifier_hash: blake3:...
#   openbao_ref: secret/drill-acme/identity/recovery/u-alice/1
#   audit_event_id: ae_id_recovery_envelope_created_001

# User is shown the recovery code ONCE (for printing)
# Output (one-time):
#   recovery_code: XXXX-XXXX-XXXX-XXXX (BIP-39 mnemonic)
#   Save this! It will not be shown again.
```

Now simulate Alice losing all devices + recovering:

```sh
# 1. User uses recovery code on a new device
oya identity recovery initiate \
    --tenant drill-acme \
    --user u-alice \
    --recovery-code XXXX-XXXX-XXXX-XXXX \
    --replacement-device new-device-001
# Cedar evaluates:
#   - identity::recovery::initiate ✓
#   - recovery code matches verifier_hash ✓
# Output:
#   recovery_grant_id: rg_drill_001
#   ceremony_state: awaiting_passphrase

# 2. User provides passphrase to derive wrapping key
oya identity recovery complete \
    --recovery-grant-id rg_drill_001 \
    --passphrase "correct horse battery staple cellphone walrus iceberg" \
    --new-device-credential-id-hash ... \
    --new-device-passkey-attestation ...
# Server:
#   - Derives wrapping key from passphrase + Argon2id (matches verifier_hash ✓)
#   - Unwraps recovery secret from OpenBao envelope
#   - Binds new device credential
#   - Rotates all active sessions (per ADR-ID-001 Constraint ID-C14)
#   - Increments recovery_epoch
# Output:
#   new_session_id: s_drill_002
#   acr: aal3_recovery_ceremony
#   credential_epoch: 2
#   recovery_epoch: 2
#   revoked_sessions: 1 (old session s_drill_001)
#   revoked_delegated_grants: 0
#   audit_event_id: ae_id_recovery_completed_001
```

Verify the old session is revoked:

```sh
oya identity session show --session s_drill_001
# Output: state=revoked, revoked_reason=recovery_ceremony, revoked_at=...
```

Acceptance: recovery envelope creation + redemption + session rotation verified.

## Day 5 — AAGUID revocation runbook + step-up

Walk the AAGUID-revocation runbook. Read `runbooks/aaguid-revocation.md`. Scenario: FIDO Alliance announces a vendor batch has a vulnerability (hypothetical). Runbook covers:

1. Identify from the FIDO Metadata Service refresh (auto-detected by AAGUID worker).
2. Mark AAGUID `xx-xx-xx` as `revoked` in catalog.
3. Existing CredentialBindings with this AAGUID: state=needs-replacement (sessions still valid until next step-up).
4. Notify affected tenants via Kafka event `identity.aaguid.revoked.v1`.
5. Affected users: cannot register new credentials with this AAGUID; existing credentials require step-up to a different authenticator within 30 d.

Simulate the runbook:

```sh
oya identity aaguid revoke \
    --aaguid ee882879-721c-4913-9775-3dfcce97072a \
    --reason "Vendor batch X-2026-05 vulnerability disclosed" \
    --grace-period-days 30 \
    --notify-tenants true
# Output:
#   affected_credentials: 1 (Alice's)
#   notification_sent: 1 tenant (drill-acme)
#   audit_event_id: ae_id_aaguid_revoked_001
```

Step-up to a different authenticator (per ADR-ID-001):

```sh
# Alice has a backup YubiKey 5C with a different AAGUID
oya identity session step-up \
    --session s_drill_002 \
    --required-acr aal3_hardware_bound \
    --excluded-credential-id-hashes blake3:7c4a2b8e9f...
# (User authenticates with backup YubiKey)
# Output:
#   stepup_id: su_drill_001
#   new_acr: aal3_hardware_bound
#   amr: [hwk, mfa]
#   audit_event_id: ae_id_stepup_completed_001
```

Acceptance: AAGUID revocation runbook walked; step-up verified.

## What you've learned

- demo_trial bootstrap + tenant + user + passkey registration.
- WebAuthn Level 3 registration + authentication ceremonies.
- OIDC token issuance with all required claims (acr, amr, tenant_id, principal_id, audience_type, home_cell, credential_epoch, recovery_epoch).
- Recovery envelope creation + redemption (operator-undecryptable).
- Session rotation on recovery completion.
- AAGUID revocation + step-up to alternative authenticator.

Next week: paid with per_seat billing_component promotion (hardware passkeys + external IdP OIDC federation + HRIS SCIM + multi-context principal resolver), paid with per_usage billing_component tour (mandatory hardware for high-risk roles + JIT IT approval + continuous risk scoring + session class tiers), paid with compliance_pack gating tour (FIPS 140-3 L3 HSM + per-pack AAGUID allowlist + regulator-observable recovery ceremony), and your first production shadow on a recovery ceremony approval.
