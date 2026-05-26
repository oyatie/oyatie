---
doc_class: ComplianceMapping
title: Compliance Cross-Mapping
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-foundry
deciders: council-privacy, ops-security, axis-foundry, council-architecture
related_adrs: [ADR-0024, ADR-0026, ADR-0028, ADR-0117, ADR-0131, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/intelligence-eval/threat-model.md
  - microservices/intelligence-eval/dpia.md
review_cadence: annually + on every framework amendment material to processing
doc_status: published
---

# Compliance Cross-Mapping (foundry-eval µservice)

## Purpose

Map each compliance-framework control to the foundry-eval artifact + mechanism that satisfies it. Auditor-grade traceability for SOC 2 Type 2, ISO 27001:2022, GDPR, EU AI Act, NIST AI RMF, OWASP LLM Top 10, HIPAA, KR PIPA, APPI, PDPA, APRA-CPS, DPDPA, LGPD, UAE PDPL, KSA PDPL, SAMA.

## SOC 2 Type 2 (2017 TSC + 2022 PoF)

| Control | Coverage | Artifact |
|---|---|---|
| CC1.x (control environment) | council sign-off; ADR governance | `dpia.md` §"Sign-off"; ADR-0024; ADR-0131 |
| CC2.x (communication) | runbooks; tenant DPA disclosures | `runbooks/*`; `legal/dpa-template.md` |
| CC3.x (risk assessment) | this threat-model + DPIA | `threat-model.md`; `dpia.md` |
| CC4.x (monitoring) | Mimir + ClickHouse + audit-chain | `dashboards/*`; foundry-evidence |
| CC5.x (control activities) | LEAN lanes; per-microservice-layout | per ADR-0131 lane suite |
| CC6.1 (logical access) | OIDC + Cedar + ServiceAccount RBAC | `policy/tenant-scope.cedar`; `policy/ci-scope.cedar` |
| CC6.2 (authorization) | per-tenant scope binding | `policy/tenant-scope.cedar` |
| CC6.3 (privileged access) | JIT via OpenBao | `policy/two-person-admin-ops.md` |
| CC6.6 (encrypted transmission) | mTLS + SPIFFE on internal paths | service mesh config |
| CC6.7 (data classification) | data_class annotations + LEAN check | `oya-check-data-class` lane |
| CC7.1 (system operations) | runbooks + on-call | `runbooks/*`; `incident-response.md` |
| CC7.2 (incident response) | `incident-response.md` | `incident-response.md` |
| CC7.4 (anomaly detection) | dashboards + Mimir alerts | `dashboards/*` |
| CC8.1 (change management) | PR + ChangeSet + per-changeset evidence | ADR-0110; `evidence/multispectrum/` |
| CC9.1 (vendor risk management) | provider model API contracts | `legal/provider-contracts.md` |

## ISO 27001:2022 (Annex A)

| Control | Coverage | Artifact |
|---|---|---|
| A.5.7 (threat intelligence) | quarterly adversarial cohort refresh | `policy/adversarial-cohort-freshness.md` (referenced in threat-model.md) |
| A.5.10 (acceptable use) | capability-owner role definition | `policy/tenant-scope.cedar` |
| A.5.14 (information transfer) | data-residency + cross-border SCC | `policy/data-residency.md` |
| A.5.15 (access control) | Cedar policy framework | ADR-0140; `policy/*.cedar` |
| A.5.17 (authentication) | OIDC + MFA + Cosign + Ed25519 | OIDC + Cosign + audit-chain |
| A.5.23 (cloud services security) | OCI native services hardened | `iac/` + CIS Benchmark |
| A.5.26 (incident response planning) | `incident-response.md` | `incident-response.md` |
| A.5.27 (lessons learned) | postmortem culture per Google SRE | `incident-response.md` §"Postmortem" |
| A.5.30 (ICT readiness) | DR + multi-region | `multi-region.md` |
| A.5.31 (legal/statutory) | per-pack overlay | `policy/data-residency.md` per-pack |
| A.5.32 (intellectual property) | Cosign + repo-level CODEOWNERS | branch protection |
| A.5.33 (records protection) | append-only audit-chain | foundry-evidence |
| A.5.34 (PII protection) | DPIA + per-subject DEK + DSR | `dpia.md`; `runbooks/` |
| A.8.2 (privileged access) | JIT via OpenBao | `policy/two-person-admin-ops.md` |
| A.8.3 (information access restriction) | tenant-isolation invariants | `policy/tenant-isolation.md` |
| A.8.5 (secure authentication) | OIDC + mTLS + Cosign | OIDC + Cosign |
| A.8.7 (protection against malware) | gVisor / Kata sandbox + image scanning | `iac/helm/gpu-runner-pool/` |
| A.8.11 (data masking) | OTel redactor + Secret<T> shim | secondary redactor on replay-engine ingress |
| A.8.12 (data leakage prevention) | secret-scanner + DEK envelope | `oya-governance-evidence-secret-scan` |
| A.8.15 (logging) | EU AI Act §17 + audit-chain | foundry-evidence; §17 evidence emission |
| A.8.16 (monitoring activities) | Mimir + Grafana + Alertmanager | `dashboards/*` |
| A.8.20 (network security) | Kubernetes NetworkPolicy + Istio mesh | `iac/helm/` |
| A.8.21 (network services security) | Envoy + WAF | ingress config |
| A.8.23 (web filtering) | egress allowlist per case pod | NetworkPolicy |
| A.8.25 (secure development lifecycle) | ChangeSet + LEAN + per-changeset evidence | ADR-0110 |
| A.8.26 (application security requirements) | Cedar + OWASP LLM Top 10 | `policy/*.cedar` |
| A.8.27 (secure system architecture) | per-microservice-layout + clean-arch | ADR-0131; ADR-0056 |
| A.8.28 (secure coding) | clippy + cargo-deny + LEAN | CI lanes |

## GDPR (Articles cited inline in threat-model + DPIA)

| Article | Coverage | Artifact |
|---|---|---|
| Art. 5 (principles) | DPIA §"necessity + proportionality" | `dpia.md` Step 4 |
| Art. 6 (lawful basis) | per-purpose mapping | `dpia.md` §2.4 |
| Art. 9 (special category) | synthetic-PHI-only; per-subject DEK envelope | `policy/synthetic-phi-only.md` |
| Art. 13/14 (information) | tenant DPA disclosures | `legal/dpa-template.md` |
| Art. 17 (right-to-erasure) | DSR cascade + per-subject DEK shred | `runbooks/` + ADR-0024 §"Resolved 1" |
| Art. 22 (automated decisions) | profiling is capability-quality, not natural-person profiling | DPIA §"Necessity of profiling" |
| Art. 25 (privacy-by-design) | TOMs catalogued | `threat-model.md` Mitigations Catalog |
| Art. 28 (processor) | per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | audit-chain | foundry-evidence |
| Art. 32 (security of processing) | every mitigation | threat-model.md |
| Art. 33 (breach notification) | `incident-response.md` §"Sev-1 breach" | `incident-response.md` |
| Art. 35 (DPIA) | this DPIA | `dpia.md` |
| Art. 44–50 (transfers) | per-pack region pinning + SCC | `policy/data-residency.md` |

## EU AI Act (Regulation 2024/1689)

| Article | Coverage | Artifact |
|---|---|---|
| Art. 9 (risk management for high-risk AI) | threat-model + dpia | `threat-model.md`; `dpia.md` |
| Art. 10 (data governance) | eval-set authoring; contamination check; synthetic-PHI | `policy/synthetic-phi-only.md`; `oya-check-eval-set-contamination` |
| Art. 14 (human oversight) | publish-gate is human-approved (PR review of eval-set) + automated gate is reversible | branch protection + reverse-cutover |
| Art. 15 (accuracy + robustness + cybersecurity) | every EvalRun emission carries §15 evidence | OpenAPI §EuAiActSection15Evidence; eval-runner emission schema |
| Art. 17 (logging) | every event audit-chain-emitted; 24mo retention (6y for high-risk per Art. 17(2)) | foundry-evidence; cost-budget.md retention |
| Art. 27 (FRIA) | this DPIA serves as deployer FRIA | `dpia.md` Step 1 |

## NIST AI RMF (AI 100-1)

| Function | Coverage | Artifact |
|---|---|---|
| Govern | council-architecture + axis-foundry sign-off | ADR-0024; DPIA sign-off |
| Map (1.x: legal context; 2.x: TEVV) | DPIA §2; threat-model §"Scope" + §"Trust Boundaries" | `dpia.md`; `threat-model.md` |
| Map-1.2 (contamination assessment) | contamination check + cohort design | T-A-01 mitigation |
| Measure-2.7 (adversarial-cohort sustainability) | quarterly cohort refresh | T-A-03 mitigation |
| Measure-2.10 (judge bias) | judge rotation + κ check | T-A-02 mitigation |
| Manage (incident + change) | runbooks + incident-response | `runbooks/*`; `incident-response.md` |

## OWASP LLM Top 10 (2024)

| Risk | Coverage | Artifact |
|---|---|---|
| LLM01: Prompt Injection | adversarial cohort: prompt-injection sub-cohort | ADR-0024 §"Adversarial cohort"; threat-model T-I-07 |
| LLM02: Insecure Output Handling | per-case egress allowlist + sandbox | threat-model T-E-01 |
| LLM03: Training Data Poisoning | eval-set contamination check | T-A-01 |
| LLM04: Denial of Service | per-capability-owner rate limit + GPU pool autoscale | T-D-01 |
| LLM05: Supply Chain | Cosign + Rekor + cargo-deny | T-S-01; T-T-01 |
| LLM06: Sensitive Information Disclosure | per-subject DEK + DP-noise + Cedar | T-I-01; T-I-02; T-I-03 |
| LLM07: Insecure Plugin Design | foundry-runtime sandbox (cross-cuts to foundry-runtime) | upstream µservice |
| LLM08: Excessive Agency | autonomy-ceiling T2 enforced | `capabilities/*.yaml` autonomy_level field |
| LLM09: Overreliance | reverse-cutover automated + parity verdicts published | T-S-03 mitigation |
| LLM10: Model Theft | sandbox + per-case ephemeral pod + provider contract | threat-model T-E-01 |

## HIPAA (pack-us-healthcare)

| Section | Coverage | Artifact |
|---|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | this DPIA | `dpia.md` |
| §164.308(a)(3)(ii)(C) (workforce clearance) | 2-person rule | `policy/two-person-admin-ops.md` |
| §164.308(a)(4)(ii)(B) (access authorization) | auditor JIT scope | T-I-05 mitigation; `policy/auditor-scope.cedar` |
| §164.310(d)(2)(i) (disposal) | per-subject DEK shred | T-S-04 mitigation |
| §164.312(a)(1) (access control) | per-tenant KEK + per-subject DEK | T-I-01 mitigation |
| §164.312(b) (audit controls) | audit-chain emission; 6y retention | cost-budget.md retention |
| §164.314 (organizational) | BAA per-tenant | `legal/baa-template.md` |
| §164.316(b)(2) (retention) | 6y for pack-us-healthcare | cost-budget.md |
| §164.502 (minimum necessary) | synthetic-PHI-only + redactor | `policy/synthetic-phi-only.md` |
| §164.514 (de-identification) | HHS expert-determination | `policy/synthetic-phi-only.md` §P-4 |

## KR PIPA (pack-kr)

| Article | Coverage | Artifact |
|---|---|---|
| Art. 3 (collection limitation) | data_class + minimisation | LEAN check |
| Art. 15 (consent) | tenant DPA | `legal/dpa-template.md` |
| Art. 17 (provision/transfer) | per-pack region pinning | `policy/data-residency.md` |
| Art. 18 (use/provision restriction) | Cedar tenant-scope | `policy/tenant-scope.cedar` |
| Art. 22-2 (cross-border) | SCC + KR PIPC notification | `policy/data-residency.md` |
| Art. 23 (sensitive PI) | per-subject DEK envelope + salt rotation | T-L-02; T-I-01 |
| Art. 24 (unique identification info) | hashed customer-id | T-L-02 |
| Art. 25 (personal video info) | N/A | – |
| Art. 28 (delegation) | per-tenant DPA | `legal/dpa-template.md` |
| Art. 29 (safeguards) | 12 prescribed safeguards covered | threat-model.md mitigations |
| Art. 29-2 (security measures notification) | breach notification | `incident-response.md` |
| Art. 33 (영향평가) | this DPIA | `dpia.md` |
| Art. 36 (rectification/erasure) | DSR cascade SLA 30d | T-L-07 |

## APPI (pack-jp)

| Article | Coverage | Artifact |
|---|---|---|
| Art. 17 (purpose of use) | tenant DPA | `legal/dpa-template.md` |
| Art. 18 (proper acquisition) | data_class annotation | LEAN check |
| Art. 20 (security management) | TOMs | threat-model.md |
| Art. 21 (cross-border notification) | pack-jp region pinning | `policy/data-residency.md` |
| Art. 23 (joint use) | joint controllership clause | DPIA §2.3 |
| Art. 24 (cross-border transfer) | SCC equivalent | `policy/data-residency.md` |
| Art. 27 (sensitive info consent) | synthetic-PHI-only | `policy/synthetic-phi-only.md` |

## PDPA (pack-sg)

| Section | Coverage | Artifact |
|---|---|---|
| Part III §11-26 (Protection Obligation) | TOMs | threat-model.md |
| Part IV (Retention) | data residency | `policy/data-residency.md` |
| MAS Notice 644 / MAS-TRM | financial-vertical overlay | per-pack overlay |

## APRA-CPS 234 (pack-au)

| Section | Coverage | Artifact |
|---|---|---|
| §29-44 (Information Security) | TOMs + threat-model | threat-model.md |
| Privacy Act 1988 APP 1-13 | DPIA | `dpia.md` |

## DPDPA 2023 (pack-in)

| Section | Coverage | Artifact |
|---|---|---|
| §6-10 (consent + notice + processing limits) | tenant DPA | `legal/dpa-template.md` |
| §10-11 (data fiduciary obligations + DPIA-equivalent) | this DPIA | `dpia.md` |

## LGPD (pack-br)

| Article | Coverage | Artifact |
|---|---|---|
| Art. 6, 7, 11 (principles + lawful basis + sensitive) | DPIA §"necessity" | `dpia.md` Step 4 |
| Art. 38 (RIPD) | this DPIA | `dpia.md` |
| BACEN Res. 4.893/2021 | financial overlay | per-pack overlay |

## UAE PDPL + KSA PDPL + SAMA

Pack-overlay sections cited inline in threat-model.md per-pack section.

## References

- `microservices/intelligence-eval/threat-model.md`.
- `microservices/intelligence-eval/dpia.md`.
- ADR-0024, ADR-0026, ADR-0028, ADR-0117, ADR-0131, ADR-0133, ADR-0140.
