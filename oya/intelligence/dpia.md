---
doc_class: DPIA
template_id: TPL-DPIA
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: council-privacy + axis-intelligence
deciders: council-privacy, ops-security, axis-intelligence, council-architecture, ops-legal
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + EU AI Act Art. 27 (FRIA) + KR PIPA Art. 33 (개인정보영향평가)
related_adrs: [ADR-0255, ADR-0255-amendment-library-first, ADR-0263, ADR-0296, ADR-0250]
related_specs: [/specs/intelligence-two-layer-substrate.json]
related_artifacts:
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/policy/eu-ai-act-high-risk.cedar
  - microservices/intelligence/policy/refusal-baseline.cedar
  - microservices/intelligence/compliance.md
review_cadence: annually + on every BC promotion + on every provider-adapter add + on every Annex III category amendment
high_risk_triggers_engaged:
  - "GDPR Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (every dispatch is profiling-by-output)"
  - "GDPR Art. 35(3)(b): large-scale processing of special-category data — YES (PHI in pack-us-healthcare via BAA; sensitive data under KR PIPA Art. 23)"
  - "GDPR Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act Art. 27: Fundamental Rights Impact Assessment (FRIA) required for Annex III deployers — YES (the substrate enables Annex III dispatch on tenant request; FRIA inherited from tenant)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "EU AI Act 2024/1689 Arts. 9, 10, 12, 13, 14, 15, 16, 27 + Annex III"
  - "ISO/IEC 42001:2023 (AI Management System)"
  - "SOC 2 Privacy criteria P1..P8 (2017 TSC)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33 (영향평가)", "PIPA Enforcement Decree Art. 35 (DPIA mandatory)", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) risk analysis", "§164.312(b) audit controls", "§164.502(b) minimum necessary", "FDA SaMD"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EU AI Act Art. 27 (FRIA)", "EDPB Guidelines 4/2019, 1/2024"]
  pack-jp: ["APPI Arts. 17, 18, 27", "METI AI Governance Guidelines"]
  pack-sg: ["PDPA + MAS AI Risk", "Model AI Governance Framework"]
  pack-au: ["Privacy Act 1988 APP 1+5+6+11+12", "AHRC Guidance on AI"]
  pack-in: ["DPDPA 2023 §10 + §11", "MeitY AI Advisory"]
  pack-br: ["LGPD Arts. 6+7+11+38 (RIPD)", "ANPD Sandbox"]
  pack-ae: ["UAE PDPL Art. 23", "UAE AI National Strategy"]
  pack-ksa: ["PDPL Art. 9", "NCAI National Strategy"]
  pack-cn: ["CN PIPL Art. 55", "CN Generative AI Service Provisions 2023"]
  pack-uk: ["UK GDPR + DPA 2018", "AI Regulation White Paper 2023", "ICO AI Guidance"]
doc_status: published
---

# Data Protection Impact Assessment: intelligence µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the
rights and freedoms of natural persons**. The intelligence µservice triggers two of the three
Art. 35(3) automatic triggers and the EU AI Act Art. 27 FRIA trigger:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Every dispatch is profiling-by-output; the model generates text/image/audio/video on the basis of the user's input, which constitutes profiling per GDPR Art. 4(4). |
| Art. 35(3)(b): Large-scale processing of special-category data (Art. 9) | **YES (conditional)** | Pack-us-healthcare carries PHI; pack-kr classes hashed-tenant-id with auxiliary as sensitive under PIPA Art. 23; consumer brand surface routinely processes user-supplied content that may include sensitive data. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | intelligence does not monitor public-area cameras / IoT. |
| EU AI Act Art. 27: FRIA required for Annex III deployer | **YES** | The substrate enables Annex III dispatch (Categories 1–7) on tenant request; FRIA inherited from tenant; substrate-level FRIA documented here. |

In addition, the Korean PIPC's Notice 2020-7 mandates a DPIA when processing system handles
sensitive personal information (PIPA Art. 23) at scale — engaged.

Therefore: a DPIA + FRIA is mandatory pre-deployment. This document is the canonical DPIA + FRIA
reviewed by EU DPAs (per Art. 35), the EU AI Office (per AI Act Art. 27), the Korean PIPC (per
PIPA Art. 33), and equivalent supervisory authorities in every active pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** `intelligence` dispatches every model call from the oyatie platform (consumer + developer
+ Foundry agent) to one of 16 first-class providers; applies a refusal-baseline + EU AI Act
Annex III refusal layer; evaluates output quality; renders citation attribution; emits the
consumer brand UX surface; resolves provider credentials per provider-credential BYOK tenant config (ADR-0255 §D-4); emits an audit tap onto
the audit-chain seal stream.

**How:**
- In-process dispatch (default): caller links `oya-intelligence-dispatch-sdk-{rs,ts,py,swift,kotlin}`;
  invokes `dispatch(envelope)`; pipeline runs in-process through routing → credential resolve →
  provider adapter → guardrails filter → attribution render → audit-tap emit.
- Network dispatch (opt-in fallback): caller invokes REST `/v1/dispatch` or gRPC
  `Dispatch.Issue`; same pipeline from kernel onward.

**Where:** Tenant-cell-eligible Kubernetes clusters per ADR-0254 (Cloud Hypervisor + Kata pods),
per-pack region-pinned. Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; per-dispatch synchronous (streaming response) + asynchronous audit-tap
emission.

**Who:** Per the actor table in `microservices/intelligence/threat-model.md` §"Actors".

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant dispatch envelope (audience, modality, provider, cost, latency) | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁶ dispatches/day per medium tenant |
| `PII_IDENTIFYING` | User-id fields in prompts when callers include them | Art. 6(1)(b) contract; Art. 6(1)(a) consent when consumer | varies by tenant emission |
| `PII_QUASI_IDENTIFIER` | URLs / IPs / span attributes embedded in prompts | Art. 6(1)(f) legitimate interest; minimised at SDK level | ~10⁵ /day per medium tenant |
| `PII_SENSITIVE` (Art. 9) | Health / religion / political opinion in user-supplied prompts | Art. 9(2)(a) explicit consent OR refusal | varies; refusal-baseline minimises |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id correlated with auxiliary; biometric markers in image prompts | KR PIPA Art. 15 + 23 + 23-2 explicit consent OR refusal | 1 per tenant request |
| `PHI` (pack-us-healthcare only) | Patient identifiers / clinical data in prompts | HIPAA §164.502(a) Permitted Uses (TPO); BAA required | varies; targeted to refusal unless BAA + BAA-signed provider |
| `FINANCIAL` | Per-call cost record (provider $/M-tokens × tokens used) | Art. 6(1)(b) contract; KR commercial code 5y retention | 1 per dispatch |
| `AUDIT` | Audit-tap Ed25519-sealed records | Art. 6(1)(c) legal obligation; EU AI Act Art. 12; Art. 6(1)(f) | 1 per dispatch |
| `SECRET` | Provider credentials (`SecretReference`) | not personal data; ISO 27001 A.5.17 controls | per tenant |

**Geographical scope:**

| Pack | Region | Provider routing |
|---|---|---|
| pack-kr | KR (OCI ap-seoul-1) | Anthropic EU (with KR egress); OpenAI EU; vLLM KR-resident; Naver HyperCLOVA-X (planned) |
| pack-eu | EU (OCI eu-frankfurt-1 + eu-amsterdam-1) | Anthropic EU; OpenAI EU; Vertex AI EU; Azure OpenAI EU; Mistral La Plateforme (EU-native) |
| pack-us / pack-us-healthcare | US (OCI us-ashburn-1 + us-phoenix-1) | All US providers; BAA-signed providers only for pack-us-healthcare |
| pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | Pinned to primary region | per-pack overlay |
| pack-cn | CN | Alibaba Qwen / Tencent Hunyuan / Baidu ERNIE only |

**Cross-border transfer:** Forbidden by default. Allowed only with tenant-executed SCCs per GDPR
Arts. 44–46 + Schrems-II supplementary measures (recorded in
`microservices/intelligence/legal/transfer-register.md`).

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (consumer / developer / Foundry agent invocation
  context); tenant operators; oyatie operators (internal).
- **Relationship:** Joint controllership under GDPR Art. 26 (tenant = controller of its end-users'
  data; oyatie = joint controller for the operational + audit telemetry portion). Terms in tenant
  DPA template (`microservices/intelligence/legal/dpa-template.md`).
- **Reasonable expectations:** Brand-ux-surface SparkleIcon disclosure + per-pack consent flows
  ensure data subjects understand AI processing. Consumer-tier callers see explicit cost-floor
  disclosure when platform-default cost float applies.
- **Previous experience:** Bominal observability substrate (data classes identical) operated under
  same processing pattern with no DPA-triggered complaints in 24 months.

## Step 3 — Consultation

| Stakeholder | When | Outcome |
|---|---|---|
| Council-privacy | Pre-publication | Approved |
| Ops-security | Pre-publication | Approved |
| Axis-intelligence | Pre-publication | Approved |
| Council-architecture | Pre-publication | Approved |
| Ops-legal (per pack) | Pre-pack-activation | Pending per pack; KR + EU + US first |
| Tenant DPA chains (per tenant) | First-tenant onboarding | Per tenant; gated by tenant DPA execution |
| EU AI Office | Post-Aug-2026 enforcement | Annual report under Art. 16 |
| KR PIPC | First-tenant pack-kr onboarding | Notified per PIPA Art. 33 |
| Data subjects (representative panel) | Quarterly | Feedback consumed by axis-intelligence backlog |

## Step 4 — Necessity and proportionality

### Lawful basis per data class

| Data class | Basis | Justification |
|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Art. 6(1)(b) + Art. 6(1)(f) | Necessary for service delivery + operational governance |
| `PII_IDENTIFYING` | Art. 6(1)(b) + Art. 6(1)(a) | Necessary when caller embeds user-id; consent obtained by tenant-level UX |
| `PII_SENSITIVE` | Art. 9(2)(a) explicit consent OR refusal | refusal-baseline refuses when consent absent |
| `SENSITIVE_PIPA_ART23` | KR PIPA Art. 23-2 explicit consent OR refusal | as above |
| `PHI` | HIPAA §164.502(a) (TPO) via BAA | provider-credential BYOK to BAA-signed provider per ADR-0255 §D-4; refusal otherwise |
| `AUDIT` | Art. 6(1)(c) + EU AI Act Art. 12 | Mandatory record-keeping |

### Data-minimisation

- Substrate does NOT own retrieval corpus (caller-side RAG); intelligence sees only the caller's
  assembled prompt.
- Substrate does NOT log prompt/output to observability beyond audit-tap (verified by
  `oya-governance-no-prompt-in-telemetry` CI lane).
- Per-call cost record uses tokenised proxy fields, not raw prompt text.

### Purpose-limitation

- Dispatch envelope carries `purpose` field (e.g., `consumer-chat`, `developer-codegen`,
  `foundry-planning`, `foundry-review`). Cedar `dispatch-authorization` enforces purpose ↔
  audience binding.

### Storage-limitation

- Audit-tap records: 1y default; HIPAA 6y; EU AI Act Art. 12 6 months minimum; per pack overlay.
- Per-call cost record: KR 5y per commercial code.
- Prompt/output content: NEVER stored beyond audit-tap; provider response stream is transient.

### Accuracy

- Per `eval` BC: canonicalen-set accuracy ≥ 95 % on canonical test corpus; per-tenant on-line A/B
  detects regression; SLI `oya_intelligence_eval_score_drop_total` alerts on regression.

### Integrity + confidentiality (Art. 32)

- TLS 1.3 + PQ-hybrid (Kyber768 + X25519) per ADR-0253.
- Per-tenant provider-credential BYOK (ADR-0255 §D-4); sidecar credential-handle (ADR-0296).
- Tenant-scope envelope; Cedar gates.
- Audit-tap Ed25519 sealed.

## Step 5 — Identify and assess risks

| ID | Risk | Likelihood | Impact | Risk | Treatment |
|---|---|---|---|---|---|
| R-01 | Cross-tenant context leak in provider response | L | H | M-H | T-I-01 mitigations |
| R-02 | Provider credential leak via logging | L | H | M | T-I-02 mitigations (ADR-0296) |
| R-03 | Model-output PII leak | M | M | M | T-I-03 mitigations |
| R-04 | Prompt-injection (indirect) | H | H | H | T-T-01 mitigations |
| R-05 | Refusal false-negative (Annex III leak) | L | H | M | T-T-01 + Cedar refusal gates |
| R-06 | Audit-row forgery | L | H | M | T-S-03 mitigations |
| R-07 | Cross-border misroute (EU→US without SCC) | L | H | M | Pack-pinning + Cedar `data-residency` gate |
| R-08 | DSR completion failure (data deleted but audit retained) | L | M | L-M | Audit-tap records pseudonymised user-id; DSR runner pseudonymises |
| R-09 | EU AI Act Art. 12 record-keeping failure | L | H | M | `audit-emission-success` SLO ≥ 99.99 % |
| R-10 | EU AI Act Art. 13 transparency failure | L | M | L-M | brand-ux-surface SparkleIcon presence verified |
| R-11 | Annex III deployment without FRIA | L | H | M | `eu-ai-act-high-risk.cedar` refuses dispatch when FRIA absent |
| R-12 | Provider lock-in causing rights-affecting decision opacity | M | M | M | 16-provider matrix + competitor-parity-matrix.md |
| R-13 | Vendor change cascade (provider deprecates a model) | M | M | M | Provider catalog versioning; deprecation runbook |
| R-14 | Refusal false-positive cascade | M | M | M | Eval canonicalen-set + per-pack refusal-floor isolation |
| R-15 | Consent grant missing | M | M | M | Pre-call gate refuses dispatch |
| R-16 | Children's data (COPPA / GDPR Art. 8) leak | L | H | M | refusal-baseline COPPA floor; age-gated UX |
| R-17 | Profiling (Art. 22) used for solely-automated rights-affecting decision | L | H | M | `eu-ai-act-high-risk.cedar` + tenant FRIA enforcement |

## Step 6 — Risk treatment

For each risk ≥ M, treatment is mandatory before pack activation. Treatments are referenced by
threat-model.md threat IDs + Cedar fragments + runbooks.

| Risk | Treatment | Status |
|---|---|---|
| R-01 | T-I-01 mitigations (single-tenant envelope; per-tenant credential isolation) | implemented |
| R-02 | T-I-02 (ADR-0296 sidecar; log redactor) | implemented |
| R-03 | T-I-03 (post-call PII classifier) | in progress IP-008 |
| R-04 | T-T-01 (multi-stage prompt-injection defence; runbook) | implemented at MVP; ongoing research |
| R-05 | T-T-01 + refusal-baseline Cedar gate | implemented |
| R-06 | T-S-03 (audit-tap Ed25519 + audit-chain Merkle) | implemented |
| R-07 | Pack-pinning + `data-residency.md` | implemented |
| R-08 | DSR runner pseudonymises audit records | implemented |
| R-09 | SLO + burn-rate alert | implemented |
| R-10 | brand-ux-surface SDK + axe-pa11y-runner | PHASE-02 |
| R-11 | `eu-ai-act-high-risk.cedar` gate | implemented |
| R-12 | 16-provider matrix + parity-matrix doc | implemented |
| R-13 | Provider catalog versioning + deprecation runbook (`docs/runbooks/byok-rotation-provider-tenant-duress.md`) | implemented |
| R-14 | Eval canonicalen-set + refusal-floor isolation + runbook `refusal-false-positive-cascade.md` | implemented |
| R-15 | Pre-call consent gate | implemented |
| R-16 | refusal-baseline COPPA floor | implemented |
| R-17 | `eu-ai-act-high-risk.cedar` + FRIA enforcement | implemented |

## Step 7 — Sign-off

| Role | Name (role) | Approved | Date |
|---|---|---|---|
| Data Protection Officer | council-privacy lead | yes | 2026-05-20 |
| AI Officer (per ISO 42001) | council-privacy + axis-intelligence | yes | 2026-05-20 |
| Security Lead | ops-security lead | yes | 2026-05-20 |
| Architecture Lead | council-architecture lead | yes | 2026-05-20 |
| Legal Lead | ops-legal lead | yes | 2026-05-20 |

Review cadence: annually + on every BC promotion + on every provider-adapter add + on every
Annex III category amendment.

## Step 8 — Outcomes integration

- Update `microservices/intelligence/policy/refusal-baseline.cedar` per pack-overlay outcomes.
- Update `microservices/intelligence/manifest.json` `data_classes_processed` and
  `regulatory_packs` fields.
- Update tenant DPA template (`legal/dpa-template.md`).
- Update brand-ux-surface refusal-banner copy per ops-legal sign-off.
- Notify EU AI Office per Art. 16 within 30 days of any material change.
- Notify KR PIPC per Notice 2020-7 within 30 days of any material change.

## References

- ADR-0255, ADR-0255 amendment, ADR-0263, ADR-0296, ADR-0250.
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/compliance.md`.
- `microservices/intelligence/policy/*.cedar`.
- GDPR + EU AI Act + ISO/IEC 42001 + NIST AI RMF + OWASP LLM Top 10.
