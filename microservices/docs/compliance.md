---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140, ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
doc_status: published
---

# Compliance Matrix — docs µservice

## Purpose

Enumerate compliance frameworks engaged by docs, the controls satisfied, and where each control is evidenced.

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |
| WCAG 2.2 AA | Accessibility surface (editor + export) | §"WCAG 2.2 AA" below |
| SLSA Level 3 | Supply-chain (CRDT library, Pandoc, WeasyPrint, Chromium, ClamAV) | §"SLSA L3" below |
| NIST SSDF | Secure development | §"NIST SSDF" below |
| OWASP ASVS v4.0 | Web service | §"OWASP ASVS" below |
| CIS Kubernetes Benchmark | Cluster substrate | §"CIS K8s" below |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + state-level (CCPA / CMIA / NY SHIELD) |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + NIS2 + eIDAS 910/2014 (PAdES) + EU AI Act 2024/1689 |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 |
| pack-in | DPDPA 2023 §6-11 + RBI Master Direction on IT Outsourcing |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + SAMA Cybersecurity Framework 2017 |

### Clinical-content overlays

| Framework | Engaged when | Notes |
|---|---|---|
| FDA 21 CFR Part 11 | clinical-notes tenant in pack-us-healthcare | audit-chain Ed25519 seal satisfies §11.10(e) audit trail + §11.50 electronic signature |
| ICH GCP E6(R2) | clinical-research-document tenant | retention + integrity per §4.9 + §5.5 |

## SOC 2 Mapping

| TSC | Control | Docs evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | this compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC4.1 | Demonstrates evaluation | audit-chain emission per doc lifecycle + LEAN check coverage |
| CC4.2 | Selects monitoring activities | observability dashboards + per-changeset evidence |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies + per-block ACL |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys + share-link Ed25519 |
| CC6.3 | Authorises | Cedar policies |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | embed-resolver source-side ACL passthrough + per-block ACL |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001 Mapping

| Annex A Control | Docs evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use) | policy/editor-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar + RLS + per-block ACL |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation + share-link key rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.26 (response to security incidents) | incident-response.md |
| A.5.27 (lessons from incidents) | post-incident review process |
| A.5.28 (collection of evidence) | audit-chain seal |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | this compliance.md per-pack overlays |
| A.5.32 (intellectual property) | dependencies + licenses in `catalog/*.yaml` |
| A.5.33 (records protection) | retention + legal-hold (Object Lock) |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access rights) | OpenBao JIT + 2-person rule |
| A.8.3 (info access restriction) | RLS + Cedar + per-block ACL |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.7 (protection against malware) | ClamAV / OPSWAT attachment scan; HTML sanitiser |
| A.8.11 (data masking) | redaction in cross-tenant projection + export |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan + gVisor egress block |
| A.8.15 (logging) | observability + audit-chain |
| A.8.16 (monitoring activities) | dashboards + alerts |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.21 (security of network services) | per-tenant API key + rate limits |
| A.8.23 (web filtering) | WAF at ingress |
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0139 SLO-gated promotion |
| A.8.26 (application security requirements) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 |
| A.8.28 (secure coding) | LEAN check `oya-check-ooxml-import-fidelity` + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.33 (test information) | synthetic test tenants per `ci-scope.cedar` |
| A.8.34 (audit + protection of audit systems) | audit-chain immutability + 2-person rule on admin |

## GDPR Mapping

| Article | Docs evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | per-block ACL + redaction in export |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain |
| Art. 6(1) lawful basis | dpia.md §2.4 |
| Art. 9 special-category | pack-us-healthcare overlay + pack-kr flagged-doc |
| Art. 13/14 transparency | tenant DPA template + AI-assist labelling |
| Art. 17 right-to-erasure | DSR cascade + hold-vs-erasure policy |
| Art. 22 automated decision | AI-assist is suggestion-only; no legal-effect on subject (unless pack-eu HR T1/T2 which is refused at Cedar) |
| Art. 25 by design + default | type-system separation + Cedar policy + per-block ACL |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Pack-overlay detail: pack-kr (KR PIPA + ISMS-P + 전자문서법)

| PIPA Article | Docs evidence |
|---|---|
| Art. 15 (consent for collection) | tenant onboarding consent flow |
| Art. 17 (cross-border transfer) | default-residency + SCC clause |
| Art. 18 (use beyond stated purpose) | dpia.md purpose-limitation §2.4 |
| Art. 22-2 (sensitive personal info, identifier-based) | flagged-doc Cedar policy |
| Art. 23 (sensitive personal info) | per-doc sensitivity flag + access restrictions |
| Art. 23-2 (cross-border sensitive) | pack-pinning + SCC |
| Art. 24 (uniquely identifying) | hashed tenant ID + salt rotation |
| Art. 28 (storage period) | retention bounded per asset table |
| Art. 29 (technical safeguards) | 12-safeguard mapping in threat-model.md |
| Art. 29-2 (data leakage prevention) | LEAN checks + DLP |
| Art. 33 (DPIA / 영향평가) | dpia.md |

| ISMS-P §§ | Docs evidence |
|---|---|
| §2.1 (information security policy) | this compliance.md + policy/* |
| §2.3 (asset management) | catalog/*.yaml |
| §2.5 (human security) | 2-person rule + JIT |
| §2.7 (access control) | RLS + Cedar + per-block ACL |
| §2.9 (operational security) | runbooks/* |
| §2.10 (communications security) | mesh mTLS + WAF |
| §2.11 (cryptography) | tenant-DEK + audit-chain Ed25519 + share-link signing |
| §2.12 (incident management) | incident-response.md |

| 전자문서법 §§ | Evidence |
|---|---|
| Art. 5 (integrity of electronic documents) | audit-chain Ed25519 |
| Art. 6 (storage of electronic documents) | retention + legal hold (S3 Object Lock) |
| Art. 7 (e-signature equivalence) | OIDC + JIT |

## Pack-overlay detail: pack-us-healthcare (HIPAA)

| 45 CFR §§ | Docs evidence |
|---|---|
| §164.308(a)(1)(ii)(A) risk analysis | dpia.md + threat-model.md |
| §164.308(a)(3) workforce security | OpenBao JIT + 2-person rule |
| §164.308(a)(4) info access management | Cedar + RLS + per-block ACL |
| §164.310(a) facility access | inherited from cloud-k8s |
| §164.312(a) access control | RLS + Cedar + per-block ACL |
| §164.312(b) audit controls | audit-chain Ed25519 + retention ≥ 6y |
| §164.312(c) integrity | audit-chain Merkle |
| §164.312(d) person authentication | OIDC + MFA |
| §164.312(e) transmission security | mesh mTLS |
| §164.314(a) BAA | legal/baa-template.md |
| §164.316 documentation | retain artifacts ≥ 6y |
| §164.502(a) Permitted Uses (TPO) | tenant DPA |
| §164.502(b) Minimum Necessary | per-block ACL type-narrowing |
| §164.504(e) BAA terms | BAA template |
| §164.512 disclosures permitted | dpia.md |
| §164.514 de-identification | redaction in export |

State-level:
- CCPA Cal. Civ. Code §1798.100: GDPR-Art-15 equivalent, DSR cascade satisfies.
- CMIA Cal. Civ. Code §56: medical info disclosure; pack-us-healthcare enforces.
- NY SHIELD Act: breach notification + reasonable security.

## Pack-overlay detail: pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

| Control | Citation | Docs implementation |
|---|---|---|
| Lawful basis | GDPR Art. 6 | per-purpose admission via Cedar |
| Right to erasure | GDPR Art. 17 | document-store-usecase erasure orchestrator + legal-hold reconciliation |
| Right to portability | GDPR Art. 20 | DOCX / Markdown / HTML / PDF / EPUB export per PRD FR-09 |
| DPIA | GDPR Art. 35 | this DPIA |
| Cross-border | Chapter V | per-pack EU residency; SCC for cross-pack |
| AI Act limited-risk (Art. 50) | EU AI Act 2024/1689 | AI-assist labels in UI |
| AI Act high-risk (Annex III §3) | EU AI Act 2024/1689 | T1/T2 HR-context overlays REFUSED at Cedar layer pending ADR-DOCS-0005 conformity assessment |
| ePrivacy | Art. 5(3) | doc web-UI tracking-free posture |
| eIDAS PAdES | 910/2014 | PAdES B-LT signed PDF export per pack-eu overlay |
| NIS2 | 2022/2555 | 24h+72h+1mo incident timelines |

## Pack-overlay detail: pack-us (CCPA / CPRA / sectoral)

| Control | Citation | Docs implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export per PRD FR-09 |
| Right to delete | CCPA §1798.105 | document-store deletion orchestrator |
| Sale of PD opt-out | CCPA §1798.120 | no sale; documented in `legal/sub-processors.md` |
| SOC 2 | TSC 2017+2022 | annual SOC 2 Type 2 |

## Pack-overlay detail: pack-jp (APPI)

| Control | Citation | Docs implementation |
|---|---|---|
| Specified-purpose | APPI Art. 17 | consent-recorded purposes per tenant onboarding |
| Leak notification | APPI Art. 22 | 3-business-day notification per incident-response.md |
| Cross-border | APPI Art. 24 | per-pack jp-tokyo-1; cross-pack consent-gated |

## Pack-overlay detail: pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/docs-compliance-overlay.md`.

Highlights:
- **PDPA 2012**: Part III Protection + Part IV Retention + Part VI Transfer.
- **APP 8 + APP 11 + APP 12**: cross-border + security + access.
- **APRA-CPS 234**: information security for financial-services tenants.
- **DPDPA 2023**: §6-11.
- **LGPD Arts. 33-36**: cross-border.
- **UAE PDPL** + **KSA PDPL**: cross-border + impact assessment + Sharia retention.

## WCAG 2.2 AA

| Success Criterion | Docs evidence |
|---|---|
| 1.1.1 Non-text content (alt-text required) | block-types kernel enforces alt-text on image blocks; export validation |
| 1.3.1 Info + relationships (heading hierarchy) | block-types schema validates heading order |
| 1.4.3 Contrast (minimum) | design-system primitives enforce; per-export Pa11y check |
| 2.1.1 Keyboard | editor + reader fully keyboard-navigable |
| 2.4.6 Headings + labels | block-types schema enforces |
| 3.3.1 Error identification | suggestion + comment surfaces accessible error states |
| 4.1.2 Name, role, value | ARIA roles on every block primitive |
| 4.1.3 Status messages | live-region announcements for collab updates |

Verification: `oya-governance-wcag-22-aa-conformance` LEAN lane runs axe-core + Pa11y on every export sample + UI test corpus.

## SLSA L3

- All Layer-A dependencies (Loro, Pandoc, WeasyPrint, Chromium, ClamAV, KaTeX) pinned + LTS per ADR-DOCS-0001 / ADR-DOCS-0003.
- All Layer-B crates built reproducibly + signed (sigstore/cosign).
- Provenance attestation per release.

## NIST SSDF

- Per-PR LEAN gates (linting, type-checking, port-location, layer-correctness).
- Per-release threat-model + DPIA review.
- Per-incident post-incident review with corrective actions tracked.

## OWASP ASVS v4.0

- V1: Architecture, design + threat modelling → threat-model.md + ADRs.
- V2: Authentication → OIDC + MFA + share-link Ed25519.
- V3: Session management → WS gateway tenant lease.
- V4: Access control → Cedar + RLS + per-block ACL.
- V5: Validation, sanitisation, encoding → `ammonia` HTML sanitiser + OOXML strict parser.
- V7: Error handling + logging → audit-chain seals.
- V8: Data protection → tenant-DEK envelope + Object Lock.
- V9: Communication → mTLS + TLS 1.3.
- V10: Malicious code → ClamAV + OPSWAT attachment scan.
- V11: Business logic → CRDT op signature + suggestion state machine.
- V12: Files + resources → attachment scanner + size limits.
- V14: Configuration → Helm charts + LTS pins.

## CIS Kubernetes Benchmark

- Inherited from cloud-k8s µservice; docs deployments comply with security-context (runAsNonRoot, readOnlyRootFilesystem, drop ALL caps) per the Helm template.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| PIPC examination | on-trigger | council-privacy |
| ANPD (Brazil) | on-trigger | council-privacy |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| WCAG 2.2 AA audit | bi-annually | external accessibility firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |
| AI Act conformity (HR-context) | per tenant opt-in | council-privacy + axis-docs |

## References

- ADR-0028 (Bominal), ADR-0117, ADR-0135, ADR-0140.
- ADR-DOCS-0001 through ADR-DOCS-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164; FDA 21 CFR Part 11.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- WCAG 2.2 (W3C, 2023-10).
- SLSA Specification v1.0.
- NIST SSDF SP 800-218.
- OWASP ASVS v4.0.
- CIS Kubernetes Benchmark v1.9.
- EU AI Act Regulation (EU) 2024/1689; eIDAS Regulation 910/2014.
