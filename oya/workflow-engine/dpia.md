---
doc_class: DPIA
template_id: TPL-DPIA
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-workflow
deciders: council-privacy, ops-security, axis-workflow, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0035, ADR-0056, ADR-0103, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/workflow.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/workflow-engine/threat-model.md
  - microservices/workflow-engine/policy/data-residency.md
  - microservices/workflow-engine/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — YES (cross-µservice orchestration adapter processes every workload event)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare clinical workflows; sensitive data under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
doc_status: published
---

# Data Protection Impact Assessment: workflow-engine µservice

## Step 1 — Identify the need for a DPIA

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation | **YES** | The engine processes every typed event published by every µservice on behalf of every tenant; aggregation across runs constitutes systematic profiling of tenant behavior. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional on pack)** | pack-us-healthcare clinical workflows carry PHI in step payloads; pack-kr PIPA Art. 23 classes correlation-prone tenant signals as sensitive. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Engine does not monitor public area. |

KR PIPC Notice 2020-7 mandates DPIA when handling sensitive PII at scale — engaged.

DPIA is mandatory pre-deployment. This document is reviewed by EU DPAs (Art. 35) and KR PIPC (PIPA Art. 33) at first-tenant onboarding in each jurisdiction.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** The engine ingests typed events from every workload µservice; routes them through workflow specs as state machines + DAGs; persists durable run state in Postgres + Citus; emits per-run audit-chain seals; replays event log on cold-start or operator-initiated debug; emits Workflow events consumed by downstream subscribers.

**How:** Tenant SDK → engine REST (mTLS + OIDC) → spec compiled at submit time → run starts → step body executed in Wasmtime sandbox → state checkpointed to Postgres → audit-chain seal emitted → next event dispatched.

**Where:** Per-pack region-pinned engine clusters (pack-kr → KR / pack-eu → EU / pack-us → US / etc.); each pack has its own Postgres + Citus cluster, Valkey cluster, ClickHouse replica.

**When:** Continuous; sub-second event-to-action; per-run audit seal within 1s of completion.

**Who:** Per actor table in `threat-model.md`.

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant run starts, step status, retry counts | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest | ~10⁸ events/day per medium tenant |
| `PII_IDENTIFYING` | User-id fields in step payloads (when emitted by workload µservice) | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation (audit) | varies per tenant |
| `PII_QUASI_IDENTIFIER` | Request URLs/IPs embedded in step payloads | Art. 6(1)(f) legitimate interest; minimised at SDK | ~10⁶ payload fields/day per medium tenant |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id in topic namespace | KR PIPA Art. 15 + 23 + 23-2 | 1 per tenant request |
| `PHI` (pack-us-healthcare) | Patient identifiers / clinical data in clinical workflow payloads | HIPAA §164.502(a) (TPO) per BAA | targeted to 0 via SDK redactor; non-zero residual |
| `AUDIT` | Run-history seals; state transitions | Art. 6(1)(c) legal obligation | 1 per state transition |
| `SECRET` | Tenant SDK API keys | not personal data; ISO 27001 A.5.17 controls | — |

**Geographical scope:** Per pack:
- pack-kr: KR (ap-seoul-1)
- pack-eu: EU (eu-frankfurt-1 + eu-amsterdam-1 DR)
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1 DR)
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs for GDPR-scope tenants per Arts. 44-46.

### 2.3 Context

- **Data subjects:** End-users of tenant applications; tenant operators (admin users); oyatie operators.
- **Relationship:** Joint controllership with tenant per GDPR Art. 26; tenant DPA `legal/dpa-template.md`.
- **Reasonable expectations:** Tenants expect workflow execution per service-level contract; end-users expect operational data processing per tenant's privacy notice.
- **Previous experience:** Bominal workflow engine (predecessor) operated under same pattern; no DPA-triggered complaints in 24mo.
- **Industry codes:** OpenTelemetry semantic conventions for event tracing.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Execute tenant-authored workflow specs | Necessary for contracted service | Art. 6(1)(b) contract |
| Route cross-µservice events via adapter | Necessary for operational integrity of multi-product oyatie | Art. 6(1)(b) + 6(1)(f) legitimate interest |
| Persist run history for audit | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) legal obligation |
| Replay-as-debugger for tenant operators | Tenant-contracted feature | Art. 6(1)(b) contract |
| Cross-tenant analytics (anonymised; future) | Optional product-improvement use | Art. 6(1)(f) + DP analysis |
| Marketing / unrelated commercial use | NOT a purpose | N/A — excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representatives | Scheduled pre-GA | Feedback folded into Step 6 |
| Data subjects (tenant end-users) | Indirect via tenant onboarding | Joint-controllership clause carries upstream-disclosure |
| Supervisory authority | Art. 36 prior consultation: NOT triggered (no residual high risk after mitigations; §6 + §7) | If residual > Medium, triggered |
| ops-security | YES — co-author of `threat-model.md` | Threat model + DPIA share residual risk |
| Engineering teams | YES | SDK redactor + `data_class` annotations enforced at CI |
| External auditor | At first audit cycle | Cross-references this DPIA |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary? | YES — workflow execution + cross-product orchestration cannot be performed without event processing. |
| Less intrusive alternative? | Considered: synthetic-only execution (skip real-user payloads). Rejected: synthetic signal does not exercise real workflow logic. Current design uses real payloads but redacts PII at SDK level. |
| Is processing proportionate? | YES — collection limited to: typed events with `data_class` annotation; step payloads classified at spec authoring; SDK redactor strips `PII`-class fields. Per Art. 5(1)(c) data minimisation. |
| Public / private substantial interest? | YES — operational reliability of tenant workflows; legitimate interest in tenant DPA. |
| Could anonymised data work? | PARTIALLY — pseudonymisation (hashed tenant-id) applied at topic namespace boundary. Full anonymisation would prevent per-tenant SLO + audit; pseudonymisation is the proportionate compromise. |
| Lawful basis | Identified per purpose in §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care) + HIPAA BAA. pack-kr sensitive data: PIPA Art. 23(2) (explicit consent at tenant onboarding). |
| Transfer basis (Arts. 44-46) | Per §2.2: SCC-only; default residency by pack. |
| Retention | Per asset class in `threat-model.md` §"Assets". HIPAA pack: ≥ 6y for audit-relevant data. PIPA: erasure-on-request via DSR cascade. |
| Rights of data subjects | Honoured per §6: access (Art. 15), rectification (16), erasure (17), restriction (18), portability (20), objection (21), automated-decision-protections (22). |

## Step 5 — Identify and assess risks

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | PII leakage via step payload logging | M-H | H | **H** |
| R-02 | Cross-tenant run-state leak (rival tenant infers business signal) | L-M | H | **H** |
| R-03 | Long retention enables surveillance pattern across years | M | M-H | **M-H** |
| R-04 | Automated execution affects end-users (workflow-driven action they did not consent to) | M | M | **M** |
| R-05 | Sub-processor breach exposes step payloads | L | H | **M** |
| R-06 | Spec author misconfiguration exposes end-user data via overly-permissive event payload | M | M-H | **M-H** |
| R-07 | End-user DSR (right-to-erasure) incomplete because data spread across runs / event log / ClickHouse replica | M | M | **M** |
| R-08 | Joint-controllership confusion: tenant doesn't disclose engine processing to its end-users | M-H | M | **M-H** |
| R-09 | Cross-border transfer of EU-resident data via mis-pinned engine cluster | L | H | **M** |
| R-10 | Children's data processed without parental consent (pack-in) | L | H | **M-H** |
| R-11 | PHI processed without BAA (pack-us-healthcare; tenant ships clinical workflows) | M | H | **H** |
| R-12 | Hashed tenant-id re-identified via small-tenant auxiliary data | L | M | **L-M** |
| R-13 | Replay-debugger access by auditor pivots cross-tenant | L | H | **M** |
| R-14 | Side-effect step (HTTP POST to external) executes against wrong end-user during replay | L | H | **M** |
| R-15 | Operator override cancels production run affecting end-user | L | M-H | **L-M** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE/LINDDUN threat in `threat-model.md`.

## Step 6 — Measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (PII in payloads) | SDK PII redactor with `data_class` honour; sample rate 0.1% in prod payload logs; quarterly synthetic-PII drill; refuse spec submissions that classify obvious-PII as INTERNAL_ONLY | Residual M | axis-workflow + each spec author |
| R-02 (cross-tenant leak) | Citus partition + RLS; per-tenant SDK key; LEAN check `oya-governance-citus-rls-enforced`; annual pen-test; weekly threat hunt | L | ops-security |
| R-03 (long retention) | Retention per asset class explicit; aggressive defaults; DSR cascade honours Art. 17; cold-tier data aggregated | L-M | council-privacy |
| R-04 (automated execution affects end-users) | Engine is operational decision-maker, NOT solely-automated decision producing legal effects on data subjects per Art. 22; tenant retains supervisory role; operator pause/cancel available | L | axis-workflow |
| R-05 (sub-processor breach) | Sub-processor list at `legal/sub-processors.md`; per-vendor DPA; quarterly sub-processor review | M (sub-processor risk irreducible) | council-privacy |
| R-06 (spec misconfig) | Spec PR review by CODEOWNERS; LEAN check `oya-check-data-class` validates payload classification; tenant template library with vetted defaults | L-M | axis-workflow + council-privacy |
| R-07 (DSR incompleteness) | DSR cascade scans Postgres + outbox + ClickHouse for identifier; 30-day SLA; cold-tier search supported | M (best-effort within retention) | council-privacy |
| R-08 (joint-controllership) | Tenant DPA mandates upstream disclosure; onboarding checklist verifies disclosure-in-tenant-notice; non-disclosure = onboarding refused | L-M | council-privacy + gtm-customer-success |
| R-09 (cross-border misroute) | Pack-pinning enforced at engine cluster level; route by tenant pack tag; misroute = config error caught by integration test | L | axis-workflow |
| R-10 (children's data) | Tenant DPA includes child-data clause; tenant must affirm parental-consent; engine inherits tenant's age-gating | L | council-privacy |
| R-11 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before tenant ingest enabled | L | council-privacy + sales-legal |
| R-12 (tenant-id re-id) | Salted hash; salt rotated 12mo; audit-chain notes rotation; small-tenant cardinality protection | L | ops-security |
| R-13 (auditor mis-pivot) | Auditor JIT tokens tenant-scoped at debugger folder level; pen-test annually | L | ops-security |
| R-14 (side-effect during replay) | Side-effect ledger: replays ignore side-effecting steps unless explicit operator-requested re-execution; replay-flag carried through bus | L | axis-workflow |
| R-15 (operator override on end-user) | 2-person rule + audit chain + 30min recovery window for soft-cancel | L | ops-security |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| ISO (ops-security chair) | `pending` | TBA |
| µservice owner (axis-workflow lead) | `pending` | TBA |
| council-architecture chair | `pending` | TBA |

**DPO advice:** Residual risks after mitigations are all L or M (no H residuals). Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-01 (PII leakage).
- Annual DPIA review.
- Re-trigger on pack activation (each pack engages distinct frameworks).

**Outcomes documented:**
- Records-of-processing register (Art. 30): `microservices/workflow-engine/legal/ropa.md`.
- Joint-controllership template: `microservices/workflow-engine/legal/dpa-template.md`.

## Per-Pack Overlay Sections

### pack-kr

PIPA Art. 33 + Enforcement Decree Art. 35 require DPIA-equivalent for systems processing sensitive PII at scale; this document fulfils that obligation.

- PIPA Art. 23: hashed tenant-id treated as sensitive when correlated.
- PIPA Art. 23-2: KR sensitive data stays in pack-kr cluster.
- PIPA Art. 28: telemetry retention bounded; per asset table.
- PIPA Art. 29: cross-mapped to 12 safeguards in `compliance.md`.
- PIPC Notice 2020-7: this DPIA follows the 7-step methodology.
- PIPA Art. 33-2: council-privacy chair serves DPO role for KR-resident tenants.

### pack-us-healthcare

HIPAA risk analysis (§164.308(a)(1)(ii)(A)); this document fulfils that requirement.

- §164.502(a) TPO: workflow execution falls under Operations.
- §164.502(b) Minimum Necessary: SDK redactor enforces minimum-necessary on step payloads.
- §164.504(e) Business Associate: oyatie operates as BA; BAA at `legal/baa-template.md`.
- §164.310 Physical: inherited from cloud-k8s + HIPAA-eligible OCI regions.
- §164.312(b) Audit Controls: Ed25519 audit-chain + retention ≥ 6y for HIPAA-tagged tenants.
- §164.404 Notification: breach notification chain in `incident-response.md`.

### pack-eu

GDPR Art. 35 DPIA for EU-resident tenant processing.

- EDPB Guidelines 4/2019 (Art. 25): explicit alignment in §4 + §6.
- EDPB Guidelines 9/2022 (breach notification): 72h notification documented in `incident-response.md`.
- NIS2: when oyatie crosses thresholds, 24h + 72h + 1mo timelines apply.
- eIDAS 910/2014: Ed25519 audit-chain seals as AdES; Art. 26 satisfied.
- Schrems II + Arts. 44-46: no cross-border transfer without SCC.

### pack-jp

APPI voluntary DPIA per PIPC voluntary scheme.

- APPI Art. 17 (purpose of use): declared at tenant onboarding.
- APPI Art. 21 (cross-border): pack-jp residency in JP.
- APPI Art. 27 (sensitive data consent): tenant-of-tenant sensitive-data inheriting.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/dpia-overlay.md` carry pack-specific legal-citation depth. Each follows this document's 7-step structure substituting local laws:

- pack-sg: PDPA 2012 Part III + IV; MAS Notice 644 for financial.
- pack-au: Privacy Act 1988 APP 1-13; APRA-CPS 234.
- pack-in: DPDPA 2023 §10 + §11; §9 (children's data).
- pack-br: LGPD Art. 38 (RIPD); ANPD methodology.
- pack-ae: UAE PDPL 45/2021 Art. 23.
- pack-ksa: KSA PDPL M/19/2021 Art. 9.

## Re-review Triggers

- Annually (Q2).
- New pack activation.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028 (Bominal): Audit chain.
- ADR-0035 (Bominal): Workflow engine.
- ADR-0103 (Bominal): Workflow hexagonal migration.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- ADR-0140: Cedar policy enforcement.
- `microservices/workflow-engine/threat-model.md`.
- `microservices/workflow-engine/policy/data-residency.md`.
- `microservices/workflow-engine/compliance.md`.
- `microservices/workflow-engine/incident-response.md`.
- `microservices/workflow-engine/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md`.
- ICO DPIA template.
- CNIL DPIA methodology.
- EDPB Guidelines 4/2019 + 9/2022.
- PIPC Notice 2020-7.
- GDPR Arts. 35 + 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- LGPD Art. 38; ANPD methodology.
- DPDPA 2023 §10-§11.
