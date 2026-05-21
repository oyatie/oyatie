---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-ontology, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/ontology/threat-model.md
  - microservices/ontology/dpia.md
  - microservices/ontology/policy/type-isolation.md
  - microservices/ontology/policy/data-residency.md
  - microservices/ontology/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (ontology µservice)

## Purpose

The canonical control-to-framework mapping for the ontology µservice. Tells external auditors (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / etc.) exactly which control implementation satisfies which framework clause, with pointers to evidence artifacts.

## Enforced Frameworks (every µservice; every pack)

### SOC 2 Type 2 (2017 Trust Services Criteria + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | COSO Principle 1: Integrity and ethical values | Code-of-conduct + signed-commit policy; CODEOWNERS reviewed quarterly | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | COSO Principle 2: Board oversight | Council-architecture quarterly review of this µservice | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/ontology/CODEOWNERS` |
| CC1.4 | Commitment to competence | Onboarding + training programs | `docs/standards/onboarding.md` |
| CC1.5 | Accountability for performance | Per-µservice SLO targets + on-call rotation | `microservices/ontology/PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `runbooks/oncall-rotation.md` |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` |
| CC2.3 | Communication with external parties | DPA + BAA + tenant onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per ADR + per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule | `tenant-isolation.md` (referenced from `policy/type-isolation.md`) §"Audit Trail" |
| CC3.4 | Significant change risk | Change-management via PR review + LEAN lanes | branch-protection.yaml |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission on every state transition | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar policy + Postgres RLS + signed commits | `policy/*.cedar` |
| CC5.3 | Policy + procedure deployment | Per-µservice runbooks + standards | `runbooks/*` |
| CC6.1 | Logical and physical access | OIDC + MFA + Cedar policy + JIT via OpenBao | `policy/tenant-scope.cedar`, `auditor-scope.cedar`, `ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant API keys + SPIFFE | `tenant-isolation.md` §"Tenant Identity Model" |
| CC6.3 | Adds/removes access | OpenBao access lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Postgres RLS + Citus strict + Cedar | `policy/type-isolation.md` TI-01..TI-13 |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype CI lanes | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + per-tenant rate limits + auto-scaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | `/specs/agentic-slo-gated-promotion.json` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | branch-protection.yaml |
| CC9.1 | Risk mitigation | Multi-region + DR pair + automated rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice |
| P2 | Choice and consent | Tenant onboarding consent capture (OpenBao tenant-resolver) |
| P3 | Collection | OTel SDK + property-tier annotation enforcement |
| P4 | Use, retention, disposal | Retention matrix in `data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own data; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | Object Type schema validation + audit-chain integrity |
| P8 | Monitoring and enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS in transit + cross-pack-replication-forbidden | `data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + Postgres RLS | `type-isolation.md` |
| A.5.17 | Authentication information | OpenBao secret rotation (30d / 90d) | OpenBao audit |
| A.5.18 | Access rights | Cedar policy managed via PR; UI editing forbidden | branch-protection.yaml |
| A.5.23 | Information security for use of cloud services | OCI HIPAA-eligible regions for pack-us-healthcare | `data-residency.md` |
| A.5.24 | Information security incident management planning + preparation | Incident-response playbook | `incident-response.md` |
| A.5.25 | Assessment and decision on info security events | Severity classification | `incident-response.md` |
| A.5.26 | Response to info security incidents | Severity-driven runbook | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from incidents | Postmortem + ADR successor-IP | `runbooks/postmortem-template.md` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for BCDR | Multi-region DR + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory + contractual requirements | This document + per-pack overlays | `compliance.md` (this) |
| A.5.32 | Intellectual property rights | License-policy CI lane | `oya-check-license-policy` |
| A.5.33 | Protection of records | Audit-chain immutability + Postgres + Kafka retention | `data-residency.md` |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar policy | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Postgres RLS + Citus strict + Cedar | `type-isolation.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | branch-protection.yaml |
| A.8.5 | Secure authentication | OIDC + MFA + per-tenant SPIFFE | `type-isolation.md` |
| A.8.7 | Protection against malware | Trivy + Grype + Cosign signed images | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | Function projection tier filter | `type-isolation.md` |
| A.8.12 | Data leakage prevention | Cross-tenant query refusal + DP aggregation | `type-isolation.md` TI-07 |
| A.8.14 | Redundancy of information processing facilities | Postgres HA + RF ≥ 3 | `multi-region.md` |
| A.8.15 | Logging | Audit-chain + structured logs | ADR-0028 |
| A.8.16 | Monitoring activities | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Networks security | NetworkPolicy + Istio mTLS | `cloud-k8s` µservice |
| A.8.21 | Security of network services | Same | Same |
| A.8.23 | Web filtering | WAF + OWASP CRS | `cloud-iac` µservice |
| A.8.24 | Use of cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | ADR-0028 + `policy/encryption.md` |
| A.8.25 | Secure development lifecycle | LEAN lanes + PR review + spec-driven-development | `docs/standards/*` |
| A.8.26 | Application security requirements | OpenAPI schema enforcement + Cedar policy + LEAN | `contracts/openapi/*.yaml` |
| A.8.27 | Secure system architecture | Clean architecture (ADR-0056 + ADR-0105) | ADR-0056 + ADR-0105 |
| A.8.28 | Secure coding | Cedar fuzz-testing + `cargo clippy` + `cargo deny` | LEAN lanes |
| A.8.32 | Change management | PR review + LEAN gates | branch-protection.yaml |
| A.8.33 | Test information | Synthetic test data only in non-prod | `docs/standards/testing.md` |
| A.8.34 | Protection during audit testing | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership | DPA |
| 5(1)(b) | Purpose limitation | DPIA §2.4 | DPIA |
| 5(1)(c) | Data minimisation | Property tier filter + OTel redactor | OTel config |
| 5(1)(d) | Accuracy | Object Type schema validation + audit-chain | LEAN |
| 5(1)(e) | Storage limitation | Retention matrix | `data-residency.md` |
| 5(1)(f) | Integrity + confidentiality | Postgres RLS + Cedar + encryption | `type-isolation.md` |
| 5(2) | Accountability | This document + DPIA + ROPA | `legal/ropa.md` |
| 6 | Lawful basis | Art. 6(1)(b)/(c)/(f) | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) for PHI; explicit consent for KR sensitive | DPIA §4 |
| 13 + 14 | Information to data subjects | Tenant notice; joint-controllership cascade | DPA |
| 17 | Right to erasure | DSR cascade | `data-residency.md` §"DSR" |
| 22 | Automated decision-making | Agent gateway autonomy_tier; tenant DPA carve-out | DPIA §6 R-06 |
| 25 | Privacy by design + default | Pseudonymisation + RLS + DSR built-in | `type-isolation.md` + `data-residency.md` |
| 28 | Processor terms | DPA template | `legal/dpa-template.md` |
| 30 | Records of processing | ROPA register | `legal/ropa.md` |
| 32 | Security of processing | Threat-model + tenant-isolation + Cedar | `threat-model.md` |
| 33 | Breach notification (72h) | Incident-response procedure | `incident-response.md` |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | Not triggered (residual ≤ Medium) | DPIA §7 |
| 44–46 | Cross-border transfers | SCC-only; Schrems-II supplementary | `legal/transfer-register.md` |

### EU AI Act 2024/1689 (where the agent gateway is high-risk)

| Art. | Requirement | Implementation |
|---|---|---|
| Art. 9 | Risk management for high-risk AI | DPIA R-14; tenant-opt-in for high-risk Functions |
| Art. 10 | Data governance | Per-property tier + DUB enforcement |
| Art. 11 | Technical documentation | This compliance.md + DPIA + threat-model |
| Art. 12 | Record-keeping | Audit-chain on every agent tool-call |
| Art. 13 | Transparency to deployers + users | Tenant DPA + transparency notice |
| Art. 14 | Human oversight | Cedar autonomy_tier ceiling; manual override |
| Art. 15 | Accuracy, robustness, cybersecurity | Per-Function SLO + Cedar gate + audit-chain |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.1 정책 | Annual ISMS-P policy review | `compliance.md` (this) |
| KR-ISMS-P §2.2 위험관리 | Annual risk assessment | `threat-model.md` + `dpia.md` |
| KR-ISMS-P §2.3 인적보안 (HR security) | Background check + onboarding training | `docs/standards/onboarding.md` |
| KR-ISMS-P §2.4 물리적보안 | Inherited from OCI ap-seoul-1 | Oracle attestation |
| KR-ISMS-P §2.5 접근통제 | OpenBao JIT + Cedar policy | `policy/*.cedar` |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + AES-256-GCM | ISO 27001 A.8.24 |
| KR-ISMS-P §2.7 시스템 | LEAN lanes + supply-chain | governance µservice |
| KR-ISMS-P §2.8 운영 | Runbooks + incident-response | `runbooks/*` + `incident-response.md` |
| KR-ISMS-P §2.9 사고관리 | Severity 1/2 reporting per KR PIPA Art. 34 | `incident-response.md` |
| KR-ISMS-P §2.10 개인정보처리 | DPIA + DSR + retention | `dpia.md` + `data-residency.md` |
| KR-ISMS-P §2.11 위탁관리 | Sub-processor list + DPA cascade | `legal/sub-processors.md` |
| KR-ISMS-P §2.12 위반관리 | Audit-chain tampering detection | ADR-0028 |
| KR PIPA Art. 3 | Collection minimization | Property tier filter |
| KR PIPA Art. 15 + 17 | Consent for collection + use | Tenant onboarding consent |
| KR PIPA Art. 18 | Use limitation | Purpose declared in DPIA |
| KR PIPA Art. 22-2 | Sensitive data special protections | `type-isolation.md` + `data-residency.md` (salt rotation) |
| KR PIPA Art. 23 + 23-2 | Sensitive data + cross-border | DPIA + `data-residency.md` (KR pack-pinned) |
| KR PIPA Art. 24 | Resident registration number | Not processed; redactor strips |
| KR PIPA Art. 25 | Image data | Not applicable |
| KR PIPA Art. 28 | Storage limitation | Retention matrix |
| KR PIPA Art. 29 | Technical safeguards | Mapped in `threat-model.md` per-pack-kr |
| KR PIPA Art. 29-2 | Encryption requirement | Inherited from A.8.24 |
| KR PIPA Art. 33 | DPIA + DPO + impact assessment | `dpia.md` + council-privacy chair |
| KR PIPA Art. 33-2 | DPO appointment notification | council-privacy chair registered with PIPC |
| KR PIPA Art. 34 | Breach notification (72h to PIPC + 72h to data subjects) | `incident-response.md` |
| KR 전자문서법 Art. 5 | Electronic document integrity | Ed25519 audit-chain seals |
| KR 전자문서법 Art. 6 | Electronic document storage | Postgres immutable + audit-chain |
| KR 전자문서법 Art. 7 | Electronic document verification | Audit-chain Merkle proofs |

### pack-us-healthcare (HIPAA Privacy + Security + Breach Notification Rules)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | Annual risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk management | Mitigations in §6 of `dpia.md` | DPIA |
| §164.308(a)(3) Workforce security | Background checks + JIT | OpenBao + ops-security |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar policy + Postgres RLS | `type-isolation.md` |
| §164.308(a)(5) Security awareness + training | Onboarding + annual refresher | `docs/standards/onboarding.md` |
| §164.308(a)(6) Security incident procedures | `incident-response.md` | Incident response |
| §164.308(a)(7) Contingency plan | `multi-region.md` + `runbooks/*` | DR plan |
| §164.310(a) Facility access controls | OCI HIPAA-eligible regions | Oracle attestation |
| §164.312(a)(1) Access control | Postgres RLS + Cedar | `type-isolation.md` |
| §164.312(b) Audit controls | Audit-chain emission | ADR-0028 |
| §164.312(c)(1) Integrity | Ed25519 + Postgres constraints | `threat-model.md` T-T-05 |
| §164.312(d) Person/entity authentication | OIDC + MFA + SPIFFE | `type-isolation.md` |
| §164.312(e)(1) Transmission security | TLS 1.3 in transit | `policy/encryption.md` |
| §164.314(a)(1) Business associate contracts | BAA template | `legal/baa-template.md` |
| §164.316(a)+(b)(2) Policies + procedures + 6y retention | `compliance.md` + retention matrix | This file + `data-residency.md` |
| §164.502(a) Permitted uses + disclosures (TPO) | Purpose limited to operations | DPIA §2.4 |
| §164.502(b) Minimum necessary | Property tier filter | OTel config |
| §164.514 De-identification | Pseudonymisation + tenant-isolation | `type-isolation.md` |
| §164.404 Notification to individuals (60d max) | Breach response chain | `incident-response.md` |
| §164.406 Notification to media (1000+ individuals) | Comms templates | `incident-response.md` |
| §164.408 Notification to HHS (60d/annual) | OCR reporting | `incident-response.md` |

### pack-eu (GDPR Arts. cited above + EDPB Guidelines + eIDAS + NIS2 + EU AI Act)

- EDPB Guidelines 4/2019 (Art. 25 by design): per `dpia.md` §6 + `type-isolation.md`.
- EDPB Guidelines 9/2022 (breach notification): integrated in `incident-response.md`.
- EDPB Recommendations 01/2020 (Schrems II): supplementary measures in `legal/schrems-supplementary-measures.md`.
- eIDAS 910/2014 Art. 26 (AdES): Ed25519 audit-chain seals.
- NIS2 (2022/2555): incident reporting timelines.
- EU AI Act 2024/1689 Arts. 9–15: see above table.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA §2.4 |
| APPI Art. 20 | Security control measures | `type-isolation.md` + `threat-model.md` |
| APPI Art. 21 | Supervision of employees + entrustees | `legal/sub-processors.md` |
| APPI Art. 23 | Third-party provision restrictions | DPA + cross-border SCCs |
| APPI Art. 24 | Cross-border transfer restrictions | `data-residency.md` JP-pack pinning |
| APPI Art. 26-2 | Data breach reporting | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | Tenant DPA consent capture |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/compliance-overlay.md`. Each follows this document's structure.

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact older than 90 days is referenced as "current" without a refresh date stamp.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence (config snapshot, lane-run output, audit-chain seal).
- `microservices/ontology/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence.

Per-framework continuous-compliance runs:
- Daily: SOC 2 CC4.x + CC7.x; ISO 27001 A.8.15 + A.8.16.
- Weekly: CC8.x; A.5.27.
- Monthly: CC3.x; A.5.7.
- Quarterly: this entire matrix re-validated.
- Annually: full re-attestation by external auditor.

### Audit evidence delivery

External auditors receive a frozen evidence pack per `docs/templates/evidence-pack-template.md`; auditor JIT token (per `policy/auditor-scope.cedar`) scopes their read.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency --microservice ontology` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 audit: external auditor sign-off recorded.
- Annual ISO 27001:2022 audit: same.

## References

- `microservices/ontology/threat-model.md`.
- `microservices/ontology/dpia.md`.
- `microservices/ontology/policy/{type-isolation, data-residency}.md`.
- `microservices/ontology/policy/*.cedar`.
- `microservices/ontology/incident-response.md`.
- ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0123 (hyperscaler-maturity); ADR-0139 (SLO); ADR-0131 (per-microservice flat layout); ADR-0140 (Cedar).
- SOC 2 Type 2: TSC 2017 + 2022 PoF — `aicpa.org`.
- ISO/IEC 27001:2022 + 27002:2022 — `iso.org`.
- GDPR — `gdpr-info.eu`; EDPB — `edpb.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`.
- HIPAA — `hhs.gov/hipaa`.
- APPI — `ppc.go.jp`.
- PDPA (SG) — `pdpc.gov.sg`; MAS — `mas.gov.sg`.
- Privacy Act 1988 (AU) — `oaic.gov.au`; APRA — `apra.gov.au`.
- DPDPA 2023 (IN) — `meity.gov.in`.
- LGPD (BR) — `gov.br/anpd`.
- UAE PDPL — `mohre.gov.ae`.
- KSA PDPL — `sdaia.gov.sa`; SAMA — `sama.gov.sa`.
- EU AI Act 2024/1689 — `eur-lex.europa.eu/eli/reg/2024/1689/oj`.

---



## §day-one-cert-readiness
This anchor is closed for `ontology` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `ontology` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +15 more.
- Example: `cedar-evaluate` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `cedar-evaluate` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `ontology` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`, `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`; +19 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `cedar-evaluate` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.ontology.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `ontology` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `cedar-evaluate` touches those data classes.
- Signal sources: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +14 more.
- Example event class: `oya.ontology.cedar.evaluate.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `ontology` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.ontology.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `cedar-evaluate` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `cedar-evaluate` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `ontology` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`, `ontology.unknown`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `ontology.unknown` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `ontology` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +15 more.
- Example: `cedar-evaluate` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.ontology` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/ontology/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.
- Example: `cedar-evaluate` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `ontology` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`, `microservices/ontology/iac/ech-config.yaml`, `microservices/ontology/iac/edge-waf.yaml`, `microservices/ontology/iac/helm/cedar-policy-engine/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `cedar-evaluate` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `ontology` is in annual full-scope pentest and every major `cedar-evaluate` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`, `microservices/ontology/iac/ech-config.yaml`, `microservices/ontology/iac/edge-waf.yaml`, `microservices/ontology/iac/helm/cedar-policy-engine/Chart.yaml`; +18 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `ontology` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `ontology` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `cedar-evaluate` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `ontology` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/ontology/catalog/oya-ontology-action-engine-usecase.yaml`, `microservices/ontology/catalog/oya-ontology-agent-gateway-rest.yaml`, `microservices/ontology/catalog/oya-ontology-app.yaml`, `microservices/ontology/catalog/oya-ontology-audit-chain-worker.yaml`, `microservices/ontology/catalog/oya-ontology-cedar-fragment-coverage-adapter.yaml`, `microservices/ontology/catalog/oya-ontology-entity-store-adapter-clickhouse.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `cedar-evaluate` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `ontology` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `cedar-evaluate` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `cedar-evaluate` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `ontology` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `ontology.unknown`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `cedar-evaluate` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `ontology`; owner `axis-ontology`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/ontology/capabilities/cedar-evaluate.yaml`, `microservices/ontology/capabilities/query-execute.yaml`, `microservices/ontology/capabilities/type-register.yaml`.
- API surfaces cited: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar/policy artifacts cited: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- SLO and dashboard evidence: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +16 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/ontology/contracts/asyncapi/ontology-events.yaml`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/contracts/proto/ontology.proto`.
- Cedar binding: `microservices/ontology/policy/abuse-defence.cedar`, `microservices/ontology/policy/auditor-scope.cedar`, `microservices/ontology/policy/ci-scope.cedar`, `microservices/ontology/policy/cross-tenant-refusal.cedar`, `microservices/ontology/policy/data-residency.md`, `microservices/ontology/policy/ontology-write-quota.cedar`; +3 more.
- State/event binding: `ontology.unknown`.
- Capability binding: `cedar-evaluate`, `query-execute`, `type-register`.
- SLO binding: `microservices/ontology/slos/action-invocation-availability.openslo.yaml`, `microservices/ontology/slos/audit-chain-emission-completeness.openslo.yaml`, `microservices/ontology/slos/dynamic-layer-freshness.openslo.yaml`, `microservices/ontology/slos/function-read-availability.openslo.yaml`, `microservices/ontology/slos/function-read-latency.openslo.yaml`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`.
- Runbook binding: `microservices/ontology/runbooks/cedar-fragment-rollback.md`, `microservices/ontology/runbooks/clickhouse-rebalance.md`, `microservices/ontology/runbooks/cross-tenant-leak-recovery.md`, `microservices/ontology/runbooks/object-type-deprecation.md`, `microservices/ontology/runbooks/ontology-bot-score-recalibration.md`, `microservices/ontology/runbooks/ontology-read-library-fallback.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `ontology`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `ontology`.
- `policy-engine` supplies the signed Cedar corpus while `ontology` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `ontology` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `ontology`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `ontology` applies the most restrictive policy and emits a degraded-mode audit event.
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

