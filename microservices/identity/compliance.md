---
doc_class: Compliance
template_id: TPL-COMPLIANCE
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + council-compliance
frameworks: [SOC2-CC6, ISO27001-A9, GDPR-Art32, HIPAA-164308a4, NIST-SP-800-63B, PCI-DSS-v4-Sec8]
---

# Compliance — identity µservice

Cross-framework mapping. Every control declares: framework citation, evidence path, gate/lane that enforces, owner.

## SOC 2 — Trust Services Criteria (2017, revised 2024)

| Control | Citation | Implementation | Evidence | Lane / Gate |
|---|---|---|---|---|
| CC6.1 | Logical and physical access controls | OIDC bearer + Cedar PDP per ADR-0183; mTLS via SPIFFE; Postgres RLS | `IdentityOidcTokenIssued` events; Cedar policy artefacts | lean-a17-authz-tier-discipline |
| CC6.2 | Authentication of users | Passkey/WebAuthn L3 first; TOTP fallback; SMS forbidden | `IdentityWebAuthnRegistered`; FIDO-MDS3 attestation | webauthn-rs conformance test |
| CC6.3 | Authorisation | Cedar PDP + ACR step-up gates per ADR-0189 | `IdentityStepUpGranted`; per-policy deny audit | lean-a15-step-up-acr-coverage |
| CC6.6 | Logical access boundaries | Per-tenant Postgres RLS; per-pack Zitadel Instance; no cross-pack replication | residency Cedar policy; NetworkPolicy artefact | layered-architecture-discipline fitness gate |
| CC6.7 | Restriction of access to information assets | OpenBao SecretReference for all signing keys; HSM in regulated packs | OpenBao audit-emit; KekAttested events | lean-a11-raw-secret-emission |
| CC6.8 | Prevention of unauthorised changes | Audit-chain Merkle + Ed25519 seal of every change | `IdentityUserProvisioned`, `IdentityScimRequestReceived` events | audit-emit-completeness SLO ≥ 1.0 |
| CC7.2 | System monitoring | Grafana dashboards `identity-overview`, `passkey-funnel`, `scim-provisioning-health` | dashboard JSON in `dashboards/` | dashboard-coverage gate |

## ISO 27001 — Annex A (2022)

| Control | Citation | Implementation |
|---|---|---|
| A.5.16 | Identity management | this entire µservice |
| A.5.17 | Authentication information | WebAuthn credentials + OIDC sessions; rotation per JWKS schedule |
| A.5.18 | Access rights | Cedar policy per resource; ACR-gated for sensitive |
| A.8.2 | Privileged access rights | `acr=critical` for admin ops; JIT IT-approval bridge |
| A.8.3 | Information access restriction | per-tenant SCIM bearers; per-pack residency |
| A.8.5 | Secure authentication | passkey-first; phishing-resistant by default |
| A.8.6 | Capacity management | capacity-model.md ceilings; auto-scale Zitadel pods |
| A.8.9 | Configuration management | Helm + Kustomize; per-pack overlays; manifest.json authoritative |
| A.8.15 | Logging | audit-chain seal of 18 distinct events |
| A.8.16 | Monitoring activities | OnCall paging on RotationOverdue, SignCountRegression, IdpFailover |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + denied panic/unwrap/expect in workspace lints |

## GDPR — Article 32 (Security of processing)

| Requirement | Implementation |
|---|---|
| (a) Pseudonymisation/encryption of personal data | TLS 1.3 in transit; AES-256-GCM at rest (Postgres) + HSM-backed signing keys |
| (b) Ongoing confidentiality, integrity, availability, resilience | Audit-chain integrity; per-pack HA; multi-AZ within pack |
| (c) Ability to restore availability and access in timely manner after incident | RTO 30s, RPO 0 (realtime tier per ADR-0152); Postgres PITR per pack |
| (d) Regular testing, assessing and evaluating the effectiveness | quarterly DR drill `identity-failover-drill`; pen test annually |

## HIPAA — §164.308(a)(4) (Information Access Management)

| Standard | Implementation |
|---|---|
| (i) Isolating health care clearinghouse functions | pack-us-healthcare is dedicated; cross-pack replication forbidden |
| (ii)(A) Access authorisation | Cedar policy with `principal.role` + tenant binding |
| (ii)(B) Access establishment & modification | SCIM provisioning with audit emit; HRIS reconciliation |
| (ii)(C) Access reviews | quarterly export of `oya-identity-user-list` per pack per tenant; reviewer-attested |

§164.312(a)(2)(i) Unique user identification: `userName` is unique per tenant; UUIDv7 server-assigned `id`.
§164.312(a)(2)(iii) Automatic logoff: per-ACR session age (4h elevated, 1h sensitive, 15min critical).
§164.312(a)(2)(iv) Encryption: AES-256-GCM at rest; HSM-backed signing.
§164.312(c)(1) Integrity: audit-chain Merkle hash.
§164.312(d) Person or entity authentication: WebAuthn AAL2-AAL3.
§164.316(b)(2)(i) Retention: 6 years for audit logs in pack-us-healthcare.

## NIST SP 800-63B (Digital Identity Guidelines, Dec 2024 revision)

| AAL | ACR (ADR-0189) | Authenticator types accepted | Reauthentication |
|---|---|---|---|
| AAL1 | routine | Passkey OR password+TOTP | every 24h |
| AAL2 | elevated | Passkey synced (multi-factor crypto) | every 4h |
| AAL2+ | sensitive | Passkey + recent presentation | every 1h |
| AAL3 | critical | Hardware authenticator FIDO-MDS3 L2+ + IT approval | every 15min |

Verifier impersonation resistance (AAL3 mandatory): FIDO2/WebAuthn satisfies via origin + RP-ID binding.

§5.1.3 SMS OTP: restricted; **NOT** accepted by oyatie identity µservice.
§5.2.5 Verifier compromise resistance: WebAuthn public-key model — verifier compromise does NOT reveal authenticator secret.
§5.2.7 Verifier-CSP key escrow: forbidden — private keys never leave authenticator.

## PCI-DSS v4.0 — Requirement 8 (Identify users and authenticate access)

| Sub-req | Implementation |
|---|---|
| 8.2.1 | Unique IDs per user |
| 8.3.1 | Strong authentication (MFA required) |
| 8.3.2 | Strong cryptography for authenticators (FIDO2 ECDSA-P256 / Ed25519) |
| 8.3.6 | MFA for all access to CDE (every `acr ≥ elevated` for finance-µservice routes) |
| 8.3.9 | Multi-factor authentication for non-console access into the CDE (any admin op = `acr=sensitive` or higher) |
| 8.6.1 | Application & system accounts (m2m via OIDC client_credentials grant + SPIFFE SVID) |
| 10.5.1 | Audit log retention ≥ 1 year, 3 months immediately available |

## KR PIPA (Personal Information Protection Act) — Enforcement Decree

| Article | Implementation |
|---|---|
| Art. 23 (sensitive info) | `BEHAVIORAL_TENANT_PRODUCT` + `SENSITIVE_PIPA_ART23` data classes refused outside pack-kr |
| Art. 28 (overseas transfer) | pack-kr is sovereign; no cross-pack identity replication |
| Art. 29 (safeguards) | HSM-backed KEK; OpenBao with audit emit; mTLS everywhere |
| Art. 30 (retention) | ≥1 year audit log; KR-FSS sector ≥5 years |

## Evidence inventory

| Evidence | Path | Cadence |
|---|---|---|
| User-list export per tenant | `evidence/identity-user-list-<pack>-<tenant>-<date>.json` | quarterly |
| Audit-chain proof of seal | `evidence/audit-chain-seal-identity-<window>.json` | weekly |
| AAGUID allowlist diff | `evidence/aaguid-allowlist-<pack>-<date>.json` | quarterly |
| JWKS rotation log | `evidence/jwks-rotation-<pack>-<window>.json` | daily |
| SCIM bearer rotation log | `evidence/scim-bearer-rotation-<pack>-<window>.json` | 90-daily |
| DR drill report | `evidence/identity-dr-drill-<date>.json` | quarterly |
| Pen test report | `evidence/identity-pen-test-<year>.pdf` | annual |
| DPIA approval | `evidence/dpia-approval-identity-<date>.json` | annual |
| SOC 2 attestation supporting evidence | `evidence/soc2-cc6-<period>.json` | annual |
