---
doc_class: DPIA
template_id: TPL-DPIA
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-sheets
deciders: council-privacy, ops-security, axis-sheets, council-design-system, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act 2024 conformity assessment
related_adrs: [ADR-0028, ADR-0056, ADR-0065, ADR-0103, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-SHEETS-0005]
related_specs: [/specs/microservices/sheets.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/sheets/threat-model.md
  - microservices/sheets/policy/data-residency.md
  - microservices/sheets/compliance.md
  - microservices/sheets/policy/editor-isolation.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, AI-formula provider, or XLSX import library version
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — YES (Sheets processes every tenant cell-edit session)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare clinical workbooks; sensitive data under PIPA Art. 23; AI-formula prompts may carry end-user data; connected-sheets queries against external databases may pull PII)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act 2024 (high-risk system): conditional YES if AI-formula used for credit/insurance/employment scoring (Annex III); conformity-assessment in scope"
doc_status: published
---

# Data Protection Impact Assessment: sheets µservice

## Step 1 — Identify the need for a DPIA

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation | **YES** | Sheets processes every tenant cell-edit session; aggregation across sessions constitutes systematic profiling of tenant modelling behavior. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional on pack)** | pack-us-healthcare clinical workbooks carry PHI references; pack-kr PIPA Art. 23 classes correlation-prone editor signals as sensitive; AI-formula prompts may inherit end-user identifiers from tenant prose; connected-sheets queries against external databases may pull PII. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Sheets does not monitor public area. |
| EU AI Act 2024 (high-risk AI system) | **YES (conditional)** | AI-formula drafting in regulated domains (healthcare/finance/HR/credit/insurance) classifies under AI Act Annex III if formulas drive automated decisions about persons. Per ADR-SHEETS-0005: T2 cross-µservice scope treated as high-risk by default; T1 advisory may be high-risk depending on domain. Conformity assessment required. |

KR PIPC Notice 2020-7 mandates DPIA when handling sensitive PII at scale — engaged.

DPIA is mandatory pre-deployment. This document is reviewed by EU DPAs (Art. 35) and KR PIPC (PIPA Art. 33) at first-tenant onboarding in each jurisdiction, plus EU AI Act conformity-assessment when AI-formula used in regulated domain workflows.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** Sheets receives tenant authoring inputs (cell value edits, formula entries, drag-fill operations, conditional formatting rules, pivot configs, chart configs, prose for AI-formula); persists workbook + cell state; emits CRDT ops to collab participants; on save, emits a cell-edit event to the cell µservice + audit-chain seal; enforces per-seat licensing via Cedar; enforces per-range ACL via Cedar; renders per-jurisdiction overlays for data-class markers; runs recalc engine on dependency-graph; bridges to foundry-runtime for AI-formula + smart-fill + anomaly detection; bridges to external data sources via connected-sheets.

**How:** Tenant browser loads Leptos WASM bundle from CDN → OIDC tenant-binding established → editor REST issues workbook session → CRDT ops route through WebSocket gateway → cell-edit emits canonical cell event → cell µservice durably persists; recalc engine fires dependency-graph topological recalc on each edit.

**Where:** Per-pack region-pinned Sheets clusters; each pack has its own Postgres + Valkey + S3 + Arrow/Parquet substrate; CDN is global edge with per-pack cache keys.

**When:** On-demand; sub-second sheet-open; per-save audit seal within 1s of submit; recalc within seconds for 100k cells.

**Who:** Per actor table in `threat-model.md`.

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant authoring sessions, cell edits, save events | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest | ~10⁶ events/day per medium tenant |
| `PII_IDENTIFYING` | User-id columns in workbooks; AI-formula prose mentioning end-users; connected-sheets results | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation (audit) | varies per tenant |
| `PII_QUASI_IDENTIFIER` | Author OIDC `sub`; IP addresses in editor session metadata | Art. 6(1)(f) legitimate interest; minimised at SDK | ~10⁴ events/day per tenant |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id in CDN cache key | KR PIPA Art. 15 + 23 + 23-2 | 1 per tenant per CDN segment |
| `PHI` (pack-us-healthcare) | Patient identifiers in clinical-workbook columns; AI-formula prose | HIPAA §164.502(a) (TPO) per BAA | targeted near-zero via SDK redactor + per-range ACL; non-zero residual |
| `AUDIT` | Cell-edit events; license-gate evaluation events; sharing-change events; AI-formula invocations | Art. 6(1)(c) legal obligation | 1 per state transition |
| `SECRET` | Per-tenant SDK API keys; connected-sheets external-source credentials | not personal data; ISO 27001 A.5.17 | — |

**Geographical scope:** Per pack:
- pack-kr: KR (ap-seoul-1)
- pack-eu: EU (eu-frankfurt-1 + eu-amsterdam-1 DR)
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1 DR)
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs for GDPR-scope tenants per Arts. 44-46. AI-formula routing inherits pack residency.

### 2.3 Context

- **Data subjects:** End-users referenced in workbook cells; tenant operators (admins authoring workbooks); tenant analysts; oyatie operators.
- **Relationship:** Joint controllership with tenant per GDPR Art. 26; tenant DPA `legal/dpa-template.md`.
- **Reasonable expectations:** Tenants expect spreadsheet authoring per service contract; end-users referenced in cells expect operational data processing per tenant's privacy notice.
- **Previous experience:** Net-new µservice per ADR-0135; no Bominal predecessor; DPIA reviewed pre-launch.
- **Industry codes:** OpenTelemetry semantic conventions for editor-session tracing.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Provide tenant spreadsheet authoring | Necessary for contracted service | Art. 6(1)(b) contract |
| Persist workbook + cell state for resume + collab | Necessary for tenant UX | Art. 6(1)(b) contract |
| Emit cell-edit events to cell µservice + audit-chain | Necessary for downstream durability + audit | Art. 6(1)(b) contract + Art. 6(1)(c) legal obligation |
| Per-seat licensing enforcement | Contractual billing | Art. 6(1)(b) contract |
| Per-range ACL enforcement (ADR-SHEETS-0006) | Tenant data-access control | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest (defense-in-depth) |
| AI-formula drafting + smart-fill (optional; per-tenant opt-in) | Optional product feature | Art. 6(1)(a) consent + Art. 6(1)(b) contract |
| Connected-sheets external-source query (optional; tenant configures) | Optional product feature; tenant integrates | Art. 6(1)(b) contract |
| XLSX import/export (optional) | Optional product feature | Art. 6(1)(b) contract |
| Embed-bridge into docs + slides | Contracted feature | Art. 6(1)(b) contract |
| Trigger-bridge to workflow-engine | Optional product feature | Art. 6(1)(b) contract |
| Audit retention | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) legal obligation |
| Marketing / unrelated commercial use | NOT a purpose | N/A — excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representatives | Scheduled pre-GA | Feedback folded into Step 6 |
| Data subjects (tenant end-users referenced in workbooks) | Indirect via tenant onboarding | Joint-controllership upstream-disclosure |
| Supervisory authority | Art. 36 prior consultation: NOT triggered (no residual high risk after mitigations) | If residual > Medium, triggered |
| ops-security | YES — co-author of `threat-model.md` | Threat model + DPIA share residual risk |
| Engineering teams | YES | SDK redactor + `data_class` annotations enforced at CI |
| External auditor | At first audit cycle | Cross-references this DPIA |
| EU AI Act notified body (if AI-formula used in regulated domain) | Conditional | Conformity assessment required pre-deployment per ADR-SHEETS-0005 |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary? | YES — spreadsheet authoring + cell graph emission cannot be performed without editor session processing. |
| Less intrusive alternative? | Considered: text-only CSV editor (no formula engine; no AI-formula). Rejected: business power user persona cannot use text-only. Current design uses cell grid with PII redaction + data_class markers + per-range ACL. |
| Is processing proportionate? | YES — collection limited to: workbook + cell state, CRDT ops (transient), cell-edit events (audit). Cell-level data classification via `data_class`. SDK redactor strips PII for AI-formula. Per Art. 5(1)(c) data minimisation. |
| Public / private substantial interest? | YES — operational reliability of tenant spreadsheet authoring; legitimate interest in tenant DPA. |
| Could anonymised data work? | PARTIALLY — pseudonymisation (hashed tenant-id in CDN cache; OIDC sub for author identity) applied. Full anonymisation prevents per-seat licensing + audit. Pseudonymisation is the proportionate compromise. |
| Lawful basis | Identified per purpose in §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care) + HIPAA BAA. pack-kr sensitive data: PIPA Art. 23(2) (explicit consent at tenant onboarding). AI-formula of sensitive content: tenant explicit opt-in + per-session consent. |
| Transfer basis (Arts. 44-46) | SCC-only; default pack residency. AI-formula routes to pack-resident provider by default. |
| Retention | Per asset class in `threat-model.md` §"Assets". HIPAA pack: ≥ 6y for audit. PIPA: erasure-on-request via DSR cascade. |
| Rights of data subjects | Honoured per §6: access (Art. 15), rectification (16), erasure (17), restriction (18), portability (20), objection (21), automated-decision-protections (22). |
| AI Act 2024 (AI-formula) | Conditional high-risk classification per Annex III; conformity assessment + transparency obligations apply when AI-formula drafts in regulated domain; per ADR-SHEETS-0005. |

## Step 5 — Identify and assess risks

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | XSS injection in editor surface exposes session token + cross-tenant workbook access | M | H | **H** |
| R-02 | Cross-tenant workbook leak (rival tenant infers business signal from cell contents) | L-M | H | **H** |
| R-03 | AI-formula prompt leaks end-user PII to LLM provider | H | H | **H** |
| R-04 | Per-seat license attribution retention exceeds end-user consent for audit | M | M | **M** |
| R-05 | Sub-processor breach exposes workbook state | L | H | **M** |
| R-06 | Workbook author misconfigures cell config; end-user PII surfaced in cell without classification | M | M-H | **M-H** |
| R-07 | End-user DSR (right-to-erasure) incomplete because data spread across workbooks / AI prompts / connected-sheets / version-history | M | M | **M** |
| R-08 | Joint-controllership confusion: tenant doesn't disclose Sheets processing to its end-users | M-H | M | **M-H** |
| R-09 | Cross-border transfer of EU-resident data via mis-routed AI-formula provider | L | H | **M** |
| R-10 | Children's data processed without parental consent (pack-in; cell contains child data) | L | H | **M-H** |
| R-11 | PHI processed without BAA (pack-us-healthcare; tenant drafts clinical workbooks) | M | H | **H** |
| R-12 | Hashed tenant-id re-identified via small-tenant auxiliary data | L | M | **L-M** |
| R-13 | AI-formula hallucinates formula citing invented end-user identifiers | M | M | **M** |
| R-14 | Connected-sheets external-source credentials leaked via cell payload | L | H | **M** |
| R-15 | Operator JIT elevation abused to read tenant workbooks | L | H | **M** |
| R-16 | Per-tenant branding mid-render exploited for cross-tenant XSS (anti-pattern violation) | L | H | **M** |
| R-17 | EU AI Act non-conformity if AI-formula used in regulated domain without conformity assessment | L (early) → M (post-AI-Act-enforcement) | H | **M-H** |
| R-18 | CDN purge gap serves stale WASM with known vulnerability | M | M | **M** |
| R-19 | Editor session resume after long disconnect surfaces stale data classification | L | M | **L-M** |
| R-20 | XLSX upload malware compromises tenant browser (defeats gVisor) | L | H | **M** |
| R-21 | XLSX export of sensitive PII / PHI without tenant intent (export-discipline mishap) | M | M-H | **M-H** |
| R-22 | Per-range ACL misconfigured (operator grants broader access than intended) | M | M | **M** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE/LINDDUN threat in `threat-model.md`.

## Step 6 — Measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (XSS) | Strict CSP + Trusted Types + Leptos virtual-DOM only + LEAN XSS-vector-scan + annual pen-test | L | axis-sheets + ops-security |
| R-02 (cross-tenant leak) | Citus partition + RLS + per-tenant SDK key + LEAN check + annual pen-test + weekly threat hunt | L | ops-security |
| R-03 (AI-formula PII leak) | SDK PII redactor + tenant disclosure + BYO-LLM option + zero-retention LLM models preferred + audit-emission | M (residual; redactor heuristic) | axis-sheets + council-privacy + foundry-runtime |
| R-04 (license retention) | Retention bounded; audit-chain forensic vs operational distinction; DSR cascade for end-user erasure | L | council-privacy + tenancy |
| R-05 (sub-processor breach) | Sub-processor list; per-vendor DPA; quarterly review | M | council-privacy |
| R-06 (cell misconfig) | data_class markers visible in cell-grid; LEAN check `oya-check-data-class`; Cedar policy preview before share; tenant template library | L-M | axis-sheets + council-privacy |
| R-07 (DSR cascade) | DSR runner scans workbook cells + AI prompts + connected-sheets results + version-history + comments + audit-chain; 30-day SLA | M (best-effort within retention) | council-privacy |
| R-08 (joint-controllership) | Tenant DPA mandates upstream disclosure; onboarding checklist verifies | L-M | council-privacy + gtm-customer-success |
| R-09 (cross-border AI-formula mis-route) | Pack-pinned AI-formula routing; foundry-runtime enforces; integration test verifies | L | axis-sheets + foundry-runtime |
| R-10 (children's data) | Tenant DPA child-data clause; engine inherits tenant's age-gating | L | council-privacy |
| R-11 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before tenant ingest enabled | L | council-privacy + sales-legal |
| R-12 (tenant-id re-id) | Salted hash; salt rotated 12mo; audit-chain notes rotation | L | ops-security |
| R-13 (AI-formula hallucination) | Completion validated against formula-engine grammar; user explicit-accept before save; audit-chain emission | M (residual) | axis-sheets + council-privacy |
| R-14 (connected-sheets credentials) | External-source credentials NEVER stored in cell payload; OpenBao reference only; LEAN `oya-governance-no-secrets-in-cell-payload` | L | ops-security + axis-sheets |
| R-15 (operator override) | 2-person rule + audit chain + read-pattern anomaly alert | L | ops-security |
| R-16 (mid-render branding) | **FORBIDDEN by anti-pattern policy**; LEAN check enforces | L | council-design-system + ops-security |
| R-17 (AI Act non-conformity) | Pre-deployment conformity assessment per ADR-SHEETS-0005; transparency UI; opt-in consent | L | council-privacy + axis-sheets + sales-legal |
| R-18 (CDN purge gap) | CDN purge SLI; versioned bundle path; browser-side version pin | L | cloud-iac + axis-sheets |
| R-19 (stale classification on resume) | Session resume re-evaluates classification; warning banner | L | axis-sheets |
| R-20 (XLSX malware) | gVisor sandbox + ClamAV + OPSWAT + size cap + decompression-bomb detection; LEAN check enforces | L | ops-security + axis-sheets |
| R-21 (XLSX export PII) | ACL-aware export masking; export audit-chain; tenant DPA on export discipline; data-class markers in XLSX metadata | M (operator-discretion residual) | axis-sheets + council-privacy |
| R-22 (range ACL misconfig) | ACL UI preview before save; LEAN test corpus; quarterly ACL drift audit | L-M | axis-sheets + ops-security |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| ISO (ops-security chair) | `pending` | TBA |
| µservice owner (axis-sheets lead) | `pending` | TBA |
| council-architecture chair | `pending` | TBA |
| council-design-system chair | `pending` | TBA |

**DPO advice:** Residual risks after mitigations: most are L or M; R-03 AI-formula prompt leakage holds at M residual due to heuristic redactor; R-11 PHI handling at H residual is conditional on BAA enforcement; R-21 XLSX export PII at M residual is operator-discretion within ACL by design. Art. 36 prior consultation NOT triggered absent BAA breach. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-03 (AI-formula redactor effectiveness).
- Annual DPIA review.
- Re-trigger on pack activation, AI-formula provider change, or AI Act enforcement milestone.

**Outcomes documented:**
- Records-of-processing register (Art. 30): `microservices/sheets/legal/ropa.md`.
- Joint-controllership template: `microservices/sheets/legal/dpa-template.md`.
- AI Act 2024 conformity assessment record (when applicable): `microservices/sheets/legal/ai-act-conformity.md`.

## Per-Pack Overlay Sections

### pack-kr

PIPA Art. 33 + Enforcement Decree Art. 35 require DPIA-equivalent for systems processing sensitive PII at scale; this document fulfils.

- PIPA Art. 23: hashed tenant-id treated as sensitive when correlated.
- PIPA Art. 23-2: KR sensitive data stays in pack-kr; AI-formula routes KR-resident provider only.
- PIPA Art. 28: telemetry retention bounded.
- PIPA Art. 29: cross-mapped to 12 safeguards in `compliance.md`.
- PIPC Notice 2020-7: this DPIA follows 7-step methodology.
- PIPA Art. 33-2: council-privacy chair serves DPO role for KR-resident tenants.

### pack-us-healthcare

HIPAA risk analysis (§164.308(a)(1)(ii)(A)); this document fulfils.

- §164.502(a) TPO: spreadsheet authoring falls under Operations.
- §164.502(b) Minimum Necessary: per-range ACL + SDK redactor + data_class markers enforce minimum-necessary.
- §164.504(e) Business Associate: oyatie operates as BA; BAA at `legal/baa-template.md`.
- §164.310 Physical: inherited from cloud-k8s + HIPAA-eligible OCI regions.
- §164.312(b) Audit Controls: Ed25519 audit-chain + retention ≥ 6y for HIPAA-tagged tenants.
- §164.404 Notification: breach notification chain in `incident-response.md`.

### pack-eu

GDPR Art. 35 DPIA for EU-resident tenant processing.

- EDPB Guidelines 4/2019 (Art. 25): explicit alignment in §4 + §6.
- EDPB Guidelines 9/2022 (breach notification): 72h notification.
- NIS2: 24h/72h/1mo timelines apply when thresholds crossed.
- eIDAS 910/2014: Ed25519 audit-chain seals as AdES; signed XLSX exports per eIDAS Art. 26.
- Schrems II + Arts. 44-46: no cross-border transfer without SCC; AI-formula routes EU-resident provider.
- EU AI Act 2024: AI-formula used in regulated domain triggers conformity assessment + transparency obligations per ADR-SHEETS-0005.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/sheets-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- New pack activation.
- AI-formula provider change.
- XLSX import library version bump (calamine major version).
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- EU AI Act enforcement milestone.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028, 0056, 0065, 0103, 0117, 0126, 0130, 0131, 0140.
- ADR-SHEETS-0005 (AI-formula bounds).
- `microservices/sheets/threat-model.md`.
- `microservices/sheets/policy/data-residency.md`.
- `microservices/sheets/compliance.md`.
- `microservices/sheets/incident-response.md`.
- `microservices/sheets/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa, ai-act-conformity}.md`.
- ICO DPIA template; CNIL DPIA methodology.
- EDPB Guidelines 4/2019 + 9/2022.
- PIPC Notice 2020-7.
- GDPR Arts. 35 + 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- EU AI Act 2024 Annex III + Art. 9 + Art. 13.
- LGPD Art. 38.
- DPDPA 2023 §10-§11.
