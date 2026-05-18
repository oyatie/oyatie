---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-observability, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/observability/threat-model.md
  - microservices/observability/dpia.md
  - microservices/observability/policy/tenant-isolation.md
  - microservices/observability/policy/data-residency.md
  - microservices/observability/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (observability µservice)

## Purpose

The canonical control-to-framework mapping for the observability µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / etc.) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact. Continuous-compliance-evidence emission keeps this matrix machine-verifiable; the `oya-governance-compliance-evidence-recency` lane enforces freshness.

## Enforced Frameworks (every µservice; every pack)

### SOC 2 Type 2 (2017 Trust Services Criteria + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | COSO Principle 1: Integrity and ethical values | Code-of-conduct + signed-commit policy; CODEOWNERS reviewed quarterly | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | COSO Principle 2: Board oversight | Council-architecture quarterly review of this µservice | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/observability/CODEOWNERS` (Slice C) |
| CC1.4 | Commitment to competence | Onboarding + training programs | `docs/standards/onboarding.md` |
| CC1.5 | Accountability for performance | Per-µservice SLO targets + on-call rotation | `microservices/observability/PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `runbooks/oncall-rotation.md` (Slice B7) |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` §"Escalation" |
| CC2.3 | Communication with external parties | DPA + BAA + tenant onboarding | `legal/dpa-template.md` (Slice D) |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per ADR + per IP | `evidence/multispectrum/` per µservice |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule for sensitive ops | `tenant-isolation.md` §"Audit Trail" |
| CC3.4 | Significant change risk | Change-management via PR review + LEAN lanes | Slice D `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission on every state transition | ADR-0028 + audit-chain µservice |
| CC5.1 | Control activities | LEAN lanes (50+ checks across governance µservice) | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar policy + Mimir multi-tenancy + signed commits | `policy/*.cedar` |
| CC5.3 | Policy and procedure deployment | Per-µservice runbooks + standards | `docs/standards/*.md` + `microservices/*/runbooks/` |
| CC6.1 | Logical and physical access | OIDC + MFA + Cedar policy + JIT elevation via OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant API keys + SPIFFE identity | `tenant-isolation.md` §"Tenant Identity Model" |
| CC6.3 | Adds/removes access | OpenBao access lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Mimir multi-tenancy + reserved tenants | `tenant-isolation.md` TI-01..TI-07 |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade for disposal | `data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype CI lanes; weekly CVE scan | `/specs/supply-chain.json` (existing) |
| CC7.1 | System operations | HA Mimir + per-tenant rate limits + auto-scaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | `/specs/agentic-slo-gated-promotion.json` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | `tasks/plan.md` (this changeset) |
| CC9.1 | Risk mitigation | Multi-region + DR pair + automated rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` (Slice D) |

**Privacy Criteria (P1–P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices communication | DPA template + tenant onboarding notice |
| P2 | Choice and consent | Tenant onboarding consent capture (OpenBao tenant-resolver) |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation enforcement |
| P4 | Use, retention, disposal | Retention matrix in `data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators can read own data; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | OpenSLO manifest schema validation + audit-chain integrity |
| P8 | Monitoring and enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use of information and other associated assets | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS in transit + cross-pack-replication-forbidden | `data-residency.md` §"Cross-Pack Replication" |
| A.5.15 | Access control | OIDC + Cedar + Mimir multi-tenancy | `tenant-isolation.md` |
| A.5.17 | Authentication information | OpenBao secret rotation (30d/90d cadences) | OpenBao audit log |
| A.5.18 | Access rights | RBAC managed via OpenTofu; UI editing forbidden | `iac/terraform/grafana-rbac.tf` (Slice C) |
| A.5.23 | Information security for use of cloud services | OCI HIPAA-eligible regions for pack-us-healthcare | `data-residency.md` §"pack-us-healthcare" |
| A.5.24 | Information security incident management planning + preparation | Incident-response playbook | `incident-response.md` |
| A.5.25 | Assessment and decision on information security events | Severity classification | `incident-response.md` §"Severity Definitions" |
| A.5.26 | Response to information security incidents | Severity-driven response runbook | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from information security incidents | Post-incident review template + ADR-#### successor-IP | `runbooks/postmortem-template.md` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for business continuity | Multi-region DR + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory and contractual requirements | This document + per-pack overlays | `compliance.md` (this file) |
| A.5.32 | Intellectual property rights | License-policy CI lane | `oya-check-license-policy` (governance µservice) |
| A.5.33 | Protection of records | Audit-chain immutability + Mimir retention | `data-residency.md` §"Retention" |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar policy | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT elevation via OpenBao; 2-person rule for sensitive ops | OpenBao audit |
| A.8.3 | Information access restriction | Mimir multi-tenancy + Cedar | `tenant-isolation.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | `branch-protection.yaml` |
| A.8.5 | Secure authentication | OIDC + MFA + per-tenant SPIFFE identity | `tenant-isolation.md` |
| A.8.7 | Protection against malware | Trivy + Grype container scanning + signed images (Cosign) | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | OTel SDK PII redactor | OTel redaction processor |
| A.8.12 | Data leakage prevention | Cross-tenant query refusal + DP aggregation | `tenant-isolation.md` TI-03 |
| A.8.14 | Redundancy of information processing facilities | Mimir HA + replication-factor ≥ 3 | `multi-region.md` |
| A.8.15 | Logging | Audit-chain + Loki structured logs | ADR-0028 |
| A.8.16 | Monitoring activities | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Networks security | Network policies + Istio mTLS | `cloud-k8s` µservice (sub-processor) |
| A.8.21 | Security of network services | Same | Same |
| A.8.23 | Web filtering | WAF + OWASP CRS at ingress | `cloud-iac` µservice |
| A.8.24 | Use of cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | ADR-0028 + `policy/encryption.md` |
| A.8.25 | Secure development life cycle | LEAN lanes + PR review + spec-driven-development | `docs/standards/*` |
| A.8.26 | Application security requirements | OpenAPI schema enforcement + Cedar policy + LEAN | `contracts/openapi/*.yaml` (Slice C) |
| A.8.27 | Secure system architecture | Clean architecture (ADR-0056 + ADR-0105) | ADR-0056 + ADR-0105 |
| A.8.28 | Secure coding | Cedar fuzz-testing + `cargo clippy` + `cargo deny` | LEAN lanes |
| A.8.32 | Change management | PR review + LEAN gates | `branch-protection.yaml` |
| A.8.33 | Test information | Synthetic test data only in dev/staging; no prod-data in non-prod | `docs/standards/testing.md` |
| A.8.34 | Protection of information systems during audit testing | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership clause | DPA |
| 5(1)(b) | Purpose limitation | Purpose declared in DPIA §2.4 | DPIA |
| 5(1)(c) | Data minimisation | OTel SDK redactor + sampling | OTel config |
| 5(1)(d) | Accuracy | OpenSLO schema validation + audit-chain | LEAN |
| 5(1)(e) | Storage limitation | Retention matrix | `data-residency.md` |
| 5(1)(f) | Integrity + confidentiality | Mimir multi-tenancy + encryption | `tenant-isolation.md` |
| 5(2) | Accountability | This document + DPIA + ROPA | `legal/ropa.md` (Slice D) |
| 6 | Lawful basis | Art. 6(1)(b) contract + 6(1)(c) legal-obligation + 6(1)(f) legitimate-interest | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) for PHI (pack-us-healthcare); explicit consent via tenant onboarding for KR PIPA sensitive | DPIA §4 |
| 13 + 14 | Information to data subjects | Tenant notice; joint-controllership cascade | DPA |
| 17 | Right to erasure | DSR cascade per `data-residency.md` | DSR runner |
| 22 | Automated decision-making | Operational decision carve-out (not solely automated producing legal effects) | DPIA §6 R-04, R-05 |
| 25 | Privacy by design + default | Pseudonymisation by default; multi-tenancy enforced; DSR built-in | `tenant-isolation.md` + `data-residency.md` |
| 28 | Processor terms | DPA template | `legal/dpa-template.md` (Slice D) |
| 30 | Records of processing | ROPA register | `legal/ropa.md` (Slice D) |
| 32 | Security of processing | Threat-model + tenant-isolation + Cedar | `threat-model.md` |
| 33 | Breach notification (72h) | Incident-response procedure | `incident-response.md` §"Regulatory notifications" |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | Not triggered (residual risks ≤ Medium) | DPIA §7 |
| 44–46 | Cross-border transfers | SCC-only; Schrems-II supplementary measures | `legal/transfer-register.md` (Slice D) |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.1 정책 (policy) | Annual ISMS-P policy review | `compliance.md` (this) annual review |
| KR-ISMS-P §2.2 위험관리 (risk management) | Annual risk assessment | `threat-model.md` + `dpia.md` |
| KR-ISMS-P §2.3 인적보안 (HR security) | Background check + onboarding training | `docs/standards/onboarding.md` |
| KR-ISMS-P §2.4 물리적보안 (physical) | Inherited from OCI ap-seoul-1 datacenter | Oracle attestation |
| KR-ISMS-P §2.5 인적보안 (access control) | OpenBao JIT + Cedar policy | `policy/*.cedar` |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + AES-256-GCM at rest | ISO 27001 A.8.24 |
| KR-ISMS-P §2.7 시스템 (system) | LEAN lanes + supply-chain | governance µservice |
| KR-ISMS-P §2.8 운영 (operations) | Runbooks + incident-response | `runbooks/*` + `incident-response.md` |
| KR-ISMS-P §2.9 사고관리 (incident management) | Severity 1/2 reporting per KR PIPA Art. 34 (72h to PIPC) | `incident-response.md` §"pack-kr regulatory" |
| KR-ISMS-P §2.10 개인정보처리 (PII processing) | DPIA + DSR + retention | `dpia.md` + `data-residency.md` |
| KR-ISMS-P §2.11 위탁관리 (sub-processor) | Sub-processor list + DPA cascade | `legal/sub-processors.md` (Slice D) |
| KR-ISMS-P §2.12 위반관리 (violation management) | Audit-chain tampering detection | ADR-0028 |
| KR PIPA Art. 3 | Collection minimization | OTel SDK PII redactor |
| KR PIPA Art. 15 + 17 | Consent for collection + use | Tenant onboarding consent |
| KR PIPA Art. 18 | Use limitation | Purpose declared in DPIA |
| KR PIPA Art. 22-2 | Sensitive data special protections | `tenant-isolation.md` + `data-residency.md` (salt rotation) |
| KR PIPA Art. 23 + 23-2 | Sensitive data + cross-border | DPIA + `data-residency.md` (KR pack-pinned) |
| KR PIPA Art. 24 | Resident registration number protection | Not processed; redactor strips |
| KR PIPA Art. 25 | Image data (e.g., CCTV) protection | Not applicable (no CCTV) |
| KR PIPA Art. 28 | Storage limitation | Retention matrix |
| KR PIPA Art. 29 | Technical safeguards | Mapped in `threat-model.md` per-pack-kr overlay |
| KR PIPA Art. 29-2 | Encryption requirement | Inherited from A.8.24 |
| KR PIPA Art. 33 | DPIA + designate DPO + impact assessment | `dpia.md` + council-privacy chair |
| KR PIPA Art. 33-2 | DPO appointment notification | council-privacy chair registered with PIPC |
| KR PIPA Art. 34 | Breach notification (72h to PIPC + 72h to data subjects) | `incident-response.md` |
| KR 전자문서법 Art. 5 | Electronic document integrity | Ed25519 audit-chain seals |
| KR 전자문서법 Art. 6 | Electronic document storage | Mimir immutable blocks + audit-chain |
| KR 전자문서법 Art. 7 | Electronic document verification | Audit-chain Merkle proofs |

### pack-us-healthcare (HIPAA Privacy + Security + Breach Notification Rules)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | Annual risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk management | Mitigations in §6 of `dpia.md` | DPIA |
| §164.308(a)(3) Workforce security | Background checks + JIT elevation | OpenBao + ops-security |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar policy + Mimir multi-tenancy | `tenant-isolation.md` |
| §164.308(a)(5) Security awareness + training | Onboarding + annual refresher | `docs/standards/onboarding.md` |
| §164.308(a)(6) Security incident procedures | `incident-response.md` | Incident response |
| §164.308(a)(7) Contingency plan | `multi-region.md` + `runbooks/*` | DR plan |
| §164.310(a) Facility access controls | OCI HIPAA-eligible regions | Oracle attestation |
| §164.312(a)(1) Access control | Mimir multi-tenancy + Cedar | `tenant-isolation.md` |
| §164.312(b) Audit controls | Audit-chain emission | ADR-0028 |
| §164.312(c)(1) Integrity | Ed25519 + Mimir block validation | `threat-model.md` T-T-02, T-T-04 |
| §164.312(d) Person/entity authentication | OIDC + MFA + SPIFFE | `tenant-isolation.md` |
| §164.312(e)(1) Transmission security | TLS 1.3 in transit | `policy/encryption.md` |
| §164.314(a)(1) Business associate contracts | BAA template | `legal/baa-template.md` (Slice D) |
| §164.316(a)+(b)(2) Policies + procedures + 6y retention | `compliance.md` + retention matrix | This file + `data-residency.md` |
| §164.502(a) Permitted uses + disclosures (TPO) | Purpose limited to operations | DPIA §2.4 |
| §164.502(b) Minimum necessary | OTel SDK redactor | OTel config |
| §164.514 De-identification | Pseudonymisation + tenant-isolation | `tenant-isolation.md` |
| §164.404 Notification to individuals (60d max) | Breach response chain | `incident-response.md` |
| §164.406 Notification to media (1000+ individuals) | Comms templates | `incident-response.md` |
| §164.408 Notification to HHS (60d/annual) | OCR reporting via `incident-response.md` | OCR portal |

### pack-eu (GDPR Arts. cited above + EDPB Guidelines + eIDAS + NIS2)

- EDPB Guidelines 4/2019 (Art. 25 data protection by design): satisfied per `dpia.md` §6 + `tenant-isolation.md`.
- EDPB Guidelines 9/2022 (breach notification): integrated in `incident-response.md` §"GDPR Art. 33 notification (72h)".
- EDPB Recommendations 01/2020 (post-Schrems II): supplementary measures in `legal/schrems-supplementary-measures.md` (Slice D).
- eIDAS 910/2014 Art. 26 (Advanced Electronic Signature): Ed25519 audit-chain seals satisfy AdES requirements when sealing EU-tenant transaction records.
- NIS2 (2022/2555): incident reporting timelines (24h initial + 72h detailed + 1mo final) in `incident-response.md`.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA §2.4 |
| APPI Art. 20 | Security control measures | `tenant-isolation.md` + `threat-model.md` |
| APPI Art. 21 | Supervision of employees + entrustees | `legal/sub-processors.md` |
| APPI Art. 23 | Third-party provision restrictions | DPA + cross-border SCCs |
| APPI Art. 24 | Cross-border transfer restrictions | `data-residency.md` JP-pack pinning |
| APPI Art. 26-2 | Data breach reporting (PPC + data subject) | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | Tenant DPA consent capture |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/compliance-overlay.md` carry the full local-law citation matrix. Each overlay follows this document's structure:
- pack-sg: PDPA 2012 (Singapore PDPC + MAS Notice 644 for finance)
- pack-au: Privacy Act 1988 APP 1–13 + APRA-CPS 234 (for finance)
- pack-in: DPDPA 2023 + RBI Master Direction on IT Outsourcing 2023
- pack-br: LGPD + BACEN Res. 4.893/2021 (for finance)
- pack-ae: UAE PDPL Federal Decree-Law 45/2021
- pack-ksa: KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity Framework 2017

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact older than 90 days is referenced as "current" without a refresh date stamp. Forces quarterly re-validation of every control-implementation cited above.

### Evidence emission

For every framework × control cited, an evidence artifact lives at one of:
- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence (config snapshot, lane-run output, audit-chain seal)
- `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence

Per-framework continuous-compliance runs:
- Daily: SOC 2 CC4.x (monitoring) + CC7.x (operations); ISO 27001 A.8.15 + A.8.16 (logging + monitoring)
- Weekly: CC8.x (change management); A.5.27 (learning from incidents)
- Monthly: CC3.x (risk assessment refresh); A.5.7 (threat intelligence refresh)
- Quarterly: this entire matrix re-validated; auditor-ready evidence snapshot frozen
- Annually: full re-attestation by external auditor

### Audit evidence delivery

External auditors receive a frozen evidence pack per `docs/templates/evidence-pack-template.md`; auditor JIT token (per `policy/auditor-scope.cedar`) scopes their read; engagement window bounded; every read audit-chain-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 audit: external auditor sign-off recorded in `evidence/audits/soc2/<year>-type2-report.pdf`.
- Annual ISO 27001:2022 audit: recorded analogously.
- Per-pack audit cadences per local law (KR PIPC review on demand; HIPAA covered-entity audits on demand; GDPR DPA inquiries on demand).

## References

- `microservices/observability/threat-model.md`.
- `microservices/observability/dpia.md`.
- `microservices/observability/policy/{tenant-isolation, data-residency}.md`.
- `microservices/observability/policy/*.cedar`.
- `microservices/observability/incident-response.md`.
- ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0123 (hyperscaler maturity claim gate); ADR-0139 (SLO gate); ADR-0131 (per-microservice flat layout); ADR-0140 (Cedar policy).
- SOC 2 Type 2: TSC 2017 + 2022 Points of Focus — `aicpa.org/topic/audit-assurance/audit-and-assurance-greater-than-soc-2`.
- ISO/IEC 27001:2022 + ISO/IEC 27002:2022 — `iso.org/standard/27001`.
- GDPR — `gdpr-info.eu`; EDPB Guidelines — `edpb.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`.
- HIPAA — `hhs.gov/hipaa`.
- APPI — `ppc.go.jp`.
- PDPA (SG) — `pdpc.gov.sg`; MAS — `mas.gov.sg`.
- Privacy Act 1988 (AU) — `oaic.gov.au`; APRA — `apra.gov.au`.
- DPDPA 2023 (IN) — `meity.gov.in`.
- LGPD (BR) — `gov.br/anpd`.
- UAE PDPL — `mohre.gov.ae`.
- KSA PDPL — `sdaia.gov.sa`; SAMA — `sama.gov.sa`.
