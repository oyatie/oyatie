---
doc_class: DPIA
template_id: TPL-DPIA
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-guardrails
deciders: council-privacy, ops-security, axis-foundry-guardrails, council-architecture
methodology: ICO DPIA template + CNIL DPIA + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act Art. 9 risk-management
related_adrs: [ADR-0022, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/intelligence-guardrails/threat-model.md
  - microservices/intelligence-guardrails/policy/guardrail-enforcement.md
  - microservices/intelligence-guardrails/policy/data-residency.md
  - microservices/intelligence-guardrails/compliance.md
review_cadence: annually + on every classifier-model rollout + on every Cedar bundle change + on every pack activation
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (per-prompt classification is systematic profiling)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in pack-us-healthcare; sensitive data under PIPA Art. 23)"
  - "GDPR Art. 22 — solely-automated decision producing significant effects on data subject — YES (block decisions can deny a tenant's user from receiving a response)"
  - "EU AI Act high-risk — guardrails is a safety component; risk-management system mandatory per Art. 9"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44–50"
  - "ISO 27001:2022 A.5.34 (privacy + PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1–P8, 2017 TSC)"
  - "EU AI Act Arts. 9 (risk-management), 10 (data + data-governance), 11 (technical-documentation), 12 (record-keeping), 13 (transparency), 14 (human-oversight), 15 (accuracy + robustness + cybersecurity)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 23-2, 24, 25, 28, 29, 29-2, 33", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis), §164.312(b) (audit), §164.502(b) (minimum necessary), §164.514 (de-identification), §164.504(e) (BAA)"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EU AI Act Art. 9 (risk-management system mandatory for high-risk AI)", "EDPB Guidelines 4/2019, 9/2022"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV, MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12, OAIC APP guidelines"]
  pack-in: ["DPDPA 2023 §§10 + 11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38 (RIPD), ANPD methodology"]
  pack-ae: ["UAE PDPL FDL 45/2021 Art. 23"]
  pack-ksa: ["PDPL RD M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: foundry-guardrails µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) + EU AI Act Art. 9 + KR PIPA Art. 33 + HIPAA §164.308(a)(1)(ii)(A) all engage:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | **YES** | Per-prompt classification is systematic; verdict influences whether a tenant's end-user receives a response. |
| Art. 35(3)(b): Special-category at scale | **YES (conditional)** | Pack-us-healthcare PHI; pack-kr Art. 23 sensitive. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Public-area systematic monitoring | NO | n/a |
| GDPR Art. 22: solely-automated decision-making with significant effects | **YES** | Block decision is automated; can deny information delivery; significant effect to data subject. |
| EU AI Act Art. 9: high-risk AI system risk-management | **YES** | foundry-guardrails is a safety component of an AI system; risk-management system mandatory. |
| HIPAA §164.308(a)(1)(ii)(A) risk-analysis | **YES (conditional)** | Pack-us-healthcare activation triggers HIPAA risk-analysis substantially equivalent to DPIA. |
| KR PIPC Notice 2020-7 (sensitive data at scale) | **YES (conditional)** | Pack-kr activation triggers KR DPIA-equivalent. |

DPIA mandatory pre-deployment. This document is the canonical DPIA + AI Act risk-management record reviewed by EU DPAs / Korean PIPC / HIPAA OCR examiners at first-tenant onboarding in each pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** foundry-guardrails accepts a candidate prompt (pre-invocation) + provider output (post-invocation) + tenant context (tenant_id_hashed, capability, autonomy-ceiling-claim) from foundry-runtime; runs classifier ensemble + Cedar policy evaluation; returns verdict (allow / block / redact + reason). Verdict + decision metadata emitted to foundry-evidence + audit-chain. No prompt / output content persisted by guardrails itself.

**How:** foundry-runtime → guardrails REST/gRPC (mTLS + SPIFFE) → classifier-model serving (in-cluster ONNX) + Cedar engine (in-process) + Postgres rule-store (per-pack HA) + optional LLM-judge fallback (via foundry-providers). Verdict → AsyncAPI publisher → foundry-evidence + audit-chain.

**Where:** Per-pack region-pinned cluster (pack-kr → KR / pack-eu → EU / pack-us → US / pack-jp → JP / etc.); pack-pinning enforces residency per ADR-0117.

**When:** Real-time (every capability invocation); 60-second classifier-model freshness check; rule hot-reload on PR merge.

**Who:** Per actor table in `threat-model.md` §"Actors". Sole in-cluster caller: foundry-runtime.

### 2.2 Scope of the processing

**Personal-data classes processed (transient in-memory only):**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-invocation prompt + output text | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest (safety) | ~10⁷ invocations/day per medium tenant |
| `PII_IDENTIFYING` | User-id fields or quoted strings in prompts | Art. 6(1)(b) + 6(1)(f) | varies; redacted from logs |
| `PII_QUASI_IDENTIFIER` | URLs / IPs / quasi-identifiers in prompts | Art. 6(1)(f); minimised at SDK | varies |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id | KR PIPA Art. 15 + 23 (explicit consent at onboarding) | 1 per request |
| `PHI` (pack-us-healthcare) | Patient identifiers in prompt content | HIPAA §164.502(a) Permitted Uses (Operations) per BAA | varies; redactor-targeted |
| `AUDIT` | GuardrailDecision verdicts | Art. 6(1)(c) legal obligation + 6(1)(f) | 1 per invocation |
| `SECRET` | Cosign signing keys; provider tokens (via foundry-providers) | n/a (not personal data) | per OpenBao |
| Classifier scores | Per-detector outputs | Art. 6(1)(b) + 6(1)(f) | 1 vector per invocation |

**Geographical scope:** Per-pack residency; cross-pack transfer of prompts / outputs / rules / classifier-models / decisions forbidden by default.

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs for GDPR-scope tenants.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (tenant's customers); tenant operators; oyatie operators.
- **Joint controllership:** Per GDPR Art. 26 with tenant; tenant DPA includes upstream disclosure clause.
- **Reasonable expectations:** Tenant operators expect a safety floor; end-users expect operational safety; oyatie's processing is disclosed in tenant's privacy notice via joint-controllership cascade.
- **Industry codes:** Voluntary alignment with NIST AI RMF 1.0; OWASP LLM Top 10; MITRE ATLAS.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Pre-invocation prompt classification** | Necessary for safety floor | Art. 6(1)(b) + 6(1)(f) |
| **Post-output validation** | Necessary for output safety | Art. 6(1)(b) + 6(1)(f) |
| **Autonomy-tier gate enforcement** | Necessary for ADR-0022 compliance | Art. 6(1)(b) + 6(1)(c) |
| **Content-safety rule evaluation** | Necessary for per-pack regulatory compliance | Art. 6(1)(c) (legal obligation: KR PIPA, HIPAA, EU AI Act) |
| **Jailbreak detection** | Necessary for cybersecurity | Art. 6(1)(c) + 6(1)(f) |
| **Audit-chain emission** | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR + EU AI Act | Art. 6(1)(c) |
| **Marketing / unrelated commercial use** | NOT a purpose | excluded |

Purposes are explicit + legitimate + specified at tenant onboarding via DPA template (Art. 5(1)(b)).

## Step 3 — Consultation

| Stakeholder | Consulted | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | sign-off pending; §7 |
| Tenant representative (3 prospective tenants) | Scheduled pre-GA | Folded into §6 |
| Data subjects | Indirect via tenant onboarding | Joint-controllership cascade |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Not triggered (residual ≤ M after mitigations) | If residual > M, Art. 36 prior consultation triggered |
| EU AI Act notified body | At first AI-Act-scoped tenant onboarding | Required for high-risk AI |
| Information-security (ops-security) | YES | Threat-model + DPIA share residual catalog |
| Engineering teams | YES | data_class annotations enforced |
| External auditor | At first audit cycle | Cross-references this DPIA |

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary for purpose? | YES — safety floor cannot be enforced without inspecting content. |
| Less intrusive alternative? | Considered: client-side filtering only. Rejected: bypassable; doesn't meet AI Act Art. 14 human-oversight requirement. |
| Proportionate to purpose? | YES — content inspected in-memory only, not persisted; minimum-necessary classifier outputs; per-pack scope. |
| Public interest / substantial private interest? | YES — safety + cybersecurity + regulatory compliance. |
| Anonymised / pseudonymised alternative? | Pseudonymisation applied (hashed tenant ID); full anonymisation defeats per-tenant entitlement model. |
| Lawful basis | §2.4 |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h) + HIPAA BAA. pack-kr Art. 23: explicit consent. |
| Transfer basis (Arts. 44–46) | SCC-only; default residency by pack. |
| Retention | Per asset class in `threat-model.md` §"Assets & Data Classification". Prompt + output NOT persisted by guardrails; persisted by foundry-evidence under that µservice's retention. |
| Rights of data subjects | Honoured per §6: access, rectification, erasure, restriction, portability, objection, Art. 22 automated-decision protections. |

## Step 5 — Identify and assess risks to data subjects

Scored Likelihood × Severity (data-subject perspective).

| ID | Risk | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Block decision wrongly denies legitimate request (false-positive harms end-user) | M | M | **M** |
| R-02 | Allow decision wrongly passes unsafe content (false-negative harms end-user) | M | H | **H** |
| R-03 | Prompt content leakage via guardrails logs | M | H | **H** |
| R-04 | Classifier-model leakage of training-data PII via model-inversion | L | M | **L-M** |
| R-05 | Per-tenant Cedar overlay leaks tenant business intent cross-tenant | L | M | **L-M** |
| R-06 | Sev-1 jailbreak success exposes end-user to unsafe content | L | H | **M** |
| R-07 | Automated decision (Art. 22) lacks explanation | M | M | **M** |
| R-08 | Joint-controllership confusion: end-user unaware of guardrail processing | M-H | M | **M-H** |
| R-09 | DSR cascade incomplete: prompts not retrievable for erasure (not persisted) | L | L | **L** |
| R-10 | LLM-judge fallback discloses prompt cross-pack | L | H | **M** |
| R-11 | EU AI Act high-risk classifier deployed without notified-body assessment | L | H | **M** |
| R-12 | Children's data (DPDPA §9) processed without parental consent | L | H | **M-H** |
| R-13 | PHI in pack-us prompt without BAA | M | H | **H** |
| R-14 | Adversarial prompt evasion exposes end-user | M | H | **M-H** |
| R-15 | Multi-turn drift evades single-turn classifier | M | M-H | **M** |

Cross-reference: every risk has at least one mitigation in §6 + one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (false-positive) | Per-tenant FP escalation budget; rule-author review queue; shadow→enforce rollout; decision detail explanation per GDPR Art. 22 | L-M (engineering discipline floor) | axis-foundry-guardrails |
| R-02 (false-negative) | Multi-detector ensemble; LLM-judge ambiguous fallback; post-output validator (second line); baseline-fixture catalogue; monthly red-team | L | axis-foundry-guardrails |
| R-03 (prompt leakage in logs) | OTel redactor; data_class annotation; `oya-check-data-class` lane; synthetic-PII drills | M (engineering discipline floor) | every owner |
| R-04 (model inversion) | Verdict-only classifier output; LLM-judge with no-data-disclosure template; training-data provenance documented | L | axis-foundry-guardrails |
| R-05 (cross-tenant rule leak) | Cedar `tenant-scope`; Postgres RLS; typed `RuleStore` port | L | axis-foundry-guardrails |
| R-06 (Sev-1 jailbreak) | Sev-1 auto-incident; post-mortem; classifier retraining; red-team monthly | L | axis-foundry-guardrails |
| R-07 (Art. 22 explanation) | Decision detail endpoint surfaces block_reason + cedar_policy_ids + classifier_model_versions; tenant operator can request human review | L | axis-foundry-guardrails |
| R-08 (joint-controllership) | Tenant DPA includes upstream disclosure clause; tenant onboarding checklist verifies privacy-notice; non-disclosure = onboarding refused | L-M | council-privacy + gtm |
| R-09 (DSR cascade) | Prompts not persisted by guardrails; foundry-evidence handles DSR cascade for any persisted decision history; 30d SLA | L | council-privacy |
| R-10 (LLM-judge cross-pack) | LLM-judge endpoint pack-pinned; foundry-providers enforces pack residency | L | axis-foundry-guardrails + axis-foundry-providers |
| R-11 (EU AI Act notified body) | Notified-body assessment scheduled before pack-eu activation | L | council-privacy + ops-legal |
| R-12 (children's data) | DPDPA §9 + GDPR Art. 8: tenant DPA includes child-data clause; oyatie inherits tenant's age-gating | L (residual on tenant) | council-privacy |
| R-13 (PHI without BAA) | pack-us-healthcare requires BAA before ingest enabled; non-signed tenants routed to non-PHI pack | L | council-privacy + sales-legal |
| R-14 (adversarial prompt evasion) | Multi-detector ensemble + canonicalisation pre-pass + red-team monthly | M (cat-and-mouse) | axis-foundry-guardrails |
| R-15 (multi-turn drift) | Session-aware classifier when turn-count > 5; per-session block-rate SLO | M | axis-foundry-guardrails |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | `pending` | TBA |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-foundry-guardrails lead) | `pending` | TBA |
| council-architecture chair | `pending` | TBA |
| EU AI Act notified body (pack-eu activation) | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all rated L or M. Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-03, R-14, R-15.
- Annual review of this DPIA.
- Re-trigger DPIA on every classifier-model rollout + every pack activation + every Cedar bundle change.

**Outcomes documented:**
- Mitigations in §6 in-scope for Slice A / B / C / D authoring.
- Records-of-processing (Art. 30): `microservices/intelligence-guardrails/legal/ropa.md` (Slice D).
- Joint-controllership: `legal/dpa-template.md`.
- EU AI Act risk-management system: this document + PRD + threat-model + compliance + ongoing classifier-rollout sign-offs.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + ISMS-P)

- **PIPA Art. 33 + Enforcement Decree Art. 35**: 영향평가 fulfilled.
- **PIPA Art. 23 (sensitive)**: default-block on sensitive content; tenant consent required for entitlement.
- **PIPA Art. 23-2 (cross-border)**: pack-kr-resident; no cross-pack.
- **PIPC Notice 2020-7**: 7-step methodology followed.
- **PIPA Art. 33-2 (DPO)**: council-privacy chair serves PIPA DPO role.

### pack-us-healthcare (HIPAA)

- **§164.308(a)(1)(ii)(A) Risk Analysis**: this DPIA fulfils.
- **§164.502(a) Permitted Uses (TPO)**: Operations.
- **§164.502(b) Minimum Necessary**: prompt + output handled in-memory only; not persisted.
- **§164.504(e) BAA**: `legal/baa-template.md`.
- **§164.312(b) Audit Controls**: audit-chain seals + retention ≥ 6y.

### pack-eu (GDPR + EU AI Act + EDPB + NIS2 + eIDAS)

- **GDPR Art. 35**: this document.
- **GDPR Art. 22 (automated decision)**: §6 R-07 explanation right.
- **EU AI Act Art. 9 (risk-management)**: this DPIA + threat-model + PRD + compliance form the risk-management system; ongoing monitoring via shadow-mode metrics.
- **EU AI Act Art. 10 (data + data-governance)**: training-data provenance documented.
- **EU AI Act Art. 11 (technical-documentation)**: this document + classifier model-cards.
- **EU AI Act Art. 12 (record-keeping)**: audit-chain seals.
- **EU AI Act Art. 13 (transparency)**: block_reason + cedar_policy_ids.
- **EU AI Act Art. 14 (human-oversight)**: FP budget + tenant override + rule-author review queue.
- **EU AI Act Art. 15 (accuracy + robustness + cybersecurity)**: ensemble + Cosign + red-team.
- **EDPB Guidelines 4/2019 (Art. 25 by design + default)**: explicit alignment.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h notification chain in `incident-response.md`.
- **NIS2**: when oyatie crosses thresholds, 24h+72h+1mo timelines.
- **eIDAS**: Ed25519 audit-chain seals = AdES.
- **Schrems II + Arts. 44-46**: SCC-only; pack-eu-resident.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-guardrails-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- On every classifier-model rollout (always re-DPIA at least the risk-analysis section).
- On every Cedar policy bundle change.
- On every new pack activation.
- On supervisory-authority guidance change affecting any enforced framework.
- Post-incident.

## References

- ADR-0022, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140.
- `microservices/intelligence-guardrails/threat-model.md`.
- `microservices/intelligence-guardrails/compliance.md`.
- `microservices/intelligence-guardrails/policy/{tenant-isolation, data-residency, guardrail-enforcement}.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022.
- GDPR Art. 35 + 36; KR PIPA Art. 33; HIPAA §164.308(a)(1)(ii)(A); LGPD Art. 38.
- EU AI Act (Reg 2024/1689) Arts. 9-15.
- NIST AI RMF 1.0.
