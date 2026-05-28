---
doc_class: DPIA
template_id: TPL-DPIA
microservice: tasks
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-tasks
methodology: ICO DPIA + CNIL DPIA + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act Annex IV
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145), ADR-TASKS-0006]
related_artifacts:
  - microservices/tasks/threat-model.md
  - microservices/tasks/policy/task-isolation.md
  - microservices/tasks/policy/data-residency.md
  - microservices/tasks/compliance.md
  - microservices/tasks/capabilities/T0-suggest.yaml
  - microservices/tasks/capabilities/T1-assist.yaml
  - microservices/tasks/capabilities/T2-auto.yaml
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or AI capability promotion (T0 → T1 → T2)
high_risk_triggers_engaged:
  - "GDPR Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (T1 priority-suggest + auto-categorise; T2 auto-assign profile task assignees)"
  - "GDPR Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in clinical-task assignment via pack-us-healthcare)"
  - "GDPR Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act Annex III §4 (employment-context auto-assign) — engages high-risk AI conformity-assessment pathway"
doc_status: published
---

# Data Protection Impact Assessment: tasks µservice

## Step 1 — Need for a DPIA

Tasks processes per-task content (titles, descriptions, custom fields), attendance state (assignee + watcher), dependency graphs (which person is bottlenecking which work), and emits operational decisions about work assignment that — in employment contexts — fall under EU AI Act Annex III §4. **All three relevant Art. 35(3) automatic triggers engaged** plus EU AI Act conformity-assessment trigger:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| GDPR Art. 35(3)(a) Systematic + extensive evaluation including profiling | YES | T1 priority-suggest + auto-categorise + T2 auto-assign profile task assignees + observe work patterns |
| GDPR Art. 35(3)(b) Large-scale special-category | YES (conditional) | Clinical-task assignment (pack-us-healthcare) carries PHI; HR-context task descriptions may carry GDPR Art. 9 categories |
| GDPR Art. 35(3)(c) Public-area monitoring | NO | — |
| **EU AI Act Annex III §4 (employment-context)** | **YES** | Auto-assign affecting work allocation = high-risk AI |
| KR PIPA Art. 33 영향평가 mandate | YES | Pack-kr sensitive personal information at scale |

DPIA + EU AI Act conformity-assessment mandatory pre-deployment. Reviewed by EU DPAs (Art. 35) + KR PIPC (Art. 33) at first-tenant onboarding per jurisdiction, **and** by an EU AI Act notified body before T2 auto-assign in employment-context goes live (per ADR-TASKS-0006).

## Step 2 — Describe the processing

### 2.1 Nature

**What:** Task CRUD with title/description/status/priority/assignee/due-date/labels/parent; project + view + dependency-edge + recurrence + custom-field; bulk-edit; cross-project search via Meilisearch; importers (CSV + Jira + Asana + Trello + Linear + Todoist); T0/T1/T2 AI capabilities (suggest, assist, auto); bidirectional workflow-engine bridge.

**How:** REST ingress → Postgres task-store (per-tenant RLS + tenant-DEK envelope encryption) → Valkey view-cache → Meilisearch per-tenant index → Workflow events to workflow-engine + audit-chain + mail + messenger + calendar + foundry-runtime + observability.

**Where:** Per-pack region-pinned Postgres + Valkey + Meilisearch (pack-kr → KR; pack-eu → EU; pack-us → US; pack-us-healthcare → BAA-eligible US; pack-jp → JP; etc.). Residency enforced via ADR-0117 + ADR-0140.

**When:** Continuous; on-demand for user actions; recurring background sweeps for retention + recurrence-expansion + search-index-rebuild + webhook-fanout + importer-runner.

**Who:** Per the actor table in `threat-model.md` §"Actors".

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PROFESSIONAL_TASK_CONTENT` | task title, description, custom-field values | Art. 6(1)(b) contract + 6(1)(f) legitimate interest | 10⁵ tasks/day per medium tenant |
| `PERSONAL_TASK_CONTENT` | personal task entries | Art. 6(1)(a) consent + 6(1)(b) | 10⁴/day per active user |
| `PII_IDENTIFYING` | assignee emails, watcher emails, person-typed custom-field values | Art. 6(1)(b) contract | 10× task count |
| `PII_BEHAVIORAL` | time-tracking ticks (M02-onward); work-pattern inferences | Art. 6(1)(a) explicit consent | per opt-in user |
| `PHI` (pack-us-healthcare only) | clinical-task assignment content under BAA | HIPAA §164.502(a) Permitted Uses | per BAA tenant |
| `AUDIT` | task lifecycle records | Art. 6(1)(c) legal obligation | 1 per task mutation |
| `SECRET` | tenant-DEK, API keys, Meilisearch master keys | not personal data | managed via OpenBao |
| **AI inference inputs / outputs (T1/T2)** | feature vectors, classification outputs, decision rationales | Art. 6(1)(f) legitimate interest + EU AI Act Annex IV documentation | per-T1/T2-invocation |

**Geographical scope:** per pack (per §2.1).

**Cross-border transfer:** forbidden by default; allowed with tenant-executed SCCs per Arts. 44–46 per `multi-region.md`.

### 2.3 Context

- **Data subjects:** end-users (the tenant's employees + assignees + watchers); tenant operators; oyatie operators (internal).
- **Relationship:** joint controllership with tenant (GDPR Art. 26) for end-user task data; oyatie sole processor for operational metadata. **For T2 auto-assign in employment-context, tenant is the controller; oyatie is the processor + AI-system-provider per EU AI Act.**
- **Reasonable expectations:** tenant operators expect operational task management; end-users (employees) expect work-allocation per tenant's privacy notice + employment contract; auto-assign in employment context requires explicit upstream disclosure.
- **Previous experience:** Bominal Tasks inheritance per ADR-0231-0233; no DPA-triggered complaints in inheritance period.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Task management | Contracted | Art. 6(1)(b) |
| Cross-task dependency tracking | Operational benefit; contractual | Art. 6(1)(f) + 6(1)(b) |
| T0 suggest (smart-time, title, room, agenda) | Optional benefit | Art. 6(1)(f) legitimate interest |
| T1 priority-suggest + auto-categorise | Optional benefit; opt-in | Art. 6(1)(a) consent + 6(1)(f) |
| **T2 auto-assign in employment-context** | **Operational benefit; OPT-IN + CONFORMITY-ASSESSED** | **Art. 6(1)(a) explicit consent + EU AI Act Annex III §4 conformity** |
| Cross-µservice bridges (create-task-from-{email,message,event}) | Operational benefit | Art. 6(1)(f) |
| Audit-chain emission | Records-of-processing (Art. 30) | Art. 6(1)(c) |
| Legal-hold preservation | Legal obligation | Art. 6(1)(c) |
| Marketing / unrelated commercial use | NOT a purpose | N/A |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Sample of prospective tenants | Scheduled pre-GA | Feedback folded into §6 |
| End-users (indirect via tenant) | Joint-controllership clause | Tenant disclosure obligation |
| Supervisory authority (DPA / PIPC / OCR / EU AI Act notified body) | Art. 36 prior consultation triggered for T2 auto-assign in pack-eu employment-context until conformity assessment complete | Held until ADR-TASKS-0006 conformity ADR |
| Information security (ops-security) | YES | Shared residual catalog with threat-model |
| Engineering (axis-tasks + each consuming µservice) | YES | LEAN gates enforced |
| External auditor (SOC 2 / EU AI Act notified body) | At first audit cycle | Cross-references DPIA |
| Workers' council (where tenant employment-context) | Tenant-level obligation (per Works Council Directive / 근로기준법 / German Mitbestimmung) | Surface to tenant onboarding checklist |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary? | YES — task management cannot occur without task content. T2 auto-assign is optional; T0/T1 are optional. |
| Less-intrusive alternative? | T1 + T2 considered against: pure human assignment (slower but no AI risk); manual rule-based assignment (limited expressiveness). T2 chosen with Cedar-gating + conformity assessment to permit narrow employment-context deployment. |
| Proportionate? | YES — minimum-necessary at type level; T2 limited to opt-in tenants + opt-in users + employment-context conformity-bound. |
| Anonymisation possible? | Partial — search index can use redacted projection; full anonymisation incompatible with assignment purpose. |
| Lawful basis | Per §2.4 |
| Special-category (Art. 9) | pack-us-healthcare: Art. 9(2)(h) (health care provision) + HIPAA BAA. HR-context task descriptions: Art. 9(2)(b) for legitimate-employment-purposes. |
| Transfer basis | SCCs only; default residency by pack. |
| Retention | per task / pack; HIPAA pack ≥ 6y; employment-context per `pack-kr` 근로기준법 Art. 41 = 3y minimum; default 24mo + per-tenant policy override. |
| Subject rights | Art. 15/16/17/18/20/21/22 honoured per §6. |
| **EU AI Act compliance (T2 employment-context)** | **REFUSED at Cedar layer until ADR-TASKS-0006 conformity-assessment ADR ships + Art. 14 reversibility window + Art. 50 user labelling wired** |

## Step 5 — Risks to data subjects

| ID | Risk | L | S | Score |
|---|---|---|---|---|
| R-01 | Personal-task content leaks into Professional-context query | M-H | H | **H** |
| R-02 | Cross-tenant search-index leak | M | H | **H** |
| R-03 | T2 auto-assign creates disparate-impact for protected class (employment-context) | M (without mitigation) / L (with conformity assessment + fairness audit) | H | **H pre-mitigation / L post-** |
| R-04 | Time-tracking ticks (M02-onward) enable employee surveillance | M | H | **H** |
| R-05 | Importer source forges assignee identity | M | H | **H** |
| R-06 | Webhook payload over-projection leaks data to public webhook URL | M | M | **M** |
| R-07 | Long retention enables surveillance pattern across years | M | M-H | **M-H** |
| R-08 | Automated recurring tasks create attendee notification storm | L | M | **L-M** |
| R-09 | DSR right-to-erasure incomplete due to recurring + legal-hold overlap | M | M | **M** |
| R-10 | Joint-controllership: tenant doesn't disclose oyatie's processing to end-users | M-H | M | **M-H** |
| R-11 | PHI processed without BAA (pack-us-healthcare tenant doesn't sign BAA but ships clinical tasks) | M | H | **H** |
| R-12 | Sub-processor breach (Postgres cluster operator / cloud provider / Meilisearch operator) | L | H | **M** |
| R-13 | Cross-border transfer of EU-resident task data via mis-routed REST ingress | L | H | **M** |
| R-14 | Foundry-runtime classifier inference leaks task content to model provider | L | H | **M** |
| R-15 | Tenant-DEK leaked via log → mass decryption | L | H | **M** |
| R-16 | Auditor mis-pivot across tenants | L | H | **M** |
| R-17 | T2 auto-assign deployed without works-council consultation (in jurisdictions that require it) | M | H | **H pre-onboarding-checklist** |
| R-18 | Dependency-graph cycle prevention algorithm DoS via deliberate complex graph | L | M | **L-M** |

Cross-reference: every risk has at least one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Risk-reducing measures

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 | Rust type-level Personal vs Professional separation; Cedar `task-isolation.cedar`; LEAN check `oya-check-context-isolation` | L | axis-tasks |
| R-02 | Per-tenant index name + master key prefix; LEAN check + property test on adapter; annual pen-test | L | ops-security |
| R-03 | T2 auto-assign in employment-context REFUSED at Cedar until ADR-TASKS-0006 conformity ADR ships; fairness audit per `slos/auto-assign-fairness-correctness.openslo.yaml`; bias-audit annually | L (post-mitigation) | axis-tasks + council-privacy |
| R-04 | Time-tracking opt-in per employee (not tenant); per-employee retention bounded; aggregation-only reporting except for the employee themselves + 2-person-rule admin | M (employment-context surveillance baseline) | council-privacy |
| R-05 | Importer assignee strict resolution (ADR-TASKS-0001); pre-import validator surfaces mapping for tenant-operator review | L | axis-tasks |
| R-06 | Per-webhook destination validation + circuit-breaker; LEAN check refuses webhook to public IP without tenant signature; per-subscription field-projection | L | axis-tasks |
| R-07 | Aggressive retention defaults; DSR cascade; cold-storage per-task access requires admin JIT | L-M | council-privacy |
| R-08 | Recurring horizon bounded at 5y; per-tenant recurrence rate-limit | L | axis-tasks |
| R-09 | DSR cascade with legal-hold overlap policy: erasure honoured except where hold; partial-erasure preserves task minus identifier | M (hold-vs-erasure tension accepted) | council-privacy |
| R-10 | Tenant DPA mandates upstream disclosure; tenant-onboarding checklist verifies; works-council consultation required where applicable | L-M | council-privacy |
| R-11 | pack-us-healthcare onboarding requires BAA pre-ingest; non-signed tenants pre-flighted to non-PHI pack | L | council-privacy |
| R-12 | Sub-processor list at `legal/sub-processors.md`; DPA + SCCs per sub-processor; quarterly review; Meilisearch operator under DPA | M (sub-processor risk irreducible) | council-privacy |
| R-13 | Pack-pinning at ingress; route by pack tag; LEAN check refuses cross-pack route | L | axis-tasks |
| R-14 | Foundry-runtime tenant-DEK-wrapped prompt; no cross-tenant training; private-inference posture per `T0-suggest.yaml` / `T1-assist.yaml` / `T2-auto.yaml` `private_inference` field | L | axis-tasks + foundry-runtime |
| R-15 | Secret-scanner CI lane; `Secret<T>` type strips Debug; 90d rotation; rotation event re-encrypts | M (human-error baseline) | ops-security |
| R-16 | Auditor JIT tokens tenant-scoped at row level; pen-test annually | L | ops-security |
| R-17 | Onboarding checklist for any tenant onboarding T2 auto-assign in employment-context includes works-council consultation evidence requirement | L | council-privacy + gtm-customer-success |
| R-18 | Bounded BFS per ADR-TASKS-0002 with 50ms p99 budget; per-tenant rate limit on dependency-edge writes | L | axis-tasks |

## Step 7 — Sign-off

| Sign-off | Status |
|---|---|
| DPO (council-privacy) | `pending` |
| Information Security Officer (ops-security) | `pending` |
| µservice owner (axis-tasks) | `pending` |
| Council-architecture | `pending` |
| **EU AI Act notified body (T2 auto-assign in employment-context)** | **Held until ADR-TASKS-0006 conformity-assessment ADR ships** |

**DPO advice:** Residual risks all L or M after mitigations for T0/T1 capabilities. T2 auto-assign in employment-context is HELD at Cedar layer pending EU AI Act conformity assessment. Art. 36 prior consultation NOT triggered for T0/T1; triggered for T2 employment-context. Proceed with first-tenant onboarding for T0/T1 subject to:
- Quarterly review of R-09 (DSR vs hold tension).
- Annual review of this DPIA.
- Re-trigger on each pack activation.
- Conformity assessment + DPO sign-off required before any tenant enables T2 auto-assign in employment-context.

## Per-Pack Overlays

### pack-kr (KR PIPA + 근로기준법 + 직장 갑질 + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 mandate 개인정보영향평가. This document fulfils that.

- **PIPA Art. 23 (sensitive)**: per-task sensitivity flag; flagged tasks carry additional access restrictions.
- **PIPA Art. 28 (storage period)**: retention bounded per asset table.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6.
- **PIPC Notice 2020-7 methodology**: Steps 1–7 align.
- **근로기준법 Art. 41**: employment-record retention 3y minimum; task assignment history is employment-record-adjacent for full-time-employee tenants → 1095d retention floor.
- **근로기준법 Art. 23 (anti-discrimination)**: auto-assign fairness audit; T2 auto-assign in pack-kr employment context REFUSED at Cedar layer pending fairness-audit + ADR-TASKS-0006 (analogous to EU AI Act refusal).
- **직장 갑질 protections**: T2 auto-assign of high-workload tasks REFUSED at Cedar layer.

### pack-us-healthcare (HIPAA + ADA + EEOC)

HIPAA §164.308(a)(1)(ii)(A) requires risk-analysis substantially equivalent to a DPIA. This document fulfils that.

- **§164.502(a) Permitted Uses (TPO)**: clinical task assignment falls under Treatment + Operations.
- **§164.502(b) Minimum Necessary**: cross-tenant search projection enforces at type level.
- **§164.504(e) BAA**: BAA template at `legal/baa-template.md`.
- **§164.310 Physical Safeguards**: inherited from cloud-k8s.
- **§164.312(b) Audit Controls**: Ed25519 audit-chain seal + retention ≥ 6y.
- **§164.404 Notification**: breach chain in `incident-response.md` ≤ 60-day window.
- **FDA 21 CFR Part 11**: clinical-task touches research subjects → audit-chain seal satisfies e-signature.
- **ADA 42 USC §12101 (accommodation tasks)**: WCAG 2.2 AA + accessibility audit.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

This document is the GDPR Art. 35 DPIA for EU tenant processing **and the EU AI Act Annex IV technical documentation pointer for T2 auto-assign**.

- **EDPB Guidelines 4/2019 (Art. 25)**: by-design + by-default verified in §4 of dpia.md.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain in `incident-response.md`.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo when threshold-engaged.
- **eIDAS 910/2014 Art. 26**: audit-chain Ed25519 seals satisfy AdES.
- **Schrems II + Arts. 44–46**: SCC-gated transfers only; transfer register kept.
- **GDPR Art. 22 (automated decision)**: T2 auto-assign in employment-context refused at Cedar layer until conformity ADR ships + Art. 14 reversibility window.
- **EU AI Act Annex III §4 (employment-context)**: T2 auto-assign REFUSED at Cedar layer pending ADR-TASKS-0006 + conformity assessment + notified-body audit + Art. 50 user labelling + Art. 14 human oversight (reversibility 30s) all wired.

### pack-us (CCPA + EEOC + Title VII + state AI laws)

- **CCPA + CPRA**: subject-rights per §6.
- **EEOC UGESP 1978 (29 CFR §1607) + Title VII**: T2 auto-assign in pack-us employment context REFUSED at Cedar layer until fairness-audit per `slos/auto-assign-fairness-correctness.openslo.yaml` is green + bias-audit annually.
- **NY Local Law 144 (AEDT)**: bias-audit for any automated-employment-decision-tool; T2 auto-assign refused for pack-us-NY until AEDT audit complete.
- **CO AI Act HB23-1041**: refused until disclosure + opt-out wired.
- **ADA 42 USC §12101**: WCAG 2.2 AA accessibility.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/tasks-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- On every new pack activation.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- Post-incident (Sev-1 or Sev-2).
- **AI capability promotion: T0 → T1 → T2** (mandatory).
- **EU AI Act conformity-assessment scope expansion** (mandatory).

## References

- ADR-0028 (Bominal), ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140, ADR-TASKS-0006.
- `microservices/tasks/threat-model.md`, `compliance.md`, `policy/*.cedar`, `multi-region.md`, `incident-response.md`, `legal/{dpa-template,baa-template,sub-processors,transfer-register,ropa}.md`, `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, `capabilities/T2-auto.yaml`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36; KR PIPA Art. 33; HIPAA 45 CFR §164.308.
- EU AI Act (EU) 2024/1689 — Annex III §4 + Annex IV + Art. 14 + Art. 50 + Art. 22.
- 근로기준법 Arts. 23/41; 직장 갑질 protections.
- EEOC UGESP 1978; Title VII; ADA; NY Local Law 144; IL AI Video Interview Act; CO AI Act HB23-1041.
- ISO 30414 (HR analytics).
- WCAG 2.2 AA.
- `microservices/calendar/dpia.md` — sibling reference template.
