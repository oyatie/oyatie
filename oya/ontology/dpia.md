---
doc_class: DPIA
template_id: TPL-DPIA
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-ontology
deciders: council-privacy, ops-security, axis-ontology, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0006, ADR-0028, ADR-0055, ADR-0056, ADR-0059, ADR-0106, ADR-0107, ADR-0117, ADR-0122, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/ontology.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/ontology/threat-model.md
  - microservices/ontology/policy/type-isolation.md
  - microservices/ontology/policy/data-residency.md
  - microservices/ontology/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (Ontology stores typed entity history of subjects across products + LLM agent gateway makes semi-automated decisions)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI in pack-us-healthcare; sensitive data under PIPA Art. 23; financial data under pack-kr-FSS)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34, A.5.31"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33, 34", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019 (Art. 25)", "EDPB Guidelines 9/2022 (breach notification)", "EU AI Act 2024/1689 Arts. 9-15 (high-risk AI)"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "OAIC guidelines"]
  pack-in: ["DPDPA 2023 §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38 (RIPD; ANPD methodology)"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["KSA PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: ontology µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The ontology µservice triggers two of three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Ontology persists typed entity history (Patient, Customer, Employee, etc.) across every product; the agent gateway invokes Functions on behalf of LLMs producing semi-automated decisions affecting data subjects. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional)** | pack-us-healthcare carries PHI in Object Types; pack-kr KR PIPA Art. 23 sensitive data; pack-kr-FSS financial data. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Ontology does not monitor public-area cameras. |

Also: PIPC Notice 2020-7 (KR) mandates a DPIA for sensitive personal information processing at scale; engaged for pack-kr.

The EU AI Act 2024/1689 (Arts. 9–15) applies when the agent gateway is used in a high-risk AI context (e.g., access to health, employment, education entity types); engaged conditionally for pack-eu.

Therefore: a DPIA is mandatory pre-deployment.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** Ontology persists typed Object Type instances (entities) across every oyatie product; persists Link Type edges; invokes Action Types via Cedar gate; evaluates Function reads; mirrors history to ClickHouse for OLAP queries; emits Merkle/Ed25519 audit-chain seals; dispatches LLM tool-calls via agent gateway.

**How:** SDK / REST → Cedar policy gate → Postgres + Citus (RLS) + outbox → Kafka → ClickHouse + audit-chain Merkle tree → Ed25519 seal via OpenBao Transit.

**Where:** Per-pack region-pinned Postgres + Citus + ClickHouse clusters (pack-kr → KR / pack-eu → EU / pack-us → US / etc.) running in shared substrate Kubernetes clusters on oyatie's `cloud-k8s` substrate. Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; write rate ~50 k Object Type writes/s per cell baseline; ~1 k Actions/s; ~10 k Function reads/s; LLM tool-calls vary by agent traffic.

**Who:** Per the actor table in `microservices/ontology/threat-model.md` §"Actors". External tenant operators; customer applications; workload µservices; LLM agents; CI runner; ontology workers; council operators; external auditors.

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Order/Payslip/Invoice/Workflow-run instances | Art. 6(1)(b) contract necessity | ~10⁹ Object Type writes/day per medium tenant |
| `PII_IDENTIFYING` | User/employee/customer Object Types | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation | varies by tenant emission |
| `PII_QUASI_IDENTIFIER` | Hashed national-id / hashed device-id in entity properties | Art. 6(1)(f) legitimate interest; pseudonymised at SDK level | per-tenant |
| `SENSITIVE_PIPA_ART23` | Korean residents' sensitive data; tenant identifier mapping | KR PIPA Art. 15 + 23 + 23-2 (explicit consent + sensitive processing) | per tenant request |
| `PHI` (pack-us-healthcare only) | Patient + Encounter + Medication + DiagnosticReport Object Types | HIPAA §164.502(a) Permitted Uses (TPO) per BAA | varies; per tenant |
| `AUDIT` | Action invocation receipts; audit-chain Merkle nodes; Ed25519 seals | Art. 6(1)(c) legal obligation | 1 per Action; ~10⁵/day per medium tenant |
| `SECRET` | Per-tenant API keys, Ed25519 signing keys | not personal data; ISO 27001 A.5.17 | varies |

**Geographical scope:** Per pack (see `multi-region.md`):
- pack-kr: KR (ap-seoul-1).
- pack-eu: EU (eu-frankfurt-1 + eu-amsterdam-1 DR pair).
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1).
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned.

**Cross-border transfer:** Forbidden by default. Allowed only with tenant SCCs (GDPR Arts. 44–46) recorded in `microservices/ontology/legal/transfer-register.md`.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications; tenant operators; oyatie operators (internal).
- **Relationship to data subjects:** Joint controllership with the tenant under GDPR Art. 26 (the tenant is controller of its end-users' data; oyatie is joint controller for the entity-management portion).
- **Reasonable expectations:** Tenant operators expect typed entity persistence; end-users expect operational data per the tenant's own privacy notice.
- **Previous experience:** Bominal Object Graph (predecessor; renamed to Ontology per ADR-0055 / ADR-0122) operated under the same processing pattern; no DPA-triggered complaints in 24 months.
- **Industry codes:** Voluntary alignment with FHIR (pack-us-healthcare) + OpenTelemetry semantic conventions reduces ambiguity.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Persist typed entities for tenant products** | Necessary for tenant's contracted feature delivery | Art. 6(1)(b) contract |
| **Cross-product entity sharing via Functions** | Necessary for product composition (ADR-0059 adapter layer) | Art. 6(1)(b) + Art. 6(1)(f) legitimate interest |
| **Audit-chain Merkle/Ed25519 seal emission** | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) legal obligation |
| **Agent gateway: LLM tool-call dispatch on behalf of subject's data** | Tenant-contracted feature; subject-aware (autonomy tier ceiling) | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest (with Art. 22 carve-out) |
| **DSR cascade (right-to-erasure)** | Required by GDPR Art. 17 + PIPA Art. 36 + DPDPA §12 + LGPD Art. 18(V)-(VI) | Art. 6(1)(c) legal obligation |
| **Cross-tenant aggregations (DP-noised; future)** | Optional product-improvement | Art. 6(1)(f) + DP analysis in `policy/dp-analysis.md` |
| **Marketing / unrelated commercial use** | NOT a purpose | N/A |

The purposes are explicit, legitimate, and specified at the point of tenant onboarding via the DPA template.

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representative (sample of 3 prospective tenants) | Scheduled — pre-GA | Feedback to be folded into Step 6 |
| Data subjects (end-users) | Indirect via tenant onboarding | Joint-controllership clause |
| Supervisory authority (EU DPA / KR PIPC) | Prior consultation (Art. 36) — NOT triggered (residual risks ≤ Medium after mitigations) | If residual > Medium, Art. 36 triggered |
| Information security team (ops-security) | YES — co-author of threat-model.md | Shared residual-risk catalog |
| Engineering teams (axis-ontology + workload µservices) | YES | SDK + Cedar fragments + LEAN lanes |
| External auditor (SOC 2 / ISO 27001 firm) | At first audit cycle | Cross-references this DPIA |

DPO independent advice + sign-off recorded at §7.

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary to achieve the purpose? | YES — typed entity persistence + cross-product sharing cannot be performed without Ontology. |
| Is there a less intrusive alternative? | Considered: per-product private DBs without cross-product adapter. Rejected: violates ADR-0059 cohesion thesis + duplicates compliance surfaces. Current design centralises governance. |
| Is processing proportionate to the purpose? | YES — per-property tier classification + Cedar gate + Function projection limits exposure to minimum necessary. |
| Does processing achieve a public interest or substantial private interest? | YES — operational reliability + audit-grade provenance + DSR honour. |
| Could the purpose be achieved by anonymised / pseudonymised data? | PARTIALLY — `tenant_id` is hashed; subject identifiers can be hashed within Object Types per Bominal ADR-0008. Full anonymisation defeats tenant-contracted feature. Pseudonymisation is the proportionate compromise. |
| Lawful basis (Art. 6) | Per purpose in §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h); pack-kr sensitive data: PIPA Art. 23(2). |
| Transfer basis (Arts. 44–46) | SCCs only per §2.2. |
| Retention | Per asset class in `threat-model.md` §"Assets". |
| Rights of data subjects | Honoured per §6 mitigations. |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-tenant Object Type leak | L-M | H | **M-H** |
| R-02 | Cross-pillar leak (org-pillar data accessed via person-pillar context) | M | H | **H** |
| R-03 | Property-tier escape (Tier1Sensitive exposed via Function projection) | M | H | **H** |
| R-04 | LLM agent gateway leaks subject data to LLM-provider context | M | H | **H** |
| R-05 | Long retention enables surveillance pattern (longitudinal profile) | M | M-H | **M-H** |
| R-06 | Automated Action invocation affects subject (e.g., auto-discharge) | M | H | **H** |
| R-07 | Audit-chain seal tampered / lost → no provenance of subject's history | L | H | **M** |
| R-08 | DSR erasure incomplete because retention spread data across Object Types + Links + audit chain | M | M | **M** |
| R-09 | Joint-controllership confusion (tenant doesn't disclose oyatie's processing to end-users) | M-H | M | **M-H** |
| R-10 | Re-identification via auxiliary data on hashed subject_id | L | M | **L-M** |
| R-11 | Cross-border transfer of EU-resident data via misrouted Object Type write | L | H | **M** |
| R-12 | Children's data (DPDPA §9) processed without parental consent | L | H | **M-H** |
| R-13 | PHI processed without BAA (pack-us-healthcare) | M | H | **H** |
| R-14 | EU AI Act high-risk classification breached (agent gateway dispatches Function affecting health/employment access) | L-M | H | **M-H** |
| R-15 | Cross-tenant Link Type permits subject's data to traverse to unrelated tenant | L | H | **M** |

Cross-reference: every risk has at least one mitigation in §6 + corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (cross-tenant leak) | Postgres FORCE RLS + Citus strict mode + Cedar default-deny + LEAN runtime probe; annual pen-test | L | ops-security + axis-ontology |
| R-02 (cross-pillar leak) | `pillar.cedar` evaluator + 2-person rule for cross-pillar grants + audit-chain | L | axis-ontology + council-privacy |
| R-03 (tier escape) | Function projection tier filter + LEAN lane on every projection; max-tier ceiling claim | L | axis-ontology |
| R-04 (LLM context leak) | Tier-filtered tool-call payload + tenant opt-in for Tier1Sensitive + per-LLM DPA + agent-gateway rate limit | M (irreducible LLM-provider boundary) | axis-ontology + council-privacy |
| R-05 (long retention surveillance) | Retention matrix per asset class + DSR cascade + aggressive defaults | L-M | council-privacy |
| R-06 (automated Action) | Cedar autonomy_tier ceiling + 2-person rule for high-impact Actions + Art. 22 carve-out + tenant override available | L | axis-ontology + council-privacy |
| R-07 (audit-chain tampering) | Append-only triggers + Merkle verification + cross-µservice chain-of-chains + HSM-backed keys | L | axis-ontology + audit-chain |
| R-08 (DSR cascade gap) | Per-Object-Type scan + completeness manifest + 30d SLA + best-effort within retention windows | M | council-privacy + axis-ontology |
| R-09 (joint-controllership confusion) | DPA mandates upstream disclosure clause + onboarding checklist | L-M | council-privacy + gtm-customer-success |
| R-10 (re-identification) | Salted hash + per-tenant salt + 12mo rotation + audit-chain notes rotation | L | ops-security |
| R-11 (cross-border misroute) | Pack-pinning at SDK + integration test catches at CI; runtime detector | L | axis-ontology |
| R-12 (children's data) | Tenant DPA includes child-data clause + age-gating inheritance | L (residual depends on tenant) | council-privacy |
| R-13 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before ingest enabled | L | council-privacy + sales-legal |
| R-14 (EU AI Act high-risk) | Per-Function classification of high-risk usage + Cedar autonomy_tier strictness + log + transparency notice; per-tenant opt-in to high-risk Actions | L-M | council-privacy + axis-ontology |
| R-15 (cross-tenant link via grant) | Cedar CrossTenantLinkGrant + 2-person rule + audit-chained + data_class cap | L | ops-security + axis-ontology |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-ontology lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are M (R-04 LLM, R-07 retention, R-08 DSR, R-09 joint-controllership, R-14 AI Act); all considered acceptable given mitigations + tenant disclosure. Art. 36 prior consultation NOT triggered. DPO advises proceeding subject to:
- Quarterly review of R-04 (LLM context leak residual) — LLM-provider DPA updates.
- Annual review of this DPIA.
- Re-trigger on any pack-activation; each new pack engages distinct legal frameworks; pack-overlay sections must be filled prior to first-tenant in that pack.
- Re-trigger on any EU AI Act amendment relevant to agent gateway.

**Outcomes documented:**
- Mitigations adopted: every measure in §6 is in-scope for Slice A authoring of this phase.
- Records-of-processing entry (per GDPR Art. 30): `microservices/ontology/legal/ropa.md` (Slice D successor-IP).
- Joint-controllership template: `microservices/ontology/legal/dpa-template.md` (Slice D).

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 require a 개인정보영향평가 for systems processing sensitive personal information at scale. This document fulfils that obligation.

- **PIPA Art. 23 (sensitive personal information)**: hashed subject identifiers treated as sensitive when correlated with auxiliary data; salt-rotation (R-10).
- **PIPA Art. 23-2 (sensitive cross-border)**: KR-resident sensitive data stays in pack-kr cluster.
- **PIPA Art. 28 (storage period)**: bounded retention; non-essential data removed within statutory minimum.
- **PIPA Art. 29 (technical safeguards)**: §6 measures map directly.
- **PIPC Notice 2020-7**: this DPIA follows the prescribed 7-step methodology.
- **PIPA Art. 33-2 (DPO appointment)**: council-privacy chair serves PIPA DPO role.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk analysis substantially equivalent to a DPIA.

- **§164.502(a) TPO**: Operations scope covers Ontology entity management.
- **§164.502(b) (Minimum Necessary Standard)**: Function projection tier-filters PHI; Cedar permits per role.
- **§164.504(e) (Business Associate)**: oyatie operates as Business Associate; BAA template at `legal/baa-template.md`.
- **§164.310 (Physical Safeguards)**: inherited from cloud-k8s.
- **§164.312(b) (Audit Controls)**: audit-chain seal + retention ≥ 6y for HIPAA-tagged tenants.
- **§164.404 (Notification to Individuals)**: breach chain documented in `incident-response.md`.
- **45 CFR Part 164 Subpart D (Breach Notification)**: integrated.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act 2024/1689)

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing.

- **EDPB Guidelines 4/2019 (Art. 25)**: alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: chain in `incident-response.md`.
- **NIS2**: incident-reporting timelines (24h + 72h + 1mo).
- **eIDAS 910/2014**: Ed25519 seals are AdES.
- **Schrems II + Arts. 44–46 transfers**: no cross-border without SCCs + supplementary measures.
- **Children's data (Art. 8)**: inherited via tenant age-gating.
- **EU AI Act 2024/1689 Arts. 9–15 (high-risk AI)**: agent gateway classifies tenant Functions as high-risk if they touch health/employment/education/biometric/critical-infrastructure entities; tenant opt-in required; transparency notice mandated.

### pack-jp (APPI)

APPI does not mandate DPIA-equivalent but encourages voluntary risk assessment.

- **APPI Art. 17 (purpose of use)**: declared at tenant onboarding.
- **APPI Art. 21 (cross-border transfer)**: pack-jp residency.
- **APPI Art. 27 (sensitive data consent)**: tenant DPA captures explicit end-user disclosure.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/dpia-overlay.md` carry pack-specific legal-citation depth:

- **pack-sg (PDPA 2012)**: PDPA Part III + IV; MAS Notice 644.
- **pack-au (Privacy Act 1988 APP)**: APP 1–13; APP 8 + APP 11 + APP 12; APRA-CPS 234.
- **pack-in (DPDPA 2023)**: §10 + §11; §9 (children's data); §8.
- **pack-br (LGPD)**: Arts. 38 (RIPD); ANPD methodology.
- **pack-ae (UAE PDPL Federal Decree-Law 45/2021)**: Art. 23.
- **pack-ksa (KSA PDPL Royal Decree M/19/2021)**: Art. 9 + SAMA Cybersecurity Framework 2017.

## Re-review Triggers

- Annually (Q2 each year).
- On every new pack activation.
- On any change to processing purpose or data-class taxonomy.
- On any sub-processor change.
- On any breach notification triggered.
- On supervisory-authority guidance change.
- On any EU AI Act amendment affecting the agent gateway.
- Post-incident.

## References

- ADR-0006: Ontology typed-entity layer.
- ADR-0028 (Bominal): Audit chain.
- ADR-0055 + ADR-0122: Ontology rename.
- ADR-0059: Workflow + Ontology adapter layer.
- ADR-0106 (Bominal): Ontology architecture.
- ADR-0107 (Bominal): Agent gateway.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout.
- ADR-0132 (Bominal): Pillars.
- ADR-0140: Cedar policy enforcement.
- `microservices/ontology/threat-model.md`.
- `microservices/ontology/policy/type-isolation.md`.
- `microservices/ontology/policy/data-residency.md`.
- `microservices/ontology/compliance.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + 36; KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A); LGPD Art. 38; DPDPA 2023 §10–§11.
- EU AI Act 2024/1689 Arts. 9–15.
