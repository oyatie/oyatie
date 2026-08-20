---
doc_class: MigrationPlaybook
microservice: tenancy
vendor: Azure AD B2C (Premium P1 + P2)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Azure AD B2C → oyatie tenancy

Audience: a SaaS platform team running Azure AD B2C for multi-tenant customer identity. Drivers: conglomerate parent-child hierarchy + sovereign child veto + cryptographic audit-chain + per-pack residency + ~50% TCO reduction at 5M MAU + scoped-permit data isolation primitive.

## Why this migration matters

Azure AD B2C is excellent at:

- Mature multi-tenant identity for SaaS apps.
- Integration with the broader Microsoft Azure ecosystem.
- Custom Policies (XML-based) for highly customized flows.
- Strong SAML 2.0 + OIDC + WS-Federation support.
- High Azure region availability.

oyatie tenancy adds:

- **Conglomerate parent-child hierarchy** (per ADR-0313 + ADR-TEN-001) — Azure B2C has no hierarchy primitive; each tenant is flat.
- **Sovereign child veto** — children can deny parent data-plane access (per ADR-TEN-001 sovereign path).
- **Scoped permits** — `owns` does NOT auto-grant `data_read` (per ADR-TEN-001 § Decision).
- **Cryptographic audit-chain** — Azure Activity Log is server-mutable.
- **Per-pack residency enforcement** — Azure B2C regions are coarser; pack-level rules are not first-class.
- **~50% TCO reduction** at 5M MAU (oyatie paid tenant_class baseline self-hosted ~ $528k vs Azure B2C P2 ~ $1.0M).
- **Spinoff/divestiture ceremony** (per IP-journey-j127 + j133).
- **Workforce-personal-tenant boundary** (oyatie's dual-context isolation; Azure has no equivalent).

The trade-off: Azure B2C Custom Policies (XML) is mature for highly customized auth flows. oyatie's equivalent is Cedar policies + workflow-engine flows; some Azure-specific patterns may require redesign.

## Step 1 — Inventory the Azure AD B2C estate (≤ 1-2 weeks)

```bash
# Azure CLI export
az ad b2c user-flow list --resource-group $RG --output json > ./azure-b2c-export/user-flows.json
az ad b2c custom-policy list --resource-group $RG --output json > ./azure-b2c-export/custom-policies.json
az ad b2c application-list --resource-group $RG > ./azure-b2c-export/applications.json
az ad b2c user list --resource-group $RG --output json > ./azure-b2c-export/users.json
az ad b2c identity-provider list --output json > ./azure-b2c-export/idps.json

# For SaaS platforms with thousands of customer tenants:
# Iterate over each customer tenant
for tenant in $(cat ./customer-tenants.txt); do
    az ad b2c application-list --tenant-id $tenant > ./azure-b2c-export/per-tenant/$tenant-apps.json
done
```

Document:

- Tenant count (typical SaaS B2C: 1k-10k customer tenants).
- User count per tenant (typical: 100-50k users).
- Active Custom Policies (XML; their replacement plan).
- Inbound IdPs (Facebook, Google, custom OIDC).
- Outbound apps (SAML SPs + OIDC clients).
- Tenant-level branding + UX customizations.
- Azure AD B2C MAU bucket usage.

## Step 2 — Map Azure AD B2C concepts to oyatie tenancy (≤ 1 week)

| Azure AD B2C concept | oyatie tenancy equivalent |
|---|---|
| B2C Tenant | Tenant |
| Customer user (member of tenant) | Principal (per identity µservice; ADR-ID-001) |
| User Flow | identity µservice OIDC flow + Cedar policy |
| Custom Policy (XML) | Cedar policy + workflow-engine flow |
| Identity Provider (Facebook, Google, etc.) | External IdP federation (identity µservice; per IP-011) |
| B2C Application (registered app) | OIDC client registered with oyatie identity |
| Multi-tenant scope (apps consumed across tenants) | Cross-tenant Cedar permits (per ADR-TEN-001) |
| Premium P1/P2 features (MFA, conditional access) | identity µservice continuous risk scoring + session step-up |
| Azure Activity Log | Audit-chain (`tenancy.*` events) |
| Tenant branding (logo, colors) | tenant metadata + UI theme |
| Region (Azure region) | Cell (per ADR-0009) |

## Step 3 — Data migration (≤ 4-12 weeks)

```sh
oya tenancy migrate import-azure-ad-b2c \
    --target-conglomerate-root acme-saas \
    --azure-b2c-export-dir ./azure-b2c-export/ \
    --map-each-b2c-tenant-to-oyatie-tenant true \
    --create-conglomerate-hierarchy true \
    --conglomerate-root-tenant acme-saas \
    --import-applications-as-oidc-clients true \
    --convert-custom-policies-best-effort true \
    --throttle-rate 100-tenants-per-min
```

The migration:

1. Creates oyatie conglomerate root (e.g., acme-saas).
2. Creates an oyatie tenant per Azure B2C tenant.
3. Builds parent-child relationships (acme-saas owns each customer tenant).
4. Creates initial scoped permits (acme-saas can manage billing on each customer tenant).
5. Creates principals from B2C users via identity µservice import.
6. Maps user flows + IdPs to identity µservice configurations.
7. Converts Custom Policies (XML) to Cedar policies + workflow-engine flows (best-effort; manual review required).
8. Imports applications as OIDC clients.

Backfill rate ~ 100 tenants/min at paid tenant_class baseline. 10k tenants → ~ 100 min.

Verify:

```sh
oya tenancy conglomerate show --root-tenant acme-saas
# Output:
#   conglomerate_root: acme-saas
#   total_tenants: 10 247
#   max_depth: 1 (flat customer tenants under root)
#   imported_from: azure-ad-b2c
```

## Step 4 — Cedar policy + workflow migration (≤ 4-8 weeks)

Custom Policies (XML) require manual review. The auto-converter handles common patterns:

| Azure Custom Policy element | oyatie equivalent |
|---|---|
| ClaimsProvider (technical profile) | identity µservice OIDC flow |
| OrchestrationStep | workflow-engine step |
| ClaimsTransformation | Cedar policy fragment OR workflow step |
| InputClaim / OutputClaim | OIDC token claim mapping |
| TrustFrameworkPolicy | tenant-level policy bundle |
| RelyingParty | OIDC client registration |

Use `oya tenancy custom-policy convert` to generate stubs:

```sh
oya tenancy custom-policy convert \
    --xml-input ./azure-b2c-export/custom-policies/B2C_1A_SignUpOrSignin.xml \
    --output-dir ./converted-policies/
# Output:
#   - converted-policies/B2C_1A_SignUpOrSignin.cedar
#   - converted-policies/B2C_1A_SignUpOrSignin.workflow.yaml
#   - converted-policies/B2C_1A_SignUpOrSignin.review-notes.md  # manual review items
```

## Step 5 — Application cutover (≤ 8-16 weeks)

For each application using Azure B2C:

1. Register the app as OIDC client with oyatie identity µservice.
2. Update the app's OIDC discovery URL (from Azure B2C tenant to oyatie cell endpoint).
3. Test in shadow.
4. Cutover.

During shadow, use bridge mode: oyatie identity can accept Azure B2C tokens via external IdP federation.

## Step 6 — DNS + branding cutover (≤ 1 week)

```
# Old: Azure B2C tenant URL
acme-saas.b2clogin.com/.well-known/openid-configuration
↓
# New: oyatie identity issuer URL per customer tenant
identity.acme-saas.oyatie.local/{customer_tenant_id}/.well-known/openid-configuration
```

Or use custom domain delegation per customer tenant (typical for white-label SaaS):

```
identity.{customer-tenant}.acme-saas-customer-domain.com → oyatie identity
```

## Step 7 — Azure AD B2C decommission (≤ 90-180 d post-cutover)

- Export final user data + audit logs.
- Cancel Azure B2C subscription.
- Retain audit log archive for legal-hold duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Custom Policy (XML) translation fidelity | High | Auto-converter handles 70% of common patterns; manual review for remaining 30%; provide bridge for legacy custom policies during transition |
| User MFA factor migration | High | Cannot import; users re-enroll passkey on first login; identity µservice provides bridge through external IdP federation |
| Inbound IdPs (Facebook, Google, social) | Medium | Re-configure via identity µservice external IdP federation; users may need to re-authorize on first login |
| Multi-tenant app re-registration | High | Each app needs new OIDC client config in oyatie; coordinated cutover |
| Azure Activity Log → audit-chain migration | Medium | Historical Azure logs imported for read-only reference; new events go to audit-chain |
| Tenant-level branding (logo, colors) | Low | oyatie tenant metadata supports branding; re-apply per tenant |
| Localization / language packs | Medium | oyatie supports per-tenant locale; re-configure |
| Azure B2C P2 conditional access policies | High | Cedar policy migration + continuous risk scoring (identity µservice IP-014) |
| Per-tenant custom UX (HTML/JS for sign-in pages) | Medium | oyatie's identity µservice supports per-tenant UX; manual port required |
| Azure B2C user import format (UPN) | Low | identity µservice imports preserve UPN as principal_id |
| Azure subscription billing transition | Medium | Run both for 30-60 d during shadow; cutover invoicing at end of period |
| SAML 2.0 IdP integrations | Medium | oyatie identity supports SAML 2.0 IdP via Zitadel backend (per IP-001) |
| Mobile app SDK transition (MSAL → oyatie SDK) | Medium | Provide TypeScript/Swift/Kotlin SDK; user-facing OIDC flow unchanged |
| Custom user attributes | Low | oyatie principal extended attributes; preserve attribute names |
| Tenant-data isolation (already enforced in Azure) | Low | oyatie's Cedar + RLS is stronger isolation primitive than Azure's |
| External app integrations expecting Azure B2C tokens | Medium | OIDC token format compatible; claim names may differ (e.g., `tenant_id` vs `tid`); SDK shim provided |
| Performance: 10k tenant managed by single Azure B2C tenant migration | Medium | oyatie supports multi-tenant SaaS at scale; conglomerate hierarchy + sovereign child available |
| Compliance certifications during transition | High | Bridge mode + dual-attestation for SOC 2 + ISO 27001 during transition period |
| Custom claim transformation chains | High | Manual Cedar policy + workflow-engine flow port; auto-converter provides stub |
| Identity provider trust framework | Medium | oyatie supports OIDC + SAML federation; manual config per IdP |
| Multi-region (Azure → oyatie cells) | Low | Map Azure region to oyatie cell; preserve residency requirements via pack overlays |
