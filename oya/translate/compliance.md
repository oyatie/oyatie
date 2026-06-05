---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security + axis-translate
deciders: council-privacy, ops-security, axis-translate, council-architecture, ops-compliance
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0133, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004]
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

- `buck2 build //:quality-lane-registry-authority-check # lane=compliance --microservice translate` exits 0.
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

---



## §day-one-cert-readiness
This anchor is closed for `translate` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `translate` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +13 more.
- Example: `translate-segment-suggest-and-lang-hint` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `translate-segment-suggest-and-lang-hint` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §minor-protection
This anchor is closed for `translate` against ADR-0292 §D-1: minor-user refusal, teen policy class and age-verification handling.

### Service-specific answer
- Minor exposure for `translate` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `translate-segment-suggest-and-lang-hint` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-policy-class activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `translate` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`, `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `translate-segment-suggest-and-lang-hint` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.translate.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `translate` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `translate-segment-suggest-and-lang-hint` touches those data classes.
- Signal sources: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`; +12 more.
- Example event class: `oya.translate.translate.segment.suggest.and.lang.hint.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §ml-model-lifecycle
This anchor is closed for `translate` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `True` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `translate.translate-segment-suggest-and-lang-hint` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `translate-segment-suggest-and-lang-hint` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-fairness-audit
This anchor is closed for `translate` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `translate` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `translate` never stores protected attributes solely to make a product feature easier.
- Example: `translate-segment-suggest-and-lang-hint` abuse/risk score challenge rate is compared across locale, accessibility profile, age policy class, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `translate` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.translate.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `translate-segment-suggest-and-lang-hint` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `translate-segment-suggest-and-lang-hint` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `translate` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`, `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`; +5 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `translate.bulk_translate` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `translate` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`; +12 more.
- Example: `translate-segment-suggest-and-lang-hint` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.translate` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/translate/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.
- Example: `translate-segment-suggest-and-lang-hint` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `translate` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`, `microservices/translate/iac/helm/translate/Chart.yaml`, `microservices/translate/iac/helm/translate/templates/deployment.yaml`, `microservices/translate/iac/helm/translate/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `translate-segment-suggest-and-lang-hint` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `translate` is in annual full-scope pentest and every major `translate-segment-suggest-and-lang-hint` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`, `microservices/translate/iac/helm/translate/Chart.yaml`, `microservices/translate/iac/helm/translate/templates/deployment.yaml`, `microservices/translate/iac/helm/translate/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `translate` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `translate` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `translate-segment-suggest-and-lang-hint` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `translate` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/translate/catalog/oya-translate-bulk-worker.yaml`, `microservices/translate/catalog/oya-translate-doc-adapter-libreoffice.yaml`, `microservices/translate/catalog/oya-translate-doc-adapter-pandoc.yaml`, `microservices/translate/catalog/oya-translate-langdetect-adapter-foundry-runtime.yaml`, `microservices/translate/catalog/oya-translate-qe-adapter-foundry-runtime.yaml`, `microservices/translate/catalog/oya-translate-qe-kernel.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `translate-segment-suggest-and-lang-hint` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `translate` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `translate-segment-suggest-and-lang-hint` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `translate-segment-suggest-and-lang-hint` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `translate` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `translate-segment-suggest-and-lang-hint` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `translate`; owner `axis-translate`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `bulk-translate`, `document-localization`, `language-detection`, `quality-estimation`, `real-time-stream`, `termbase-and-glossary`; +2 more.
- Capability records cited: `microservices/translate/capabilities/T0-suggest.yaml`, `microservices/translate/capabilities/T1-assist.yaml`, `microservices/translate/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar/policy artifacts cited: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/translate/contracts/asyncapi/translate-events.yaml`, `microservices/translate/contracts/openapi/translate.yaml`, `microservices/translate/contracts/proto/translate.proto`.
- Cedar binding: `microservices/translate/policy/ai-act-overlay.md`, `microservices/translate/policy/auditor-scope.cedar`, `microservices/translate/policy/ci-scope.cedar`, `microservices/translate/policy/data-residency.md`, `microservices/translate/policy/public-read.cedar`, `microservices/translate/policy/translate-tenant-scope.cedar`.
- State/event binding: `translate.bulk_translate`, `translate.document_localization`, `translate.language_detection`, `translate.quality_estimation`, `translate.real_time_stream`, `translate.termbase_and_glossary`; +2 more.
- Capability binding: `translate-segment-suggest-and-lang-hint`, `translate-assist-suggest-qe-and-langdetect`, `translate-auto-translate-content-class`.
- SLO binding: `microservices/translate/slos/batch-translate-latency.openslo.yaml`, `microservices/translate/slos/data-residency-correctness.openslo.yaml`, `microservices/translate/slos/document-translate-latency.openslo.yaml`, `microservices/translate/slos/language-detection-latency.openslo.yaml`, `microservices/translate/slos/mt-engine-availability.openslo.yaml`, `microservices/translate/slos/qe-score-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/translate/runbooks/document-round-trip-corruption.md`, `microservices/translate/runbooks/glossary-conflict-resolution.md`, `microservices/translate/runbooks/mt-engine-degraded-shed.md`, `microservices/translate/runbooks/quality-estimation-rollback.md`, `microservices/translate/runbooks/real-time-caption-stream-stall.md`, `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `translate`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `translate`.
- `policy-engine` supplies the signed Cedar corpus while `translate` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `translate` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `translate`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `translate` applies the most restrictive policy and emits a degraded-mode audit event.
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

