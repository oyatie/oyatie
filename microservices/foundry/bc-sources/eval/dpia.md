---
doc_class: DPIA
template_id: TPL-DPIA
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry
deciders: council-privacy, ops-security, axis-foundry, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act Art. 27 (fundamental rights impact assessment for high-risk AI)
related_adrs: [ADR-0024, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/foundry-eval/threat-model.md
  - microservices/foundry-eval/policy/tenant-isolation.md
  - microservices/foundry-eval/policy/data-residency.md
  - microservices/foundry-eval/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processors, or model-routing decisions
high_risk_triggers_engaged:
  - "GDPR Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (eval substrate evaluates capability + provider routing, which determines tenant-facing AI behavior)"
  - "GDPR Art. 35(3)(b): large-scale processing of special-category data — YES (pack-us-healthcare PHI via synthetic; pack-kr PIPA Art. 23 sensitive; replay traces may contain unredacted PII from source µservices)"
  - "GDPR Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act Art. 27 (FRIA for high-risk AI): YES — foundry-eval directly informs model-routing for high-risk AI use cases"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "EU AI Act Arts. 9, 10, 15, 17, 27"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII)"
  - "SOC 2 Privacy criteria (P1-P8, 2017 TSC)"
  - "NIST AI 100-1 (AI Risk Management Framework)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514 (de-identification)"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019", "EU AI Act Art. 27 (FRIA)", "EDPB Guidelines 9/2022"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1, 5, 6, 11, 12"]
  pack-in: ["DPDPA 2023 §10 + §11"]
  pack-br: ["LGPD Arts. 6, 7, 11, 38 (RIPD)"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9", "SAMA Cybersecurity Framework"]
doc_status: published
---

# Data Protection Impact Assessment: foundry-eval µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. EU AI Act Art. 27 separately requires a **Fundamental Rights Impact Assessment (FRIA)** for high-risk AI use cases. The foundry-eval µservice triggers both:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| GDPR Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Eval-runner produces per-capability verdicts that directly inform model-routing decisions affecting tenant-end-user-facing AI behaviour |
| GDPR Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional)** | Pack-us-healthcare PHI via synthetic fixtures; pack-kr PIPA Art. 23 sensitive; replay traces may carry unredacted PII from source µservices |
| GDPR Art. 35(3)(c): Public-area monitoring | NO | foundry-eval does not monitor public-area data |
| EU AI Act Art. 27 FRIA | **YES** | foundry-eval is part of the high-risk AI pipeline (it gates model decisions); deployer-level FRIA mandatory |
| KR PIPC Notice 2020-7 (mandatory DPIA criteria) | **YES** | System processes PIPA Art. 23 sensitive data at scale |

A DPIA + FRIA combined document is mandatory pre-deployment. This document is the canonical artefact reviewed by EU DPAs (Art. 35), the EU AI Office (Art. 27), the Korean PIPC (PIPA Art. 33), and equivalent authorities at first-tenant onboarding.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** foundry-eval ingests eval-set manifests (per-capability YAML + Cosign sig) and replay-trace samples (from production µservice OTel emission); executes per-case dispatches against provider model APIs or in-house variants; computes per-cohort aggregates + parity-deltas; emits eligibility verdicts that gate capability publish + routing-preference + in-house-cutover; shreds per-subject DEKs on DSR.

**How:** Capability-owner authors `eval-sets/<capability>/v<n>.evalset.yaml` + Cosign signs → registry validates → eval-runner-worker dispatches Kubernetes Jobs to GPU pool → per-case pod invokes provider model API → result stored in S3 (encrypted) → aggregate written to ClickHouse + Postgres → verdict emitted to foundry-evidence + foundry-runtime + foundry-providers.

**Where:** Per-pack region-pinned dedicated foundry-eval clusters (pack-kr → KR OCI ap-seoul-1; pack-eu → EU eu-frankfurt-1; pack-us → US us-ashburn-1; pack-us-healthcare → HIPAA-eligible US region). Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; nightly cadence per capability; on-demand publish-gate runs; on-demand A/B routing-preference runs; on-demand model-upgrade replay runs.

**Who:** Per actor table in `microservices/foundry-eval/threat-model.md` §"Actors".

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6 + Art. 9) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-capability invocation counts; per-case latency / error / pass-rate | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest (operational) | ~10⁶ eval-cases/day per capability |
| `PII_IDENTIFYING` | User-id fields in source-µservice trace spans flowing into replay store | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation (audit) | varies; targeted to 0 via redactor |
| `PII_QUASI_IDENTIFIER` | URLs / IPs in span attributes from source µservices | Art. 6(1)(f) + minimisation | varies |
| `SENSITIVE_PIPA_ART23` | Hashed tenant-id when correlated with auxiliary data | KR PIPA Art. 15 + 23 + 23-2 (sensitive PI with explicit consent) | 1 per tenant request |
| `PHI` (pack-us-healthcare) | Patient identifiers in source-µservice traces (if redactor missed); synthetic-PHI in eval-set fixtures | HIPAA §164.502(a) Permitted Uses per BAA; never live PHI in eval-sets by policy `policy/synthetic-phi-only.md` | targeted to 0 live; varies in replay |
| `AUDIT` | EvalRun / Parity / Divergence / Cutover events | Art. 6(1)(c) legal obligation + Art. 6(1)(f) | 1 per evaluation |
| `SECRET` | Provider API keys; ClickHouse pwd; Cosign signing keys; per-subject DEKs; per-tenant KEKs | not personal data; ISO 27001 A.5.17 controls | varies |

**Geographical scope:** Per pack:
- pack-kr: KR (ap-seoul-1) — KR tenant data + eval-set baselines stay in KR.
- pack-eu: EU (eu-frankfurt-1) — EU tenant data stays in EU.
- pack-us / pack-us-healthcare: US (us-ashburn-1) — US data stays in US; HIPAA pack pinned to BAA-eligible region.
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned to primary region.

**Cross-border transfer:** Forbidden by default per `multi-region.md`. Allowed only with tenant-executed SCCs for GDPR-scope tenants per Arts. 44-46; recorded in `microservices/foundry-eval/legal/transfer-register.md`.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (the tenant's customers); tenant operators (administrative users); capability owners (oyatie engineers); oyatie operators. Mapping in §3.
- **Relationship to data subjects:** Joint controllership with the tenant under GDPR Art. 26 for the operational telemetry portion. Joint-controllership terms recorded in tenant DPA template (`microservices/foundry-eval/legal/dpa-template.md`).
- **Reasonable expectations:** Tenant operators expect operational eval of capabilities they consume (per service contract). End-users expect data-class-respecting processing per tenant privacy notice; oyatie discloses via joint-controllership clause.
- **Industry codes:** Voluntary alignment with OpenTelemetry semantic conventions + EU AI Act high-risk AI obligations + NIST AI RMF + OWASP LLM Top 10.

### 2.4 Purpose of the processing

| Purpose | Lawful basis | Necessity |
|---|---|---|
| Capability publish gating | Art. 6(1)(b) + Art. 6(1)(f) | Necessary to prevent tenant exposure to untested capabilities |
| Nightly drift detection | Art. 6(1)(f) | Necessary to detect provider-side drift impacting tenant outcomes |
| A/B routing-preference evidence | Art. 6(1)(f) | Necessary to defend routing decisions under cost / quality pressure |
| In-house cutover decision | Art. 6(1)(f) | Necessary for ADR-0026 cutover; reduces vendor lock-in |
| Regression detection via replay | Art. 6(1)(b) + Art. 6(1)(c) (audit) | Necessary for accuracy + robustness obligations under EU AI Act Art. 15 |
| EU AI Act §17 logging | Art. 6(1)(c) | Legal obligation |

## Step 3 — Consultation

| Consultation | Engaged | Status |
|---|---|---|
| Internal: council-privacy + ops-security + axis-foundry + council-architecture | YES | Sign-off pending |
| Tenant DPO (where applicable) | At onboarding | per-tenant DPA |
| External DPA (GDPR Art. 36 prior consultation) | Triggered only if residual risk = H after mitigation | Not currently triggered |
| EU AI Office (FRIA notification) | Per Art. 27 deployer obligation | At first high-risk AI deployment in EU |
| Korean PIPC | Notice when triggers PIPA Art. 33 | At pack-kr launch |
| Data subjects | via tenant's privacy notice (joint controllership) | tenant-mediated |

## Step 4 — Assess necessity + proportionality

| Principle | Demonstration |
|---|---|
| Lawful basis (Art. 6) | Each purpose linked to a lawful basis (§2.4) |
| Special-category processing (Art. 9) | PHI restricted to synthetic-only by policy `policy/synthetic-phi-only.md`; live-PHI excluded by construction |
| Data minimisation (Art. 5(1)(c)) | Per-case dispatch only consumes the eval-case input; broader tenant data never read by eval-runner |
| Storage limitation (Art. 5(1)(e)) | Replay traces 24mo max (6y pack-us-healthcare per HIPAA §164.316(b)(2)); per-subject DEKs shred on DSR within 30d |
| Accuracy (Art. 5(1)(d)) | Eval substrate IS the accuracy mechanism; Art. 5(1)(d) self-served |
| Integrity + confidentiality (Art. 5(1)(f)) | Cosign + KEK + DEK + audit-chain — threat-model.md mitigations cover |
| Transparency (Arts. 12-14) | Tenant DPA discloses; verdict history visible to tenant operators with appropriate scope |
| Necessity of profiling | Profiling here = automated capability-quality assessment, not natural-person profiling per Art. 22 → does not constitute automated decision-making affecting data subjects directly |

## Step 5 — Risk register

Cross-referenced with `threat-model.md` STRIDE / LINDDUN catalog. Format: ID; Description; Likelihood (L/M/H); Impact (L/M/H); Risk; Mitigation; Residual.

| ID | Description | L | I | Risk | Mitigation | Residual |
|---|---|---|---|---|---|---|
| R-01 | PII leakage into replay traces via source-µservice trace emission gap (T-I-02) | H | H | **H** | OTel redactor at source + secondary redactor at replay-ingress + per-subject DEK + DSR shred | M |
| R-02 | Cross-tenant query leak via ClickHouse misconfiguration (T-I-01) | M | H | **H** | ClickHouse RBAC + tenant_id partition + DP-noise on cross-tenant aggregates + LEAN check | L |
| R-03 | Eval-set tampering → false publish-gate pass (T-T-01) | L | H | **M** | Cosign + Rekor inclusion-proof + signed commits | L |
| R-04 | Baseline-output tampering at S3 (T-T-02) | L | H | **M** | SSE-KMS + Object Lock + per-object Cosign + monthly block validator | L |
| R-05 | Insider deletes eval-run history to hide regression (T-E-05) | L | H | **M** | Sealed partition + 2-person rule + JIT + audit-chain + 30d soft-delete window | L |
| R-06 | GPU sandbox escape (T-E-01) | L | H | **M** | gVisor / Kata + seccomp + AppArmor + NetworkPolicy + CIS Benchmark | L |
| R-07 | Capability owner forges Cosign signature (T-S-01) | L | H | **M** | Hardware-token binding + Rekor inclusion-proof + per-capability CODEOWNERS | L |
| R-08 | Eval-set training-data contamination (T-A-01) | M | M | **M** | contamination_check_run_at field + LEAN check + adversarial cohort design | M (industry baseline) |
| R-09 | LLM-as-judge bias (T-A-02) | M | M | **M** | Per-quarter judge rotation + κ ≥ 0.7 consistency | M (industry baseline) |
| R-10 | DSR cascade fails to shred within 30d SLA (T-L-07) | M | M | **M** | DSR cascade runner + dashboard SLO + soft-shred 7d window + per-subject DEK | M (retention-window-bounded) |
| R-11 | Cross-border misroute (pack-eu eval data flows to non-EU cluster) | L | H | **M** | Pack-pinning enforced at OTel emission + replay-engine ingest validates pack-tag | L |
| R-12 | Provider API key leak via eval-time error trace (T-I-06) | M | H | **H** | Per-case ephemeral pod + secret-scanner + Secret<T> shim + rotation 30d | M (human baseline) |
| R-13 | Replay determinism divergence > 100ms (operational risk) | M | M | **M** | Deterministic-seed cohort + monitoring + replay-engine-fail-closed | L |
| R-14 | EU AI Act §15 (accuracy) evidence-schema missing on eval-run emission | L | H | **M** | Schema regression test + emission validator + LEAN check | L |
| R-15 | Subject re-identification via auxiliary data on tenant-id hash (T-L-02) | L | M | **L-M** | Salt rotation 12mo + audit-chain notes | L |

## Step 6 — Mitigation measures

Per threat-model.md §"Mitigations Catalog". Each R-NN above maps to the catalogued mitigation:

| Risk ID | Mitigation ref | Status |
|---|---|---|
| R-01 | Threat-model T-I-02 mitigations | Implemented + LEAN-check |
| R-02 | T-I-01 mitigations | Implemented + LEAN-check |
| R-03..R-04 | T-T-01..T-T-02 | Implemented |
| R-05 | T-E-05 | Implemented + 2-person rule policy `policy/two-person-admin-ops.md` |
| R-06 | T-E-01 | Implemented |
| R-07 | T-S-01 | Implemented |
| R-08 | T-A-01 | Implemented + quarterly contamination review |
| R-09 | T-A-02 | Implemented + quarterly κ check |
| R-10 | T-L-07 | DSR cascade runner in M01-P02 |
| R-11 | Pack-routing enforcement at OTel + replay-ingress | Implemented + LEAN-check `oya-check-pack-routing-conformance` |
| R-12 | T-I-06 | Implemented |
| R-13 | Threat-model T-D-03 + tests/load/replay-determinism.rs | Implemented + monitored |
| R-14 | EU AI Act §15 emission schema + schema regression test | Implemented |
| R-15 | T-L-02 | Implemented |

## Step 7 — Outcome + sign-off

### Residual risks accepted

Per threat-model.md "Residual Risk Acceptance" table. Residual M risks (R-01, R-08, R-09, R-10, R-12) accepted as quarterly-reviewable; residual L risks (all others) accepted as annually-reviewable. No residual H risks remain post-mitigation.

### Decisions

1. **Live PHI never enters eval-sets.** Eval-set baseline inputs and case prompts use synthetic-PHI fixtures only. Policy `policy/synthetic-phi-only.md` is BLOCKER on `microservices/foundry-eval/eval-sets/**`.
2. **Replay traces accept "post-redactor" data only.** Source-µservice OTel SDK redactor is the first line; replay-engine ingress runs a secondary redactor + dead-letters unannotated traces. Per-subject DEK envelope is the third line (DSR shred surface).
3. **EU AI Act §15 evidence on every emission.** Every EvalRun, ParityVerdict, ReplayDivergence, and InHouseCutoverEligible event carries the §15 (accuracy + robustness) + §17 (logging) evidence-schema fields by construction. Schema regression test BLOCKER.
4. **Cosign-with-Rekor-inclusion-proof BLOCKER on eval-set load.** Detached signature alone insufficient; Rekor public-log inclusion required.
5. **Cross-border replication forbidden by default.** Pack-pinning is BLOCKER; LEAN-check enforced.
6. **Per-subject DEK shred SLA 30d.** DSR cascade runner monitored; missing the SLA is a Sev-2 incident.
7. **Adversarial cohort quarterly rotation BLOCKER.** Stale cohorts (no rotation > 4mo) refuse capability re-publish.
8. **Judge model rotation quarterly + κ ≥ 0.7.** Inter-judge consistency monitored.

### Sign-off

- council-privacy: `pending`
- ops-security: `pending`
- council-architecture: `pending`
- axis-foundry: `pending`
- For pack-eu deployment: EU AI Office Art. 27 FRIA notification: `not yet triggered`
- For pack-kr deployment: PIPC PIPA Art. 33 notification: `not yet triggered`

## Step 8 — Review

This DPIA reviews annually + on:
- Any change to processing purpose, data classes, or sub-processor list.
- Any new pack activation.
- Any incident affecting personal data (Sev-1 / Sev-2).
- Any EU AI Act / GDPR / KR PIPA / HIPAA amendment material to processing.
- Any new adversarial cohort or judge model rotation.

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 33 영향평가 specifics: this DPIA's structure conforms to PIPC Notice 2020-7 §3 (체계적 평가).
- 신용정보법 §32 (when finance capability evaluated): eval data for credit-decisioning capabilities subject to additional FSS oversight; FSS audit-export channel via `legal/fss-export.md`.

### pack-us-healthcare

- HIPAA §164.308(a)(1)(ii)(A) risk analysis: this DPIA serves as the HIPAA risk analysis.
- Synthetic-PHI-only policy: per `policy/synthetic-phi-only.md`; HHS de-identification expert-determination required + recorded.
- BAA per-tenant: `legal/baa-template.md`.

### pack-eu

- GDPR Art. 35 + Art. 36 prior-consultation trigger: triggered only if residual H risk persists post-mitigation; currently not triggered.
- EU AI Act Art. 27 FRIA: this DPIA + threat-model + EU AI Act §15/§17 evidence emission combine into the deployer-level FRIA.
- Schrems-II compliance: EU eval data EU-resident; cross-border replication forbidden by default.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections at `regional-packs/<pack>/foundry-eval-dpia-overlay.md`.

## References

- `microservices/foundry-eval/threat-model.md`.
- `microservices/foundry-eval/compliance.md`.
- `microservices/foundry-eval/policy/synthetic-phi-only.md`.
- `microservices/foundry-eval/policy/tenant-isolation.md`.
- `microservices/foundry-eval/policy/data-residency.md`.
- `microservices/foundry-eval/policy/two-person-admin-ops.md`.
- ADR-0024, ADR-0026, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0140.
- ICO DPIA template — `ico.org.uk/.../data-protection-impact-assessments-dpias`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
- PIPC Notice 2020-7 (Korean DPIA methodology).
- EDPB Guidelines 4/2019 on Art. 25.
- EDPB Guidelines 9/2022 on personal data breach notification.
- EU AI Act (Regulation 2024/1689) Art. 27 (FRIA).
- NIST AI 100-1 (AI RMF).
