---
doc_class: DPIA
template_id: TPL-DPIA
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-tenancy
deciders: council-privacy, ops-security, axis-tenancy, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 (개인정보영향평가)
related_adrs: [ADR-0018, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - tenancy/threat-model.md
  - tenancy/policy/rls-isolation.md
  - tenancy/policy/data-residency.md
  - tenancy/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (per-tenant lifecycle is systematic monitoring of tenant identity)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare; sensitive data under PIPA Art. 23; biometric / health data potentially in tenant operational state via downstream µservices)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 26, 28, 30, 32, 33, 34, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8, 2017 TSC)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 23-2, 24, 25, 28, 29, 29-2, 33, 33-2, 34, 36", "PIPA Enforcement Decree Art. 35 (DPIA mandatory criteria)", "PIPC Notice 2020-7 (DPIA methodology)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis)", "§164.312(b) (audit controls)", "§164.502(b) (minimum necessary)", "§164.514 (de-identification)"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019 on Art. 25", "EDPB Guidelines 9/2022 on breach notification"]
  pack-jp: ["APPI Arts. 17, 18, 21, 23, 24, 26-2, 27"]
  pack-sg: ["PDPA Part III (Protection) + Part IV (Retention)", "MAS Notice 644 (Technology Risk Management)"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "OAIC Privacy Impact Assessment guidance"]
  pack-in: ["DPDPA 2023 §10 (data fiduciary obligations) + §11 (DPIA-equivalent)"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 14 + 38 (RIPD)", "ANPD DPIA methodology"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23 (impact assessment)"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Data Protection Impact Assessment: tenancy µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The tenancy µservice triggers two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Per-tenant lifecycle is systematic; tenancy is the load-bearing authority on which tenant exists, when, where, with what jurisdiction — this evaluation drives the operational fate of every tenant. |
| Art. 35(3)(b): Large-scale processing of special-category data (Art. 9) | **YES (conditional)** | Pack-us-healthcare tenants process PHI (Art. 9(2)(h)); pack-kr handles KR PIPA Art. 23 sensitive data via hashed-tenant-id with auxiliary; oyatie's downstream µservices process biometric / health data via this tenant boundary. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | tenancy does not monitor public-area cameras / IoT. |

**Additionally:** the Korean PIPC's Notice 2020-7 mandates a DPIA when a processing system handles sensitive personal information (PIPA Art. 23) at scale — engaged. PIPA Art. 33 + Enforcement Decree Art. 35 require a 개인정보영향평가 — engaged.

Therefore: a DPIA is mandatory pre-deployment. **The tenancy DPIA is the most consequential of any µservice's DPIA in oyatie because a compromise here cascades to every other µservice's tenant data.** This document is the canonical DPIA reviewed by EU DPAs (per Art. 35), the Korean PIPC (per PIPA Art. 33), HIPAA OCR (per §164.308(a)(1)(ii)(A)), and equivalent supervisory authorities at first-tenant onboarding in every pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** tenancy ingests:
- Tenant lifecycle CRUD intents (create/activate/suspend/resume/delete) from platform-operators + tenant-operators.
- Tenant identifier ↔ hashed-tenant-id mapping (KR PIPA Art. 23 sensitive when combined with auxiliary data).
- JWT issuance + verification on every request to every µservice (the load-bearing tenant-identity primitive).
- RLS policy state (per-table predicates; declarative YAML + Postgres pg_policies live state).
- Cell-assignment state (tenant → Citus shard → cloud-cell).
- DSR (Data Subject Request) intents + per-µservice erasure receipts + proof-of-erasure Merkle roots.

**How:** OIDC + MFA → REST/SDK → tenant-lifecycle-rest → tenant-lifecycle-usecase → tenant-lifecycle-adapter-postgres → Postgres + Citus (RLS-enforced) → Workflow event emission (lifecycle events) → audit-chain Ed25519 seal → per-µservice Workflow consumption + state mutation.

**Where:** Per-pack region-pinned Postgres + Citus + Patroni clusters (pack-kr → KR / pack-eu → EU / pack-us → US / pack-jp → JP / etc.) running on oyatie's `cloud-k8s` substrate. Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; lifecycle mutations occur as platform-operator + tenant-operator actions; validate hot path runs on every µservice request (10⁴–10⁷ QPS); JWT signing-key rotation every 30d; DSR cascades on tenant request (regulator-bound SLA).

**Who:** Per the actor table in `tenancy/threat-model.md` §"Actors". External tenant operators; customer applications; platform operators; DPO (council-privacy chair); internal µservices; Workflow event consumers; external auditors; regulators.

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `SENSITIVE_PIPA_ART23` | `canonical_tenant_id` ↔ hashed `tenant_id` mapping (sensitive under KR PIPA Art. 23 when combined with auxiliary data) | KR PIPA Arts. 15 + 23 (sensitive personal info with explicit consent at tenant onboarding) | 1 per tenant; ~10⁶ total at scale |
| `BEHAVIORAL_TENANT_PRODUCT` | Tenant lifecycle history (created_at, suspended_at, resumed_at), plan tier, jurisdiction, cell assignment | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest (operational) | ~N rows per tenant lifecycle event |
| `PII_IDENTIFYING` | Tenant-operator OIDC subject (admin user identity); platform-operator audit-chain attribution | Art. 6(1)(c) legal obligation (audit / records-of-processing) | varies |
| `PHI` (pack-us-healthcare only) | Indirect: tenancy stores tenant identifier; PHI lives in downstream µservices but is *governed by tenancy's isolation primitives* | HIPAA §164.502(a) TPO via BAA | targeted to 0 direct; isolation-by-design |
| `AUDIT` | Lifecycle events + RLS policy installs + DSR receipts + proof-of-erasure certificates | Art. 6(1)(c) legal obligation (record-keeping); Art. 6(1)(f) legitimate interest | 1 record per state transition |
| `SECRET` | JWT signing keys, Postgres replication password, Patroni REST tokens | not personal data; ISO 27001 A.5.17 controls | varies |

**Geographical scope:** Per pack:
- pack-kr: KR (OCI ap-seoul-1).
- pack-eu: EU (eu-frankfurt-1 + eu-amsterdam-1 DR pair).
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1 DR pair); HIPAA Covered Entity tenants on BAA-eligible regions.
- pack-jp / pack-sg: JP / SG single-region.
- pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned to its primary region with DR pair.

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs (Standard Contractual Clauses) for GDPR-scope tenants per Arts. 44–46; recorded in `microservices/tenancy/legal/transfer-register.md`.

### 2.3 Context of the processing

- **Data subjects:** Tenant operators (admin users of the tenant; whose OIDC identities tenancy stores as audit attribution); platform operators (oyatie internal); the tenant entity itself (legal person; one tenant_id ↔ one organisation); indirectly, the tenant's end-users (whose data lives in downstream µservices governed by tenancy's isolation primitives — joint controllership per Art. 26).
- **Relationship to data subjects:** Direct controller for tenant-operator OIDC subject mapping + audit attribution; joint controllership with the tenant (under Art. 26) for the tenant's end-users.
- **Reasonable expectations:** Tenant operators expect operational metadata (per service-level contract). The tenant entity expects regulator-bound jurisdiction pinning. End-users (the tenant's customers) expect operational data isolation per the tenant's own privacy notice; oyatie's processing is disclosed in the tenant's notice via the joint-controllership transparency clause.
- **Previous experience:** Bominal tenancy (predecessor substrate per ADR-0018) operated under the same processing pattern; no DPA-triggered complaints in 24 months. Inherited lessons captured per `feedback_bominal_inheritance_precedence.md`.
- **Industry codes:** None directly applicable; voluntary alignment with the OpenSLO-shape RLS policy authoring pattern reduces ambiguity in data-class declaration.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Issue + verify tenant identity** (JWT issuance + verification) | Necessary for the tenant's contracted SLA (every µservice's request authentication) | Art. 6(1)(b) contract |
| **Provision tenant isolation** (RLS policy install + cell assignment) | Necessary for operational integrity; prevents cross-tenant data exposure | Art. 6(1)(b) + Art. 6(1)(f) legitimate interest |
| **Lifecycle management** (suspend / resume / delete) | Necessary for business operations + contractual fulfilment | Art. 6(1)(b) + Art. 6(1)(c) legal obligation (DSR / billing) |
| **DSR cascade execution** | Mandatory for GDPR Art. 17 / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18 | Art. 6(1)(c) legal obligation |
| **Audit-chain emission** | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 records-of-processing | Art. 6(1)(c) legal obligation |
| **Cross-pack residency enforcement** | Necessary for regulatory residency requirements | Art. 6(1)(c) + KR PIPA Art. 23-2 + GDPR Arts. 44–50 |
| **Marketing / unrelated commercial use** | NOT a purpose | N/A — explicitly excluded |

The purposes are explicit, legitimate, and specified at the point of tenant onboarding via the DPA template (Art. 5(1)(b) purpose-limitation).

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (DPO) | YES — council-privacy chair | Sign-off pending; see §7 |
| Tenant representative (sample of 3 prospective tenants for first-rollout) | Scheduled — pre-GA | Feedback folded into Step 6 |
| Data subjects (the tenants' end-users) | Indirect via tenant onboarding notices | Joint-controllership clause carries upstream-disclosure obligation |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Prior consultation (Art. 36) — **NOT triggered** (residual risks ≤ Medium after mitigations; see §6 + §7). If §6 mitigations residual > Medium, Art. 36 prior consultation triggered. | – |
| Information security team (ops-security) | YES — co-author of threat-model.md | Threat-model + DPIA share residual-risk catalog |
| Engineering teams (axis-tenancy + every workload µservice owner) | YES | DSR handler registration enforced at CI |
| External auditor (SOC 2 / ISO 27001 firm) | At first audit cycle | Cross-references this DPIA |

DPO independent advice + sign-off recorded at §7.

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary to achieve the purpose? | YES — multi-tenant SaaS cannot operate without identity + isolation primitives. |
| Is there a less intrusive alternative? | Considered: tenant-self-managed identity (each tenant brings its own JWT issuer). Rejected: would not provide the audit-chain or DSR cascade guarantees regulators require; would not enforce cross-pack residency. The current design pseudonymises the canonical tenant identifier and never stores tenant end-user data directly. |
| Is processing proportionate to the purpose? | YES — collection limited to: tenant lifecycle metadata; pseudonymised identifier mapping; audit records. End-user PII / PHI is NOT processed by tenancy directly (it lives in downstream µservices behind tenancy's isolation boundary). Per Art. 5(1)(c) data-minimisation. |
| Does processing achieve a public interest or substantial private interest? | YES — operational integrity of multi-tenant SaaS; legitimate interest documented in DPA template. |
| Could the purpose be achieved by anonymised / pseudonymised data? | PARTIALLY — pseudonymisation (hashed `tenant_id`) IS applied at the cross-µservice boundary. Full anonymisation would prevent per-tenant SLA authority + DSR cascade attribution + regulator-bound jurisdiction pinning, all of which require recoverable identity. Pseudonymisation is the proportionate compromise. |
| Lawful basis (Art. 6) | Identified per purpose in §2.4. |
| Special-category basis (Art. 9, if applicable) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care under contract with health professional) + HIPAA BAA covering §164.504(e). pack-kr sensitive data: PIPA Art. 23(2) (explicit consent) at tenant onboarding. |
| Transfer basis (Arts. 44–46) | Per §2.2 cross-border: SCCs only; default residency by pack. |
| Retention | Per asset class in `tenancy/threat-model.md` §"Assets & Data Classification". Defaults: tenant metadata 7y after deletion completes (covers DSR audit horizon + statutory minimum); audit records ≥ 1y default, ≥ 6y for HIPAA, ≥ 5y for KR-FSS, indefinite for proof-of-erasure (regulator-disclosable). |
| Rights of data subjects | Honoured per §6 mitigations: access (Art. 15), rectification (Art. 16), erasure (Art. 17), restriction (Art. 18), portability (Art. 20), objection (Art. 21), automated-decision-protections (Art. 22). |

## Step 5 — Identify and assess risks to data subjects

Risks below are scored on Likelihood (L/M/H) × Severity (L/M/H); Severity is from the perspective of the data subject, not oyatie.

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | RLS bypass exposes every tenant's data simultaneously (catastrophic; covers Art. 5(1)(f)) | L | H | **H** |
| R-02 | JWT signing-key compromise allows attacker to forge any tenant's identity | L | H | **H** |
| R-03 | Tenant operator's OIDC subject leaked via audit log to unauthorised auditor | L-M | M | **M** |
| R-04 | Cross-pack misroute violates residency (e.g., EU tenant data ends up in US cluster) | L | H | **M** |
| R-05 | Tenant deletion incomplete — residual data in some µservice exposes end-users post-DSR | M | M-H | **M-H** |
| R-06 | Hashed `tenant_id` re-identified via auxiliary data for very small tenant populations | L | M | **L-M** |
| R-07 | Tenant operator OIDC subject ↔ tenant_id correlated by malicious auditor | L | M | **L-M** |
| R-08 | Cell-assignment record reveals tenant business behavior via cell-load patterns | L | L | **L** |
| R-09 | Joint-controllership confusion — tenant doesn't disclose oyatie's processing to its end-users | M-H | M | **M-H** |
| R-10 | Children's data (DPDPA §9; pack-in) processed without parental consent at tenant DPA layer | L | H | **M-H** |
| R-11 | PHI processed without BAA (pack-us-healthcare; tenant ships clinical data without signing) | M | H | **H** |
| R-12 | Tenant deletion executed without verifiable proof — regulator cannot confirm Art. 17 fulfilment | L | H | **M** |
| R-13 | Auditor reads non-scoped tenant during engagement (Cedar policy misconfig) | L | M-H | **L-M** |
| R-14 | Patroni HA failure causes ≥ 1min validate-path outage → cascading lockouts | L | M | **L-M** |
| R-15 | DSR submission rate-limited; legitimate tenant DSR delayed beyond 30d SLA | L | M | **L-M** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE / LINDDUN threat in `tenancy/threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (RLS bypass) | `oya-governance-rls-no-superuser-bypass` + `rls-force-on-tenant-tables` CI lanes; Postgres role separation (no `bypassrls` on app role); continuous DB-state validator; 2-person rule for DBA JIT; weekly synthetic cross-tenant probe drill | L (multi-layer; bypass requires Postgres role compromise AND CI lane evasion AND state-validator evasion) | axis-tenancy + ops-security |
| R-02 (JWT-key compromise) | Ed25519 + OpenBao HSM-backed (where available); 30d rotation + 30d grace; fingerprint advertised via Workflow; alg-confusion-attack defended at verifier (T-T-02 mitigation) | L | ops-security |
| R-03 (audit-log leak to auditor) | Auditor Cedar scope-bound + window-bound; pen-test annually; OIDC subject pseudonymisation in audit log (operator-id hash) | L | ops-security + council-privacy |
| R-04 (cross-pack misroute) | Pack-pinning enforced at OTel-collector + at tenancy-lifecycle-adapter; integration tests; runtime detector `oya_tenancy_pack_misroute_total` triggers Sev-1 | L | axis-tenancy + ops-sre-reliability |
| R-05 (DSR incomplete) | LEAN check `oya-governance-dsr-handler-conformance`: every µservice must register handler; quarterly drill; missing-receipt halt-and-escalate with DPO sign-off path; receipt-aggregation Merkle root publishes "expected_n vs received_n" | M (engineering-discipline residual; acceptable given drill + halt-and-escalate) | council-privacy + every µservice owner |
| R-06 (re-identification) | Salt rotation 12mo (audit-chain seal on rotation event); small-tenant detection triggers extra DP-noise injection on cross-tenant aggregates | L | ops-security |
| R-07 (operator-tenant correlation) | Audit-log access JIT-only + tenant-scope-bound; operator-id hash in audit log; cross-tenant operator-id correlation requires DBA JIT (2-person rule) | L | ops-security + council-privacy |
| R-08 (cell-load behavior signal) | Cell-load aggregates published at pack level only; per-tenant cell load is BEHAVIORAL_TENANT_PRODUCT; never cross-tenant exposed | L | axis-tenancy |
| R-09 (joint-controllership confusion) | Tenant DPA template mandates upstream-disclosure clause; tenant onboarding checklist verifies disclosure-in-tenant-privacy-notice; non-disclosure = onboarding refused | L-M | council-privacy + gtm-customer-success |
| R-10 (children's data) | DPDPA §9 + GDPR Art. 8: tenant DPA includes child-data clause; tenant attests parental-consent process | L (residual depends on tenant) | council-privacy |
| R-11 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before ingest enabled; non-signed tenants pre-flighted to non-PHI pack; BAA template at `legal/baa-template.md` | L | council-privacy + sales-legal |
| R-12 (DSR proof unverifiable) | Proof-of-erasure aggregates per-µservice signed receipts under a Merkle root; regulator-disclosable artifact; cryptographic verification path documented | L | council-privacy + audit-chain |
| R-13 (auditor pivot) | Auditor Cedar policy `auditor-scope.cedar`: tenant-scope + engagement window; pen-test annually | L | ops-security |
| R-14 (Patroni HA outage) | 3-node minimum + quorum DCS + auto-failover ≤ 10s; quarterly failover drill; validate hot path tolerates ≤ 10s blip per AC-14 | L | ops-sre-reliability |
| R-15 (DSR delayed) | Cascade SLA timer monitors against per-pack legal SLA (30d / 15d); escalation to ops at 80% of window; DPO sign-off override if cause is legitimate (e.g., legal-hold) | L | council-privacy + axis-tenancy |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-tenancy lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all rated L or M (no H or M-H residuals remain after mitigations for risks within tenancy's direct authority). R-05 + R-09 + R-15 retain M residuals because they require ongoing engineering discipline + tenant cooperation; these are accepted with quarterly review. Therefore Art. 36 prior consultation with the supervisory authority is **NOT triggered** for the tenancy substrate itself; consultation may be triggered separately by a workload µservice's DPIA if its residual is higher.

The DPO advises proceeding with first-tenant onboarding subject to:
- Quarterly review of R-05 (DSR completeness) — engineering-discipline metric over time.
- Annual review of this DPIA.
- Re-trigger DPIA on any pack-activation (each new pack engages distinct legal frameworks).
- Re-trigger DPIA on any new sub-processor (Citus vendor, Patroni operational support, OpenBao vendor, etc.).

**Outcomes documented:**
- Mitigations adopted: every measure in §6 is in-scope for the IP-001 through IP-015 authoring (see `tenancy/PHASE-01-TENANCY-SUBSTRATE-STABLE.md`).
- Records-of-processing register entry (per GDPR Art. 30): `microservices/tenancy/legal/ropa.md`.
- Joint-controllership template: `microservices/tenancy/legal/dpa-template.md`.

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 mandate a 개인정보영향평가 (DPIA-equivalent) for systems processing sensitive personal information at scale. This document fulfils that obligation for KR tenants.

Additional KR-specific considerations:
- **PIPA Art. 23 (sensitive personal information)**: hashed `tenant_id` treated as sensitive when correlated with auxiliary data. Mitigation: salted-hash rotation (R-06).
- **PIPA Art. 23-2 (sensitive data cross-border transfer)**: KR-resident data stays in pack-kr cluster; no cross-pack replication.
- **PIPA Art. 28 (storage period)**: tenant metadata retention bounded per the asset table; non-essential data removed within statutory minimum.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6 measures to the 12 prescribed safeguards.
- **PIPC Notice 2020-7 methodology**: this DPIA's structure (Steps 1–7) follows PIPC's prescribed 7-step methodology.
- **KR PIPA Art. 33-2 (DPO appointment)**: oyatie's council-privacy chair serves the PIPA DPO role for KR-resident tenants.
- **KR PIPA Art. 34 (breach notification, 72h to PIPC + 72h to data subjects)**: incident-response.md reflects.
- **KR PIPA Art. 36 (right-to-deletion)**: DSR cascade fulfils within 30d.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires a risk analysis substantially equivalent to a DPIA. This document fulfils that requirement for HIPAA-scope tenants.

Additional HIPAA considerations:
- **§164.502(a) (Permitted Uses)**: TPO (Treatment + Payment + Operations) is the permitted scope; tenancy substrate falls under Operations.
- **§164.502(b) (Minimum Necessary)**: tenancy stores only tenant-level metadata, not end-user PHI; minimum-necessary by design.
- **§164.504(e) (Business Associate)**: oyatie operates as Business Associate for HIPAA-scope tenants; BAA template at `microservices/tenancy/legal/baa-template.md`.
- **§164.310 (Physical Safeguards)**: inherited from cloud-k8s µservice's DPIA + cloud provider's HIPAA-eligibility certification.
- **§164.312(b) (Audit Controls)**: Ed25519 audit-chain seals + audit log retention ≥ 6y for HIPAA-tagged tenants.
- **§164.404 (Notification to Individuals)**: breach-notification chain in `incident-response.md`.
- **45 CFR Part 164 Subpart D (Breach Notification)**: integrated into incident-response.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + DORA)

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing.

Additional EU considerations:
- **EDPB Guidelines 4/2019 (Art. 25 by design)**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h notification chain in `incident-response.md`.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo reporting timelines when oyatie crosses Annex I/II thresholds.
- **eIDAS 910/2014**: Ed25519 seals + proof-of-erasure are AdES (advanced electronic signatures); QES (qualified) requires certified TSP — scheduled-for-distinct-tracked-work.
- **DORA (2022/2554)**: financial-services tenants in pack-eu trigger DORA Chapter II + III + VI requirements.
- **Schrems II + Art. 44–46 transfers**: no cross-border transfer of EU-resident data without tenant-executed SCCs; transfer register kept.
- **Children's data (Art. 8)**: inherited via tenant's age-gating; oyatie does not directly process children's data.

### pack-jp (APPI)

APPI Arts. 17–27 cover most processing rules; APPI does not mandate a DPIA-equivalent but encourages voluntary risk assessment under the PPC's voluntary scheme. This document satisfies that voluntary assessment.

- **APPI Art. 17 (purpose of use)**: declared at tenant-onboarding.
- **APPI Art. 21 (cross-border transfer)**: pack-jp data JP-resident.
- **APPI Art. 23 (joint use)**: tenant-of-tenant data joint-use disclosure required.
- **APPI Art. 26-2 (breach notification)**: 72h to PPC.
- **APPI Art. 27 (sensitive data consent)**: tenant DPA captures consent.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/tenancy-dpia-overlay.md` carry pack-specific legal-citation depth. Each overlay follows this document's 7-step structure with the local PII law's articles substituted in:

- **pack-sg (PDPA 2012)**: PDPA Part III + IV; MAS Notice 644 for financial-services tenants.
- **pack-au (Privacy Act 1988 APP)**: APP 1–13; APP 8 + APP 11 + APP 12 most relevant; APRA-CPS 234 for financial-services tenants; OAIC NDB scheme.
- **pack-in (DPDPA 2023)**: §10 + §11 (DPIA-equivalent); §9 (children's data); §8 (data fiduciary obligations).
- **pack-br (LGPD)**: Arts. 38 (RIPD) + ANPD methodology; cross-border via ANPD-approved SCCs.
- **pack-ae (UAE PDPL Federal Decree-Law 45/2021)**: Art. 23 impact-assessment; Art. 9 lawful-basis.
- **pack-ksa (KSA PDPL Royal Decree M/19/2021)**: Art. 9 impact-assessment + DPO notification; SAMA Cybersecurity Framework 2017.

## Re-review Triggers

This DPIA re-reviews on:
- Annually (Q2 each year).
- On every new pack activation.
- On any change to processing purpose (§2.4) or data-class taxonomy.
- On any sub-processor change (`legal/sub-processors.md`).
- On any breach notification triggered (per Art. 33 + state laws).
- On supervisory-authority guidance change affecting any enforced framework.
- On Patroni / Citus / Postgres major-version upgrade with security advisory.
- Post-incident (any Sev-1 or Sev-2 affecting tenancy).

## References

- ADR-0018 (Bominal): Tenancy + RLS posture; inherited.
- ADR-0028 (Bominal): Audit chain.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `tenancy/threat-model.md`.
- `tenancy/policy/rls-isolation.md`.
- `tenancy/policy/data-residency.md`.
- `tenancy/compliance.md`.
- `tenancy/incident-response.md`.
- `microservices/tenancy/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md`.
- ICO DPIA template — `ico.org.uk`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
- EDPB Guidelines 4/2019 (Art. 25 by design and default).
- EDPB Guidelines 9/2022 (personal data breach notification).
- PIPC Notice 2020-7 (KR DPIA methodology).
- GDPR Art. 35 + Art. 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- LGPD Art. 38; ANPD methodology.
- DPDPA 2023 §10–§11.
