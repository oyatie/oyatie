---
doc_class: ComplianceMatrix
microservice: forms
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-legal-compliance + council-privacy + ops-security
review_cadence: quarterly (per ADR-0133 axis-4)
doc_status: published
---

# Forms — Compliance Matrix

This document maps every regulatory regime that applies to the forms µservice to the specific control point that satisfies it, and the CI lane / runbook / dashboard / SLO that asserts the control. **No control is a TODO** (per `feedback_autonomous_implementation_artifacts.md`).

## 1. GDPR (Regulation (EU) 2016/679)

Applies in pack-eu by primary residency; applies extraterritorially per Art. 3(2) to any tenant offering forms to EU subjects.

| Article | Obligation | Control | Where |
|---|---|---|---|
| Art. 5(1)(a) Lawfulness, fairness, transparency | Forms surfaces lawful basis at authoring; submitter consent notice (Art. 7) explicit per form | `form.v1.consent_notice` mandatory for any field with `data_class=PII_*` | Builder validation lane |
| Art. 5(1)(b) Purpose limitation | Each form declares purpose; cross-purpose reuse blocked | Cedar policy on response re-export | `policy/tenant-scope.cedar` PERMIT-9 |
| Art. 5(1)(c) Data minimisation | AI-form-build outputs flagged for over-asking; non-essential PII rejected at publish | `oya-forms-data-minimisation-conformance` | CI lane |
| Art. 5(1)(d) Accuracy | Submitter rectification path always-on per FR-24 | DSR cascade runner | `oya-dsr-cascade-runner` |
| Art. 5(1)(e) Storage limitation | Per-pack retention table; auto-purge after window | `oya-forms-retention-conformance` | `policy/data-residency.md` §Retention |
| Art. 5(1)(f) Integrity & confidentiality | Per-tenant DEK envelope encryption + TLS 1.3 in transit | ADR-FORMS-0003 | Column-level encryption + Istio mTLS |
| Art. 7 Conditions for consent | Explicit, granular, withdrawable; pre-ticked boxes forbidden | Form schema validation rejects pre-checked PII consent | Builder lint |
| Art. 9 Special categories | Tagged at authoring; explicit consent + DPIA mandatory; pack-eu PHI-equivalent fields require HIPAA pack or explicit DPIA waiver | `data_class=SENSITIVE_GDPR_ART9` flags | `dpia.md` §Special-category |
| Art. 12-22 Data subject rights | DSR cascade; access + rectification + erasure + portability + objection | `oya-dsr-cascade-runner` | Per-pack SLA |
| Art. 22 Automated decision-making | T2 AI-form-build outputs reviewed by tenant; no solely-automated form pass/fail | ADR-FORMS-0005 §"Decision" | Reviewer gate |
| Art. 25 DPbDD | Default config: minimal-fields, max-privacy, captcha-on, audit-on | `oya-forms-default-privacy-baseline` | CI lane |
| Art. 28 Processor | Forms is processor for tenants who are controllers; DPA template + sub-processor list | `legal/dpa-template.md`, `legal/sub-processors.md` | Annual review |
| Art. 30 Records of processing | ROPA generated per tenant from form metadata | `legal/ropa.md` | DPIA-linked |
| Art. 32 Security of processing | Pen-test annual; chaos-drill quarterly; Cedar default-deny | `threat-model.md` | SOC 2 evidence |
| Art. 33 Breach notification | ≤ 72h notification path via incident-response runbook | `runbooks/pii-leak-incident-p0.md` | Tested annually |
| Art. 35 DPIA | Mandatory for special-category, AI-form-build, large-scale (≥ 10k subjects) | `dpia.md` | Updated per release |
| Arts. 44-50 International transfers | SCC-only for cross-border; pack-routing default | `policy/data-residency.md` | Annual register update |

## 2. KR PIPA (Personal Information Protection Act, Korea)

Applies in pack-kr by primary residency.

| Article | Obligation | Control |
|---|---|---|
| Art. 15 Collection | Lawful basis declared per form; consent explicit | Builder validation |
| Art. 17 Use & provision | Cross-purpose use prohibited; explicit re-consent | Cedar PERMIT-9 |
| Art. 22-2 Personal Identification Number (RRN) | RRN field explicitly forbidden unless tenant has KISA-approved basis | Builder rejects raw-RRN field; offers alternative-ID field |
| Art. 23 Sensitive information | Tagged; processing controls | `data_class=SENSITIVE_PIPA_ART23` |
| Art. 24 Unique Identifier | Tightly scoped; consent rules | DB column-level encryption |
| Art. 28 Storage limitation | Bounded retention; minimum data | Retention table in `policy/data-residency.md` |
| Art. 28-2 Pseudonymisation | Available when statistical / research use | Tooling at export |
| Art. 36 Right to erasure | Honoured ≤ 30d | `oya-dsr-cascade-runner` |
| Enforcement Decree Art. 30 | Audit log retention ≥ 1y (KR-FSS sector: 5y) | Audit-chain retention |
| PIPC Notice 2020-7 | Overseas transfer notification | `legal/transfer-register.md` |

## 3. HIPAA (45 CFR Parts 160, 162, 164)

Applies in pack-us-healthcare with active BAA.

| Section | Obligation | Control |
|---|---|---|
| §164.308 Administrative safeguards | Workforce training; access management; risk analysis | OpenBao access; quarterly review |
| §164.310 Physical safeguards | OCI us-ashburn-1 HIPAA-eligible facility | OCI BAA |
| §164.312(a) Access control | RBAC + Cedar + audit | `policy/tenant-scope.cedar` |
| §164.312(b) Audit controls | Audit-chain per PHI read | `oya-forms-audit-chain-coverage` |
| §164.312(c) Integrity | Ed25519 seal + checksum | Audit-chain seal SDK |
| §164.312(d) Person/entity authentication | OIDC + MFA for PHI access | Tenancy entitlement |
| §164.312(e) Transmission security | TLS 1.3 + Istio mTLS | mesh policy |
| §164.316(b) Documentation retention | 6 years | `policy/data-residency.md` §Retention |
| §164.530(j) Records | 6-year retention | Same |
| §164.404 Breach notification | ≤ 60d to subjects; ≤ 60d to HHS | `runbooks/pii-leak-incident-p0.md` |

## 4. EU AI Act (Regulation (EU) 2024/1689)

Applies to T2 AI-form-build per ADR-FORMS-0005.

| Article | Obligation | Control |
|---|---|---|
| Art. 9 Risk management | Per-release risk review of AI-form-build | `legal/ai-act-conformity.md` |
| Art. 10 Data governance | Training data not used (BYO-LLM); prompt provenance recorded | `dashboards/ai-form-build-quality.json` |
| Art. 11 Technical documentation | Maintained per release; pointer in conformity-assessment | `legal/ai-act-conformity.md` |
| Art. 12 Record-keeping | 90-day prompt + completion log | Audit-chain |
| Art. 13 Transparency | Tenant + submitter notified when AI authored form | Builder UI label + submitter banner |
| Art. 14 Human oversight | Tenant explicitly reviews + accepts; reviewer-agent for cross-µservice | ADR-FORMS-0005 §"Decision" |
| Art. 15 Accuracy + robustness | Schema-valid completion rate ≥ 80% (SLI) | `slos/ai-form-build-quality.openslo.yaml` |
| Annex III §4 (high-risk employment) | If form used for employment screening → high-risk; mandatory CE-marking + notified body | High-risk classification + DPIA trigger |
| Art. 50 Transparency of AI-generated content | Form metadata exposes `ai_build_origin` | Form spec field |
| Art. 72 Post-market monitoring | Quarterly safety-signal review | `dashboards/ai-form-build-quality.json` |

## 5. eIDAS (Regulation (EU) 910/2014)

| Article / Annex | Obligation | Control |
|---|---|---|
| Art. 25 Legal effects of electronic signatures | SES/AES/QES classification per signature | ADR-FORMS-0006 |
| Annex I (QES requirements) | QES requires QSCD + qualified certificate | Per-tenant tier-G+ entitlement; CA list |
| ETSI EN 319 122/132/142 | XAdES/CAdES/PAdES profiles | Per-format signer worker |

## 6. PCI DSS v4 (Payment Card Industry)

Applies when payment field enabled.

| Requirement | Obligation | Control |
|---|---|---|
| Req 1 Network security | mTLS + WAF + NetworkPolicy egress allowlist | Helm + Istio |
| Req 3 Protect stored data | No PAN stored; tokenisation at fintech (Tier-G) | Payment adapter; scope-reduction proof |
| Req 4 Encrypt transmission | TLS 1.3 minimum | WAF config |
| Req 6 Secure systems | SLSA L3 + NIST SSDF | `legal/slsa-l3.md` |
| Req 8 Authenticate users | OIDC + MFA for tenant operators | Tenancy |
| Req 10 Log + monitor | Audit-chain + Prometheus + Mimir | observability bridge |
| Req 11 Test security | Annual pen-test + quarterly chaos | `threat-model.md` |
| Req 12 Information security policy | Policy at `policy/*.cedar` + Cedar default-deny | Cedar bundle |

## 7. WCAG 2.2 AA (W3C Recommendation, 5 October 2023)

Every form renderer pass blocked on AA failure.

| Criterion | Test | Lane |
|---|---|---|
| 1.4.3 Contrast | axe-core | `oya-governance-wcag22-conformance` |
| 2.1.1 Keyboard | manual + Cypress | Same |
| 2.4.7 Focus visible | Visual regression | Same |
| 2.5.8 Target size (minimum) | Auto-measure | Same |
| 3.3.7 Redundant entry | Pre-fill aware | Same |
| 3.3.8 Accessible authentication | OIDC ≤ AAL2 | Tenancy |
| 4.1.3 Status messages | Live regions on validation | Renderer |

## 8. APPI / PDPA / DPDPA / LGPD / UAE PDPL / KSA PDPL

Per-pack overlay enforcement:

| Pack | Statute | Key control |
|---|---|---|
| pack-jp | APPI | Pseudonymisation; cross-border notification |
| pack-sg | PDPA | DNC registry honour; consent withdrawal |
| pack-in | DPDPA 2023 | Significant Data Fiduciary obligations; pseudo + child-protection |
| pack-br | LGPD | DPO contact; ANPD breach notification |
| pack-ae | UAE PDPL | Cross-border via SCC-equivalent |
| pack-ksa | KSA PDPL | NCA notification; processor registration |

Each pack ships an overlay file at `regional-packs/<pack>/forms-compliance-overlay.md` (mirroring sheets / workflow-studio convention).

## 9. SOC 2 Type 2 + ISO 27001:2022

Forms participates in the oyatie-wide SOC 2 / ISO 27001 program; controls evidenced via:

- Audit-chain seals (every state change).
- Cedar policy bundle (access controls).
- DPIA + AI-Act-conformity (privacy + AI).
- Pen-test report (annual).
- Chaos-drill ledger (quarterly).
- Workforce training ledger.
- Sub-processor list + DPA.

## 10. OWASP ASVS v4 + CIS K8s + SLSA L3 + NIST SSDF

| Framework | Coverage |
|---|---|
| OWASP ASVS v4 L2 | All non-FinTech forms |
| OWASP ASVS v4 L3 | Payment-enabled + healthcare-pack forms |
| CIS Kubernetes Benchmark | All cluster nodes; weekly scan |
| SLSA Level 3 | Builds reproducible; provenance signed |
| NIST SSDF SP 800-218 | PO / PS / PW / RV practices in CI |

## References

- `policy/data-residency.md`.
- `threat-model.md`.
- `dpia.md`.
- `legal/{dpa-template, sub-processors, transfer-register, ai-act-conformity, slsa-l3, ropa, baa-template, schrems-supplementary-measures}.md` (authored per-tenant under operational layer).
- All ADRs cited inline.
