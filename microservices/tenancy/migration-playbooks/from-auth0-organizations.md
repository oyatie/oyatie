---
doc_class: MigrationPlaybook
microservice: tenancy
source_vendor: Auth0 Organizations
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — Auth0 Organizations → oyatie tenancy

Audience: a platform-operator currently using Auth0 Organizations (Okta Customer Identity) for multi-tenant SaaS who wants to migrate tenant lifecycle + isolation to oyatie's `tenancy` µservice over 8-12 weeks.

Outcome: all Auth0 organizations migrated as oyatie tenants, per-tenant database isolation upgraded from Auth0's logical-only model, sovereign-pack residency enabled where applicable, Auth0 Organizations decommissioned.

## Phase 0 — discovery (week 1)

1. Inventory Auth0 Organizations:
   ```sh
   auth0 organizations list --json > organizations.json
   ```
2. For each organization:
   - Name + display_name.
   - Branding (logo, colors, custom domain).
   - Enabled connections (which IdPs the org allows).
   - Member count + member roles.
   - Custom metadata + app_metadata.
3. Inventory data isolation patterns in your app:
   - How does your app currently scope data per Auth0 org?
   - Is the org_id in JWT used as a query filter?
   - Are there cross-org-leak risks in your code?
4. Inventory commercial exposure:
   - Auth0 contract end date.
   - Commercial plan (B2B Pro / B2B Enterprise).
   - MAU (Monthly Active Users) consumption.
   - Number of Active Orgs.

Deliverable: `migration-plan.md`.

## Phase 1 — stand up oyatie + IAM bridge (week 2)

1. Deploy oyatie tenancy IaC into the target cell.
2. Configure oyatie iam µservice to federate with Auth0 during migration (Auth0 still issues tokens; oyatie validates the org_id claim).
3. Smoke-test: provision a sample tenant in oyatie; verify the tenant record + RLS policies + Cedar bindings exist.

## Phase 2 — tenant migration (weeks 3-5)

For each Auth0 Organization, provision an equivalent oyatie tenant:

```sh
oya tenancy tenant-provision \
    --legal-name "<auth0 display_name>" \
    --audience-type b2b-organization \
    --country-code <inferred from members> \
    --data-residency-region <inferred> \
    --external-id-mapping "auth0:<auth0_org_id>"
```

The `external-id-mapping` field links the oyatie tenant to the Auth0 org for in-flight reconciliation. Your app can look up either ID during the migration period.

For each member of the Auth0 org:
1. Create the equivalent oyatie iam principal.
2. Bind to the new oyatie tenant.
3. Map Auth0 role to Cedar role (typically `tenant_admin`, `tenant_member`).
4. Send invitation email with migration context.

Bulk migrate via:
```sh
oya tenancy bulk-migrate-from-auth0 \
    --input organizations.json \
    --members-input members.json \
    --dry-run  # remove for real migration
```

## Phase 3 — application code updates (weeks 6-7)

Your app currently:
- Receives Auth0 JWT.
- Extracts `org_id` claim.
- Uses `org_id` as a query filter (e.g. `WHERE auth0_org_id = X`).

Update your app to:
- Receive oyatie JWT (issued by `iam` µservice).
- Extract `tenant_id` claim.
- Use `tenant_id` (with the substrate-enforced RLS — your queries don't need explicit WHERE because RLS handles it).

This is the biggest code change. Plan: 1-2 weeks per major service, depending on how deeply Auth0 was integrated.

## Phase 4 — IdP federation cutover (week 8)

If your customers use Auth0 connections (Google/GitHub/SAML/etc), reconfigure those as oyatie iam federations:

```sh
oya iam federation-create \
    --tenant <oyatie-tenant-id> \
    --provider-type "saml" \
    --provider-config-from-auth0 <auth0-connection-id>
```

The substrate translates Auth0 connection config to oyatie iam federation config. Customer SAML metadata stays the same; only the redirect URLs change.

Communicate to customers: "On 2026-05-25 at 14:00 UTC, your login redirect URL will change from auth0.com to iam.your-platform.com. Bookmarks may need updating. Your password / SSO continues to work."

## Phase 5 — cutover (weeks 9-10)

1. Day-of-cutover: switch your app to issue oyatie JWTs instead of validating Auth0 JWTs.
2. Auth0 tokens already issued continue to work for their TTL (typically 24 h); new logins go through oyatie iam.
3. After 7 days (all Auth0 tokens expired): disable Auth0 token validation in your app entirely.
4. Monitor for: missed customer logins, broken integrations, RLS misconfigurations.

## Phase 6 — Auth0 wind-down (weeks 11-12)

1. Disable Auth0 Organizations (keep Auth0 if you still use it for other identity flows).
2. Receive final Auth0 invoice; pay residuals.
3. Update tenant ARCHITECTURE.md to reference oyatie tenancy + iam exclusively.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Auth0 org_id used as foreign key in your DB | Preserve as `legacy_id` in oyatie tenant record; updated app code uses `tenant_id` |
| Auth0 user_id ≠ oyatie principal_id | Create a mapping table during migration; gradually phase out reference to Auth0 user_id |
| Custom Auth0 rules (server-side JavaScript) | Reimplement as Cedar policies or `iam` µservice integrations; budget extra time |
| Auth0 Actions (custom auth flow hooks) | Map to oyatie iam hooks (PreSignIn, PostSignIn); some Actions require reimplementation |
| Customers using Auth0 universal login | They redirect to oyatie's iam universal login; update their bookmarks + documentation |
| Auth0 M2M tokens | Provision oyatie iam machine-tokens for the same purposes |
| Customer SAML connections | Re-provision; SAML metadata stays the same but redirect URLs change |
| Per-org branding (logo, colors) | Migrate to oyatie tenancy branding fields; substrate renders branded login pages |
