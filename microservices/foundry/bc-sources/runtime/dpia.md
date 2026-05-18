---
doc_class: DPIA
template_id: TPL-DPIA
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-runtime
deciders: council-privacy, ops-security, axis-foundry-runtime, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act Art. 9 (risk management)
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0140]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/foundry-runtime/threat-model.md
  - microservices/foundry-runtime/policy/runtime-isolation.md
  - microservices/foundry-runtime/policy/data-residency.md
  - microservices/foundry-runtime/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or pack activation
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (per-session conversation memory + tool-call decisioning is systematic profiling at scale)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in pack-us-healthcare; sensitive under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act Art. 6 (high-risk classification): conditional on tenant capability use case (clinical decision support, employment, education, essential services → high-risk per Annex III)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "EU AI Act Arts. 9 (risk management), 10 (data governance), 13 (transparency), 14 (human oversight), 15 (accuracy + robustness + cybersecurity)"
  - "ISO 27001:2022 A.5.34 + A.5.31"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3/15/17/18/22-2/23/24/25/28/29/29-2/33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7", "KR FSC AI Guideline 2024"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) + §164.312(b) + §164.502(b) + §164.514", "FDA SaMD pre-market"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EDPB Guidelines 4/2019 (Art. 25)", "EDPB Guidelines 9/2022 (breach notification)", "EU AI Act Arts. 9–15"]
  pack-jp: ["APPI Arts. 17, 18, 27", "METI AI Governance Guidelines 2024"]
  pack-sg: ["PDPA Part III + IV", "MAS FEAT Principles"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "AHRC AI guidance"]
  pack-in: ["DPDPA 2023 §10 + §11", "MeitY AI Advisory 2024"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38", "ANPD AI guidance"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23", "UAE Charter for Responsible AI"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9", "SDAIA Generative AI guidelines"]
doc_status: published
---

# Data Protection Impact Assessment: foundry-runtime µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is likely to result in **high risk** to data subjects. The foundry-runtime µservice triggers two of the three Art. 35(3) automatic triggers, plus a conditional EU AI Act high-risk classification:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Per-session conversation memory + per-turn tool-call decisioning constitute systematic profiling at scale; the runtime is the load-bearing decision-execution layer for hosted agents. |
| Art. 35(3)(b): Large-scale special-category processing | **YES (conditional)** | pack-us-healthcare carries PHI; pack-kr PIPA Art. 23 sensitive class triggered when tenant capability touches sensitive data. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | N/A |
| EU AI Act Art. 6 (high-risk classification) | **YES (conditional)** | Tenant capabilities targeting Annex III use cases (clinical decision support, employment, education, essential services) are high-risk; runtime is the deployer's tooling for those use cases. |

KR PIPC Notice 2020-7 mandates a DPIA when system handles sensitive PII at scale — engaged.

EU AI Act Art. 9 (risk management) requires risk-assessment for high-risk systems — engaged when tenant capability is high-risk.

Therefore: **mandatory DPIA pre-deployment**. This document is the canonical DPIA reviewed by EU DPAs (Art. 35), KR PIPC (PIPA Art. 33), HIPAA OCR (when pack-us-healthcare engaged), and EU AI Act notified bodies (when tenant capabilities are high-risk per Annex III).

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** foundry-runtime accepts capability invocation requests; resolves capability descriptors from the registry mirror; materialises runtime pods (or attaches warm-pool pods); maintains per-session conversation history + tool-call scratchpad across multi-turn interactions; dispatches LLM + tool calls through `foundry-providers` and safety checks through `foundry-guardrails`; emits invocation telemetry to `foundry-evidence` and observability.

**How:** Tenant operator (Workflow Studio) or workload µservice (workflow-engine) → REST/gRPC invocation → AutonomyGate (refusal-or-permit) → registry-cache descriptor read → executor dispatches via providers + guardrails over mTLS → step events emitted → session-state Redis read/write + Postgres mutation log → invocation lifecycle records sealed in audit-chain → invocation completion event consumed by waiting workflow.

**Where:** Per-pack region-pinned runtime clusters (pack-kr → KR / pack-eu → EU / pack-us → US / etc.). Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; on every tenant or workload invocation.

**Who:** Per actor table in `threat-model.md` §"Actors".

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate (per medium tenant) |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant invocation rates, conversation history, scratchpad | Art. 6(1)(b) contract + 6(1)(f) legitimate interest | ~10⁷ invocations/day |
| `PII_IDENTIFYING` | User-id / email / IDs in session payload when emitted by capability | Art. 6(1)(b) contract; Art. 6(1)(c) legal (audit) | varies per capability |
| `PII_QUASI_IDENTIFIER` | URLs, IPs, user-agent in tool-call working data | Art. 6(1)(f) legitimate interest; minimised at SDK | ~10⁶ attributes/day |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id `X-Scope-OrgID` correlated with auxiliary | KR PIPA Art. 15 + 23 + 23-2 with explicit consent | 1 per invocation |
| `PHI` (pack-us-healthcare only) | Patient IDs / clinical data when capability invocation touches clinical workflow | HIPAA §164.502(a) TPO via BAA | targeted to 0 via guardrail redactor |
| `AUDIT` | Invocation lifecycle events; autonomy-violation events; Ed25519 seals | Art. 6(1)(c) legal obligation | 1 per invocation + 1 per violation |
| `SECRET` | Redis AUTH / Postgres credentials / SPIFFE material | not personal data; ISO 27001 A.5.17 | varies |

**Geographical scope:** Per pack (pack-kr KR / pack-eu EU / pack-us US / pack-us-healthcare US HIPAA-eligible / pack-jp JP / pack-sg SG / pack-au AU / pack-in IN / pack-br BR / pack-ae AE / pack-ksa KSA).

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. SCC-mediated exception for GDPR-scope only with tenant-executed SCCs + EDPB-recommended supplementary measures.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (the tenant's customers); tenant operators; oyatie operators.
- **Joint controllership** with tenant under GDPR Art. 26; recorded in DPA template.
- **Reasonable expectations:** tenant operators expect hosted agent runtime per SLA; end-users expect data collection per the tenant's own privacy notice; oyatie's processing disclosed via joint-controllership clause.
- **Previous experience:** Bominal had no Foundry-class runtime equivalent; oyatie is greenfield; no DPA-triggered complaints baseline.
- **Industry codes:** OpenTelemetry semantic conventions + OpenAPI 3.2 + AsyncAPI 3.1 for downstream visibility.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Execute capability invocations on behalf of tenant | Necessary for SLA | Art. 6(1)(b) |
| Maintain session-state for multi-turn coherence | Necessary for product experience | Art. 6(1)(b) |
| Enforce autonomy tier at dispatch (refuse over-ceiling) | Operational integrity; EU AI Act Art. 14 human oversight | Art. 6(1)(b) + 6(1)(f) + EU AI Act Art. 14 |
| Emit invocation telemetry to evidence + observability | Operational + compliance | Art. 6(1)(b) + 6(1)(c) |
| Audit-chain seal of every invocation event | Mandatory for SOC 2 + ISO + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) |
| Marketing / unrelated commercial use | NOT a purpose | N/A — explicitly excluded |

Purposes explicit, legitimate, specified at tenant onboarding per Art. 5(1)(b).

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representative (sample of 3) | Scheduled pre-GA | Feedback into §6 |
| Data subjects | Indirect via tenant onboarding notices | Joint-controllership cascade |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Art. 36 prior consultation — NOT triggered (residual ≤ Medium after mitigations) | If §6 residual > Medium for any high-risk class, prior consultation triggered |
| EU AI Act notified body | Conditional on tenant capability classification (pack-eu + Annex III use) | Engaged per-tenant when high-risk capability registered |
| Information security team (ops-security) | YES — co-author of threat-model.md | Shared residual-risk catalog |
| Engineering teams (axis-foundry-runtime + siblings) | YES | data_class annotations enforced at CI |
| External auditor (SOC 2 / ISO firm) | At first audit cycle | Cross-references this DPIA |

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary? | YES — capability invocation cannot be executed without conversation state + capability descriptor read. |
| Less intrusive alternative? | Considered: stateless agents (no session memory). Rejected: defeats multi-turn experience contract. Current design retains session memory with minimisation (data_class annotation + guardrail redactor + sampling). |
| Proportionate? | YES — session-state contains only the conversation memory necessary to maintain coherence; tool-call working data carries `data_class` annotations; guardrails strip injected adversarial content; provider credentials never resident in runtime. |
| Public / private substantial interest? | YES — tenant SLA + operational reliability + safety enforcement. |
| Anonymised alternative? | PARTIAL — pseudonymisation via hashed tenant_id at the boundary; full anonymisation defeats per-tenant SLA. |
| Lawful basis (Art. 6) | Identified per purpose in §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h); pack-kr sensitive: PIPA Art. 23(2) explicit consent at onboarding. |
| Transfer basis (Arts. 44–46) | Per §2.2: SCCs only. |
| Retention | Per asset table in threat-model §"Assets". Defaults: 14d Redis hot + 90d Postgres cold + 6y for HIPAA scope. PIPA erasure on request via DSR cascade. |
| Rights of data subjects | Honoured per §6 mitigations; Art. 15/16/17/18/20/21/22 + EU AI Act Arts. 13 + 14 transparency + human-oversight. |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Prompt-injection causes session content leakage across turns | M-H | H | **H** |
| R-02 | Cross-tenant session leak (Redis prefix misconfig) | L-M | H | **H** |
| R-03 | Long retention enables behavioural surveillance | M | M-H | **M-H** |
| R-04 | Automated capability invocation affects end-user (decision side-effect) | M | M | **M** |
| R-05 | Autonomy ceiling bypass produces unauthorised side-effect on end-user | L | H | **M** |
| R-06 | Sub-processor (provider / cloud / Redis cluster operator) breach | L | H | **M** |
| R-07 | Tenant misconfig: capability descriptor leaks descriptor-stage PII | M | M-H | **M-H** |
| R-08 | DSR cascade incomplete (end-user across multiple sessions / packs) | M | M | **M** |
| R-09 | Joint-controllership confusion: tenant doesn't disclose to end-users | M-H | M | **M-H** |
| R-10 | Re-identification via session timing + cardinality (small tenant) | L | M | **L-M** |
| R-11 | Cross-border transfer of EU-resident session via misroute | L | H | **M** |
| R-12 | Children's data (DPDPA §9; pack-in) processed without parental consent | L | H | **M-H** |
| R-13 | PHI processed without BAA (pack-us-healthcare) | M | H | **H** |
| R-14 | Hashed tenant-id re-identified via small-tenant auxiliary | L | M | **L-M** |
| R-15 | Auditor mis-pivots tenant-A → tenant-B during engagement | L | H | **M** |
| R-16 | High-risk EU AI Act capability deployed without notified body engagement | L | H | **M** |
| R-17 | Provider credential leak via runtime memory dump | L | H | **M** |
| R-18 | Capability descriptor tampering produces malicious tool-call | L | H | **M** |
| R-19 | Invocation lifecycle record tampered → false audit trail | L | H | **M** |

Cross-reference: every risk has at least one mitigation in §6 and one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (prompt-injection) | Foundry-guardrails BEFORE provider dispatch; output guardrails before session persistence; per-tenant adversarial pattern detector; OWASP LLM Top 10 mitigations applied | Residual M (adversarial baseline) | foundry-guardrails + axis-foundry-runtime |
| R-02 (cross-tenant session leak) | Redis tenant-prefix enforced by SessionStore; LEAN check; per-tenant Redis ACL; integration test cross-tenant returns empty | L | axis-foundry-runtime + ops-security |
| R-03 (long retention) | Retention bounded; DSR cascade 30d; cold-tier aggregated; admin JIT for cold-tier reads | L-M | council-privacy |
| R-04 (automated decisions) | Capability invocation not solely-automated for Art. 22 purposes (operational); tenant tier-2-plus capabilities require human-in-loop ack | L | axis-foundry-runtime |
| R-05 (autonomy ceiling bypass) | AutonomyGate first step; signed ceiling cache; violation emits AutonomyViolationDetected | L | axis-foundry-runtime + ops-security |
| R-06 (sub-processor breach) | Sub-processor list maintained; quarterly security review; DPA + SCCs per applicable | M (irreducible) | council-privacy + cloud-secrets |
| R-07 (tenant misconfig) | Capability descriptor schema validation; data_class enforcement at supervisor; runtime refuses unannotated descriptors | L-M | axis-foundry-runtime + axis-foundry |
| R-08 (DSR incompleteness) | DSR cascade in session-state worker scans hot + cold; soft-delete 30d grace | M (best-effort within retention) | council-privacy |
| R-09 (joint-controllership confusion) | DPA template mandates upstream disclosure clause; onboarding refuses non-disclosure | L-M | council-privacy + gtm |
| R-10 (re-identification) | Per-tenant cardinality limits; DP noise on cross-tenant aggregates | L | axis-foundry-runtime |
| R-11 (cross-border misroute) | Pack-pinning at OTel collector level; integration test catches misroute | L | axis-foundry-runtime |
| R-12 (children's data) | Tenant DPA child-data clause; parental-consent affirmation | L (depends on tenant) | council-privacy |
| R-13 (PHI without BAA) | pack-us-healthcare onboarding refuses until BAA signed | L | council-privacy + sales-legal |
| R-14 (tenant-id re-id) | Salted hash + 12mo rotation | L | ops-security |
| R-15 (auditor mis-pivot) | Auditor JIT tokens tenant-scoped; pen-test annually | L | ops-security |
| R-16 (high-risk EU AI Act capability without notified body) | Capability registration carries Annex III classification flag; notified body engagement required before activation in pack-eu | L | council-privacy + council-architecture |
| R-17 (provider credential leak) | Architectural: credentials never resident in runtime (foundry-providers holds them); coredumps disabled; e2e test verifies isolation | L | foundry-providers + ops-security |
| R-18 (descriptor tampering) | Ed25519 signature on every descriptor; validated at cache load | L | axis-foundry-runtime |
| R-19 (lifecycle tampering) | audit-chain Ed25519 + Merkle seal | L | audit-chain |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| ISO (ops-security chair) | `pending` | TBA |
| µservice owner (axis-foundry-runtime lead) | `pending` | TBA |
| council-architecture chair | `pending` | TBA |
| EU AI Act notified body | conditional | engaged per high-risk tenant capability registration |

**DPO advice:**
Residual risks after §6 mitigations rated L or M (no H or M-H residuals remain). Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-01 (prompt-injection residual).
- Annual review of this DPIA.
- Re-trigger DPIA on any pack activation OR any high-risk EU AI Act capability registration.
- EU AI Act Art. 9 risk-management review on every capability tier elevation.

**Outcomes documented:**
- Mitigations adopted: every measure in §6 in-scope for IP-001 through IP-015 (`PHASE-01-AGENT-RUNTIME-AND-CAPABILITY-EXECUTION.md`).
- ROPA register entry: `microservices/foundry-runtime/legal/ropa.md`.
- Joint-controllership: `microservices/foundry-runtime/legal/dpa-template.md`.

## Per-Pack Overlay Sections

### pack-kr (PIPA + ISMS-P + FSC AI Guideline 2024)

PIPA Art. 33 + Enforcement Decree Art. 35 require 개인정보영향평가 for sensitive PII at scale — engaged. PIPC Notice 2020-7 methodology followed.

- **PIPA Art. 23**: hashed tenant-id sensitive when correlated; salt rotation per R-14.
- **PIPA Art. 23-2**: KR-resident sensitive data stays in pack-kr.
- **PIPA Art. 28**: bounded retention.
- **PIPA Art. 29**: every §6 measure maps to a prescribed safeguard.
- **PIPA Art. 33-2**: council-privacy chair serves PIPA DPO role.
- **KR FSC AI Guideline 2024 §3**: AutonomyGate is the human-in-loop control; FSC notification on AutonomyViolation > threshold.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk analysis substantially equivalent to DPIA — engaged.

- **§164.502(a) TPO**: operations scope only.
- **§164.502(b) Minimum Necessary**: data_class minimum-necessary redaction.
- **§164.504(e) Business Associate**: BAA template at `legal/baa-template.md`.
- **§164.312(b) Audit Controls**: audit-chain seal; retention ≥ 6y for PHI sessions.
- **§164.404 / §164.406 / §164.408**: breach notification chain in `incident-response.md`.
- **FDA SaMD pre-market**: clinical decision support capabilities carry FDA classification tag; runtime refuses unclassified in pack-us-healthcare.

### pack-eu (GDPR + EU AI Act + EDPB + NIS2 + eIDAS)

This document is the GDPR Art. 35 DPIA AND the EU AI Act Art. 9 risk-management artifact for high-risk capability deployments.

- **EDPB Guidelines 4/2019 (Art. 25)**: explicit alignment §4 + §6.
- **EDPB Guidelines 9/2022 (breach)**: 72h chain in `incident-response.md`.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo timelines.
- **eIDAS 910/2014**: Ed25519 seals as AdES for EU-tenant invocation records.
- **EU AI Act Art. 9**: this risk register + threat-model are the foundational artifacts; reviewed pre-deployment per high-risk capability.
- **EU AI Act Art. 10**: data_class taxonomy + retention + tenant-scope enforcement.
- **EU AI Act Art. 13 (transparency)**: capability descriptor includes purpose + autonomy tier.
- **EU AI Act Art. 14 (human oversight)**: AutonomyGate.
- **EU AI Act Art. 15 (cybersecurity)**: foundry-guardrails + provider-credential isolation + circuit-breakers.
- **Schrems II + Arts. 44–46**: no cross-border EU-resident transfer without SCCs + supplementary measures.
- **Children's data (Art. 8)**: via tenant age-gating.

### pack-jp (APPI + METI AI Governance)

APPI Arts. 17–27 + METI voluntary AI governance scheme satisfied.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- Every new pack activation.
- Every high-risk EU AI Act capability registration (per Annex III).
- Every change to processing purpose (§2.4) or data-class taxonomy.
- Every sub-processor change.
- Every breach notification.
- Every supervisory-authority guidance change.
- Post-incident (Sev-1/Sev-2).
- Every autonomy-tier ceiling raise.

## References

- ADR-0022; ADR-0024; ADR-0025; ADR-0028 (Bominal audit-chain); ADR-0117; ADR-0139; ADR-0131; ADR-0132; ADR-0140.
- `microservices/foundry-runtime/threat-model.md` — paired security artifact.
- `microservices/foundry-runtime/policy/{runtime-isolation, data-residency}.md`.
- `microservices/foundry-runtime/compliance.md`.
- `microservices/foundry-runtime/incident-response.md`.
- `microservices/foundry-runtime/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019, 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36; KR PIPA Art. 33 + Enforcement Decree Art. 35; HIPAA 45 CFR §164.308(a)(1)(ii)(A); LGPD Art. 38; DPDPA 2023 §10–§11.
- EU AI Act (Regulation 2024/1689) Arts. 6 + 9 + 10 + 13 + 14 + 15 + Annex III.
- OWASP Top 10 for LLM Applications 2025.
