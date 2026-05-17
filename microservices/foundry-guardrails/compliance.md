---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-foundry-guardrails, council-architecture, ops-compliance
related_adrs: [ADR-0022, ADR-0028, ADR-0117, ADR-0123, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-guardrails/threat-model.md
  - microservices/foundry-guardrails/dpia.md
  - microservices/foundry-guardrails/policy/tenant-isolation.md
  - microservices/foundry-guardrails/policy/data-residency.md
  - microservices/foundry-guardrails/policy/guardrail-enforcement.md
  - microservices/foundry-guardrails/incident-response.md
review_cadence: annually + on every enforced-framework version update + on every classifier-model rollout
doc_status: published
---

# Compliance Control-to-Framework Mapping (foundry-guardrails µservice)

## Purpose

Authoritative control-to-framework mapping for foundry-guardrails. Tells an external auditor exactly which control satisfies which clause, with evidence pointers. Continuous-compliance-evidence emission keeps this matrix machine-verifiable; the `oya-governance-compliance-evidence-recency` lane enforces freshness.

## Enforced Frameworks (every tenant; every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | COSO 1: integrity + ethical values | Code-of-conduct + signed-commit policy | `docs/standards/code-review.md` |
| CC1.2 | COSO 2: board oversight | Council quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Org structure | RACI per µservice | `microservices/foundry-guardrails/CODEOWNERS` (Slice C) |
| CC1.4 | Competence | Onboarding | `docs/standards/onboarding.md` |
| CC1.5 | Accountability | Per-µservice SLO + on-call | `PRD.md` §Performance + `incident-response.md` |
| CC2.1-2.3 | Communication | Status page + runbooks + DPA/BAA | `legal/dpa-template.md` (Slice D) |
| CC3.1 | Risk identification | Annual threat-model + DPIA | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity | Multispectrum per IP | `evidence/multispectrum/` |
| CC3.3 | Fraud risk | Audit-chain Ed25519 + 2-person rule | `policy/tenant-isolation.md` |
| CC3.4 | Change risk | PR review + LEAN lanes | branch-protection |
| CC4.1 | Internal monitoring | LEAN + self-SLO | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emit | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes | governance µservice |
| CC5.2 | Tech controls | Cedar + Postgres RLS + signed commits | `policy/*.cedar` |
| CC5.3 | Policy deployment | Per-µservice runbooks + standards | `runbooks/*` + `docs/standards/*` |
| CC6.1 | Logical + physical access | OIDC + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authn + authz | OIDC + per-pod SPIFFE | `policy/tenant-isolation.md` |
| CC6.3 | Adds/removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Cedar + Postgres RLS + SPIFFE | `policy/tenant-isolation.md` |
| CC6.7 | Transmission + disposal | mTLS + KMS + DSR cascade | `policy/data-residency.md` |
| CC6.8 | Vulnerability mgmt | cargo deny + Trivy + Grype + weekly CVE | `/specs/supply-chain.json` |
| CC7.1 | System ops | HA classifier pool + rate limits + HPA | `capacity-model.md` |
| CC7.2 | Monitoring | Self-SLO + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly | Burn-rate + jailbreak-rate alarms | `/specs/agentic-slo-gated-promotion.json` |
| CC7.4 | Incident response | Severity-classified | `incident-response.md` |
| CC8.1 | Change mgmt | PR review + LEAN gates + Cedar bundle versioning | branch-protection |
| CC9.1 | Risk mitigation | Multi-region + DR + auto-rollback for classifier | `multi-region.md` + `runbooks/classifier-model-rollback.md` |
| CC9.2 | Vendor risk | Sub-processor list + DPA | `legal/sub-processors.md` (Slice D) |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice | DPA template + tenant onboarding |
| P2 | Choice + consent | OpenBao tenant-resolver onboarding consent |
| P3 | Collection | Prompt + output minimised at API; not persisted |
| P4 | Use, retention, disposal | Retention matrix; DSR cascade |
| P5 | Access | Tenant operator can read own decisions; Art. 22 explanation |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | Cedar v4 + shadow→enforce + golden fixtures |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Quarterly threat-model + monthly red-team | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + tenant-scope | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + cross-pack-forbidden | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + Postgres RLS | `policy/tenant-isolation.md` |
| A.5.17 | Authentication info | OpenBao rotation 30d/90d | OpenBao audit |
| A.5.18 | Access rights | Cedar-managed; UI editing forbidden | `policy/*.cedar` |
| A.5.23 | Cloud services info-security | OCI HIPAA-eligible regions for pack-us-hc | `policy/data-residency.md` |
| A.5.24-5.28 | Incident mgmt | `incident-response.md` | runbooks |
| A.5.30 | BC | Multi-region DR | `multi-region.md` |
| A.5.31 | Legal/statutory | This file + per-pack | `compliance.md` |
| A.5.32 | IP rights | License-policy CI lane | governance |
| A.5.33 | Protection of records | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy + PII | DPIA + DSR + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Info access restriction | Cedar + Postgres RLS | `policy/tenant-isolation.md` |
| A.8.4 | Source code access | CODEOWNERS + branch-protection | branch-protection |
| A.8.5 | Secure authn | OIDC + MFA + SPIFFE | `policy/tenant-isolation.md` |
| A.8.7 | Malware protection | Trivy + Grype + Cosign signed images | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | OTel redactor; prompt-hash in logs | OTel config |
| A.8.12 | DLP | Cedar deny on cross-tenant + secret-leak detector | `policy/tenant-scope.cedar` |
| A.8.16 | Monitoring | Self-SLO + OnCall | `failure-modes.md` |
| A.8.23 | Web filtering | WAF + OWASP CRS | cloud-iac |
| A.8.24 | Cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | `policy/encryption.md` (Slice D) |
| A.8.25 | Secure SDLC | LEAN + PR review + spec-driven | `docs/standards/*` |
| A.8.26 | App security requirements | OpenAPI schema + Cedar + LEAN | `contracts/openapi/*.yaml` |
| A.8.27 | Secure architecture | Clean architecture (ADR-0056 + ADR-0105) | ADRs |
| A.8.28 | Secure coding | clippy + cargo deny + Cedar fuzz | LEAN |
| A.8.32 | Change mgmt | PR + LEAN | branch-protection |
| A.8.33 | Test info | Synthetic only in non-prod | `docs/standards/testing.md` |
| A.8.34 | Audit info protection | Auditor JIT + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation |
|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership |
| 5(1)(b) | Purpose limitation | DPIA §2.4 |
| 5(1)(c) | Data minimisation | Prompt + output not persisted; OTel redactor |
| 5(1)(d) | Accuracy | Cedar schema validation + shadow→enforce |
| 5(1)(e) | Storage limitation | Retention matrix; not-persisted-by-default |
| 5(1)(f) | Integrity + confidentiality | Cedar + Postgres RLS + mTLS |
| 5(2) | Accountability | This doc + DPIA + ROPA |
| 6 | Lawful basis | Art. 6(1)(b) + (c) + (f) per purpose |
| 9 | Special category | Art. 9(2)(h) for PHI; explicit consent for PIPA Art. 23 |
| 13 + 14 | Information to data subjects | Tenant notice cascade |
| 17 | Right to erasure | DSR cascade (foundry-evidence owns history; guardrails not persisted) |
| 22 | Automated decision-making | Decision detail endpoint + FP escalation + tenant override |
| 25 | Privacy by design + default | Default-deny + pseudonymisation |
| 28 | Processor terms | DPA template |
| 30 | Records of processing | ROPA register |
| 32 | Security of processing | Threat-model + Cedar + multi-detector |
| 33 | Breach notification (72h) | `incident-response.md` |
| 35 | DPIA | This DPIA |
| 36 | Prior consultation | Not triggered (residual ≤ M) |
| 44-46 | Cross-border transfers | SCC-only; pack-pinning |

### EU AI Act (Reg 2024/1689) — Arts. 9-15

foundry-guardrails is a safety component of an AI system; the AI Act high-risk regime applies.

| Art. | Requirement | Implementation |
|---|---|---|
| Art. 9 | Risk-management system | DPIA + threat-model + PRD + compliance.md form the risk-management system; ongoing monitoring via shadow-mode metrics + Sev-1 incident retraining |
| Art. 10 | Data + data-governance | Classifier training-data provenance documented in model-cards; no PII / PHI in corpus |
| Art. 11 | Technical-documentation | PRD + threat-model + DPIA + compliance + model-cards (foundry-evidence) |
| Art. 12 | Record-keeping | Audit-chain Ed25519 seals on every decision; ≥ 1y retention (6y for HIPAA) |
| Art. 13 | Transparency | Decision detail (block_reason + cedar_policy_ids + classifier_model_versions) returned on every block |
| Art. 14 | Human-oversight | FP escalation budget + tenant Cedar-entitlement override + rule-author review queue + classifier-rollback runbook |
| Art. 15 | Accuracy + robustness + cybersecurity | Multi-detector ensemble + canonicalisation + red-team monthly + Cosign + shadow→enforce + per-tenant rate limits |

## Suggested Frameworks (per-pack)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| ISMS-P §2.1 정책 | Annual policy review | `compliance.md` review |
| ISMS-P §2.2 위험관리 | Annual risk assessment | `threat-model.md` + `dpia.md` |
| ISMS-P §2.3 인적보안 | Onboarding + training | `docs/standards/onboarding.md` |
| ISMS-P §2.4 물리적보안 | OCI datacenter inheritance | Oracle attestation |
| ISMS-P §2.5 인적보안 (접근통제) | OpenBao JIT + Cedar | `policy/*.cedar` |
| ISMS-P §2.6 암호화 | TLS 1.3 + AES-256-GCM | ISO A.8.24 |
| ISMS-P §2.7 시스템 | LEAN + supply-chain | governance |
| ISMS-P §2.8 운영 | Runbooks + incident-response | `runbooks/*` |
| ISMS-P §2.9 사고관리 | KR PIPA Art. 34 72h to PIPC | `incident-response.md` |
| ISMS-P §2.10 개인정보처리 | DPIA + DSR + retention | `dpia.md` + `policy/data-residency.md` |
| ISMS-P §2.11 위탁관리 | Sub-processor DPA cascade | `legal/sub-processors.md` |
| ISMS-P §2.12 위반관리 | Audit-chain tampering detection | ADR-0028 |
| PIPA Art. 3 | Collection minimization | Not-persisted-by-default |
| PIPA Art. 15 + 17 | Consent | Tenant onboarding |
| PIPA Art. 18 | Use limitation | DPIA §2.4 |
| PIPA Art. 22-2 | Sensitive special protections | Cedar default-deny on Art. 23 categories |
| PIPA Art. 23 + 23-2 | Sensitive + cross-border | DPIA + `policy/data-residency.md` (pack-pinned) |
| PIPA Art. 24 | RRN protection | Not processed; redactor strips |
| PIPA Art. 25 | Image-data | n/a |
| PIPA Art. 28 | Storage limitation | Not persisted by guardrails |
| PIPA Art. 29 | Technical safeguards | Mapped in `threat-model.md` per-pack-kr overlay |
| PIPA Art. 29-2 | Encryption | A.8.24 |
| PIPA Art. 33 | DPIA + DPO | `dpia.md` + council-privacy chair |
| PIPA Art. 33-2 | DPO appointment notification | council-privacy registered |
| PIPA Art. 34 | Breach notification (72h) | `incident-response.md` |
| 전자문서법 Art. 5 | EDoc integrity | Ed25519 audit-chain |
| 전자문서법 Art. 6 | EDoc storage | audit-chain immutability |
| 전자문서법 Art. 7 | EDoc verification | Merkle proofs |

### pack-us-healthcare (HIPAA)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk Analysis | Annual | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk Management | Mitigations | DPIA §6 |
| §164.308(a)(3) Workforce security | Background + JIT | OpenBao + ops-security |
| §164.308(a)(4)(ii)(B) Access auth | Cedar + Postgres RLS | `policy/tenant-isolation.md` |
| §164.308(a)(5) Awareness + training | Onboarding + annual refresher | `docs/standards/onboarding.md` |
| §164.308(a)(6) Security incident | `incident-response.md` | Incident response |
| §164.308(a)(7) Contingency plan | `multi-region.md` + runbooks | DR |
| §164.310(a) Facility access | OCI HIPAA-eligible | Oracle attestation |
| §164.312(a)(1) Access control | Cedar + Postgres RLS | `policy/tenant-isolation.md` |
| §164.312(b) Audit controls | Audit-chain emit | ADR-0028 |
| §164.312(c)(1) Integrity | Ed25519 + Cosign | `threat-model.md` T-T-03 + T-T-04 |
| §164.312(d) Person/entity authn | OIDC + MFA + SPIFFE | `policy/tenant-isolation.md` |
| §164.312(e)(1) Transmission security | TLS 1.3 | `policy/encryption.md` |
| §164.314(a)(1) BAA | BAA template | `legal/baa-template.md` |
| §164.316(a)+(b)(2) Policies + 6y retention | This + retention matrix | `policy/data-residency.md` |
| §164.502(a) Permitted Uses (TPO) | Operations | DPIA §2.4 |
| §164.502(b) Minimum necessary | Not persisted; minimal classifier outputs | DPIA |
| §164.504(e) Business Associate | BAA | `legal/baa-template.md` |
| §164.514 De-identification | OTel redactor | OTel config |

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

See "Enforced Frameworks" above; per-pack adds:
- **EDPB Guidelines 4/2019 (Art. 25)**: explicit alignment in DPIA §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain documented.
- **NIS2 (2022/2555)**: Annex I/II thresholds → 24h+72h+1mo reporting.
- **eIDAS 910/2014**: Ed25519 = AdES.
- **Schrems II + Arts. 44-46**: SCC-only; pack-eu-resident.
- **EU AI Act**: see "Enforced Frameworks".

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-guardrails-compliance-overlay.md`.

## Continuous-Compliance Evidence Emission

- Every decision + rule mutation + classifier rollout + Cedar bundle change emits an audit-chain seal.
- `oya-governance-compliance-evidence-recency` lane verifies seals are present + signed + within retention.
- Quarterly auditor read pack: scoped JIT auditor token; Grafana decision dashboards + Postgres rule-store reads via Cedar `auditor-scope.cedar`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency --microservice foundry-guardrails` — exit 0.
- Annual external SOC 2 audit; biennial ISO 27001 surveillance.
- pack-eu activation triggers EU AI Act notified-body assessment.
- pack-us-healthcare activation triggers HIPAA risk-analysis fresh sign-off.
- pack-kr activation triggers PIPC 영향평가 fresh sign-off.

## References

- `microservices/foundry-guardrails/threat-model.md`.
- `microservices/foundry-guardrails/dpia.md`.
- `microservices/foundry-guardrails/policy/*.md` + `*.cedar`.
- ADR-0028 (Bominal audit-chain).
- ADR-0022 (autonomy ceiling).
- ADR-0140 (Cedar substrate).
- SOC 2 Type 2 (2017 TSC + 2022 PoF).
- ISO 27001:2022.
- GDPR (Regulation 2016/679).
- EU AI Act (Regulation 2024/1689).
- HIPAA (45 CFR Part 164).
- KR PIPA + ISMS-P + 전자문서법.
- LGPD (Lei 13.709/2018).
- DPDPA 2023.
- APPI 改正 2022.
- PDPA 2012 (SG).
- Privacy Act 1988 + APP (AU).
- UAE PDPL FDL 45/2021.
- KSA PDPL RD M/19/2021.
- NIST AI RMF 1.0.
- OWASP LLM Top 10 (2025).
- MITRE ATLAS.
