---
doc_class: DPIA
template_id: TPL-DPIA
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + council-compliance
gdpr_article: 35
related_adrs: [ADR-0156, ADR-0162, ADR-0187, ADR-0188, ADR-0189, ADR-0190]
---

# DPIA — identity µservice (GDPR Art. 35)

This Data Protection Impact Assessment covers the processing performed by the `identity` µservice, which by definition holds and processes PII for every end-user of every consumer µservice. Per GDPR Art. 35(3)(b) "systematic and extensive evaluation of personal aspects... including profiling" applies because step-up authentication evaluates the user's authentication-context class (`acr`).

## 1. Description of the processing

| Item | Value |
|---|---|
| Controller | tenant (B2B oyatie customer) |
| Processor | oyatie (this µservice) |
| Nature of processing | identity verification, credential storage, session management, provisioning lifecycle, audit emission |
| Scope | every authenticated end-user of every consumer µservice |
| Context | multi-tenant SaaS; per-pack residency (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa) |
| Purposes (Art. 5(1)(b)) | authn, authz, provisioning, audit (each declared in OIDC `purpose` claim) |

## 2. Data categories processed

| Category (ADR-0156 PII registry) | Examples | Pack residency | Retention |
|---|---|---|---|
| PII_IDENTIFYING | `userName`, `email`, `externalId`, `displayName`, `id` | per-pack | active + purpose-bounded; 30d post-deletion (GDPR Art. 17 grace), 6y for US-healthcare (HIPAA §164.316(b)(2)) |
| PII_QUASI_IDENTIFYING | `givenName`, `familyName`, `department`, `costCenter`, `manager` | per-pack | as above |
| AUTHENTICATION | WebAuthn `credential_id`, `public_key` (CBOR), `aaguid`, `transports`, `sign_count`, OIDC `jti`, refresh-token hash | per-pack | active + 24h post-revoke for replay-defense; rotated keys disposed |
| AUDIT | per-event seal (Merkle leaf hash + Ed25519 sig + tenant_id + user_id + acr + reason) | per-pack | KR PIPA ≥1y; HIPAA 6y; PCI-DSS ≥1y/3mo immediate |
| INTERNAL_ONLY | per-tenant SCIM bearer hash, per-tenant Zitadel Org ID, OpenBao path | per-pack | active + 90d post-rotate |

Data NOT processed: race, religion, political opinion, biometric template, health data, sexual orientation, criminal record (all forbidden in this µservice per ADR-0156).

## 3. Lawful basis (GDPR Art. 6)

- **Contract performance** (Art. 6(1)(b)) — the user is using a tenant's service; identity verification is necessary to deliver it.
- **Legitimate interests** (Art. 6(1)(f)) — fraud prevention via step-up + audit; balancing test in §6 below.
- **Legal obligation** (Art. 6(1)(c)) — audit-log retention to satisfy AML/KYC where applicable (FIN/FSS regs in pack-kr regulated tier).

Consent (Art. 6(1)(a)) is NOT a basis for this processing; it is contract-and-legitimate-interest only. Users cannot withdraw "consent" to be identified while continuing to use the tenant's service.

## 4. Necessity & proportionality

- **Minimal collection**: only `userName` + `email` are required; `displayName`, `givenName`, `familyName` are optional. WebAuthn collects only the public key; the private key never leaves the user's device.
- **Purpose-bound claims**: every OIDC token carries a `purpose` claim (per ADR-0145); downstream Cedar policy refuses cross-purpose reuse.
- **Per-pack residency**: cross-pack identity replication is forbidden by ADR-0179 sovereign-cloud-per-regional-pack.
- **No profiling for marketing**: ACR evaluation is operational decisioning (auth posture), not behavioural profiling for commercial use.

## 5. Subject rights (GDPR Art. 12-22)

| Right | Article | Implementation |
|---|---|---|
| Access | Art. 15 | SCIM `GET /scim/v2/{tenant}/Users/{me}` + `oya` CLI `identity export` |
| Rectification | Art. 16 | SCIM PATCH; or via external IdP if federated |
| Erasure | Art. 17 | SCIM DELETE → tombstone → 30d grace → hard-delete; cascade DSR per ADR-0156 |
| Restriction | Art. 18 | SCIM PATCH `active=false` |
| Portability | Art. 20 | SCIM GET in standard schema; or via Webauthn `userVerification` export |
| Object | Art. 21 | n/a (no marketing) |
| Auto-decision | Art. 22 | ACR step-up has human override (operator-mediated `account-recovery` runbook) |

## 6. Risk assessment

| Risk | Likelihood | Severity | Residual after mitigation |
|---|---|---|---|
| Mass credential database exfiltration | low | catastrophic | low — per-tenant RLS, audit emit, encrypted at rest + HSM signing key |
| Single-tenant credential leak | medium | high | low — tenant-scoped, rotated bearer, revoke-on-suspect |
| Phishing of password fallback | high | medium | low — Passkey-first, password is fallback only; phishing-resistant primary |
| Mass passkey reset (e.g., device class compromise) | low | high | medium — runbook `mass-passkey-reset` with operator gates |
| Step-up bypass via session reuse | low | high | low — `acr_event_at` + Cedar predicate |
| Audit-log loss | low | catastrophic | low — Merkle + Ed25519 + per-pack replica + offsite tape |
| Cross-pack residency violation | low | catastrophic | very-low — Cedar deny + Kyverno admission deny + NetworkPolicy deny |
| HRIS data drift (terminated still active) | medium | medium | low — daily reconciliation + alert |

Residual risks have been accepted by council-compliance (DPIA approval pending; tracked in `evidence/dpia-approval-identity-2026-05-18.json`).

## 7. Cross-border transfer

Per ADR-0179 sovereign-cloud-per-regional-pack, no cross-border transfer occurs:

- pack-eu user data stays in EU.
- pack-kr stays in KR.
- pack-us-healthcare stays in US Sovereign HIPAA-eligible regions.
- pack-ksa stays in KSA.

If a tenant operates in two packs, two independent user records exist (one per pack); no replication crosses the boundary.

## 8. Vendor assessment

| Vendor | Data category processed | Lawful basis | Data Processing Agreement |
|---|---|---|---|
| Zitadel (open source, self-hosted) | none (self-hosted) | n/a | n/a |
| MaxMind GeoIP DB | IP-to-country mapping | legitimate interests | MaxMind DPA + offline DB |
| FIDO Alliance MDS3 | AAGUID metadata (public) | legitimate interests | public dataset |
| Okta / Entra / Workspace (when tenant federates upstream) | identity assertion | controller's choice | tenant-side DPA |
| Workday / BambooHR / Rippling (HRIS adapter consumer choice) | employee records | controller's choice | tenant-side DPA |

## 9. Consultation

- DPO review: 2026-05-15 (pending sign-off)
- Council-compliance: 2026-05-17 (approved with conditions; conditions resolved in this revision)
- Council-architecture: 2026-05-18 (approved)

## 10. DPIA outcome

**APPROVED with conditions**:
1. `lean-a18-identity-vendor-isolation` lane must be promoted to blocker within 60 days.
2. The pack-kr regulated-tier AAGUID allowlist must be reviewed quarterly by ops-security.
3. The HRIS reconciliation drift alert must be wired into Grafana OnCall before HRIS adapters ship to production.
