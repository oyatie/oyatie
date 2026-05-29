---
doc_class: ResidencyPolicy
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + council-compliance
related_adrs: [ADR-0156, ADR-0179]
---

# Data Residency Policy — identity µservice

## Authoritative rule

Identity data NEVER crosses a regulatory-pack boundary. Per ADR-0179 sovereign-cloud-per-regional-pack, the µservice is deployed independently in each pack with no replication, no cache, no shared-state crossing the boundary.

## Per-pack data inventory

| Pack | Postgres event-store | WebAuthn credentials | SCIM bearers | JWKS keys | Audit emissions |
|---|---|---|---|---|---|
| kr | KR-Seoul region | KR-Seoul | KR-Seoul OpenBao + Thales Luna HSM | KR-Seoul | KR-Seoul audit-chain |
| eu | EU-Frankfurt | EU-Frankfurt | EU-Frankfurt OpenBao + AWS CloudHSM (FR) | EU-Frankfurt | EU-Frankfurt audit-chain |
| us | US-East | US-East | US-East OpenBao + AWS CloudHSM | US-East | US-East audit-chain |
| us-healthcare | US-East-HIPAA-Eligible | US-East-HIPAA | US-East-HIPAA OpenBao + AWS CloudHSM HIPAA | US-East-HIPAA | US-East-HIPAA audit-chain |
| jp | JP-Tokyo | JP-Tokyo | JP-Tokyo OpenBao | JP-Tokyo | JP-Tokyo |
| sg | SG-Singapore | SG-Singapore | SG-Singapore OpenBao | SG-Singapore | SG-Singapore |
| au | AU-Sydney | AU-Sydney | AU-Sydney OpenBao | AU-Sydney | AU-Sydney |
| in | IN-Mumbai | IN-Mumbai | IN-Mumbai OpenBao | IN-Mumbai | IN-Mumbai |
| br | BR-São Paulo | BR-São Paulo | BR-São Paulo OpenBao | BR-São Paulo | BR-São Paulo |
| ae | AE-Dubai | AE-Dubai | AE-Dubai OpenBao + Thales DPoD (UAE) | AE-Dubai | AE-Dubai |
| ksa | KSA-Riyadh | KSA-Riyadh | KSA-Riyadh OpenBao + Thales Luna (Riyadh) | KSA-Riyadh | KSA-Riyadh |

## Cross-pack scenarios explicitly forbidden

1. **No cross-pack WebAuthn credential replication.** A Passkey registered in pack-eu CANNOT be used to authenticate in pack-us. The user must register a separate Passkey in each pack where they have presence.
2. **No cross-pack SCIM provisioning.** Enterprise IdPs that operate in multiple regions register one SCIM connection per (tenant, pack).
3. **No cross-pack JWKS sharing.** Each pack has its own signing keys; JWKS endpoints are pack-specific URLs.
4. **No cross-pack audit-chain replication.** Each pack's audit-chain is independent; aggregate views (e.g., for SOC 2 compliance) are operator-driven per-pack queries, not live replication.
5. **No cross-pack federation upstream.** If a tenant uses Google Workspace + has presence in pack-us and pack-eu, two independent federation bindings exist (one per pack); ID tokens minted in one pack are NOT valid in another.

## Cross-pack tenant operation (legitimate)

A multinational tenant with users in EU + US registers as TWO independent tenant records:
- `tenant_acme_eu` in pack-eu.
- `tenant_acme_us` in pack-us.

Each tenant has its own users, credentials, audit log, billing relationship. The customer's central IT desk operates BOTH SCIM connections.

## Enforcement

- **Network**: NetworkPolicy denies pod-to-pod traffic across pack boundaries.
- **Cedar**: `forbid when { principal.pack != resource.pack }` (in `dual-context-residency.cedar`).
- **Kyverno**: admission denies any K8s resource in pack-X that references pack-Y.
- **OpenBao**: per-pack OpenBao instances; no cross-pack mTLS trust.
- **DNS**: per-pack `identity-<pack>.oyatie.com` endpoints; global `identity.oyatie.com` is a 404 + selector.
- **Audit**: any attempted cross-pack call emits `IdentityResidencyViolationAttempt` event with full context.

## Pack-overlay specifics

### pack-kr (KR PIPA)

- KR-FSS sector data (financial services tenants in KR) MUST use HSM-backed signing (Thales Luna).
- 5y audit-log retention.
- KR PIPC notification within 72h on breach.

### pack-eu (GDPR)

- GDPR Art. 32 encryption-at-rest mandatory.
- GDPR Art. 33 breach notification within 72h.
- DPIA per Art. 35 authored (see dpia.md).

### pack-us-healthcare (HIPAA)

- BAA in force is precondition for any tenant onboarding (Cedar enforces).
- 6y audit-log retention.
- HIPAA Breach Notification within 60d (Cedar context flag `baa_in_force`).

### pack-ksa (Sovereign KSA + KSA-CITC)

- Air-gapped deployment supported.
- Thales Luna HSM mandatory.
- Sovereign data lives ONLY in KSA-Riyadh; no offshore replica.

### pack-ae (UAE)

- Thales DPoD (UAE-hosted) for HSM.
- UAE data-protection regulation compliance evidence per quarterly review.

## Verification

- `oya identity residency verify --pack <pack>` — checks no cross-pack references in config.
- `oya-check-data-residency` CI lane.
- Monthly residency-audit report: `evidence/identity/residency-audit-<pack>-<date>.json`.
