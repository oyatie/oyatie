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
