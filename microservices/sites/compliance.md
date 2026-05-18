---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0126, ADR-0140, ADR-SITES-0006]
doc_status: published
---

# Compliance Matrix — sites µservice

## Purpose

Enumerate compliance frameworks engaged by sites, the controls
satisfied, and where each control is evidenced (per artifact, lane, or
runbook).

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |
| WCAG 2.2 AA + ATAG 2.0 | accessibility | §"Accessibility Mapping" below |
| OWASP ASVS v4 | application security verification | §"OWASP Mapping" below |
| W3C Subresource Integrity | published asset integrity | LEAN `oya-check-sri-coverage` |
| NIST SSDF (SP 800-218) | secure software development | per Foundry pipeline |
| SLSA L3 | supply-chain integrity | published artifacts signed |
| CIS Kubernetes Benchmark | container hardening | IaC inherits from cloud-k8s |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + ADA Title III + Section 508 (patient portals) + CCPA / CMIA / NY SHIELD |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + EU AI Act Art. 50 + Annex III §3 + EU DSA Arts. 14/27 + NIS2 + eIDAS 910/2014 + ePrivacy 2002/58/EC Art. 5(3) |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 (financial-tenants) |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 (financial-tenants) |
| pack-in | DPDPA 2023 §6-16 + RBI Master Direction on IT Outsourcing (financial-tenants) |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 (financial-tenants) |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + SAMA Cybersecurity Framework 2017 (financial-tenants) |

## SOC 2 Mapping

| TSC | Control | Sites evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | This compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys |
| CC6.3 | Authorises | Cedar policies (`tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`) |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | type-narrowed projections + LEAN checks |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts + SLSA L3 + W3C SRI |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention; runbooks/publish-pipeline-rollback.md |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0130 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001:2022 Mapping

| Annex A Control | Sites evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use of info) | policy/editor-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar policies + RLS |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | This compliance.md per-pack overlays |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access) | OpenBao JIT + 2-person rule |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.11 (data masking) | redaction in published projection + WCAG-AA correctness lane |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan on publish |
| A.8.15 (logging) | observability + audit-chain |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.23 (web filtering) | WAF at ingress; rate-limit on anonymous reads |
| A.8.25 (secure SDLC) | LEAN gates + ADR-0130 |
| A.8.26 (application security) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 + ADR-0131 |
| A.8.28 (secure coding) | LEAN check + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.34 (audit + protection) | audit-chain immutability + 2-person rule |

## GDPR Mapping

| Article | Sites evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md §2.2 |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | type-narrowed published projection + analytics hash-bucket |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain + W3C SRI |
| Art. 6(1) lawful basis | dpia.md §2.2 |
| Art. 13/14 transparency | tenant DPA template + consent banner (where required by ePrivacy) |
| Art. 17 right-to-erasure | DSR cascade orchestrator in page-usecase + legal-hold reconciliation |
| Art. 20 portability | site-export endpoint per FR-portability |
| Art. 22 automated decision | T2 AI-page-build does not make legal-effect decisions; HR/legal/medical overlay REFUSED per ADR-SITES-0006 |
| Art. 25 by design + default | type-system separation + Cedar policy |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Accessibility Mapping (WCAG 2.2 AA + ATAG 2.0)

| Criterion | Sites evidence |
|---|---|
| WCAG 2.2 1.1.1 Non-text Content | alt-text required at publish; LEAN refuse |
| WCAG 2.2 1.3.1 Info and Relationships | heading-order LEAN check |
| WCAG 2.2 1.4.3 Contrast (Minimum) | 4.5:1 contrast check via LightningCSS + theme validator |
| WCAG 2.2 1.4.10 Reflow | responsive layout default themes |
| WCAG 2.2 2.1.1 Keyboard | all interactive blocks keyboard-navigable |
| WCAG 2.2 2.4.4 Link Purpose | link text required + LEAN |
| WCAG 2.2 2.5.8 Target Size (Minimum) | per ADR-SITES-0007 image-block min 44×44 tap target |
| WCAG 2.2 3.1.1 Language of Page | `lang` attribute required at publish |
| WCAG 2.2 3.3.7 Redundant Entry | form-block via forms µservice |
| WCAG 2.2 4.1.2 Name, Role, Value | ARIA on dynamic blocks |
| ATAG 2.0 (editor accessibility) | block editor accessible; alt-text prompt on image insert |

## OWASP ASVS v4 Mapping

| Section | Coverage |
|---|---|
| V1 Architecture | ADR-0056 + ADR-0105 + per-µservice flat layout |
| V2 Authentication | OIDC + MFA + per-tenant API key |
| V3 Session management | per-tenant session token; salt-rotation |
| V4 Access control | Cedar + RLS |
| V5 Validation, sanitization, encoding | URL percent-encoding per RFC 3986; HTML output encoding; SVG sanitisation; LightningCSS scoped CSS |
| V7 Error handling + logging | structured logging + audit-chain |
| V8 Data protection | tenant-DEK + TLS 1.3 |
| V9 Communication | mesh mTLS + WAF |
| V10 Malicious code | image-pipeline SVG strip; published JS bears SRI |
| V12 Files + resources | libvips bound on file size + resolution |
| V13 API + Web service | OpenAPI 3.1 contract |
| V14 Configuration | values.yaml + Helm pin |

## Per-pack overlays

### pack-kr (KR PIPA + KR-FSS + 전자문서법 + ISMS-P)

| Control | Citation | Sites implementation |
|---|---|---|
| Audit-chain integrity | 전자문서법 Art. 5 | Ed25519 + Merkle per Bominal ADR-0028 |
| Special-category data | KR PIPA Art. 23 | data-class `SENSITIVE_PIPA_ART23` on CMS-collection fields; Cedar refusal of anonymous rendering |
| Retention floor | KR-FSS guidelines | 1825d (5y) for financial-sector tenants |
| Notification | KR PIPA Art. 34 | 72h notification per incident-response.md |
| Cross-border | KR PIPA Art. 17 | per-pack residency; cross-pack SCC-gated |
| ISMS-P | KISA Notice 2024-X | annual recertification |

### pack-eu (GDPR + ePrivacy + EU AI Act + EU DSA + eIDAS + NIS2)

| Control | Citation | Sites implementation |
|---|---|---|
| Lawful basis | GDPR Art. 6 | per-purpose admission via Cedar; `legal/ropa.md` |
| Right to erasure | GDPR Art. 17 | page-usecase erasure orchestrator + legal-hold reconciliation |
| Right to portability | GDPR Art. 20 | site-export endpoint |
| DPIA | GDPR Art. 35 | this DPIA |
| Cross-border | Chapter V | per-pack EU residency; SCC for cross-pack |
| AI Act transparency | Art. 50 | T2 AI-page-build labelled "AI is suggesting this page — review before publish"; Art. 14 cancel window 30s |
| AI Act high-risk | Annex III §3 | T2 HR/legal/medical-context overlays REFUSED at Cedar layer pending ADR-SITES-XXXX |
| ePrivacy | Art. 5(3) | analytics first-party only; consent banner required for non-strictly-necessary cookies |
| DSA Art. 14 transparency | EU DSA | publish-refusal records carry policy citation |
| eIDAS Art. 26 AdES | EU 910/2014 | audit-chain Ed25519 satisfies |

### pack-us-healthcare (HIPAA + BAA + ADA Title III + Section 508)

| Control | Citation | Sites implementation |
|---|---|---|
| Security Rule | 45 CFR §164.308 | Risk Analysis + audit controls + encryption |
| Privacy Rule | 45 CFR §164.502(b) | minimum-necessary on CMS-collection fields; PHI data-class |
| Encryption | 45 CFR §164.312(a)(2)(iv) | Tenant-DEK envelope at rest; TLS 1.3 in transit |
| Audit controls | 45 CFR §164.312(b) | Ed25519 + Merkle audit-chain; retention ≥ 6y |
| BAA | 45 CFR §164.504(e) | per-tenant BAA per `legal/baa-template.md`; LEAN refuse pack-us-healthcare without `baa_on_file=true` |
| ADA Title III + Section 508 | (federal) | patient-portal sites refuse publish at < 100% WCAG 2.2 AA |

### pack-us (CCPA / CPRA / sectoral)

| Control | Citation | Sites implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export |
| Right to delete | CCPA §1798.105 | erasure orchestrator |
| Sale of PD opt-out | CCPA §1798.120 | no sale; `legal/sub-processors.md` |

### pack-jp (APPI)

| Control | Citation | Sites implementation |
|---|---|---|
| Purpose | APPI Art. 17 | tenant onboarding consent |
| Leak notification | APPI Art. 22 | 3-business-day per incident-response.md |
| Cross-border | APPI Art. 24 | pack-jp jp-tokyo-1; cross-pack consent-gated |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays summarised in `policy/data-residency.md`.

Highlights:
- **PDPA 2012 (sg)**: Part III Protection + Part IV Retention + Part VI Transfer.
- **APP 8 + APP 11 + APP 12** (Privacy Act 1988 au).
- **DPDPA 2023 (in)**: §6-16 consent / notice / security.
- **LGPD Arts. 33-36 (br)**: cross-border.
- **UAE PDPL** + **KSA PDPL**: cross-border + impact assessment.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| WCAG 2.2 AA conformance review | per-publish (LEAN) + annual external audit | ops-security + external accessibility firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| EU AI Act Annex III §3 conformity (HR/legal/medical overlay) | per-launch + annual | council-privacy + external AI firm |
| EU DSA transparency report | annually | council-privacy |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |

## References

- ADR-0028, ADR-0117, ADR-0126, ADR-0140, ADR-SITES-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`,
  `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- EU AI Act Regulation (EU) 2024/1689.
- EU DSA Regulation (EU) 2022/2065.
- ePrivacy Directive 2002/58/EC.
- ADA Title III + Section 508 + WCAG 2.2.
- ATAG 2.0 — w3.org/TR/ATAG20.
- OWASP ASVS v4.
- W3C Subresource Integrity.
- SLSA L3 — slsa.dev.
- NIST SSDF SP 800-218.
- CIS Kubernetes Benchmark.
- eIDAS Regulation (EU) 910/2014.
- NIS2 Directive (EU) 2022/2555.
