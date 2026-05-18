---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-foundry-runtime, council-architecture, ops-compliance
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0028, ADR-0117, ADR-0123, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-runtime/threat-model.md
  - microservices/foundry-runtime/dpia.md
  - microservices/foundry-runtime/policy/runtime-isolation.md
  - microservices/foundry-runtime/policy/data-residency.md
  - microservices/foundry-runtime/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (foundry-runtime µservice)

## Purpose

Canonical control-to-framework mapping for the foundry-runtime µservice. Tells an external auditor exactly which control implementation satisfies which framework clause, with pointer to evidence artifact. Continuous-compliance-evidence emission keeps this matrix machine-verifiable; `oya-governance-compliance-evidence-recency` lane enforces freshness.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity / ethics | Code-of-conduct + signed-commits + CODEOWNERS | `docs/standards/code-review.md` |
| CC1.2 | Board oversight | Council-architecture quarterly review | council minutes |
| CC1.3 | Organizational structure | RACI per µservice | `microservices/foundry-runtime/CODEOWNERS` |
| CC1.4 | Competence | Onboarding + training | `docs/standards/onboarding.md` |
| CC1.5 | Performance accountability | SLO targets + on-call rotation | PRD §Performance + `incident-response.md` |
| CC2.1 | External communication | Status page + tenant comms | `runbooks/oncall-rotation.md` |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` |
| CC2.3 | Communication with external parties | DPA + BAA + onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per IP | `evidence/multispectrum/*` |
| CC3.3 | Fraud risk | Audit-chain Ed25519 + 2-person rule | `policy/runtime-isolation.md` |
| CC3.4 | Significant change risk | PR review + LEAN | `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN lanes + SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Redis ACL + Postgres RLS + signed commits | `policy/*.cedar` |
| CC5.3 | Policy deployment | Runbooks + standards | `docs/standards/*` + `runbooks/*` |
| CC6.1 | Logical access | OIDC + MFA + Cedar + JIT OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authentication / authorization | OIDC + per-tenant key + SPIFFE | `policy/runtime-isolation.md` |
| CC6.3 | Access lifecycle | OpenBao access lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Redis tenant-prefix + Postgres RLS + Cedar | `policy/runtime-isolation.md` TI-01..TI-08 |
| CC6.7 | Transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA + per-tenant rate limits + HPA | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate + cardinality alerts | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN + branch protection | branch-protection.yaml |
| CC9.1 | Risk mitigation | Multi-region DR + automated rollback (capability-version level via foundry-supervisor) | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8):**

| P# | Implementation |
|---|---|
| P1 | DPA template + onboarding notice |
| P2 | Tenant consent capture at onboarding |
| P3 | OTel SDK redactor + data_class + guardrails |
| P4 | Retention matrix + DSR cascade |
| P5 | Tenant operators read own data; DSR access cascade |
| P6 | Sub-processor list + transfer register |
| P7 | Capability descriptor schema + audit-chain |
| P8 | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Implementation |
|---|---|
| A.5.7 Threat intelligence | Annual threat-model + quarterly review |
| A.5.10 Acceptable use | Cedar policy + per-tenant scoping |
| A.5.14 Information transfer | mTLS in transit + cross-pack-forbidden |
| A.5.15 Access control | OIDC + Cedar + Redis ACL + Postgres RLS |
| A.5.17 Authentication info | OpenBao secret rotation (30d/90d) |
| A.5.18 Access rights | Cedar + RBAC managed via Terraform |
| A.5.23 Information security for cloud | OCI HIPAA-eligible for pack-us-healthcare |
| A.5.24–A.5.27 Incident management | `incident-response.md` |
| A.5.28 Evidence | Audit-chain Ed25519 |
| A.5.30 BC readiness | DR pair + RPO/RTO targets |
| A.5.31 Legal / statutory | This document + per-pack overlays |
| A.5.32 IP rights | License-policy CI lane |
| A.5.33 Records protection | Audit-chain immutability + Mimir retention |
| A.5.34 Privacy / PII | DPIA + DSR + Cedar |
| A.8.2 Privileged access | JIT OpenBao + 2-person rule |
| A.8.3 Access restriction | Redis prefix + Postgres RLS + Cedar |
| A.8.4 Source code access | CODEOWNERS + branch-protection |
| A.8.5 Secure authentication | OIDC + MFA + SPIFFE |
| A.8.7 Malware protection | Trivy + Grype + Cosign |
| A.8.11 Data masking | OTel SDK redactor + guardrails |
| A.8.12 DLP | Cross-tenant query refusal |
| A.8.14 Redundancy | HA replicas + Redis cluster RF≥1 + Postgres replica |
| A.8.15 Logging | Audit-chain + Loki structured logs |
| A.8.16 Monitoring activities | Self-observability + OnCall |
| A.8.20–A.8.21 Networks security | Istio mTLS + NetworkPolicy |
| A.8.23 Web filtering | WAF + OWASP CRS |
| A.8.24 Cryptography | TLS 1.3 + Ed25519 + AES-256-GCM |
| A.8.25 SDLC | LEAN lanes + PR review |
| A.8.26 Application security | OpenAPI + Cedar |
| A.8.27 Secure architecture | ADR-0056 + ADR-0105 clean architecture |
| A.8.28 Secure coding | Cedar fuzz + clippy + cargo deny |
| A.8.32 Change management | PR + LEAN |
| A.8.33 Test information | Synthetic test data only in dev/staging |
| A.8.34 Audit testing protection | Auditor JIT tokens |

### GDPR

| Art. | Implementation |
|---|---|
| 5(1)(a) Lawfulness | Tenant notice + joint-controllership |
| 5(1)(b) Purpose | Declared in DPIA §2.4 |
| 5(1)(c) Minimisation | guardrails + data_class + sampling |
| 5(1)(d) Accuracy | Capability descriptor schema + audit-chain |
| 5(1)(e) Storage limitation | Retention matrix |
| 5(1)(f) Integrity / confidentiality | Redis prefix + Postgres RLS + encryption |
| 5(2) Accountability | This + DPIA + ROPA |
| 6 Lawful basis | Art. 6(1)(b)/(c)/(f) per purpose |
| 9 Special category | Art. 9(2)(h) PHI; PIPA Art. 23(2) sensitive |
| 13 + 14 Info to data subjects | Tenant notice cascade |
| 17 Erasure | DSR cascade |
| 22 Automated decisions | Operational carve-out; tier-2+ requires human ack |
| 25 Privacy by design | Pseudonymisation + multi-tenancy default-on + DSR built-in |
| 28 Processor | DPA template |
| 30 ROPA | `legal/ropa.md` |
| 32 Security | This + threat-model |
| 33 Breach notification (72h) | `incident-response.md` |
| 35 DPIA | `dpia.md` |
| 36 Prior consultation | Not triggered |
| 44–46 Cross-border | SCCs only |

### EU AI Act (Regulation 2024/1689) — High-Risk Posture

| Art. | Requirement | Implementation |
|---|---|---|
| Art. 6 + Annex III | High-risk classification | Capability descriptor carries Annex III classification flag; runtime refuses unclassified high-risk capability in pack-eu |
| Art. 9 Risk management | Continuous risk-management system | `dpia.md` + `threat-model.md` + quarterly review cadence |
| Art. 10 Data + data governance | Training + validation + testing data quality | data_class annotations + capability descriptor schema |
| Art. 11 Technical documentation | Documentation of high-risk systems | This document + DPIA + per-capability registration |
| Art. 12 Record-keeping | Logs throughout lifecycle | Invocation lifecycle records + audit-chain |
| Art. 13 Transparency | Disclosure to deployer + user | Capability descriptor purpose-of-use; tenant notice cascade |
| Art. 14 Human oversight | Effective human oversight | AutonomyGate refusal + per-invocation reviewable lifecycle records |
| Art. 15 Accuracy + robustness + cybersecurity | Technical robustness | foundry-guardrails + circuit-breakers + provider-credential isolation + per-invocation timeout + per-tenant rate limits |
| Art. 16–22 Provider obligations | Risk management, QMS, registration | Quality management via LEAN lanes; conformity-assessment when applicable |
| Art. 26 Deployer obligations | Provider follows instructions for use | Capability descriptor instructions enforced by runtime |
| Art. 50 Transparency to natural persons | Disclosure for emotion-recognition / categorisation systems | Capability descriptor carries disclosure tag; tenant DPA cascades |
| Art. 53 Codes of practice | Voluntary alignment | Monitored per gtm-customer-success |

## Suggested Frameworks (per-pack)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법 + KR FSC AI Guideline 2024)

| Section | Implementation |
|---|---|
| KR-ISMS-P §2.1 정책 | Annual policy review |
| KR-ISMS-P §2.2 위험관리 | `threat-model.md` + `dpia.md` |
| KR-ISMS-P §2.3 인적보안 | Onboarding + background checks |
| KR-ISMS-P §2.5 인적보안 (access) | OpenBao JIT + Cedar |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + AES-256-GCM |
| KR-ISMS-P §2.9 사고관리 | KR PIPA Art. 34 (72h to PIPC) |
| KR-ISMS-P §2.10 개인정보처리 | DPIA + DSR + retention |
| KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 23-2, 24, 25, 28, 29, 29-2, 33, 33-2, 34 | Cross-mapped in `threat-model.md` + `dpia.md` per-pack-kr overlays |
| KR 전자문서법 Arts. 5, 6, 7 | Ed25519 audit-chain on every invocation event |
| KR FSC AI Guideline 2024 §3 (human-in-loop) | AutonomyGate; FSC notification on violation > threshold |

### pack-us-healthcare (HIPAA)

| 45 CFR | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar + Redis prefix + Postgres RLS |
| §164.308(a)(6) Incident procedures | `incident-response.md` |
| §164.308(a)(7) Contingency plan | `multi-region.md` |
| §164.310 Physical safeguards | OCI HIPAA-eligible inherited |
| §164.312(a)(1) Access control | Cedar + tenant prefix |
| §164.312(b) Audit controls | Audit-chain + ≥6y retention for PHI |
| §164.312(c)(1) Integrity | Ed25519 + descriptor signature |
| §164.312(d) Authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) Transmission security | TLS 1.3 |
| §164.314(a)(1) BAA | `legal/baa-template.md` |
| §164.316(a)+(b)(2) Policies + 6y retention | This + retention matrix |
| §164.502(a) Permitted Uses (TPO) | Operations purpose only |
| §164.502(b) Minimum Necessary | OTel SDK redactor + guardrails |
| §164.514 De-identification | Pseudonymisation + tenant-isolation |
| §164.404/406/408 Breach notification | `incident-response.md` |
| FDA SaMD pre-market | Clinical-decision-support capability classification flag |

### pack-eu (GDPR + EU AI Act + EDPB + eIDAS + NIS2)

- EDPB Guidelines 4/2019: `dpia.md` §6 + `policy/runtime-isolation.md`.
- EDPB Guidelines 9/2022: `incident-response.md` §"GDPR Art. 33 (72h)".
- EDPB Recommendations 01/2020: `legal/schrems-supplementary-measures.md`.
- eIDAS 910/2014 Art. 26 AdES: Ed25519 audit-chain seals.
- NIS2: 24h + 72h + 1mo timelines in `incident-response.md`.
- EU AI Act: per-article mapping above.

### pack-jp (APPI + METI)

| Art. | Implementation |
|---|---|
| APPI Art. 17 | Purpose declared in DPIA |
| APPI Art. 18 | Purpose limitation |
| APPI Art. 20 | `policy/runtime-isolation.md` |
| APPI Art. 21 | `legal/sub-processors.md` |
| APPI Art. 23 | DPA + cross-border SCCs |
| APPI Art. 24 | `policy/data-residency.md` JP-pack pinning |
| APPI Art. 26-2 | `incident-response.md` |
| APPI Art. 27 | Tenant DPA sensitive-consent |
| METI AI Governance 2024 | Voluntary alignment; documented in council-architecture briefings |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-runtime-compliance-overlay.md`:
- pack-sg: PDPA 2012 + MAS FEAT + Veritas Toolkit (for FS tenants).
- pack-au: Privacy Act 1988 APP 1–13 + AHRC AI guidance.
- pack-in: DPDPA 2023 + MeitY AI Advisory 2024 + RBI IT Outsourcing 2023.
- pack-br: LGPD + ANPD AI guidance + BACEN Res. 4.893/2021.
- pack-ae: UAE PDPL + UAE Charter for Responsible AI.
- pack-ksa: KSA PDPL + SDAIA Generative AI guidelines + SAMA Cybersecurity Framework 2017.

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact older than 90 days is referenced as "current" without a refresh date stamp.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence.
- `microservices/foundry-runtime/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset.

Per-framework continuous-compliance runs:
- Daily: SOC 2 CC4.x + CC7.x; ISO A.8.15 + A.8.16.
- Weekly: CC8.x; A.5.27.
- Monthly: CC3.x; A.5.7.
- Quarterly: full matrix re-validation.
- Annually: external auditor re-attestation.

### Audit evidence delivery

External auditors receive frozen evidence pack per `docs/templates/evidence-pack-template.md`; JIT token (per `policy/auditor-scope.cedar`); engagement window bounded; every read audit-chain-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 + ISO 27001:2022 audits recorded in `evidence/audits/`.
- Per-pack audit cadences per local law.

## References

- `microservices/foundry-runtime/threat-model.md`; `dpia.md`; `policy/*.md`; `policy/*.cedar`; `incident-response.md`.
- ADR-0022; ADR-0024; ADR-0025; ADR-0028; ADR-0117; ADR-0123; ADR-0130; ADR-0131; ADR-0140.
- SOC 2 TSC 2017 + 2022 PoF — `aicpa.org`.
- ISO/IEC 27001:2022 + 27002:2022 — `iso.org`.
- GDPR + EDPB — `gdpr-info.eu` + `edpb.europa.eu`.
- EU AI Act (Regulation 2024/1689) — `eur-lex.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`; KR FSC AI Guideline 2024.
- HIPAA — `hhs.gov/hipaa`; FDA SaMD — `fda.gov/medical-devices/software-medical-device-samd`.
- APPI — `ppc.go.jp`; METI AI Governance 2024.
- PDPA + MAS FEAT — `pdpc.gov.sg` + `mas.gov.sg`.
- Privacy Act 1988 — `oaic.gov.au`; AHRC AI guidance.
- DPDPA 2023 + MeitY AI Advisory 2024 — `meity.gov.in`.
- LGPD + ANPD AI — `gov.br/anpd`.
- UAE PDPL + Charter for Responsible AI — `mohre.gov.ae`.
- KSA PDPL + SDAIA Generative AI — `sdaia.gov.sa`; SAMA — `sama.gov.sa`.
