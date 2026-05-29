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

---



## §day-one-cert-readiness
This anchor is closed for `identity` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `identity` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +14 more.
- Example: `oidc-token-issue` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `identity` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `oidc-token-issue` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §bootstrap-trust-chain
This anchor is closed for `identity` against ADR-0295 §D-2: Tier-1 bootstrap SPIFFE attestation and kill switch.

### Service-specific answer
- Bootstrap trust applies to `identity` control-plane deployment, CI principals, and first-run OpenBao/SPIFFE bindings.
- Stage-1 trust root is offline-rooted and time-boxed; the kill switch disables bootstrap trust after the declared window even if later stages fail.
- Workload SVIDs protect API/worker surfaces for `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- CI principals can run synthetic tests and publish evidence, but cannot read production tenant data or mint tenant-scoped credentials.
- Example: `oidc-token-issue` app pod starts only after SPIFFE identity, OpenBao policy, and Cedar CI-scope permits are all present.
- Bootstrap failures default to halt: no unauthenticated fallback and no long-lived bootstrap token.
- Evidence: sigstore/cosign attestation, audit-chain bootstrap event, branch-protection gate, and SLO smoke report.
- Tier-1 bootstrap status is listed here even for non-bootstrap services so auditors know whether the service inherits or owns the ceremony.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: SPIFFE/SPIRE workload identity is the reference pattern for the control shape described here.
- Precedent 2: AWS Nitro Enclaves attestation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `identity` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `identity` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`; +19 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `oidc-token-issue` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.identity.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `identity` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `identity` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `oidc-token-issue` touches those data classes.
- Signal sources: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.identity.oidc.token.issue.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `identity` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `identity` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.identity.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `oidc-token-issue` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `oidc-token-issue` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `identity` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`; +8 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `identity.zitadel_instance_controller` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `identity` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`; +12 more.
- Example: `oidc-token-issue` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.identity` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/identity/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.
- Example: `oidc-token-issue` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `identity` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`; +12 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `oidc-token-issue` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `identity` is in annual full-scope pentest and every major `oidc-token-issue` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`; +18 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `identity` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `identity` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `oidc-token-issue` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `identity` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/identity/catalog/oya-check-authz-tier-discipline.yaml`, `microservices/identity/catalog/oya-check-step-up-auth-coverage.yaml`, `microservices/identity/catalog/oya-identity-audit-emitter-kernel.yaml`, `microservices/identity/catalog/oya-identity-oidc-issuer-kernel.yaml`, `microservices/identity/catalog/oya-identity-scim-server-kernel.yaml`, `microservices/identity/catalog/oya-identity-step-up-orchestrator-kernel.yaml`; +23 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `oidc-token-issue` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `identity` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `identity` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `oidc-token-issue` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `oidc-token-issue` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `identity` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.
- State/event surfaces carrying classification: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `oidc-token-issue` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `identity`; owner `axis-identity`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `zitadel-instance-controller`, `oidc-issuer`, `webauthn-relying-party`, `scim-server`, `hris-adapter`, `step-up-orchestrator`; +3 more.
- Capability records cited: `microservices/identity/capabilities/multi-context-principal-resolve.yaml`, `microservices/identity/capabilities/oidc-token-issue.yaml`, `microservices/identity/capabilities/scim-user-provision.yaml`, `microservices/identity/capabilities/step-up-acr-grant.yaml`, `microservices/identity/capabilities/webauthn-authenticate.yaml`.
- API surfaces cited: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar/policy artifacts cited: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`, `INTERNAL_ONLY`.

### Primitive and API binding
- API surface binding: `microservices/identity/contracts/asyncapi/identity-events.yaml`, `microservices/identity/contracts/asyncapi/multi-context-events.yaml`, `microservices/identity/contracts/openapi/identity.yaml`, `microservices/identity/contracts/openapi/multi-context-split.yaml`, `microservices/identity/contracts/proto/identity.proto`, `microservices/identity/contracts/proto/multi_context_split.proto`.
- Cedar binding: `microservices/identity/policy/cedar-acr-predicates.cedar`, `microservices/identity/policy/context-split.cedar`, `microservices/identity/policy/data-residency.md`, `microservices/identity/policy/dual-context-residency.cedar`, `microservices/identity/policy/operator-recovery.cedar`, `microservices/identity/policy/tenant-scope.cedar`.
- State/event binding: `identity.zitadel_instance_controller`, `identity.oidc_issuer`, `identity.webauthn_relying_party`, `identity.scim_server`, `identity.hris_adapter`, `identity.step_up_orchestrator`; +2 more.
- Capability binding: `oidc-token-issue`, `webauthn-authenticate`, `step-up-acr-grant`, `multi-context-principal-resolve`, `scim-user-provision`.
- SLO binding: `microservices/identity/slos/aaguid-refresh-freshness.openslo.yaml`, `microservices/identity/slos/audit-emit-completeness.openslo.yaml`, `microservices/identity/slos/jwks-availability.openslo.yaml`, `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/identity/runbooks/brute-force-mitigation.md`, `microservices/identity/runbooks/idp-failover-drill.md`, `microservices/identity/runbooks/ip-block-incident.md`, `microservices/identity/runbooks/jwks-rotation.md`, `microservices/identity/runbooks/passkey-cross-device-debug.md`, `microservices/identity/runbooks/passkey-reset.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `identity`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `identity`.
- `policy-engine` supplies the signed Cedar corpus while `identity` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `identity` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `identity`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `identity` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

