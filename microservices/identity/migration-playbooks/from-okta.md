---
doc_class: MigrationPlaybook
microservice: identity
vendor: Okta (Workforce Identity, Customer Identity, Adaptive MFA)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Okta → oyatie identity

Audience: a team running Okta Workforce Identity for a corporate workforce IdP. Drivers: passkey-primary by default + operator-undecryptable recovery + dual-context isolation + sovereign-pack residency + ~4× TCO reduction at 50k seats + cryptographic audit-chain.

## Why this migration matters

Okta is excellent at:

- Best-in-class enterprise IdP UX.
- Massive integration ecosystem (Okta Integration Network has 7 000+ apps).
- Strong adaptive MFA + risk-engine.
- Mature SCIM provisioning.
- Okta Verify mobile authenticator.

oyatie identity adds:

- **Passkey-primary by default** (per ADR-ID-001; Okta supports passkey via FastPass but defaults to MFA-with-password).
- **Operator-undecryptable recovery envelope** (per ADR-ID-001; Okta admin can reset user credentials).
- **Dual-context (work + personal) principal isolation** (Okta uses separate Okta orgs for B2C; no aggregated dual-context view).
- **Cryptographic audit-chain** (Okta System Log is server-mutable).
- **Sovereign-pack residency** (KR-PIPA, EU-GDPR Art 9, US-HIPAA, FedRAMP-High, CN-PIPL).
- **~4× TCO reduction** at 50k seats ($888k self-hosted vs $3.7M Okta).
- **Per-pack AAGUID allowlist** at paid with compliance_pack gating.
- **Per-tenant FIPS-140-3 L3 HSM custody** at paid with compliance_pack gating.

The trade-off: Okta Integration Network's 7 000+ apps is the largest in the industry. oyatie's plugin SDK can host most enterprise SaaS apps via OIDC + SCIM, but the breadth of pre-configured integrations is smaller at launch.

## Step 1 — Inventory the Okta estate (≤ 1-2 weeks)

```bash
# Export Okta organization data
okta-cli organization export \
    --output ./okta-export/ \
    --include users,groups,apps,scim-config,mfa-factors,policies,sign-on-policies

# Or use Okta API for programmatic export
curl -X GET "https://acme-corp.okta.com/api/v1/users?limit=200" \
    -H "Authorization: SSWS $OKTA_API_TOKEN" \
    > ./okta-export/users.json
```

Document:

- User count + group count (typical: 50k users, 500-5000 groups for an enterprise).
- Active applications + their SAML/OIDC configuration.
- SCIM-enabled applications + their attribute mappings.
- MFA factors enrolled per user (push, TOTP, SMS, WebAuthn, security key).
- Sign-on policies + adaptive MFA rules.
- Domain configuration + Okta-hosted login URLs.
- Custom Okta workflows.
- Universal Directory schema customizations.
- Inbound IdPs (Active Directory, LDAP, social).
- Outbound IdPs (apps consuming Okta as IdP).

Typical mid-size: 5k-50k users, 50-500 active apps, 100-1000 sign-on policies.

## Step 2 — Map Okta concepts to oyatie identity (≤ 1 week)

| Okta concept | oyatie identity equivalent |
|---|---|
| Okta Organization | Tenant |
| Okta User | Principal (per tenant + audience_type) |
| Okta Group | Cedar role |
| Okta Application (SAML/OIDC) | Outbound OIDC client registered with oyatie issuer |
| Okta Active Directory / LDAP integration | External IdP federation (inbound) |
| Okta Universal Directory schema | oyatie principal extended attributes |
| Okta Sign-On Policy | Cedar policy on `identity::session::*` |
| Okta Adaptive MFA | Continuous risk scoring + Cedar `identity::session::step_up` |
| Okta Verify mobile app | oyatie mobile passkey enrollment |
| Okta FastPass | Hardware-backed passkey (no oyatie-specific app needed) |
| Okta SCIM Provisioning | SCIM 2.0 per ADR-identity-003 |
| Okta System Log | Audit-chain (`identity.*` events; cryptographically sealed) |
| Okta Threat Insight | Per-cell continuous risk scoring per IP-014 |
| Okta Workflows | `workflow-engine` µservice flows |

## Step 3 — Data migration (≤ 2-6 weeks per 50k users)

```sh
oya identity migrate import-okta \
    --tenant acme-corp \
    --okta-export-dir ./okta-export/ \
    --map-okta-org-to-tenant acme-corp \
    --preserve-user-ids true \
    --preserve-group-memberships true \
    --convert-mfa-factors-to-passkey-enroll-required true \
    --import-sign-on-policies-as-cedar true \
    --throttle-rate 100-users-per-sec
```

The migration:

1. Creates oyatie tenant from Okta organization.
2. Creates oyatie principals from Okta users (preserve user_id + email + display_name + custom attributes).
3. Creates Cedar roles from Okta groups (preserve memberships).
4. Imports SAML/OIDC apps as outbound OIDC clients.
5. **No password import** — passkey-bootstrap mandatory on first login (per ADR-ID-001 § Decision).
6. **No MFA factor import** — users re-enroll passkey on first login (replaces SMS/TOTP/push).
7. Imports Okta sign-on policies as Cedar policy fragments (manual review required for complex rules).
8. Imports Okta System Log into oyatie audit-chain for historical reference (1-year retention).

Backfill rate ~ 100 users/sec at paid with per_seat billing_component. 50k users → ~ 8 minutes (data migration; passkey enrollment is per-user manual).

Verify post-import counts:

```sh
oya identity tenant stats --tenant acme-corp
# Output:
#   total_principals: 50 421
#   total_groups: 1 248
#   active_applications: 247
#   sign_on_policies: 84
#   pending_passkey_enrollment: 50 421 (all users need to enroll passkey on first login)
#   imported_from: okta
```

## Step 4 — Application integration (≤ 4-12 weeks)

For each Okta-integrated SaaS app:

```sh
# Register the app as an OIDC client with oyatie
oya identity oidc client register \
    --tenant acme-corp \
    --client-id slack-acme \
    --client-name "Slack Enterprise" \
    --redirect-uris https://acme-corp.slack.com/sso/saml/* \
    --grant-types authorization_code,refresh_token \
    --scopes openid,profile,email,groups \
    --token-signing-alg EdDSA   # or ES256 per app compatibility

# Update the app's OIDC configuration to point at oyatie
# (In the app's admin console, update OIDC discovery URL from https://acme-corp.okta.com to https://identity.acme-corp.oyatie.local)
```

Slack/M365/etc. continue to work; the OIDC discovery URL change is invisible to end users.

## Step 5 — Shadow run + cutover (≤ 8-16 weeks)

Phase 1 (weeks 1-4): Okta remains primary. oyatie identity stands up alongside. Test users (~ 5%) enroll passkey on oyatie.
Phase 2 (weeks 5-8): App-by-app cutover. SAML/OIDC config updated to point at oyatie. Okta remains active for un-migrated apps.
Phase 3 (weeks 9-12): User passkey enrollment campaign. Each user enrolls passkey on oyatie (first login redirects to enrollment flow).
Phase 4 (weeks 13-16): Final apps migrated. Okta becomes read-only for archive.

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.identity_substrate.cut_over \
    --payload '{"from":"okta","to":"oyatie","cutover_at":"2026-09-15T14:00:00Z"}'
```

## Step 6 — Okta decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final Okta System Log for archive.
- Cancel Okta Workforce Identity contract.
- Retain Okta archive read-only for legal-hold + audit duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| User passkey enrollment compliance (50k users all enrolling) | High | Communications + IT-led enrollment campaigns; provide YubiKeys for high-risk roles; passwordless self-service for synced passkeys |
| App integration breakage on OIDC URL change | High | Pre-test each app in shadow; provide rollback procedure |
| Okta Workflows + custom code | High | Port to workflow-engine µservice; manual review required |
| Okta Adaptive MFA rule fidelity | Medium | Cedar policy fragments may not 1:1 match Okta's risk engine; tune over 60-90 d |
| MFA factor migration (TOTP, SMS, push) | High | Cannot import; users re-enroll passkey; provide bridge using oyatie's identity µservice OIDC federation back to Okta for transition |
| AD/LDAP integration | Medium | External IdP federation continues to work; oyatie accepts Okta tokens during bridge |
| Universal Directory custom schema | Medium | Map to oyatie principal extended attributes; preserve attribute names |
| Sign-on policies (Network Zones, etc.) | High | Cedar policy fragments; some Okta-specific concepts (Network Zone) require manual re-modeling |
| Okta Threat Insight integration | Medium | oyatie's continuous risk scoring per IP-014 replaces; behavioral signals + IP reputation + device fingerprinting |
| Mobile push notifications (Okta Verify) | Low | Passkey enrollment on mobile replaces; Apple/Google synced passkeys widely available |
| Okta Integration Network app gap | High | Pre-audit; top-50 apps must have oyatie OIDC client config ready before cutover; tail of long apps may need plugin SDK work |
| Okta-hosted login URL change | Medium | Set up 301 redirects from acme-corp.okta.com to identity.acme-corp.oyatie.local |
| Tenant admin Okta API automation | Medium | Re-implement against oyatie identity REST API; SDK provided in Rust/TypeScript/Python/Go |
| User self-service password reset → recovery envelope | High | User training: recovery now requires passphrase + recovery code; no admin-reset path |
| MFA factor self-service enrollment | Medium | Replace with passkey self-enrollment + recovery envelope creation |
| SAML 2.0 apps (oyatie focuses on OIDC) | Medium | oyatie supports SAML 2.0 via Zitadel backend; for migration, SAML config + IdP metadata can be re-issued |
| Customer Identity (CIAM) use cases | Medium | oyatie consumer_low_risk session class + audience_type=personal for B2C |
| FedRAMP / DoD authorization status | High | paid with compliance_pack gating tenant_class provides FedRAMP-High equivalent; Okta GovCloud may be required for some govt contracts during transition |
| HRIS sync (Workday/BambooHR) | Medium | Re-configure HRIS adapter per IP-009 to write to oyatie SCIM endpoint |
| Multi-domain Okta org (single Okta org → multiple oyatie tenants) | Medium | Pre-plan tenant mapping; conglomerate hierarchy per ADR-TEN-001 supports parent-child |
| Legacy SAML apps without OIDC support | Medium | oyatie SAML 2.0 IdP for legacy apps; passkey ceremony bridged via redirect |
| Compliance audit during transition (SOC 2, ISO 27001) | High | Pre-coordinate with auditor; provide bridge evidence showing both systems active + audit trail continuous |
