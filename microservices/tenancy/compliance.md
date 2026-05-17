---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-tenancy, council-architecture, ops-compliance
related_adrs: [ADR-0018, ADR-0028, ADR-0117, ADR-0123, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/tenancy/threat-model.md
  - microservices/tenancy/dpia.md
  - microservices/tenancy/policy/rls-isolation.md
  - microservices/tenancy/policy/data-residency.md
  - microservices/tenancy/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (tenancy µservice)

## Purpose

The canonical control-to-framework mapping for the tenancy µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / etc.) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact. **Tenancy is the highest-stakes µservice in oyatie's catalog; its isolation guarantees are reviewed by every other µservice's auditor as a precondition.**

## Enforced Frameworks (every µservice; every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | Integrity and ethical values | Code-of-conduct + signed-commit policy; CODEOWNERS reviewed quarterly | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/tenancy/CODEOWNERS` |
| CC1.4 | Commitment to competence | Onboarding + training | `docs/standards/onboarding.md` |
| CC1.5 | Accountability for performance | Per-µservice SLO targets + on-call rotation | `microservices/tenancy/PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `runbooks/jwt-key-rotation.md` + `incident-response.md` |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` §"Escalation" |
| CC2.3 | Communication with external parties | DPA + BAA + tenant onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per ADR + per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule for sensitive ops | `policy/rls-isolation.md` §"Audit Trail" |
| CC3.4 | Significant change risk | Change-management via PR review + LEAN lanes | `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission on every state transition | ADR-0028 + audit-chain µservice |
| CC5.1 | Control activities | LEAN lanes (rls-no-superuser-bypass, rls-force-on-tenant-tables, jwt-key-fingerprint-advertised) | `microservices/governance/` |
| CC5.2 | Technology controls | RLS + JWT + Cedar defence-in-depth | `policy/rls-isolation.md` + `policy/*.cedar` |
| CC5.3 | Policy and procedure deployment | Per-µservice runbooks + standards | `docs/standards/*.md` + `microservices/tenancy/runbooks/` |
| **CC6.1** | **Logical and physical access** | **OIDC + MFA + Cedar policy + JIT elevation via OpenBao** | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| **CC6.2** | **Authentication + authorization** | **OIDC + per-tenant JWT + SPIFFE identity** | `policy/rls-isolation.md` §"Tenant Identity Model" |
| CC6.3 | Adds/removes access | OpenBao access lifecycle + audit | OpenBao audit log |
| **CC6.6** | **Logical access control** | **Postgres RLS + JWT + Cedar (the load-bearing trio)** | `policy/rls-isolation.md` |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade for disposal | `policy/data-residency.md` + DSR cascade |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype CI lanes; weekly CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | Patroni HA + per-tenant rate limits + auto-scaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | tenancy self-OpenSLOs |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | branch-protection.yaml |
| CC9.1 | Risk mitigation | Multi-region + DR pair + automated rollback | `multi-region.md` + ADR-0130 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice |
| P2 | Choice and consent | Tenant onboarding consent capture (OpenBao tenant-resolver) |
| P3 | Collection | Tenancy stores only tenant-level metadata; data minimisation by design |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own data; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | Schema validation + audit-chain integrity |
| P8 | Monitoring and enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS in transit + cross-pack-replication-forbidden | `policy/data-residency.md` |
| **A.5.15** | **Access control** | **OIDC + Cedar + RLS + JWT** | `policy/rls-isolation.md` |
| A.5.17 | Authentication information | OpenBao secret rotation (30d JWT keys / 90d DB password) | OpenBao audit log |
| A.5.18 | Access rights | RBAC via declarative IaC; UI editing forbidden | `iac/terraform/postgres-rbac.tf` |
| A.5.23 | Information security for use of cloud services | OCI HIPAA-eligible regions for pack-us-healthcare | `policy/data-residency.md` |
| A.5.24 | Information security incident management planning | Incident-response playbook | `incident-response.md` |
| A.5.25 | Assessment and decision on information security events | Severity classification | `incident-response.md` §"Severity" |
| A.5.26 | Response to information security incidents | Severity-driven response runbook | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from information security incidents | Post-incident review template + ADR follow-up | `runbooks/postmortem-template.md` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for business continuity | Multi-region DR + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory and contractual requirements | This document + per-pack overlays | `compliance.md` |
| A.5.32 | Intellectual property rights | License-policy CI lane | `oya-check-license-policy` |
| A.5.33 | Protection of records | Audit-chain immutability + Postgres retention | `policy/data-residency.md` |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT elevation via OpenBao; 2-person rule | OpenBao audit |
| **A.8.3** | **Information access restriction** | **RLS + JWT + Cedar** | `policy/rls-isolation.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | branch-protection.yaml |
| A.8.5 | Secure authentication | OIDC + MFA + per-tenant JWT | `policy/rls-isolation.md` |
| A.8.7 | Protection against malware | Trivy + Grype container scanning + signed images (Cosign) | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | Hashed `tenant_id` pseudonymisation | `policy/rls-isolation.md` |
| **A.8.12** | **Data leakage prevention** | **Cross-tenant query refusal via RLS** | `policy/rls-isolation.md` |
| A.8.14 | Redundancy of information processing | Patroni HA + Citus + Valkey clustering | `multi-region.md` |
| A.8.15 | Logging | Audit-chain + Postgres structured logs | ADR-0028 |
| A.8.16 | Monitoring activities | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Networks security | Network policies + Istio mTLS | `cloud-k8s` |
| A.8.21 | Security of network services | Same | Same |
| A.8.23 | Web filtering | WAF + OWASP CRS at ingress | `cloud-iac` |
| A.8.24 | Use of cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | ADR-0028 + `policy/encryption.md` |
| A.8.25 | Secure development life cycle | LEAN lanes + PR review + spec-driven-development | `docs/standards/*` |
| A.8.26 | Application security requirements | OpenAPI schema enforcement + Cedar policy + LEAN | `contracts/openapi/tenancy.yaml` |
| A.8.27 | Secure system architecture | Clean architecture (ADR-0056 + ADR-0105) | ADR-0056 + ADR-0105 |
| A.8.28 | Secure coding | Cedar fuzz-testing + `cargo clippy` + `cargo deny` + SQL parameterisation | LEAN lanes |
| A.8.32 | Change management | PR review + LEAN gates (rls-no-superuser-bypass, rls-force-on-tenant-tables) | branch-protection.yaml |
| A.8.33 | Test information | Synthetic test data only; no prod-data in non-prod | `docs/standards/testing.md` |
| A.8.34 | Protection of information systems during audit testing | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/26/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership clause | DPA |
| 5(1)(b) | Purpose limitation | Purpose declared in DPIA §2.4 | DPIA |
| 5(1)(c) | Data minimisation | Tenancy stores only tenant-level metadata | DPIA |
| 5(1)(d) | Accuracy | Schema validation + audit-chain | LEAN |
| 5(1)(e) | Storage limitation | Retention matrix | `policy/data-residency.md` |
| **5(1)(f)** | **Integrity + confidentiality** | **RLS + multi-tenancy + encryption** | `policy/rls-isolation.md` |
| 5(2) | Accountability | This document + DPIA + ROPA | `legal/ropa.md` |
| 6 | Lawful basis | Art. 6(1)(b) + 6(1)(c) + 6(1)(f) | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) for PHI; explicit consent for KR PIPA Art. 23 sensitive | DPIA §4 |
| 13 + 14 | Information to data subjects | Tenant notice; joint-controllership cascade | DPA |
| **17** | **Right to erasure** | **DSR cascade across all µservices + proof-of-erasure** | DSR runner |
| 22 | Automated decision-making | Tenant lifecycle is operator-decision, not solely-automated | DPIA R-04 |
| 25 | Privacy by design + default | Pseudonymisation by default; RLS enforced; DSR built-in | `policy/rls-isolation.md` + `policy/data-residency.md` |
| 26 | Joint controllership | Tenancy is joint controller for tenant's end-users (with the tenant) | DPA |
| 28 | Processor terms | DPA template | `legal/dpa-template.md` |
| 30 | Records of processing | ROPA register | `legal/ropa.md` |
| 32 | Security of processing | Threat-model + RLS + JWT + Cedar | `threat-model.md` |
| 33 | Breach notification (72h) | Incident-response procedure | `incident-response.md` |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | Not triggered (residual ≤ Medium) | DPIA §7 |
| 44–46 | Cross-border transfers | SCC-only; Schrems-II supplementary measures | `legal/transfer-register.md` |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.1 정책 | Annual ISMS-P policy review | `compliance.md` annual review |
| KR-ISMS-P §2.2 위험관리 | Annual risk assessment | `threat-model.md` + `dpia.md` |
| KR-ISMS-P §2.3 인적보안 | Background check + onboarding training | `docs/standards/onboarding.md` |
| KR-ISMS-P §2.5 접근통제 | OpenBao JIT + Cedar | `policy/*.cedar` |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + AES-256-GCM | ISO 27001 A.8.24 |
| KR-ISMS-P §2.9 사고관리 | Severity 1/2 reporting per KR PIPA Art. 34 (72h to PIPC) | `incident-response.md` |
| KR-ISMS-P §2.10 개인정보처리 | DPIA + DSR + retention | `dpia.md` + `policy/data-residency.md` |
| KR-ISMS-P §2.12 위반관리 | Audit-chain tampering detection | ADR-0028 |
| KR PIPA Art. 15 + 17 | Consent for collection + use | Tenant onboarding consent |
| KR PIPA Art. 18 | Use limitation | Purpose declared in DPIA |
| KR PIPA Art. 22-2 | Sensitive data special protections | salt rotation + `policy/data-residency.md` |
| KR PIPA Art. 23 + 23-2 | Sensitive data + cross-border | DPIA + `policy/data-residency.md` (KR pack-pinned) |
| KR PIPA Art. 28 | Storage limitation | Retention matrix |
| KR PIPA Art. 29 | Technical safeguards | Mapped in `threat-model.md` |
| KR PIPA Art. 29-2 | Encryption requirement | A.8.24 |
| KR PIPA Art. 33 | DPIA + DPO | `dpia.md` + council-privacy chair |
| KR PIPA Art. 33-2 | DPO appointment | registered with PIPC |
| KR PIPA Art. 34 | Breach notification (72h) | `incident-response.md` |
| KR PIPA Art. 36 | Right-to-deletion | DSR cascade within 30d |
| KR 전자문서법 Art. 5–7 | Electronic document integrity + storage + verification | Ed25519 audit-chain |

### pack-us-healthcare (HIPAA Privacy + Security + Breach Notification Rules)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) | Risk management | Mitigations in §6 of `dpia.md` |
| §164.308(a)(3) | Workforce security | Background checks + JIT |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar + RLS + JWT |
| §164.308(a)(5) | Security awareness + training | Onboarding + annual refresher |
| §164.308(a)(6) | Security incident procedures | `incident-response.md` |
| §164.308(a)(7) | Contingency plan | `multi-region.md` + `runbooks/*` |
| §164.310(a) | Facility access controls | OCI HIPAA-eligible regions |
| **§164.312(a)(1)** | **Access control** | **RLS + JWT + Cedar** |
| §164.312(b) | Audit controls | Audit-chain emission |
| §164.312(c)(1) | Integrity | Ed25519 + audit-chain |
| §164.312(d) | Person/entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | TLS 1.3 in transit |
| §164.314(a)(1) | Business associate contracts | BAA template |
| §164.316(a)+(b)(2) | Policies + procedures + 6y retention | This file + `policy/data-residency.md` |
| §164.502(a) | Permitted uses (TPO) | Operations scope |
| §164.502(b) | Minimum necessary | Per-tenant minimum-data; RLS enforced |
| §164.514 | De-identification | hashed `tenant_id` pseudonymisation |
| §164.404 | Notification to individuals (60d max) | `incident-response.md` |
| §164.408 | Notification to HHS | OCR portal |

### pack-eu (GDPR + EDPB + eIDAS + NIS2 + DORA)

- EDPB Guidelines 4/2019 (Art. 25): satisfied per `dpia.md` §6 + `policy/rls-isolation.md`.
- EDPB Guidelines 9/2022 (breach notification): integrated in `incident-response.md` §"GDPR Art. 33".
- EDPB Recommendations 01/2020 (post-Schrems II): supplementary measures.
- eIDAS 910/2014 Art. 26 (AdES): Ed25519 audit-chain + proof-of-erasure.
- NIS2 (2022/2555): 24h + 72h + 1mo timelines.
- DORA (2022/2554): financial-services tenants — ICT-risk register + testing + third-party-risk.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA §2.4 |
| APPI Art. 20 | Security control measures | `policy/rls-isolation.md` + `threat-model.md` |
| APPI Art. 21 | Supervision of employees + entrustees | `legal/sub-processors.md` |
| APPI Art. 23 | Third-party provision restrictions | DPA + cross-border SCCs |
| APPI Art. 24 | Cross-border transfer restrictions | `policy/data-residency.md` JP-pack pinning |
| APPI Art. 26-2 | Breach reporting (72h) | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | Tenant DPA |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/tenancy-compliance-overlay.md` carry full local-law citation matrix:
- pack-sg: PDPA 2012 + MAS Notice 644.
- pack-au: Privacy Act 1988 APP 1–13 + APRA-CPS 234 + OAIC NDB.
- pack-in: DPDPA 2023 + RBI Master Direction on IT Outsourcing 2023.
- pack-br: LGPD + BACEN Res. 4.893/2021.
- pack-ae: UAE PDPL Federal Decree-Law 45/2021.
- pack-ksa: KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity Framework 2017.

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact older than 90 days is referenced as "current" without a refresh date stamp. Forces quarterly re-validation of every control-implementation cited above.

### Evidence emission

For every framework × control cited, an evidence artifact lives at one of:
- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence
- `microservices/tenancy/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence
- `microservices/tenancy/evidence/dsr/<dsr_id>.json` — DSR proof-of-erasure

Per-framework continuous-compliance runs:
- Daily: SOC 2 CC4.x + CC7.x; ISO 27001 A.8.15 + A.8.16
- Weekly: CC8.x; A.5.27; Postgres role-attribute audit; RLS drift detection (5min cadence within day)
- Monthly: CC3.x; A.5.7
- Quarterly: this entire matrix re-validated; auditor-ready evidence snapshot frozen
- Annually: full re-attestation by external auditor

### Audit evidence delivery

External auditors receive a frozen evidence pack per `docs/templates/evidence-pack-template.md`; auditor JIT token (per `policy/auditor-scope.cedar`) scopes their read; engagement window bounded; every read audit-chain-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency --microservice tenancy` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0; HG-TEN registered.
- Annual SOC 2 Type 2 audit: `evidence/audits/soc2/<year>-type2-report.pdf`.
- Annual ISO 27001:2022 audit.
- Per-pack audit cadences per local law.

## References

- `microservices/tenancy/threat-model.md`.
- `microservices/tenancy/dpia.md`.
- `microservices/tenancy/policy/{rls-isolation, data-residency}.md`.
- `microservices/tenancy/policy/*.cedar`.
- `microservices/tenancy/incident-response.md`.
- ADR-0018 (Bominal); ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0123 (hyperscaler maturity claim gate); ADR-0130 (SLO gate); ADR-0131 (per-microservice flat layout); ADR-0140 (Cedar policy).
- SOC 2 Type 2: TSC 2017 + 2022 PoF — `aicpa.org`.
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
