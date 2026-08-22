# IAM Engineer — First Week on `cloud-iam`

Audience: an IAM/identity engineer with AWS IAM + Okta + SAML/OIDC experience joining the `cloud-iam-*` lane.
Goal: by Friday EOD you can issue a federated principal, write a Cedar policy, translate it to AWS IAM JSON, and walk
an Okta SAML inbound federation cycle end-to-end.

## Day 1 — read before touching

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — every gate is a Cedar eval; no policy in code.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — every principal carries `tenant_id`.
- `docs/decisions/ADR-0709-general-live-apex.md — Foundry runs as `oyatie.foundry.*` principals; understand the recursion.
- ADR-0329 + ADR-0330 + ADR-0331 — the ADR-0329/0330/0331 tenant_class model.
- The Cedar 4.3 language reference (vendored at `vendor/cedar-4.3.0/CHANGELOG.md`).

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-iam-week1 .worktrees/$USER-iam-week1
cd .worktrees/$USER-iam-week1
```

## Day 2 — bring up a loopback cloud-iam cell

```bash
make dev-cell.up CELL=iam-loopback-1 PROFILE=cloud-iam-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
```

Issue your first user:
```bash
./bin/oya iam user create \
  --tenant oyatie.b2b.smb.acme-software \
  --username alice \
  --email alice@acme-software.io \
  --auth-method totp
```

Expected: a principal `User::"oyatie.b2b.smb.acme-software/alice"` materialised in the Cedar entity store and an
enrollment URL valid for 15 min. Complete TOTP enrollment in the dev UI.

Issue a token + introspect:
```bash
TOKEN=$(./bin/oya iam token issue --principal "oyatie.b2b.smb.acme-software/alice" --ttl 1h --scopes "read:tasks")
./bin/oya iam token introspect --token "$TOKEN" | jq .
```

You should see `principal`, `tenant_id`, `scopes`, `expires_at`, `cedar_entity_uid`.

## Day 3 — write your first Cedar policy

`policies/acme-software/tasks-read-allow.cedar`:
```cedar
permit (
  principal in Role::"oyatie.b2b.smb.acme-software/engineer",
  action == Action::"read",
  resource in Application::"oyatie.b2b.smb.acme-software/tasks"
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.session.mfa_verified == true
};
```

Lint + dry-eval:
```bash
./bin/oya policy lint --tenant oyatie.b2b.smb.acme-software policies/acme-software/
./bin/oya iam authorize \
  --principal "oyatie.b2b.smb.acme-software/alice" \
  --action "read" \
  --resource "Application::\"oyatie.b2b.smb.acme-software/tasks\"" \
  --context '{"session":{"mfa_verified":true}}'
```

Expected: `decision=Allow`, `determining_policies=[acme-software/tasks-read-allow]`, `eval_micros≈180`.

## Day 4 — federate Okta inbound

In a separate Okta dev tenant (`https://oyatie-dev.okta.com`), create a SAML 2.0 app named `cloud-iam-acme-software` with
ACS URL `https://loopback.cloud-iam.oyatie.local/saml/v2/acs?tenant=oyatie.b2b.smb.acme-software`.

Register the IdP:
```bash
./bin/oya iam idp register \
  --tenant oyatie.b2b.smb.acme-software \
  --kind saml-2.0 \
  --name okta-workforce \
  --metadata-url https://oyatie-dev.okta.com/app/exk.../sso/saml/metadata
```

`cloud-iam` fetches the metadata, cosign-verifies the issuer (you accept on first use), and writes a `cloud_iam.idp.registered`
audit event. Test login from the Okta tile — you should land at a principal `User::"oyatie.b2b.smb.acme-software/alice@acme-software.io"`
with a JIT-provisioned role assignment derived from Okta group `engineering` → Cedar role `engineer`.

## Day 5 — Cedar → AWS IAM JSON translation

Translate the Day 3 Cedar policy to AWS IAM:
```bash
./bin/oya iam translate \
  --tenant oyatie.b2b.smb.acme-software \
  --source-policy policies/acme-software/tasks-read-allow.cedar \
  --target aws-iam-json \
  --output translated/tasks-read-allow.iam.json
```

Output:
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": "oyatie:ReadTasks",
    "Resource": "arn:oyatie:tasks:::tenant/oyatie.b2b.smb.acme-software/*",
    "Condition": {"Bool": {"oyatie:MfaVerified": "true"}}
  }],
  "Annotation": {
    "_cedar_digest": "blake3-256:7f2c…",
    "_cedar_source": "policies/acme-software/tasks-read-allow.cedar"
  }
}
```

Translation back is forbidden by design — the digest anchors the Cedar source as authority.

## What "done with week 1" means

- [ ] You can recite the ADR-0329/0330/0331 tenant_class model and the principal context it carries.
- [ ] You issued + introspected a token; you understand the entity-UID shape.
- [ ] You authored, linted, and dry-eval'd a Cedar policy.
- [ ] You federated an Okta SAML IdP end-to-end with JIT provisioning.
- [ ] You translated a Cedar policy to AWS IAM JSON and explained the annotation.
- [ ] You read ADR-0243 + ADR-0244 + ADR-0247.

## Rookie traps

1. **Skipping `tenant_id` in Cedar.** Policies without a `principal.tenant_id == resource.tenant_id` guard leak across tenants;
   the `lean-a3-tenant-trace` CI lane catches it.
2. **Issuing long-lived AWS Access Keys.** `cloud-iam` refuses static credentials — only STS, only short-lived.
3. **Hand-editing the entity store.** Direct `UPDATE` on the CockroachDB schema bypasses the audit chain; use `oya iam` CLI.
4. **Forgetting MFA in policy `when` clauses.** A policy without `context.session.mfa_verified` opens up password-only access.
5. **Trusting unsolicited SAML metadata.** Always cosign-verify the IdP issuer on first registration.
