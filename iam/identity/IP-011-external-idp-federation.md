---
doc_class: IP
ip_id: IP-011
microservice: identity
status: ga
related_adrs: [ADR-0187]
date: 2026-05-18
owner_team: axis-identity
---

# IP-011 — External IdP federation (Google Workspace / Okta / Microsoft Entra)

## Goal

Allow tenants to bind an upstream OIDC or SAML IdP — Google Workspace, Okta Workforce, Microsoft Entra ID, OneLogin, Ping Federate — as their source of user identity, so end-users SSO without re-onboarding. The federation flow: end-user clicks "Sign in with <upstream>"; Zitadel federates the OIDC discovery; upstream IdP authenticates; ID-token returned to Zitadel; Zitadel mints an oyatie ID-token (binding `upstream_idp` claim) and emits provisioning audit.

## Files

| File | Purpose |
|---|---|
| `crates/identity-external-idp-federation-kernel/Cargo.toml` | manifest |
| `crates/identity-external-idp-federation-kernel/src/lib.rs` | `ExternalIdpFederation` trait |
| `crates/identity-external-idp-federation-domain/src/lib.rs` | per-IdP claim mapping rules |
| `crates/identity-external-idp-federation-usecase/src/lib.rs` | binding lifecycle (add/remove/disable) |
| `crates/identity-external-idp-federation-adapter/src/lib.rs` | Zitadel external-IdP API wrapper |
| `crates/identity-external-idp-federation-api/src/lib.rs` | CRUD over bindings |

## Binding lifecycle

| Op | Effect |
|---|---|
| Add | Validate OIDC discovery URL; verify JWKS reachable; configure Zitadel external-IdP; emit `IdentityExternalIdpBound` |
| Update | Update claim mapping rules; keep existing user→external mappings |
| Disable | Refuse new sign-ins via this IdP; existing sessions continue until expiry |
| Remove | Disable + delete; pre-existing users keep their oyatie IDs but cannot SSO via this IdP |

## Claim mapping per upstream

| Upstream | sub claim | email | display_name | roles |
|---|---|---|---|---|
| Google Workspace | `sub` | `email` | `name` | `hd` (Hosted Domain) maps to tenant_id allowlist |
| Okta Workforce | `sub` | `email` | `name` | `groups` |
| Microsoft Entra ID | `oid` (object_id) | `preferred_username` or `email` | `name` | `roles` claim (after configuration) |
| OneLogin | `sub` | `email` | `name` | `groups` |
| Ping Federate | `sub` | `email` | `name` | `groups` |

## JIT provisioning on first federated sign-in

If a federated user signs in for the first time and no oyatie User exists for `(external_idp, external_sub)`:
- Create oyatie User with `userName = <email-normalised>`.
- Bind `IdpBinding{ identity_provider_id, external_subject }`.
- Emit `IdentityUserProvisioned(source=federation)`.
- Apply default role per tenant policy.
- Subsequent sign-ins resolve the same User.

## SAML support

Optional per tenant. SAML uses Zitadel's SAML IdP / SP integration. Tenants supply SP metadata URL + ACS endpoint; Zitadel handles assertion verification + claim mapping.

## Tests

| Test | Mechanism |
|---|---|
| `google_workspace_jit_provisions_user_on_first_signin` | mock Workspace OIDC; sign in; user created |
| `subsequent_signin_uses_same_user` | sign in twice; same oyatie User id |
| `claim_mapping_extracts_email_from_preferred_username` | mock Entra response with no `email` |
| `tenant_id_allowlist_via_hd_claim` | Workspace `hd` mismatch → 403 |
| `disable_binding_blocks_new_signins` | disabled; sign-in returns 403 |
| `remove_binding_keeps_existing_users` | post-remove; user can sign in with Passkey directly |
| `saml_assertion_verified` | mock SAML IdP; assertion verified |
| `mismatched_audience_rejected` | SAML assertion with wrong audience → 401 |
| `audit_emitted_per_binding_op` | observe `IdentityExternalIdpBound` etc. |
| `concurrent_binding_changes_etag_protected` | two PATCH; second 412 |

## Failure modes

- **Upstream IdP discovery down**: cached config served; alert.
- **JWKS rotation upstream**: refresh on `kid` miss.
- **Claim mapping yields empty userName**: refuse provisioning; emit error event.

## Acceptance — DONE when

- 10 tests pass.
- Live Google Workspace / Okta / Entra test connections verified in staging.
- SAML IdP roundtrip works end-to-end.

## Counterpart references - 011-external-idp-federation

- Counterpart class: issuer / federation.
- GitHub enterprise SSO and ServiceNow external IdP federation are the counterpart baseline for workforce login; this IP keeps Oyatie differentiated by preserving per-pack issuer boundaries, JWKS evidence, and provider-BYOK separation.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

