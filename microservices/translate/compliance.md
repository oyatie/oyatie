---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security + axis-translate
deciders: council-privacy, ops-security, axis-translate, council-architecture, ops-compliance
related_adrs: [ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0133, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/threat-model.md
  - microservices/translate/dpia.md
  - microservices/translate/policy/credential-isolation.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/policy/ai-act-overlay.md
  - microservices/translate/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping — translate µservice

## Purpose

Canonical control-to-framework mapping for translate. Tells an external auditor which control implementation satisfies which clause, with pointers to evidence.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC; 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC6.1 | Logical access | OIDC + Cedar v4.2 + OpenBao JIT credentials | `policy/translate-tenant-scope.cedar`, `policy/credential-isolation.md` |
| CC6.2 | Authentication + auth | OIDC + SPIFFE adapter identity + 2-person rule on adapter publish | branch-protection.yaml CODEOWNERS |
| CC6.6 | Logical access control | Cedar default-deny + per-tenant RLS on Postgres + per-tenant Meilisearch index | `policy/*.cedar` |
| CC6.7 | Transmission + disposal | mTLS + ZDR negotiation + audit-chain Ed25519 envelope | adapter impl |
| CC6.8 | Vulnerability management | `cargo deny` + Sigstore attestation + adapter pin + CVE refresh on Pandoc + LibreOffice | release-automation lanes |
| CC7.1 | System operations | engine-health monitor + auto-failover + per-pack HA | `failure-modes.md`, `capacity-model.md` |
| CC7.2 | Monitoring system inputs | per-engine SLI + cost SLI + audit-chain emission | `dashboards/*.json`, observability integration |
| CC7.3 | Anomaly evaluation | response-shape validator + QE-score anomaly + burn-rate alerts | `dashboards/quality-and-tm-leverage.json` |
| CC7.4 | Incident response | severity-classified response + on-call rotation + 7 runbooks | `incident-response.md`, `runbooks/` |
| CC8.1 | Change management | PR review + LEAN gates + 2-person rule on TM/termbase commits | branch-protection.yaml |

### ISO/IEC 27001:2022

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Vendor breach-notification subscription + quarterly threat-model refresh | `threat-model.md` |
| A.5.14 | Information transfer | mTLS + ZDR + per-pack SCC + residency-bound | `policy/data-residency.md` |
| A.5.15 | Access control | OpenBao + Cedar | `policy/*.cedar` |
| A.5.17 | Authentication info | OpenBao credential isolation + zeroize-on-drop | `policy/credential-isolation.md` |
| A.5.23 | Information security for cloud services | Sub-processor list + per-vendor DPA | this doc §"Sub-processors" |
| A.5.26 | Incident response | `runbooks/` per-incident playbooks | runbooks |
| A.5.31 | Legal + statutory + regulatory | per-pack regulatory mapping table | `policy/data-residency.md` |
| A.5.32 | Intellectual property | Vendor terms compliance + tenant DPA | tenant onboarding |
| A.8.2 | Privileged access | 2-person rule + JIT elevation via OpenBao | `policy/credential-isolation.md` |
| A.8.3 | Information access restriction | Cedar + RLS + per-tenant index | `policy/*.cedar` |
| A.8.5 | Secure authentication | OIDC + mTLS + SPIFFE | adapter impl |
| A.8.11 | Data masking | Per-tenant DLP + PHI redaction (where vendor supports) | adapter impl |
| A.8.12 | Data leakage prevention | `oya-translate-credential-isolation` LEAN lane + zero-occurrence regex | LEAN |
| A.8.16 | Monitoring activities | SLI + audit-chain | observability |
| A.8.20 | Network security | Istio mTLS + egress proxy + pinned vendor CA | `cell` |
| A.8.23 | Web filtering | Egress proxy allowlist | `cell` |
| A.8.24 | Cryptography | Ed25519 + BLAKE3 + KMS keyring | adapter impl |
| A.8.25 | Secure development lifecycle | Per-µservice SDL + LEAN gates | `docs/standards/` |
| A.8.26 | Application security requirements | OWASP LLM Top 10 + ATLAS coverage | `threat-model.md` |
| A.8.27 | Secure system architecture | Per-ADR review | ADRs |

### GDPR (Reg. (EU) 2016/679; applicable when pack-eu activated)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5 | Lawfulness + minimisation | Per-vendor DPA + segment-level extraction + hash-only audit | adapter impl + DPA |
| Art. 6 | Lawful basis | Tenant contract (Art. 6(1)(b)) | tenant onboarding |
| Art. 9 | Special category data | Explicit opt-in for sensitive content classes + per-tenant DPA Art. 9 basis | `dpia.md` §1 |
| Art. 22 | Automated decision-making | translate does NOT make automated decisions with legal effect; QE is informational | ADR-TRANSLATE-0003 |
| Art. 25 | Data protection by design | OpenBao + Cedar + residency-bound + gVisor | `policy/credential-isolation.md` + `policy/data-residency.md` |
| Art. 28 | Processor obligations | Vendor sub-processor DPA + SCC | sub-processor list |
| Art. 30 | Records of processing | `TranslationCompleted` events as Art. 30 records | audit-chain |
| Art. 32 | Security of processing | mTLS + Ed25519 + Cedar + OpenBao + gVisor | full posture |
| Art. 33 | Breach notification | 72h notification per `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` | runbook |
| Art. 35 | DPIA | This µservice's DPIA | `dpia.md` |
| Arts. 44–50 | Cross-border transfers | SCC 2021/914 + supplementary measures + per-pack residency | `policy/data-residency.md` |

### EU AI Act (Reg. (EU) 2024/1689)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 9 | Risk management system | Per-vendor + per-engine risk register | `threat-model.md` + ADR-TRANSLATE-0003 |
| Art. 10 | Data governance + management | TM provenance + termbase governance | `policy/translate-tenant-scope.cedar` |
| Art. 11 | Technical documentation | This doc + ADRs | ADR-TRANSLATE-0003 |
| Art. 12 | Record-keeping (logging) | `TranslationCompleted` + `EngineRouted` + `EuAiActDisclosure` events | audit-chain |
| Art. 13 | Transparency + information to deployers | `EuAiActDisclosure` event per call (engine + model id + jurisdiction + system prompt hash + response hash) | `contracts/asyncapi/translate-events.yaml` |
| Art. 14 | Human oversight | Human-in-the-loop review for high-risk content classes via `workflow-engine` | `workflow-engine` integration |
| Art. 15 | Accuracy + robustness + cybersecurity | QE + response-shape validator + per-engine cybersecurity posture | adapter impl |
| Art. 27 | FRIA for deployers of high-risk AI | Per-tenant FRIA appended to DPIA if used for employment/credit/legal/medical contexts | `dpia.md` + ADR-TRANSLATE-0003 |
| Art. 50 | Transparency to natural persons | `EuAiActDisclosure` event + UI disclosure when tenant displays AI-generated translation | `policy/ai-act-overlay.md` |

### HIPAA (pack-us-healthcare)

| §  | Requirement | Implementation | Evidence |
|---|---|---|---|
| §164.308(a)(1) | Risk analysis | This threat-model + DPIA | `threat-model.md` + `dpia.md` |
| §164.308(a)(4) | Information access management | Cedar + OpenBao | policy fragments |
| §164.310 | Physical safeguards | Inherited from `cloud-k8s` | cloud-k8s |
| §164.312(a) | Access control + emergency | OpenBao JIT + revoke runbook | `runbooks/` |
| §164.312(b) | Audit controls | `TranslationCompleted` audit-chain | audit-chain |
| §164.312(c) | Integrity controls | BLAKE3 + Ed25519 envelope | adapter impl |
| §164.312(d) | Person/entity authentication | OIDC + SPIFFE | mesh |
| §164.312(e) | Transmission security | mTLS + ZDR + HIPAA-eligible region | adapter impl |
| §164.502(e) | BAA with vendors | per-vendor BAA executed pre-PHI | sub-processor list |

### Per-Pack Regulatory Mapping

| Pack | Frameworks | Notes |
|---|---|---|
| pack-kr | KR PIPA + ISMS-P + 전자문서법 + KR-FSS commercial code (5y retention) | per `policy/data-residency.md` |
| pack-eu | GDPR + EU AI Act + NIS2 + eIDAS | per `policy/data-residency.md` |
| pack-us | State-by-state PII laws + CCPA + (where TPO permits) | per `policy/data-residency.md` |
| pack-us-healthcare | HIPAA + HITECH + state PHI laws | per `policy/data-residency.md` |
| pack-jp | APPI Art. 24 cross-border | per `policy/data-residency.md` |
| pack-sg | PDPA + MAS-TRM | per `policy/data-residency.md` |
| pack-au | Privacy Act + APP 8 + APRA-CPS 234 | per `policy/data-residency.md` |
| pack-in | DPDPA 2023 §16 cross-border | per `policy/data-residency.md` |
| pack-br | LGPD Art. 33 cross-border + BACEN | per `policy/data-residency.md` |
| pack-ae | UAE PDPL | per `policy/data-residency.md` |
| pack-ksa | KSA PDPL + SAMA | per `policy/data-residency.md` |
| pack-cn-stub | CN Cybersecurity Law + DSL + PIPL Art. 38–43 | in-house ONLY; no external vendor; scaffolding only in M01 |

### eIDAS — signed translations (pack-eu optional)

For tenants requiring eIDAS-grade signed translation outputs (e.g., legal documents requiring qualified electronic signature), translate emits a QSeal envelope per Reg. (EU) 910/2014 Art. 35 alongside the standard Ed25519 envelope. Out-of-scope for M01 default; available behind feature flag.

### WCAG 2.2 AA — Accessibility

Translate API responses include accessibility metadata (language tag, plural-rule form, formality marker) so consuming UIs can render translated content per WCAG 2.2 AA (especially SC 3.1.1 language-of-page, SC 3.1.2 language-of-parts).

### Industry standards explicitly aligned

| Standard | Surface |
|---|---|
| OASIS XLIFF 2.1 | File import/export schema |
| LISA OSCAR TMX 1.4 | TM exchange schema |
| ISO 30042 (TBX) | Termbase schema |
| ICU MessageFormat | Placeholder preservation |
| CLDR plural rules | Plural-form handling |
| RFC 5646 BCP 47 | Language tag canonicalization |
| ISO 639-3 + ISO 639-5 | Language code resolution |
| WMT (BLEU + chrF + COMET) | Translation quality benchmarks (eval set) |
| ITU-T G.107 | Audio caption quality model (real-time stream) |
| NIST SSDF v1.1 | Secure development practices |
| SLSA L3 | Build provenance |
| OWASP ASVS v4.0 | Application security verification |
| CIS Kubernetes Benchmark | gVisor sandbox hardening |

## Sub-Processors (per pack)

| Sub-processor | Role | Packs | DPA / BAA |
|---|---|---|---|
| Anthropic | LLM-class MT for content classes requiring frontier capability | pack-kr (SCC + ZDR), pack-eu (SCC), pack-us, pack-us-healthcare (BAA + ZDR), pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| OpenAI | LLM-class MT (alternative) | pack-eu (post-SCC), pack-us, pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| Google (Cloud Translation API + AutoML) | Hosted MT + custom model | pack-kr (SCC), pack-eu, pack-us, pack-jp, pack-sg, pack-au | per-tenant DPA + (PHI) BAA |
| DeepL | Hosted MT (EU-grounded; high quality on EU pairs) | pack-eu (native), pack-kr (with PIPA Art. 28 consent), pack-jp | per-tenant DPA |
| Microsoft Translator | Hosted MT (alternative; tracked) | pack-eu, pack-us, pack-jp | per-tenant DPA + (PHI) BAA |
| Amazon Translate | Hosted MT (alternative; tracked) | pack-us, pack-eu | per-tenant DPA + (PHI) BAA |
| (oyatie in-house) | Self-served (foundry-runtime) | all packs | n/a (internal) |
| Oracle (OCI) | Infrastructure substrate | all packs | OCI DPA + per-region |

## Sub-Processor List Refresh

Per ADR-0028 audit-chain posture: any sub-processor change triggers:
1. Update this matrix.
2. Notify all affected tenants ≥ 30 days in advance per DPA.
3. Re-execute DPA addendum if needed.
4. Emit `SubProcessorChanged` event to audit-chain.

## Breach Notification

| Trigger | Notification clock | Audience | Owner |
|---|---|---|---|
| GDPR Art. 33 personal-data breach | 72 h from awareness | EU DPAs + affected subjects | council-privacy + DPO |
| KR PIPA breach (Art. 34) | 72 h to PIPC + affected subjects | PIPC + affected | council-privacy |
| HIPAA breach (PHI) | 60 days to affected; HHS notification | HHS + affected | council-privacy + ops-security |
| Vendor credential compromise (T-01) | per `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` | tenant + DPO + (if class crossed) regulators | ops-security |
| Cross-region leak (R-02 realisation) | per same runbook; Sev-1 (P0) | tenant + DPO + ALL relevant regulators | ops-security + council-privacy |

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance --microservice translate` exits 0.
- Annual SOC 2 audit; quarterly continuous-compliance evidence emission.
- Per-pack DPIA on file before first-tenant activation.

## References

- ADR-0028 audit-chain.
- ADR-0117 pack residency model.
- SOC 2 Type 2 TSC — AICPA.
- ISO/IEC 27001:2022.
- GDPR (Reg. (EU) 2016/679).
- EU AI Act (Reg. (EU) 2024/1689).
- HIPAA 45 CFR Part 160 + 164.
- KR PIPA + 전자문서법 + ISMS-P.
- APPI; PDPA SG; Privacy Act AU + APP; DPDPA 2023; LGPD; UAE PDPL; KSA PDPL.
- CN Cybersecurity Law + DSL + PIPL.
- eIDAS Reg. (EU) 910/2014.
- OASIS XLIFF / LISA TMX / ISO TBX.
- ICU MessageFormat / CLDR / RFC 5646 / ISO 639-3 + 639-5.
- WMT benchmarks; COMET; ITU-T G.107.
- WCAG 2.2 AA.
- NIST SSDF v1.1; SLSA L3; OWASP ASVS v4.0; CIS K8s.
