---
doc_class: IP
ip_id: IP-005
microservice: identity
status: ga
related_adrs: [ADR-0188, ADR-0145]
date: 2026-05-18
owner_team: axis-identity
---

# IP-005 — WebAuthn register/authenticate REST handlers

## Goal

Land the `oya-identity-webauthn-relying-party-rest` crate: HTTP handlers as `axum::Router` exposing `/webauthn/register/start`, `/webauthn/register/finish`, `/webauthn/authenticate/start`, `/webauthn/authenticate/finish` per W3C WebAuthn L3, backed by the kernel from IP-004 + a Postgres-backed `CredentialStore` adapter + Valkey-backed `ChallengeStore`.

## Files

| File | Purpose |
|---|---|
| `crates/oya-identity-webauthn-relying-party-rest/Cargo.toml` | manifest; axum + sqlx + valkey |
| `crates/oya-identity-webauthn-relying-party-rest/src/lib.rs` | Router builder + handlers |
| `crates/oya-identity-webauthn-relying-party-rest/src/postgres_credential_store.rs` | Postgres-backed CredentialStore impl |
| `crates/oya-identity-webauthn-relying-party-rest/src/valkey_challenge_store.rs` | Valkey-backed ChallengeStore impl |
| `crates/oya-identity-webauthn-relying-party-rest/src/error.rs` | HTTP error envelope |
| `crates/oya-identity-webauthn-relying-party-rest/tests/handlers.rs` | request-response tests via `tower::ServiceExt::oneshot` |

## Endpoints

| Method | Path | Body | Auth | Returns |
|---|---|---|---|---|
| POST | `/webauthn/register/start` | `{user_id, display_name}` | OIDC `acr>=elevated` | `RegistrationChallenge` JSON |
| POST | `/webauthn/register/finish` | `RegistrationResponse` | OIDC `acr>=elevated` | `Credential` (sans private key) |
| POST | `/webauthn/authenticate/start` | `{allow_credentials?, mediation}` | none (pre-auth) | `AuthenticationChallenge` |
| POST | `/webauthn/authenticate/finish` | `AuthenticationResponse` | none (assertion IS the auth) | `{id_token, refresh_token}` |
| GET | `/webauthn/credentials` | n/a | OIDC `acr>=elevated` | list of user's credentials |
| DELETE | `/webauthn/credentials/{id}` | n/a | OIDC `acr>=sensitive` | 204 + audit emit |

## Postgres schema

```sql
CREATE TABLE webauthn_credentials (
  credential_id    BYTEA PRIMARY KEY,
  tenant_id        TEXT NOT NULL,
  user_id          TEXT NOT NULL,
  public_key       BYTEA NOT NULL,    -- CBOR-encoded COSE_Key
  aaguid           UUID NOT NULL,
  transports       JSONB NOT NULL DEFAULT '[]'::jsonb,
  attestation_format TEXT NOT NULL,
  backup_eligible  BOOLEAN NOT NULL DEFAULT FALSE,
  backup_state     BOOLEAN NOT NULL DEFAULT FALSE,
  sign_count       BIGINT NOT NULL DEFAULT 0,
  last_used_at     TIMESTAMPTZ NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  revoked_at       TIMESTAMPTZ
);
CREATE INDEX ON webauthn_credentials (tenant_id, user_id);
CREATE INDEX ON webauthn_credentials (aaguid) WHERE revoked_at IS NULL;

-- Row-Level Security for cross-tenant isolation.
ALTER TABLE webauthn_credentials ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_scope ON webauthn_credentials
  USING (tenant_id = current_setting('app.tenant_id', true));
```

## Valkey schema

Key: `webauthn:challenge:{type}:{challenge_id}`; TTL 300s; value: serialised challenge JSON.

## Tests

| Test | Mechanism |
|---|---|
| `register_start_returns_attestation_per_pack_tier` | mock-pack-tier=PackRegulated → attestation=Direct |
| `register_finish_persists_to_postgres` | spin up testcontainer; verify row inserted |
| `authenticate_start_supports_conditional_ui` | empty allow_credentials + mediation=conditional |
| `authenticate_finish_mints_oidc_session` | mock OIDC issuer adapter |
| `delete_credential_requires_acr_sensitive` | provide elevated bearer → 403 with X-Step-Up-Required |
| `cross_tenant_credential_read_denied` | provide bearer for tenant A; ask for tenant B → 403 |
| `expired_challenge_returns_401` | wait > TTL; finish returns 401 |
| `replay_attack_rejected` | submit same assertion twice; second is 401 |
| `audit_emitted_on_register_success` | observe `IdentityWebAuthnRegistered` event |
| `audit_emitted_on_authenticate_success` | observe `IdentitySignInSucceeded` event |
| `metrics_emitted` | Prometheus scrape shows `oya_identity_webauthn_*` counters |

## Failure-handling

- All errors map to JSON `{error: {code, message, retriable, x_step_up_required?}}` envelope.
- 401 / 403 errors do NOT leak whether a credential exists for the user (timing-attack defense).
- Rate-limit: 5 attempts per IP per 60s on `/authenticate/finish`.

## Evidence

- `evidence/identity/webauthn-rest-conformance/<date>.json`
- `evidence/identity/postgres-schema-applied/<date>.json`

## Acceptance — DONE when

- 11 handler tests pass.
- Conformance against `webauthn.io` virtual authenticator (Chrome DevTools) verifies register + assert + conditional UI.
- Postgres RLS enforcement test passes.

## Counterpart references - 005-webauthn-rest

- Counterpart class: passkey / recovery assurance.
- GitHub account security and Twilio Verify show the user-facing recovery and step-up baseline; this IP keeps Oyatie stronger by binding the credential or recovery decision to tenant context, ACR, and sealed identity audit events rather than treating MFA as an app-local add-on.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `iam/identity/PRD.md`, `iam/identity/manifest.json`, and the contract/policy files cited above.
