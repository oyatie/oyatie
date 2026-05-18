---
doc_class: DPIA
template_id: TPL-DPIA
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-workflow
deciders: council-privacy, ops-security, axis-workflow, council-design-system, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act 2024 conformity assessment
related_adrs: [ADR-0028, ADR-0056, ADR-0065, ADR-0103, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-0164]
related_specs: [/specs/microservices/workflow-studio.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/workflow-studio/threat-model.md
  - microservices/workflow-studio/policy/data-residency.md
  - microservices/workflow-studio/compliance.md
  - microservices/workflow-studio/policy/editor-isolation.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or LLM-assist provider
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — YES (Studio processes every tenant authoring session)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare visual edits; sensitive data under PIPA Art. 23; LLM-assist prompts may carry end-user data)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act 2024 (high-risk system): conditional YES if LLM-assist used for spec drafting in regulated domains; conformity-assessment in scope"
doc_status: published
---

# Data Protection Impact Assessment: workflow-studio µservice

## Step 1 — Identify the need for a DPIA

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation | **YES** | Studio processes every tenant authoring session; aggregation across sessions constitutes systematic profiling of tenant authoring behavior. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional on pack)** | pack-us-healthcare clinical-workflow visual edits carry PHI references; pack-kr PIPA Art. 23 classes correlation-prone editor signals as sensitive; LLM-assist prompts may inherit end-user identifiers from tenant prose. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Studio does not monitor public area. |
| EU AI Act 2024 (high-risk AI system) | **YES (conditional)** | LLM-assist drafting workflow specs in regulated domains (healthcare/finance/HR) classifies under AI Act Annex III if specs drive automated decisions about persons. Conformity assessment required. |

KR PIPC Notice 2020-7 mandates DPIA when handling sensitive PII at scale — engaged.

DPIA is mandatory pre-deployment. This document is reviewed by EU DPAs (Art. 35) and KR PIPC (PIPA Art. 33) at first-tenant onboarding in each jurisdiction, plus EU AI Act conformity-assessment when LLM-assist used in regulated domain workflows.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** Studio receives tenant authoring inputs (drag-drop events, config field edits, prose for LLM-assist); persists editor session state; emits CRDT ops to collab participants; on save, emits a `workflow_spec.v1.json` document to the workflow-engine spec-store; enforces per-seat licensing via Cedar; renders per-jurisdiction overlays; streams live debugger frames from engine.

**How:** Tenant browser loads Leptos WASM bundle from CDN → OIDC tenant-binding established → editor REST issues editor session → CRDT ops route through WebSocket gateway → save emits canonical spec → engine durably registers spec.

**Where:** Per-pack region-pinned Studio clusters (pack-kr → KR / pack-eu → EU / pack-us → US / pack-us-healthcare → US-HIPAA-eligible / etc.); each pack has its own Postgres + Valkey cluster; CDN is global edge with per-pack cache keys.

**When:** On-demand; sub-second TTI; per-save audit seal within 1s of submit.

**Who:** Per actor table in `threat-model.md`.

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant authoring sessions, edits per workflow, save events | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest | ~10⁶ events/day per medium tenant |
| `PII_IDENTIFYING` | User-id fields in spec drafts; LLM-assist prose mentioning end-users | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation (audit) | varies per tenant |
| `PII_QUASI_IDENTIFIER` | Author OIDC `sub`; IP addresses in editor session metadata | Art. 6(1)(f) legitimate interest; minimised at SDK | ~10⁴ events/day per tenant |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id in CDN cache key | KR PIPA Art. 15 + 23 + 23-2 | 1 per tenant per CDN segment |
| `PHI` (pack-us-healthcare) | Patient identifiers referenced in clinical-workflow visual drafts; LLM-assist prose | HIPAA §164.502(a) (TPO) per BAA | targeted near-zero via SDK redactor; non-zero residual |
| `AUDIT` | Save events; license-gate evaluation events; collab conflict resolutions | Art. 6(1)(c) legal obligation | 1 per state transition |
| `SECRET` | Per-tenant SDK API keys; per-pack node-library signing keys | not personal data; ISO 27001 A.5.17 | — |

**Geographical scope:** Per pack:
- pack-kr: KR (ap-seoul-1)
- pack-eu: EU (eu-frankfurt-1 + eu-amsterdam-1 DR)
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1 DR)
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs for GDPR-scope tenants per Arts. 44-46. LLM-assist routing inherits pack residency: pack-eu tenants route to EU-resident LLM providers only.

### 2.3 Context

- **Data subjects:** End-users of tenant applications (referenced in spec content); tenant operators (admins authoring specs); tenant developers; oyatie operators.
- **Relationship:** Joint controllership with tenant per GDPR Art. 26; tenant DPA `legal/dpa-template.md`.
- **Reasonable expectations:** Tenants expect workflow authoring per service contract; end-users referenced in specs expect operational data processing per tenant's privacy notice.
- **Previous experience:** Bominal predecessor workflow authoring tool operated under same pattern; no DPA-triggered complaints in 24mo.
- **Industry codes:** OpenTelemetry semantic conventions for editor-session tracing.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Provide tenant visual workflow authoring | Necessary for contracted service | Art. 6(1)(b) contract |
| Persist editor session state for resume + collab | Necessary for tenant UX | Art. 6(1)(b) contract |
| Emit canonical workflow_spec.v1.json to engine | Necessary for downstream workflow execution | Art. 6(1)(b) contract |
| Per-seat licensing enforcement | Contractual billing | Art. 6(1)(b) contract |
| LLM-assist drafting (optional; per-tenant opt-in) | Optional product feature | Art. 6(1)(a) consent + Art. 6(1)(b) contract |
| Replay-debugger frontend rendering | Contracted feature | Art. 6(1)(b) contract |
| Per-jurisdiction overlay rendering | Necessary for multi-pack tenant UX | Art. 6(1)(c) legal obligation (data minimisation by overlay) |
| Audit retention | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) legal obligation |
| Cross-tenant analytics (anonymised; future) | Optional product improvement | Art. 6(1)(f) + DP analysis |
| Marketing / unrelated commercial use | NOT a purpose | N/A — excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representatives | Scheduled pre-GA | Feedback folded into Step 6 |
| Data subjects (tenant end-users referenced in specs) | Indirect via tenant onboarding | Joint-controllership upstream-disclosure |
| Supervisory authority | Art. 36 prior consultation: NOT triggered (no residual high risk after mitigations; §6 + §7) | If residual > Medium, triggered |
| ops-security | YES — co-author of `threat-model.md` | Threat model + DPIA share residual risk |
| Engineering teams | YES | SDK redactor + `data_class` annotations enforced at CI |
| External auditor | At first audit cycle | Cross-references this DPIA |
| EU AI Act notified body (if LLM-assist used in regulated domain) | Conditional | Conformity assessment required pre-deployment |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary? | YES — workflow authoring + cross-product orchestration spec emission cannot be performed without editor session processing. |
| Less intrusive alternative? | Considered: text-only spec editor (no visual canvas; no LLM-assist). Rejected: business power user persona (the largest tenant segment) cannot use text-only. Current design uses visual canvas with PII redaction + data_class markers. |
| Is processing proportionate? | YES — collection limited to: editor session state (active drafts), CRDT ops (transient), save events (audit). Spec drafts classified at field level via `data_class`. SDK redactor strips PII for LLM-assist. Per Art. 5(1)(c) data minimisation. |
| Public / private substantial interest? | YES — operational reliability of tenant workflow authoring; legitimate interest in tenant DPA. |
| Could anonymised data work? | PARTIALLY — pseudonymisation (hashed tenant-id in CDN cache; OIDC sub for author identity) applied. Full anonymisation prevents per-seat licensing + audit. Pseudonymisation is the proportionate compromise. |
| Lawful basis | Identified per purpose in §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care) + HIPAA BAA. pack-kr sensitive data: PIPA Art. 23(2) (explicit consent at tenant onboarding). LLM-assist of sensitive content: tenant explicit opt-in + per-session consent. |
| Transfer basis (Arts. 44-46) | Per §2.2: SCC-only; default pack residency. LLM-assist routes to pack-resident provider by default. |
| Retention | Per asset class in `threat-model.md` §"Assets". HIPAA pack: ≥ 6y for audit. PIPA: erasure-on-request via DSR cascade. |
| Rights of data subjects | Honoured per §6: access (Art. 15), rectification (16), erasure (17), restriction (18), portability (20), objection (21), automated-decision-protections (22). |
| AI Act 2024 (LLM-assist) | Conditional high-risk classification per Annex III; conformity assessment + transparency obligations apply when LLM-assist drafts specs in regulated domain. |

## Step 5 — Identify and assess risks

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | XSS injection in editor surface exposes session token + cross-tenant draft access | M | H | **H** |
| R-02 | Cross-tenant editor session leak (rival tenant infers business signal from draft contents) | L-M | H | **H** |
| R-03 | LLM-assist prompt leaks end-user PII to LLM provider | H | H | **H** |
| R-04 | Per-seat license attribution retention exceeds end-user consent for audit | M | M | **M** |
| R-05 | Sub-processor breach exposes editor session state | L | H | **M** |
| R-06 | Spec author misconfigures node config; end-user PII surfaced in spec without classification | M | M-H | **M-H** |
| R-07 | End-user DSR (right-to-erasure) incomplete because data spread across drafts / LLM prompts / editor sessions | M | M | **M** |
| R-08 | Joint-controllership confusion: tenant doesn't disclose Studio processing to its end-users | M-H | M | **M-H** |
| R-09 | Cross-border transfer of EU-resident data via mis-routed LLM-assist provider | L | H | **M** |
| R-10 | Children's data processed without parental consent (pack-in; spec contains child data) | L | H | **M-H** |
| R-11 | PHI processed without BAA (pack-us-healthcare; tenant drafts clinical workflows) | M | H | **H** |
| R-12 | Hashed tenant-id re-identified via small-tenant auxiliary data | L | M | **L-M** |
| R-13 | LLM-assist hallucinates spec containing invented end-user identifiers | M | M | **M** |
| R-14 | Collab participant accidentally exposed to other participant's authoring intent through CRDT op visibility | L-M | M | **L-M** |
| R-15 | Operator JIT elevation abused to read tenant drafts | L | H | **M** |
| R-16 | Per-tenant branding mid-render exploited for cross-tenant XSS (anti-pattern violation) | L | H | **M** |
| R-17 | EU AI Act non-conformity if LLM-assist used in regulated domain without conformity assessment | L (early) → M (post-AI-Act-enforcement) | H | **M-H** |
| R-18 | CDN purge gap serves stale WASM with known vulnerability | M | M | **M** |
| R-19 | Editor session resume after long disconnect surfaces stale data classification | L | M | **L-M** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE/LINDDUN threat in `threat-model.md`.

## Step 6 — Measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (XSS) | Strict CSP + Trusted Types + Leptos virtual-DOM only + LEAN XSS-vector-scan + annual pen-test | L | axis-workflow + ops-security |
| R-02 (cross-tenant leak) | Citus partition + RLS + per-tenant SDK key + LEAN check + annual pen-test + weekly threat hunt | L | ops-security |
| R-03 (LLM PII leak) | SDK PII redactor + tenant disclosure + BYO-LLM option + zero-retention LLM models preferred + audit-emission | M (residual; redactor heuristic) | axis-workflow + council-privacy + foundry-providers |
| R-04 (license retention) | Retention bounded; audit-chain provides forensic vs operational distinction; DSR cascade for end-user erasure | L | council-privacy + tenancy |
| R-05 (sub-processor breach) | Sub-processor list at `legal/sub-processors.md`; per-vendor DPA; quarterly review | M (sub-processor risk irreducible) | council-privacy |
| R-06 (spec misconfig) | data_class markers visible in visual canvas; LEAN check `oya-check-data-class`; Cedar policy preview before save; tenant template library with vetted defaults | L-M | axis-workflow + council-privacy |
| R-07 (DSR cascade incompleteness) | DSR runner scans editor sessions + LLM prompts + Postgres + audit-chain; 30-day SLA | M (best-effort within retention) | council-privacy |
| R-08 (joint-controllership) | Tenant DPA mandates upstream disclosure; onboarding checklist verifies disclosure; non-disclosure = onboarding refused | L-M | council-privacy + gtm-customer-success |
| R-09 (cross-border LLM mis-route) | Pack-pinned LLM-assist routing; foundry-providers enforces; integration test verifies routing | L | axis-workflow + foundry-providers |
| R-10 (children's data) | Tenant DPA child-data clause; engine inherits tenant's age-gating | L | council-privacy |
| R-11 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before tenant ingest enabled | L | council-privacy + sales-legal |
| R-12 (tenant-id re-id) | Salted hash; salt rotated 12mo; audit-chain notes rotation | L | ops-security |
| R-13 (LLM hallucinates identifiers) | LLM completion validated against canonical schema; user explicit-accept before save; audit-chain emission | M (residual; advisory feature) | axis-workflow + council-privacy |
| R-14 (CRDT op visibility) | Collab participants visible only with explicit invite; per-(tenant, definition) lease isolates | L | axis-workflow |
| R-15 (operator override) | 2-person rule + audit chain + read-pattern anomaly alert | L | ops-security |
| R-16 (mid-render branding) | **FORBIDDEN by anti-pattern policy**; LEAN check enforces; subsequent-to-GA-tier-promotion marketplace iframed-sandbox only | L | council-design-system + ops-security |
| R-17 (AI Act non-conformity) | Pre-deployment conformity assessment when LLM-assist enters regulated-domain workflow; transparency UI in editor; opt-in consent | L | council-privacy + axis-workflow + sales-legal |
| R-18 (CDN purge gap) | CDN purge SLI; versioned bundle path; browser-side version pin | L | cloud-iac + axis-workflow |
| R-19 (stale classification on resume) | Session resume re-evaluates classification; warning banner if descriptor changed during disconnect | L | axis-workflow |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| ISO (ops-security chair) | `pending` | TBA |
| µservice owner (axis-workflow lead) | `pending` | TBA |
| council-architecture chair | `pending` | TBA |
| council-design-system chair | `pending` | TBA |

**DPO advice:** Residual risks after mitigations: most are L or M; R-03 LLM-assist prompt leakage holds at M residual due to heuristic redactor; R-11 PHI handling at H residual is conditional on BAA enforcement. Art. 36 prior consultation NOT triggered absent BAA breach. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-03 (LLM-assist redactor effectiveness).
- Annual DPIA review.
- Re-trigger on pack activation, LLM provider change, or AI Act enforcement milestone.

**Outcomes documented:**
- Records-of-processing register (Art. 30): `microservices/workflow-studio/legal/ropa.md`.
- Joint-controllership template: `microservices/workflow-studio/legal/dpa-template.md`.
- AI Act 2024 conformity assessment record (when applicable): `microservices/workflow-studio/legal/ai-act-conformity.md`.

## Per-Pack Overlay Sections

### pack-kr

PIPA Art. 33 + Enforcement Decree Art. 35 require DPIA-equivalent for systems processing sensitive PII at scale; this document fulfils.

- PIPA Art. 23: hashed tenant-id treated as sensitive when correlated.
- PIPA Art. 23-2: KR sensitive data stays in pack-kr; LLM-assist routes KR-resident provider only.
- PIPA Art. 28: telemetry retention bounded.
- PIPA Art. 29: cross-mapped to 12 safeguards in `compliance.md`.
- PIPC Notice 2020-7: this DPIA follows 7-step methodology.
- PIPA Art. 33-2: council-privacy chair serves DPO role for KR-resident tenants.

### pack-us-healthcare

HIPAA risk analysis (§164.308(a)(1)(ii)(A)); this document fulfils.

- §164.502(a) TPO: workflow authoring falls under Operations.
- §164.502(b) Minimum Necessary: SDK redactor + data_class markers enforce minimum-necessary on draft fields.
- §164.504(e) Business Associate: oyatie operates as BA; BAA at `legal/baa-template.md`.
- §164.310 Physical: inherited from cloud-k8s + HIPAA-eligible OCI regions.
- §164.312(b) Audit Controls: Ed25519 audit-chain + retention ≥ 6y for HIPAA-tagged tenants.
- §164.404 Notification: breach notification chain in `incident-response.md`.

### pack-eu

GDPR Art. 35 DPIA for EU-resident tenant processing.

- EDPB Guidelines 4/2019 (Art. 25): explicit alignment in §4 + §6.
- EDPB Guidelines 9/2022 (breach notification): 72h notification.
- NIS2: 24h/72h/1mo timelines apply when thresholds crossed.
- eIDAS 910/2014: Ed25519 audit-chain seals as AdES.
- Schrems II + Arts. 44-46: no cross-border transfer without SCC; LLM-assist routes EU-resident provider.
- EU AI Act 2024: LLM-assist used in regulated domain triggers conformity assessment + transparency obligations.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/workflow-studio-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- New pack activation.
- LLM-assist provider change.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- EU AI Act enforcement milestone.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028 (Bominal): Audit chain.
- ADR-0065: Leptos for browser UI.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- ADR-0140: Cedar policy enforcement.
- ADR-0164 (Bominal): Workflow canonical spec format.
- `microservices/workflow-studio/threat-model.md`.
- `microservices/workflow-studio/policy/data-residency.md`.
- `microservices/workflow-studio/policy/editor-isolation.md`.
- `microservices/workflow-studio/compliance.md`.
- `microservices/workflow-studio/incident-response.md`.
- `microservices/workflow-studio/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa, ai-act-conformity}.md`.
- ICO DPIA template; CNIL DPIA methodology.
- EDPB Guidelines 4/2019 + 9/2022.
- PIPC Notice 2020-7.
- GDPR Arts. 35 + 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- EU AI Act 2024 Annex III + Art. 9 + Art. 13.
- LGPD Art. 38.
- DPDPA 2023 §10-§11.
