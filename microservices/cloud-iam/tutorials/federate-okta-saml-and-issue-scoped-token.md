# Tutorial — Federate Okta SAML, write a Cedar policy, issue a scoped token

Goal: stand up an Okta SAML inbound federation against a paid tenant_class tenant, write a Cedar policy that requires MFA + tenant scope,
and issue a short-lived scoped token that allows reading the `tasks` application. End-to-end on a loopback cell.

Pre-reqs:
- Loopback iam cell: `make dev-cell.up CELL=iam-loopback-1 PROFILE=cloud-iam-dev`
- Okta developer org (free tenant_class policy OK) — replace `oyatie-dev.okta.com` below with yours.
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`

## Step 1 — provision the Okta SAML app

In Okta admin → Applications → Create App Integration → SAML 2.0:
- App name: `cloud-iam-acme-software`
- Single-sign-on URL: `https://loopback.cloud-iam.oyatie.local/saml/v2/acs?tenant=oyatie.b2b.smb.acme-software`
- Audience URI: `urn:oyatie:cloud-iam:tenant:oyatie.b2b.smb.acme-software`
- Name ID format: `EmailAddress`
- Attribute statements: `email`, `groups` (filter regex `engineering|product|admin`)

Save the app, copy the IdP metadata URL (`https://oyatie-dev.okta.com/app/exk.../sso/saml/metadata`).

## Step 2 — register the IdP in cloud-iam

```bash
./bin/oya iam idp register \
  --tenant oyatie.b2b.smb.acme-software \
  --kind saml-2.0 \
  --name okta-workforce \
  --metadata-url https://oyatie-dev.okta.com/app/exk.../sso/saml/metadata \
  --jit-rules-file jit-rules.yaml
```

`jit-rules.yaml`:
```yaml
on_first_login:
  create_user: true
  user_uid_template: "User::\"{tenant_id}/{saml_email}\""
  email_attribute: email
group_to_role_map:
  engineering: "Role::\"{tenant_id}/engineer\""
  product:     "Role::\"{tenant_id}/product-owner\""
  admin:       "Role::\"{tenant_id}/admin\""
require_mfa_in_idp: true
```

Expected output:
```
idp_registered     : okta-workforce (saml-2.0)
metadata_signature : cosign-verified ✓
metadata_expires   : 2027-05-15T00:00:00Z (361 d)
jit_rules_lint     : OK
audit_chain_event  : ce-2026-05-20T09:12:04Z-…
```

## Step 3 — author the Cedar policy

`policies/acme-software/tasks-read-allow.cedar`:
```cedar
permit (
  principal in Role::"oyatie.b2b.smb.acme-software/engineer",
  action == Action::"read",
  resource in Application::"oyatie.b2b.smb.acme-software/tasks"
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.session.mfa_verified == true &&
  context.session.federation_idp == "okta-workforce"
};
```

Lint:
```bash
./bin/oya policy lint --tenant oyatie.b2b.smb.acme-software policies/acme-software/
```

Expected: `policies_parsed: 1, errors: 0, tenant_trace_ok: true`.

Push to the live entity store:
```bash
./bin/oya iam policy push \
  --tenant oyatie.b2b.smb.acme-software \
  --policy-file policies/acme-software/tasks-read-allow.cedar
```

## Step 4 — log in via Okta

In an incognito browser, visit `https://oyatie-dev.okta.com/app/UserHome`, click the `cloud-iam-acme-software` tile, sign in as
`alice@acme-software.io` (member of the `engineering` group).

Okta posts the SAML assertion to `cloud-iam`'s ACS endpoint. `cloud-iam`:
1. Validates the SAML signature against cached metadata.
2. Resolves JIT rules → materialises `User::"oyatie.b2b.smb.acme-software/alice@acme-software.io"` + attaches
   `Role::"oyatie.b2b.smb.acme-software/engineer"`.
3. Writes a `cloud_iam.federation.login` audit event.
4. Issues a session cookie + an opaque session token (4 h TTL).

## Step 5 — issue a scoped API token

From the dev UI (or via CLI assuming session):
```bash
./bin/oya iam token issue \
  --principal "User::\"oyatie.b2b.smb.acme-software/alice@acme-software.io\"" \
  --scopes "read:tasks" \
  --ttl 1h \
  --bind-to-fingerprint "$(./bin/oya iam current-session fingerprint)"
```

Output:
```
token            : eyJ0eXAiOi...   (opaque JWS, RS256)
issued_at        : 2026-05-20T09:21:33Z
expires_at       : 2026-05-20T10:21:33Z
principal        : User::"oyatie.b2b.smb.acme-software/alice@acme-software.io"
scopes           : [read:tasks]
fingerprint      : blake3-256:c9f4…  (bound to client)
```

## Step 6 — exercise the token

```bash
curl -H "Authorization: Bearer $TOKEN" \
     https://loopback.cloud-iam.oyatie.local/v1/authorize \
     -d '{
       "action": "read",
       "resource": "Application::\"oyatie.b2b.smb.acme-software/tasks\"",
       "context": {"session": {"mfa_verified": true, "federation_idp": "okta-workforce"}}
     }'
```

Expected:
```json
{
  "decision": "Allow",
  "determining_policies": ["acme-software/tasks-read-allow"],
  "eval_micros": 174,
  "audit_chain_event_id": "ce-2026-05-20T09:22:14Z-…"
}
```

## Step 7 — verify deny when MFA is missing

```bash
curl -H "Authorization: Bearer $TOKEN" \
     https://loopback.cloud-iam.oyatie.local/v1/authorize \
     -d '{
       "action": "read",
       "resource": "Application::\"oyatie.b2b.smb.acme-software/tasks\"",
       "context": {"session": {"mfa_verified": false, "federation_idp": "okta-workforce"}}
     }'
```

Expected:
```json
{
  "decision": "Deny",
  "determining_policies": [],
  "reasons": ["No permit matched: when-clause requires mfa_verified == true"],
  "eval_micros": 168
}
```

## Step 8 — revoke + audit

```bash
./bin/oya iam token revoke --token "$TOKEN" --reason "tutorial complete"
./bin/oya audit-chain query \
  --tenant oyatie.b2b.smb.acme-software \
  --since "1h ago" \
  --kind cloud_iam.token.*
```

You should see at minimum: `cloud_iam.token.issued`, `cloud_iam.authorize.allowed`, `cloud_iam.authorize.denied`, `cloud_iam.token.revoked`.

## What you just demonstrated

- Okta SAML federation with JIT provisioning + cosign-verified IdP metadata.
- Cedar policy with tenant-scoping + MFA-context guard.
- Scoped token issuance with fingerprint binding (≤ 1 h TTL).
- In-process Cedar eval in ≤ 200 µs (paid tenant_class policy SLO).
- Allow + Deny outcomes, both audit-chain-anchored.
- Revocation as a first-class Cedar action with its own audit event.
