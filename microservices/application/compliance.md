---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-application, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md
  - microservices/application/dpia.md
  - microservices/application/policy/route-isolation.md
  - microservices/application/policy/data-residency.md
  - microservices/application/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (application µservice)

## Purpose

The canonical control-to-framework matrix for Application Shell. Tells
external auditors (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC /
HIPAA OCR / etc.) which control implementation satisfies which framework
clause, with pointers to evidence. Continuous-compliance-evidence emission
keeps this matrix machine-verifiable via the
`oya-governance-compliance-evidence-recency` lane.

## Enforced Frameworks

### SOC 2 Type 2 (2017 Trust Services Criteria)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | COSO Integrity + ethical values | Code-of-conduct + signed-commit; CODEOWNERS quarterly review | `docs/standards/code-review.md` |
| CC1.2 | Board oversight | Council-architecture quarterly review of application µservice | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI per BC | `microservices/application/CODEOWNERS` |
| CC2.1-2.3 | Communication | Status page + tenant comms templates | `incident-response.md` §"Communication" |
| CC3.1-3.4 | Risk management | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC4.1-4.2 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC5.1-5.3 | Control activities | LEAN lanes (route-isolation, cedar-policy, audit-chain) + runbooks | `microservices/governance/` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar policy + JIT via OpenBao | `policy/*.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant API keys + SPIFFE | `threat-model.md` §"Trust Boundaries" |
| CC6.3 | Adds/removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Cedar default-deny + RLS on Postgres + Mimir multi-tenancy | `policy/tenant-scope.cedar` |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | cargo deny + Trivy + Grype CI lanes; weekly CVE scan; OWASP ZAP scan in CI | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA + per-tenant rate limits + auto-scale | `capacity-model.md` |
| CC7.2 | Monitoring | Self-observability metrics + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | observability OpenSLO |
| CC7.4 | Incident response | Severity-classified + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | `.github/branch-protection.yaml` |

### ISO 27001:2022

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Quarterly threat-model review | `threat-model.md` |
| A.5.10 | Acceptable use | Per-tenant DPA | DPA template |
| A.5.14 | Information transfer | mTLS + signed module manifest | `policy/data-residency.md` |
| A.5.15 | Access control | Cedar default-deny | `policy/route-isolation.md` |
| A.5.17 | Authn information | OpenBao secrets + KMS keys | OpenBao audit |
| A.5.23 | Cloud services info security | OCI security posture | `multi-region.md` |
| A.5.31 | Legal + regulatory | Per-pack DPIA + DPA | `dpia.md` |
| A.5.32 | IP rights | Bundle license check | `cargo deny` |
| A.5.33 | Protection of records | Audit-chain Ed25519 | `policy/data-residency.md` |
| A.8.2 | Privileged access | JIT elevation via OpenBao | `policy/auditor-scope.cedar` |
| A.8.3 | Information access restriction | RLS + Cedar | `policy/tenant-scope.cedar` |
| A.8.5 | Secure authn | OIDC + SAML + MFA | `threat-model.md` S-01..S-03 |
| A.8.7 | Protection against malware | SRI + signed manifest + iframe sandbox | `module-loader` |
| A.8.11 | Data masking | DSR cascade redaction | `policy/data-residency.md` |
| A.8.12 | Data leakage prevention | CSP + Cedar + RLS | `threat-model.md` I-01..I-05 |
| A.8.15 | Logging | Audit-chain + observability | `failure-modes.md` |
| A.8.16 | Monitoring activities | OpenSLO + OnCall | `incident-response.md` |
| A.8.20 | Network security | Istio mTLS + WAF | `threat-model.md` §"Trust boundaries" |
| A.8.21 | Network services | Per-pack ingress | `multi-region.md` |
| A.8.23 | Web filtering | CSP + bot management | `threat-model.md` T-03 |
| A.8.25 | Secure development lifecycle | LEAN lanes + threat model + DPIA pre-deploy | this matrix |
| A.8.26 | Application security requirements | OWASP ASVS Level 2 | `threat-model.md` |
| A.8.27 | Secure architecture principles | ADR-0105 + ADR-0131 | architecture |
| A.8.28 | Secure coding | clippy -D warnings + cargo deny + sqlx compile-check | CI |

### GDPR Arts. 5, 6, 13, 14, 25, 28, 30, 32, 33, 35

| Article | Control | Evidence |
|---|---|---|
| Art. 5 (principles) | Data minimisation in audit log; retention policy | `dpia.md` §2.3 |
| Art. 6 (lawfulness) | Per-processing lawful basis | `dpia.md` §4 |
| Art. 13/14 (information) | Tenant onboarding privacy notice + cookie banner | `dpia.md` |
| Art. 25 (data protection by design) | Default-deny Cedar + RLS + signed manifest | `policy/*.cedar` |
| Art. 28 (processor) | Sub-processor list in DPIA | `dpia.md` §2.2 |
| Art. 30 (records) | Per-pack processing record | `dpia.md` |
| Art. 32 (security) | This entire matrix | per row |
| Art. 33 (breach notification) | Sev-1 → ≤72 h notification | `incident-response.md` |
| Art. 35 (DPIA) | This µservice carries one | `dpia.md` |

### KR PIPA (pack-kr)

| Article | Control | Evidence |
|---|---|---|
| Art. 15 (collection consent) | Per-tenant consent at onboarding | DPA |
| Art. 17 (provision to 3rd party) | sub-processor consent | DPA |
| Art. 18 (purpose limitation) | Per-route Cedar permit | `policy/route-isolation.md` |
| Art. 23 (sensitive info) | Pack-pinned + Cedar required-attestation | `policy/tenant-scope.cedar` |
| Art. 24 (uniquely identifying info) | RLS + Cedar | `policy/tenant-scope.cedar` |
| Art. 28 (cross-border transfer) | Pack-pinning forbid-by-default | `policy/data-residency.md` |
| Art. 29 (security measures) | This matrix | per row |
| Art. 33 (impact assessment) | DPIA | `dpia.md` |

### HIPAA (pack-us-healthcare; conditional)

| 45 CFR | Control | Evidence |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | `threat-model.md` |
| §164.308(a)(3) | Workforce security | CODEOWNERS + onboarding |
| §164.310 | Physical safeguards | OCI HIPAA-eligible regions | `multi-region.md` |
| §164.312(a)(1) | Access control | Cedar + RLS | `policy/tenant-scope.cedar` |
| §164.312(b) | Audit controls | Audit-chain Ed25519 | `policy/route-isolation.md` |
| §164.312(c) | Integrity | SRI + signed manifest | `module-loader` |
| §164.312(d) | Person/entity authentication | OIDC + MFA + WebAuthn step-up | `policy/route-isolation.md` |
| §164.312(e) | Transmission security | mTLS + TLS 1.3 + HSTS | `threat-model.md` |
| §164.502(b) | Minimum necessary | Per-role Cedar + module-load gating | `policy/tenant-scope.cedar` |
| §164.514 | De-identification (when applicable) | Module-loader manifest split | `dpia.md` |

### OWASP ASVS v4 — Level 2 (Level 3 for module-loader)

| ASVS | Control | Evidence |
|---|---|---|
| 1.x Architecture | Per-BC trust-boundary model | `threat-model.md` |
| 2.x Authentication | OIDC/SAML strict verify + MFA | `threat-model.md` S-01..S-04 |
| 3.x Session | 256-bit token + HttpOnly + Secure + SameSite | `threat-model.md` T-06 |
| 4.x Access control | Cedar default-deny + RLS | `policy/tenant-scope.cedar` |
| 5.x Input validation | Leptos auto-encoding + sqlx compile check | `threat-model.md` T-03, T-05 |
| 6.x Cryptography | Ed25519 manifest + HMAC session + KMS | `module-loader` |
| 7.x Error handling + logging | Sealed error id; no stack in prod; audit-chain | `failure-modes.md` |
| 8.x Data protection | Pack-pinning + DSR cascade | `policy/data-residency.md` |
| 9.x Communications | TLS 1.3 + mTLS + HSTS | `threat-model.md` |
| 10.x Malicious code | SRI + signed manifest (Level 3) | `module-loader` |
| 11.x Business logic | Per-step Cedar + multi-step session correlation | `policy/tenant-scope.cedar` |
| 12.x File + resources | CDN content-type pin + no-exec on bundle origin | `module-loader` |
| 13.x API + web service | OpenAPI 3.2 + Cedar at every route | `contracts/openapi/application.yaml` |
| 14.x Configuration | Helm chart pinned LTS; deploy lane refuses drift | `iac/helm/` |

## Continuous Compliance Evidence

| Lane | What it asserts | Cadence |
|---|---|---|
| `oya-application-cedar-policy-compiles` | All `policy/*.cedar` valid | PR + daily |
| `oya-application-audit-chain` | Audit seal latency ≤ 1 s p99 | runtime SLO |
| `oya-application-residency-pin` | No cross-pack data leak | nightly scan |
| `oya-application-route-isolation` | Every route has tenant_scope + roles + pack | PR |
| `oya-application-cookie-scope-lint` | Cookie domain == `.app.oyatie.dev` only | PR + runtime probe |
| `oya-application-tls-config-lint` | TLS 1.3 + HSTS + cipher allow-list | PR + runtime probe |
| `oya-application-sri-hash-present` | All bundle URLs in shell HTML have SRI | PR |
| `oya-application-csp-strict` | CSP `script-src 'self' 'wasm-unsafe-eval'` | PR + runtime probe |

## References

- ADR-0028 audit chain.
- ADR-0117 packs.
- ADR-0123 cross-product auth.
- `microservices/observability/compliance.md` (precedent).

---



## §day-one-cert-readiness
This anchor is closed for `application` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `application` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +8 more.
- Example: `module-load` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `module-load` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `application` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`, `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `module-load` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.application.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `application` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `module-load` touches those data classes.
- Signal sources: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +9 more.
- Example event class: `oya.application.module.load.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `application` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.application.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `module-load` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `module-load` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `application` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`, `application.auth_gateway`, `application.frontend_bundle_serve`; +4 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `application.auth_gateway` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `application` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +11 more.
- Example: `module-load` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.application` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/application/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.
- Example: `module-load` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `application` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`, `microservices/application/iac/helm/cdn-controller/Chart.yaml`, `microservices/application/iac/helm/cdn-controller/values.yaml`; +8 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `module-load` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `application` is in annual full-scope pentest and every major `module-load` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`, `microservices/application/iac/helm/cdn-controller/Chart.yaml`, `microservices/application/iac/helm/cdn-controller/values.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `application` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `application` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `module-load` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `application` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/application/catalog/oya-application-auth-gateway-adapter-oidc.yaml`, `microservices/application/catalog/oya-application-auth-gateway-adapter-saml.yaml`, `microservices/application/catalog/oya-application-auth-gateway-adapter.yaml`, `microservices/application/catalog/oya-application-auth-gateway-api.yaml`, `microservices/application/catalog/oya-application-auth-gateway-app.yaml`, `microservices/application/catalog/oya-application-auth-gateway-domain.yaml`; +20 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `module-load` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `application` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `module-load` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `module-load` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `application` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `module-load` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `application`; owner `axis-application`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `auth-gateway`, `frontend-bundle-serve`, `module-loader`, `shell-routing`, `tenant-context`, `tenant-admin-console`.
- Capability records cited: `microservices/application/capabilities/module-load.yaml`, `microservices/application/capabilities/session-emit.yaml`, `microservices/application/capabilities/shell-render.yaml`, `microservices/application/capabilities/tenant-admin-console-control.yaml`.
- API surfaces cited: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar/policy artifacts cited: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`, `microservices/application/dashboards/module-load-success.json`; +2 more.
- Runbook/IaC evidence: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/application/contracts/asyncapi/application-events.yaml`, `microservices/application/contracts/openapi/application.yaml`, `microservices/application/contracts/openapi/tenant-admin-console.yaml`, `microservices/application/contracts/proto/application.proto`.
- Cedar binding: `microservices/application/policy/auditor-scope.cedar`, `microservices/application/policy/ci-scope.cedar`, `microservices/application/policy/data-residency.md`, `microservices/application/policy/public-read.cedar`, `microservices/application/policy/route-isolation.md`, `microservices/application/policy/tenant-admin-console.cedar`; +1 more.
- State/event binding: `application.auth_gateway`, `application.frontend_bundle_serve`, `application.module_loader`, `application.shell_routing`, `application.tenant_context`, `application.tenant_admin_console`.
- Capability binding: `module-load`, `session-emit`, `shell-render`, `tenant-admin-console-control`.
- SLO binding: `microservices/application/slos/audit-seal.openslo.yaml`, `microservices/application/slos/module-load.openslo.yaml`, `microservices/application/slos/oidc-signin.openslo.yaml`, `microservices/application/slos/route-resolve.openslo.yaml`, `microservices/application/slos/tti.openslo.yaml`.
- Runbook binding: `microservices/application/runbooks/auth-gateway-restart.md`, `microservices/application/runbooks/cdn-purge.md`, `microservices/application/runbooks/module-rollback.md`, `microservices/application/runbooks/session-storm.md`, `microservices/application/runbooks/tenant-context-recovery.md`, `microservices/application/runbooks/wasm-bundle-rebuild.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `application`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `application`.
- `policy-engine` supplies the signed Cedar corpus while `application` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `application` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `application`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `application` applies the most restrictive policy and emits a degraded-mode audit event.
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

