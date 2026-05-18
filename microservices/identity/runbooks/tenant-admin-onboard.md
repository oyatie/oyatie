---
doc_class: Runbook
runbook_id: identity-tenant-admin-onboard
microservice: identity
sev: planned
owner_team: axis-identity + ops-customer-success
date: 2026-05-18
---

# Runbook: Tenant admin onboard

## Purpose

Onboard a new tenant's admin user to the identity µservice with full credential provisioning, role assignment, SCIM bearer provisioning, and verification of first sign-in.

## Pre-requisites

- Tenancy µservice has provisioned the tenant (Zitadel Org created).
- Tenant's pack identified per ADR-0179 residency rules.
- Customer-success has provided admin's verified email + phone (for out-of-band ID-proofing).

## Procedure

1. **Create admin user**:
   ```
   oya identity user create \
     --tenant <tenant-id> \
     --pack <pack> \
     --userName <admin-email> \
     --displayName "<First Last>" \
     --role tenant-admin \
     --acr-floor sensitive
   ```
2. **Bootstrap WebAuthn**:
   - One-time-link email to admin (TTL 24h).
   - Admin clicks → WebAuthn register ceremony.
   - Verify a backup credential (YubiKey strongly recommended for tenant-admin role).
3. **Provision SCIM bearer** (if tenant uses SCIM provisioning from upstream):
   ```
   oya identity scim bearer create \
     --tenant <tenant-id> \
     --pack <pack> \
     --rotation-cadence-days 90
   ```
   - Bearer value printed ONCE; provided to tenant IT via secure channel (not email; secure secret-share).
   - Stored in OpenBao for rotation tracking.
4. **Configure upstream IdP federation** (if requested):
   ```
   oya identity federation add \
     --tenant <tenant-id> \
     --upstream <google-workspace|okta|entra|onelogin|ping> \
     --discovery-url <upstream-discovery-url>
   ```
5. **First sign-in test**:
   - Admin signs in via WebAuthn (or upstream federation).
   - Verify `IdentitySignInSucceeded` event sealed.
   - Verify session has `acr=elevated` (Passkey).
6. **Step-up test**:
   - Admin invokes a `sensitive`-class operation (e.g., create another user).
   - Verify step-up flow triggers; admin re-presents Passkey or hardware key.
7. **Audit verification**:
   - `oya identity audit replay --tenant <tenant-id> --since 1h` shows expected events.
8. **Customer-success handoff**:
   - Provide admin with: docs link, support contact, status page URL.
   - Note pack residency boundary and which SLOs apply.

## Verification checklist

- [ ] User exists in Zitadel Org for the tenant.
- [ ] At least one Passkey registered (verify AAGUID + transports).
- [ ] Backup credential registered (YubiKey or 2nd Passkey).
- [ ] SCIM bearer (if applicable) is valid and rotation scheduled.
- [ ] Upstream federation (if applicable) verified end-to-end.
- [ ] Step-up flow works.
- [ ] Audit-chain has the full provisioning trail.

## Rollback

If admin is unable to sign in within 24h:
- Investigate via `scim-provisioning-debug` runbook.
- If irrecoverable: `oya identity user reset <tenant> <user>` (operator-mediated; audit-trailed).

## Postmortem trigger

Onboard failure > 4 cycles in 30 days for the same tenant → product review; possible documentation gap.
