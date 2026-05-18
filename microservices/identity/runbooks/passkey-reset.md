---
doc_class: Runbook
runbook_id: identity-passkey-reset
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-security
sev: Sev-3 (single user) / Sev-2 (mass)
---

# Runbook: Passkey reset (user-initiated OR ops-mediated recovery)

## When to use

- User lost all Passkeys + TOTP fallback (full credential loss).
- Mass-passkey-reset event (authenticator vendor recall; FIDO-MDS3 marked AAGUID compromised).
- Account recovery for a verified-but-locked-out user.

## Pre-flight checks

- Identify user(s) by `tenant_id` + `user_id` via SCIM GET.
- Verify identity-proofing performed out-of-band (operator-mediated) — call known phone; verify via known IdP federation; confirm via tenant-CSO escalation channel.
- Open audit ticket: `evidence/identity/operator-action/<date>-<operator>-<user_id>.json`.

## Procedure (single user)

1. `oya identity user lookup --tenant <tenant> --user <email>` — confirm user exists.
2. `oya identity webauthn list --tenant <tenant> --user <user_id>` — enumerate credentials.
3. Operator confirms identity-proofing complete (sign off in audit ticket).
4. `oya identity webauthn revoke --tenant <tenant> --credential <cred_id> --reason operator-recovery --ticket <ticket-id>` — revoke each credential.
5. Pin user to `acr=critical` for next sign-in: `oya identity user pin-acr --tenant <tenant> --user <user_id> --acr critical`.
6. Email user with one-time-link to register a new Passkey (link bound to user_id; 30min TTL).
7. User clicks link → WebAuthn register ceremony → new Passkey provisioned.
8. Audit chain emits `IdentityWebAuthnRegistered(source=operator-recovery)`.
9. Operator closes audit ticket.

## Procedure (mass reset — FIDO-MDS3 AAGUID compromise)

1. ops-security identifies affected AAGUID from FIDO-MDS3 `REVOKED` status.
2. `oya identity webauthn list-by-aaguid --aaguid <AAGUID>` — enumerate affected credentials globally.
3. Per-pack notification:
   - `oya identity webauthn batch-revoke --aaguid <AAGUID> --reason "authenticator-vendor-revoked" --pack <pack>`.
4. Per-tenant comms: emit `IdentityMassPasskeyReset` event to tenancy µservice; tenancy sends per-tenant email batch.
5. Per-affected-user comms: email with re-registration link.
6. Pin all affected users to `acr=sensitive` for next sign-in (forces re-registration with non-revoked authenticator).
7. After 30 days, hard-revoke any not-yet-re-registered credentials.

## Verification

- Per user: `oya identity webauthn list --tenant <tenant> --user <user_id>` shows only the new credential.
- Audit chain: replay `IdentityWebAuthnRegistered(source=operator-recovery)` events.
- SLO impact: no degradation in `webauthn-authenticate-latency`.

## Rollback

- If a credential was revoked in error: restore from soft-delete tombstone (within 30d).
- Audit event `IdentityWebAuthnUndelete` emitted.

## Communication template

```
Subject: Action required — re-register your Passkey

Your security key/Passkey was revoked due to <reason>. Please click the link below
to register a new Passkey before <deadline>:

<one-time-link>

After <deadline>, you will need to contact your IT administrator.

— oyatie identity
```

## Postmortem trigger

Mass reset (Sev-2) → blameless postmortem within 7 days.
