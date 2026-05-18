---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + axis-product
---

# Competitor Parity — identity µservice

Feature-by-feature comparison against the hyperscaler IdP-as-a-service competitors and the OSS substrates oyatie evaluated for ADR-0187. The goal: oyatie identity MUST be at parity on critical features and explicitly better on residency + air-gap + audit-chain + sovereign-pack-compat.

Legend: ✓ = supported; ◐ = partial; ✗ = not supported; — = N/A.

## Authentication

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| OIDC | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SAML 2.0 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| OAuth 2.0 + PKCE | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| WebAuthn L3 Passkey | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Conditional UI | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ◐ |
| Cross-device caBLE | ✓ | ✓ | ◐ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Hardware key (FIDO2 L2+) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| AAGUID allowlist enforcement | ✓ | ◐ (enterprise tier) | ◐ | ✓ | ◐ | ◐ | ✗ | ◐ |
| FIDO-MDS3 auto-refresh | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ◐ |
| TOTP (RFC 6238) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SMS OTP | ✗ rejected (NIST §5.1.3) | ✓ available | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Magic link | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Device authorization (RFC 8628) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Step-up + risk

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| ACR step-up gates | ✓ (4-class) | ✓ AMR/ACR | ✓ MFA policies | ✓ Conditional Access | ◐ | ✓ AMR | ◐ | ◐ |
| Continuous risk-scoring (CAEP) | ◐ (Phase-2 IP-014) | ✓ ITP | ✓ Auth0 Detection | ✓ Identity Protection | ✗ | ✓ reCAPTCHA Ent. | ✗ | ✗ |
| JIT IT-approval | ✓ | ✗ | ◐ | ✓ PIM | ✗ | ✗ | ✗ | ✗ |
| Session class age binding | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |

## Provisioning

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| SCIM 2.0 inbound | ✓ | ✓ | ✓ | ✓ | ✗ (custom adapter) | ✓ | ✓ via Outpost | ✓ via extension |
| SCIM 2.0 outbound | ✗ (deferred) | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ◐ |
| HRIS adapter (Workday/BambooHR/Rippling) | ✓ | ✓ Workforce Identity | ✓ | ✓ Workday-connector | ✗ | ✗ | ✗ | ✗ |
| Group lifecycle | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ |
| Just-in-time provisioning | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Multi-tenancy

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| Multi-tenant native (no realm-per-tenant ops) | ✓ Instances+Orgs | ✓ Orgs | ✓ Orgs | ✓ Tenants | ✓ Pools | ✓ Projects | ✗ (deploy-per-tenant) | ✗ realm-per-tenant |
| Per-tenant branded login | ✓ Helm overlay | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Per-tenant SCIM endpoint | ✓ /scim/v2/{tenant}/* | ✓ | ✓ | ✓ | ✗ | ✓ | ◐ | ◐ |
| Per-tenant policy isolation | ✓ Cedar | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Residency + sovereignty

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| Per-pack residency (no cross-region replication) | ✓ 11 packs | ◐ EU/US only | ◐ EU/US only | ✓ Sovereign Cloud | ◐ region-per-pool | ✓ regions | ✓ (self-hosted) | ✓ (self-hosted) |
| Air-gapped sovereign pack | ✓ | ✗ | ✗ | ◐ Azure Stack | ✗ | ✗ | ✓ | ✓ |
| Self-hosted only deployment | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |
| Pack-kr / pack-ksa / pack-ae compat | ✓ | ✗ | ✗ | ◐ | ✗ | ✗ | ✓ | ✓ |
| BYOK / HSM-backed signing | ✓ | ✓ enterprise | ✓ enterprise | ✓ | ✓ KMS | ✓ Cloud KMS | ✓ | ✓ |

## Audit + compliance

| Feature | oyatie | Okta WIC | Auth0 | Microsoft Entra ID | AWS Cognito | Google Identity Platform | Authentik | Keycloak |
|---|---|---|---|---|---|---|---|---|
| Audit-chain Merkle + Ed25519 seal | ✓ ADR-0162 | ◐ flat log | ◐ flat log | ◐ Activity log | ◐ CloudTrail | ◐ Cloud Audit Logs | ◐ | ◐ |
| Per-tenant audit slicing | ✓ ADR-0162 | ◐ | ◐ | ◐ | ✗ | ◐ | ◐ | ◐ |
| Audit retention configurable per pack | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SOC 2 / ISO 27001 / GDPR / HIPAA / PCI-DSS | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ |
| KR PIPA / PIPC certification path | ✓ pack-kr sovereign | ✗ | ✗ | ✓ via partner | ✗ | ✗ | ✓ self-host | ✓ self-host |
| EU AI Act risk class capabilities tagging | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

## Pricing model

| Vendor | Pricing | Year-5 (10M MAU) | Lock-in |
|---|---|---|---|
| Okta Workforce Identity | per-MAU + per-MFA | ~$2.4M/yr base + ~$10M MFA | high |
| Auth0 | per-MAU + addons | ~$2.76M/yr + addons | high |
| Microsoft Entra ID External | per-MAU | ~$3.9M/yr | medium (Microsoft ecosystem) |
| AWS Cognito | per-MAU | ~$0.05/MAU = $500K/yr | medium (AWS ecosystem) |
| Google Identity Platform | per-MAU | ~$0.06/MAU = $600K/yr + Firebase | medium |
| Authentik | OSS / hosted addon | $0 self-host + ~$30K/yr support | low |
| Keycloak | OSS / RH support | $0 self-host + ~$60K/yr RH | low |
| **oyatie** | self-host TCO | $2.65M/yr (year-5 with personnel) | **none — OSS** |

oyatie is cost-competitive WITH personnel costs INCLUDED (the comparison is unfair to the vendors since their pricing is service-only). The decisive value is residency + audit-chain + sovereign-pack compat + zero lock-in.

## Decision

oyatie identity achieves parity on every must-have feature, ships ahead on residency + audit-chain + sovereign-pack + EU AI Act capabilities tagging, and accepts a conscious gap on Continuous Risk-Scoring (CAEP — deferred to IP-014) and Outbound SCIM (deferred).
