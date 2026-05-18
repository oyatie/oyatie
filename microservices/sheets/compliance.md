---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-sheets, council-design-system, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0065, ADR-0117, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/sheets/threat-model.md
  - microservices/sheets/dpia.md
  - microservices/sheets/policy/data-residency.md
  - microservices/sheets/policy/editor-isolation.md
  - microservices/sheets/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (sheets µservice)

## Purpose

Canonical control-to-framework mapping for sheets. Tells external auditors (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / EU AI Act notified body / WCAG 2.2 AA accessibility auditor / etc.) which control implementation satisfies which framework clause, with pointers to evidence. Continuous-compliance-evidence emission keeps this matrix machine-verifiable.

## Enforced Frameworks

### SOC 2 Type 2 (2017 Trust Services Criteria + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | Integrity and ethical values | Code-of-conduct + signed-commit policy | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/sheets/CODEOWNERS` |
| CC1.5 | Accountability | Per-µservice SLO + on-call rotation | `PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `incident-response.md` |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule | `policy/editor-isolation.md` |
| CC3.4 | Significant change risk | PR review + LEAN lanes | `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission per cell-edit | ADR-0028 + audit-chain µservice |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Citus + RLS + Strict CSP + SRI + gVisor + ClamAV/OPSWAT | `policy/*.cedar` + `policy/editor-isolation.md` |
| CC5.3 | Policy and procedure deployment | Per-µservice runbooks + standards | `docs/standards/*.md` + `runbooks/` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant SDK API keys + SPIFFE | `threat-model.md` §"Trust boundaries" |
| CC6.3 | Adds / removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Citus + RLS + WS gateway tenant binding + per-range Cedar ACL | `threat-model.md` T-I-01 + T-T-07 |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade + XLSX export ACL-aware masking | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE scan; ClamAV + OPSWAT on XLSX uploads | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA WS gateway + per-tenant rate limits + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cross-tenant collab SLI + range-ACL drift detection | `dashboards/*.json` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | this changeset |
| CC9.1 | Risk mitigation | Multi-region + DR pair + auto-rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list (including LLM providers; ClamAV / OPSWAT vendors) + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1-P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice; AI-formula consent UI |
| P2 | Choice + consent | Tenant onboarding consent + per-session AI-formula opt-in |
| P3 | Collection | SDK PII redactor + `data_class` annotation + cell-grid data_class markers |
| P4 | Use, retention, disposal | Retention matrix in `data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list (LLM providers; AV vendors); transfer register |
| P7 | Quality | Cell-edit signature verification + recalc determinism + formula-engine correctness |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + AI-formula pack-resident routing + ACL-aware XLSX export | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar (license-gate + range-ACL) + Citus + RLS | `threat-model.md` |
| A.5.17 | Authentication information | OpenBao rotation (30d / 90d) | OpenBao audit log |
| A.5.18 | Access rights | RBAC managed via Terraform | `iac/terraform/sheets-rbac.tf` |
| A.5.23 | Cloud services | OCI HIPAA-eligible for pack-us-healthcare | `policy/data-residency.md` |
| A.5.26 | Response to incidents | Severity-driven runbook | `incident-response.md` + `runbooks/*` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for BC | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory | This document + per-pack overlays | `compliance.md` (this file) |
| A.5.32 | Intellectual property rights | License-policy CI lane | `oya-check-license-policy` |
| A.5.33 | Protection of records | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar + range-ACL | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Citus + RLS + Cedar + WS gateway tenant binding + per-range ACL | `threat-model.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | `branch-protection.yaml` |
| A.8.5 | Secure authentication | mTLS + OIDC + signed SDK API keys | `threat-model.md` |
| A.8.7 | Protection against malware | gVisor sandbox + ClamAV + OPSWAT for XLSX upload; WASM SRI | `threat-model.md` T-S-04, T-T-06 |
| A.8.11 | Data masking | SDK redactor for AI-formula + ACL-aware XLSX export | `threat-model.md` T-I-05, T-I-09 |
| A.8.12 | Data leakage prevention | Strict CSP + Trusted Types + range-ACL Cedar | `threat-model.md` T-I-02 |
| A.8.15 | Logging | Audit-chain emission | ADR-0028 |
| A.8.16 | Monitoring activities | Sheets self-SLI + Grafana | `dashboards/*.json` |
| A.8.20 | Network security | Network policies; Sheets → cell/ontology/foundry-runtime/tenancy/audit-chain/workflow-engine/docs/slides/drive/forms/mail/community SDKs only | Kubernetes NetworkPolicy review |
| A.8.21 | Network services security | mTLS internal; TLS + WAF public | ingress configuration |
| A.8.23 | Web filtering | WAF + CSP | `iac/helm/*` |
| A.8.25 | Secure development life cycle | LEAN gates + multispectrum review | per-IP evidence |
| A.8.26 | Application security requirements | XSS prevention + signature verification + recalc determinism + formula-engine correctness + range-ACL | `policy/editor-isolation.md` |
| A.8.27 | Secure system architecture | Hexagonal layering per ADR-0103 | `PRD.md` §"Bounded Contexts" |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings`; sanitizers in CI; gVisor + AV for XLSX | per-IP acceptance lane |

### GDPR (relevant articles)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5 | Principles of processing | Lawful + transparent + minimised + purpose-limited | `dpia.md` §2.4 |
| Art. 6 | Lawful basis | Per purpose in DPIA §2.4 | `dpia.md` |
| Art. 9 | Special categories | pack-us-healthcare BAA; pack-kr explicit consent; AI-formula consent | `legal/baa-template.md`, `legal/dpa-template.md` |
| Art. 13 + 14 | Information to data subject | Tenant DPA upstream-disclosure clause; AI-formula transparency UI | `legal/dpa-template.md` |
| Art. 17 | Right to erasure | DSR cascade | `policy/data-residency.md` |
| Art. 22 | Automated decision-making | Sheets is authoring tool, not solely-automated-with-legal-effect; AI-formula is advisory at T1; T2 gated per ADR-SHEETS-0005 | `dpia.md` R-13 |
| Art. 25 | Privacy by design | This µservice's design satisfies; EDPB Guidelines 4/2019 alignment | `dpia.md` §4 |
| Art. 28 | Processor agreement | DPA template | `legal/dpa-template.md` |
| Art. 30 | Records of processing | ROPA | `legal/ropa.md` |
| Art. 32 | Security of processing | Every threat mitigation contributes | `threat-model.md` |
| Art. 33 | Breach notification (DPA) | 72h notification chain | `incident-response.md` |
| Art. 35 | DPIA | This document + `dpia.md` | `dpia.md` |
| Arts. 44-50 | Transfers | SCC-only; default pack-pinning; AI-formula pack-resident | `policy/data-residency.md` |

### OWASP ASVS L2 (Application Security Verification)

| ASVS Section | Requirement | Implementation |
|---|---|---|
| V1 (Architecture) | Threat modeling | `threat-model.md` |
| V2 (Authentication) | OIDC + MFA + session token rotation | `threat-model.md` §"Trust boundaries" |
| V3 (Session) | HttpOnly + Secure + SameSite; rotate on auth-state change | Sheets cookie configuration |
| V4 (Access Control) | Default-deny Cedar + per-tenant scope + per-range ACL | `policy/tenant-scope.cedar` |
| V5 (Validation) | Server-side formula-engine grammar validation; XLSX parse validation | `threat-model.md` T-I-03 |
| V7 (Errors + Logging) | No sensitive data in error responses; audit-chain emission | `threat-model.md` §"R-family" |
| V8 (Data Protection) | Data classification + retention bounds | `dpia.md` + `policy/data-residency.md` |
| V9 (Communications) | mTLS internal; TLS 1.3 public | ingress configuration |
| V11 (Business Logic) | Per-seat license-gate + per-range ACL + recalc determinism + formula correctness | `threat-model.md` T-T-05, AC-14, AC-21, AC-22 |
| V12 (Files + Resources) | SRI hashes on WASM chunks; gVisor sandbox + ClamAV/OPSWAT on XLSX | `threat-model.md` T-T-06, T-S-04 |
| V13 (API) | OpenAPI 3.2.0; gRPC proto3; AsyncAPI 3.1.0 | `contracts/*` |
| V14 (Configuration) | Strict CSP + Trusted Types + WAF | `policy/editor-isolation.md` |

### EU AI Act 2024 (conditional; when AI-formula used in regulated domain)

| AI Act Article | Requirement | Implementation |
|---|---|---|
| Art. 9 | Risk management system | AI-formula conformity assessment per `legal/ai-act-conformity.md`; per ADR-SHEETS-0005 |
| Art. 10 | Data and data governance | AI-formula prompts + completions retained 90d; data quality via formula-engine grammar validation |
| Art. 12 | Record-keeping | Audit-chain seal per AI-formula invocation; retention ≥ 6mo when in high-risk context |
| Art. 13 | Transparency + provision of information | AI-formula transparency UI in editor; tenant onboarding disclosure |
| Art. 14 | Human oversight | User explicit-accept of AI-formula draft before save (never auto-submit at T1; T2 ChangeSet review per ADR-SHEETS-0005) |
| Art. 15 | Accuracy, robustness, cybersecurity | Schema validation + signature verification + tenant approval gate + smart-fill accuracy SLO |

### WCAG 2.2 AA (Accessibility)

| WCAG Criterion | Implementation |
|---|---|
| 1.1.1 Non-text content | Cell-grid accessible labels per cell role; chart alt-text per chart |
| 1.3.1 Info and Relationships | ARIA grid semantics in Leptos canvas; sheet/row/col headers properly labelled |
| 1.4.3 Contrast (Minimum) | Design-system enforces 4.5:1 contrast for cell text |
| 2.1.1 Keyboard | Full keyboard navigation: arrow-key cell movement; Ctrl+arrow range select; F2 edit |
| 2.4.6 Headings and Labels | Sheet tabs labelled; named-ranges labelled |
| 3.3.1 Error identification | Formula-engine errors surfaced inline with per-cell precision |
| 4.1.2 Name, Role, Value | ARIA grid pattern adhered |
| 4.1.3 Status Messages | Recalc-progress status announced to screen-reader via aria-live |

## Per-Pack Overlay Sections

### pack-kr (KR-ISMS-P + KR PIPA + KR 전자문서법)

KR PIPA Art. 29 (technical safeguards) — cross-mapped to Sheets mitigations:

| PIPA safeguard | Sheets mitigation |
|---|---|
| Access control | OIDC + Cedar (license + range-ACL) + Citus + RLS + WS gateway tenant binding |
| Encryption (transit) | mTLS internal; TLS public ingress |
| Encryption (at rest) | KMS-SSE for Postgres + Redis AOF + S3 + Arrow/Parquet OCI Object Storage |
| Integrity verification | Cell-edit signature + recalc determinism + audit-chain Merkle |
| Audit log retention ≥ 1y | 3y aligned (KR-FSS sector) |
| IDS / IPS | WAF + network policies |
| Vulnerability management | `cargo deny` + Trivy + Grype + ClamAV + OPSWAT |
| Mobile / remote access | mTLS + OIDC + MFA |
| User account management | OpenBao lifecycle + per-seat Cedar |
| Logging | OTel + audit-chain |
| Patch management | Helm + ArgoCD declarative |
| Incident response | Severity-classified runbooks |

KR 전자문서법 (Electronic Document Act):
- Art. 5 (integrity preservation): Ed25519 audit-chain seal satisfies.
- Art. 6 (long-term preservation): Audit-chain immutability + version-history retention satisfy.
- Art. 7 (admissibility): Audit-chain Merkle proof is admissible evidence.

### pack-us-healthcare (HIPAA)

| HIPAA section | Requirement | Sheets implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | This document + `dpia.md` + `threat-model.md` |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar policy + 2-person rule + per-range ACL |
| §164.310(a)(1) | Facility access controls | Inherited from cloud-k8s + OCI HIPAA-eligible |
| §164.312(a)(1) | Access control (unique user ID) | OIDC + per-user identity |
| §164.312(a)(2)(i) | Emergency access | JIT elevation procedure |
| §164.312(a)(2)(ii) | Automatic logoff | Session timeout |
| §164.312(b) | Audit controls | Audit-chain emission per cell-edit + license-gate |
| §164.312(c)(1) | Integrity | Recalc determinism + audit-chain Merkle |
| §164.312(d) | Person or entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | mTLS + TLS |
| §164.316(b)(2) | Documentation retention (6y) | Repo retention + audit-chain |

AI-formula provider for pack-us-healthcare must be HIPAA BAA-eligible.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + DORA + AI Act)

- GDPR Art. 32 cross-mapped (above).
- EDPB Guidelines 4/2019 (Art. 25): every mitigation maps to TOM.
- NIS2 (2022/2555): 24h + 72h + 1mo reporting timelines when thresholds crossed.
- eIDAS 910/2014 Art. 26: Ed25519 audit-chain seals satisfy AdES; signed XLSX exports satisfy.
- DORA (2022/2554): pack-eu financial-services tenants get DORA-aligned BCDR.
- EU AI Act 2024: AI-formula conformity per §"EU AI Act 2024" above + ADR-SHEETS-0005.

### pack-jp (APPI)

| APPI Article | Requirement | Sheets implementation |
|---|---|---|
| Art. 17 | Purpose of use declaration | DPA + tenant onboarding |
| Art. 18 | Consent for sensitive data | Tenant DPA |
| Art. 20 | Security control measures | Threat-model mitigations |
| Art. 21 | Cross-border transfer | pack-jp residency; AI-formula routes JP-resident |
| Art. 23 | Joint use disclosure | Tenant DPA upstream clause |
| Art. 24 | Provision to third party | Sub-processor list |
| Art. 26-2 | Breach notification | `incident-response.md` |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/sheets-overlay.md` with full citations.

## Continuous-Compliance Evidence

The `oya-governance-compliance-evidence-recency` lane validates that each control's evidence artifact:
- Exists at its declared path.
- Was last modified within freshness window (annual default; quarterly for sensitive controls).
- Has a valid Ed25519 signature where required.

Annual external audit reads this file + evidence artifacts cited above.

## Re-review Triggers

- Annually (Q2 each year).
- On any pack activation.
- On any enforced-framework version update.
- On any sub-processor change (including LLM provider + AV vendor).
- Post-incident (Sev-1 or Sev-2).
- On any change to Sheets's processing scope.
- EU AI Act enforcement milestone.

## References

- ADR-0028, 0065, 0117, 0123, 0126, 0130, 0131, 0140.
- ADR-SHEETS-0001..0007 (local).
- `microservices/sheets/threat-model.md`.
- `microservices/sheets/dpia.md`.
- `microservices/sheets/policy/data-residency.md`.
- `microservices/sheets/policy/editor-isolation.md`.
- `microservices/sheets/incident-response.md`.
- SOC 2 Type 2 (2017 TSC + 2022 PoF).
- ISO 27001:2022 Annex A.
- GDPR EUR-Lex 2016/679.
- KR PIPA + 전자문서법 + KR-ISMS-P.
- HIPAA 45 CFR Parts 160 + 164.
- APPI 2003 (改正 2022).
- LGPD 2018.
- DPDPA 2023.
- DORA 2022/2554.
- NIS2 2022/2555.
- eIDAS 910/2014.
- EU AI Act 2024.
- OWASP ASVS v4.0.
- WCAG 2.2 AA — `w3.org/TR/WCAG22/`.
