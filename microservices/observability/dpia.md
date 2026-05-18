---
doc_class: DPIA
template_id: TPL-DPIA
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-observability
deciders: council-privacy, ops-security, axis-observability, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 (개인정보영향평가)
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0140]
related_specs: [/specs/agentic-slo-gated-promotion.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/observability/threat-model.md
  - microservices/observability/policy/tenant-isolation.md
  - microservices/observability/policy/data-residency.md
  - microservices/observability/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (burn-rate evaluation is systematic monitoring of tenant SLI signal)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare; sensitive data under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8, 2017 TSC)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33 (영향평가)", "PIPA Enforcement Decree Art. 35 (DPIA mandatory criteria)", "PIPC Notice 2020-7 (DPIA methodology)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis)", "§164.312(b) (audit controls)", "§164.502(b) (minimum necessary)", "§164.514 (de-identification)"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EDPB Guidelines 4/2019 on Art. 25 (data protection by design and default)", "EDPB Guidelines 9/2022 on personal data breach notification"]
  pack-jp: ["APPI Arts. 17, 18, 27 (consent for sensitive data + cross-border transfer)"]
  pack-sg: ["PDPA Part III (Protection Obligation) + Part IV (Retention)", "MAS Notice 644 (Technology Risk Management)"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "OAIC Australian Privacy Principles guidelines"]
  pack-in: ["DPDPA 2023 §10 (data fiduciary obligations) + §11 (DPIA-equivalent: 'data protection impact assessment')"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38 (RIPD — Relatório de Impacto à Proteção de Dados; ANPD methodology)"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23 (impact assessment)"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9 (impact assessment + DPO notification)"]
doc_status: published
---

# Data Protection Impact Assessment: observability µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The observability µservice triggers two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | The burn-rate evaluator continuously evaluates per-microservice signal across every tenant; the eligibility verdict is a quasi-automated decision affecting which tenant-deployed code reaches production. |
| Art. 35(3)(b): Large-scale processing of special-category data (Art. 9) | **YES (conditional)** | Pack-us-healthcare carries PHI in traces unless rigorously redacted; pack-kr KR PIPA Art. 23 classes hashed-tenant-id with auxiliary as sensitive. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | observability does not monitor public-area cameras / IoT. |

In addition, the Korean PIPC's Notice 2020-7 mandates a DPIA when processing system handles sensitive personal information (PIPA Art. 23) at scale — engaged.

Therefore: a DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (per Art. 35) and the Korean PIPC (per PIPA Art. 33) at first-tenant onboarding in each jurisdiction.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** observability ingests telemetry (metrics, logs, traces, profiles) emitted by every workload µservice; computes per-tenant SLI signal against tenant-authored OpenSLO targets; emits per-(microservice, sha, target_env) eligibility verdicts that gate dev→staging→production promotion; auto-rolls-back production on burn-rate breach; pages on-call.

**How:** OpenTelemetry SDK → Grafana Alloy collector (per workload µservice; mTLS + per-µservice OTel API key) → Prometheus + Mimir / Loki / Tempo / Pyroscope (multi-tenant TSDB / log / trace / profile stores) → SLO engine worker (continuous PromQL queries) → eligibility verdict metrics into Mimir → GitHub `repository_dispatch` event → promote workflow consumes + advances `release/<microservice>/<environment>` ref → optional automated rollback.

**Where:** Per-pack region-pinned Mimir clusters (pack-kr → KR / pack-eu → EU / pack-us → US / pack-jp → JP / etc.) running in dedicated observability Kubernetes clusters on oyatie's `cloud-k8s` substrate. Pack-pinning enforces residency per ADR-0117.

**When:** Continuous; 60-second evaluator cadence; 1-minute metric write rate from collectors.

**Who:** Per the actor table in `microservices/observability/threat-model.md` §"Actors". External tenant operators; customer applications; workload µservices; CI runner; SLO engine worker; council operators; external auditors.

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Per-tenant API call rates, error counts, latency distributions | Art. 6(1)(b) contract necessity + Art. 6(1)(f) legitimate interest (operational) | ~10⁹ metric samples/day per medium tenant |
| `PII_IDENTIFYING` | User-id fields in structured logs (when emitted by workload µservice) | Art. 6(1)(b) contract; Art. 6(1)(c) legal obligation (audit) | varies by tenant emission |
| `PII_QUASI_IDENTIFIER` | URLs / IPs / user-agent strings in trace spans | Art. 6(1)(f) legitimate interest; minimised at SDK level | ~10⁶ span attributes/day per medium tenant |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id (`X-Scope-OrgID`) when correlated with auxiliary data | KR PIPA Art. 15 + 23 + 23-2 (sensitive personal info processing with explicit consent) | 1 per tenant request |
| `PHI` (pack-us-healthcare only) | Patient identifiers / clinical data in traces if not redacted | HIPAA §164.502(a) Permitted Uses (Treatment + Payment + Operations) per BAA; never disclosed beyond Covered Entity / BA boundary | varies; targeted to 0 via SDK redactor |
| `AUDIT` | Promotion / rollback events; Ed25519 audit-chain seals | Art. 6(1)(c) legal obligation (record-keeping); Art. 6(1)(f) legitimate interest | 1 record per promotion / rollback |
| `SECRET` | Per-tenant OTel API key, Mimir API key, signing keys | not personal data; managed under ISO 27001 A.5.17 controls | varies |

**Geographical scope:** Per pack:
- pack-kr: KR (ap-seoul-1 OCI region) — KR-resident tenant data stays in KR.
- pack-eu: EU (eu-frankfurt-1) — EU-resident tenant data stays in EU.
- pack-us / pack-us-healthcare: US (us-ashburn-1 + us-phoenix-1) — US-resident tenant data stays in US; HIPAA Covered Entity tenants pinned to BAA-eligible regions.
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned to its primary region.

**Cross-border transfer:** Forbidden by default per multi-region.md (Slice B). Allowed only with tenant-executed SCCs (Standard Contractual Clauses) for GDPR-scope tenants per Arts. 44–46 and equivalent local provisions; recorded in `microservices/observability/legal/transfer-register.md` (Slice D).

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (the tenant's customers); tenant operators (administrative users of the tenant); oyatie operators (internal). Not all data classes apply to all categories; mapping is in §3.
- **Relationship to data subjects:** Joint controllership with the tenant under GDPR Art. 26 (the tenant is controller of its end-users' data; oyatie is joint controller for the operational telemetry portion). Joint-controllership terms recorded in the tenant DPA template (`microservices/observability/legal/dpa-template.md`, Slice D).
- **Reasonable expectations:** Tenant operators expect operational telemetry (per service-level contract). End-users (the tenant's customers) expect operational data collection per the tenant's own privacy notice; oyatie's processing is disclosed in the tenant's notice via the joint-controllership transparency clause.
- **Previous experience:** Bominal observability (predecessor substrate) operated under the same processing pattern; no DPA-triggered complaints in 24 months. Inherited lessons captured per `feedback_bominal_inheritance_precedence.md`.
- **Industry codes:** None directly applicable; voluntary alignment with OpenTelemetry semantic conventions reduces ambiguity in data-class declaration.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Compute per-microservice SLO compliance** | Necessary for the tenant's contracted service-level agreement | Art. 6(1)(b) contract |
| **Gate promotion to production on SLO health** | Necessary for operational integrity; prevents tenant-impacting bad releases | Art. 6(1)(b) + Art. 6(1)(f) legitimate interest |
| **Auto-rollback on production-tier breach** | Necessary for incident-response obligations | Art. 6(1)(b) + Art. 6(1)(c) legal obligation (incident notification under Art. 33) |
| **Operational dashboards (tenant-facing)** | Tenant-contracted feature | Art. 6(1)(b) contract |
| **Cross-tenant aggregations (anonymised; future)** | Optional product-improvement use | Art. 6(1)(f) legitimate interest + DP analysis published at `policy/dp-analysis.md` |
| **Audit-chain emission** | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 records-of-processing | Art. 6(1)(c) legal obligation |
| **Marketing / unrelated commercial use** | NOT a purpose | N/A — explicitly excluded |

The purposes are explicit, legitimate, and specified at the point of tenant onboarding via the DPA template (Art. 5(1)(b) purpose-limitation).

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (DPO) | YES — council-privacy chair | Sign-off pending; see §7 |
| Tenant representative (sample of 3 prospective tenants for first-rollout) | Scheduled — pre-GA | Feedback to be folded into Step 6 measures |
| Data subjects (the tenants' end-users) | Indirect via tenant onboarding notices | Joint-controllership clause carries upstream-disclosure obligation to the tenant |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Prior consultation (Art. 36) — NOT triggered (no residual high risk after mitigations; see §6 + §7) | If §6 mitigations residual > Medium, Art. 36 prior consultation is triggered |
| Information security team (ops-security) | YES — co-author of threat-model.md | Threat-model + DPIA share residual-risk catalog |
| Engineering teams (axis-observability + each workload µservice) | YES | SDK redactor + data-class annotations enforced at CI |
| External auditor (SOC 2 / ISO 27001 firm) | At first audit cycle | Cross-references this DPIA |

DPO independent advice + sign-off recorded at §7.

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary to achieve the purpose? | YES — operational SLO + gating cannot be performed without telemetry. |
| Is there a less intrusive alternative? | Considered: synthetic-only probes (no real-user signal). Rejected: synthetic signal misses real-world failure modes (post-deployment regressions invisible to canaries). The current design uses real-user signal but redacts PII at SDK level (data minimisation, Art. 25). |
| Is processing proportionate to the purpose? | YES — collection limited to: metric counts (no payload bodies); structured log fields with `data_class` annotation (redactor strips `PII`-class); trace span attributes with attribute-level redaction; profile data is binary-aggregate (no per-user attribution). Per Art. 5(1)(c) data-minimisation. |
| Does processing achieve a public interest or substantial private interest? | YES — operational reliability of tenant production systems; legitimate interest documented in DPA template. |
| Could the purpose be achieved by anonymised / pseudonymised data? | PARTIALLY — pseudonymisation (hashed customer-id) is applied at the `X-Scope-OrgID` boundary. Full anonymisation would prevent per-tenant SLO authority and defeat the contractual purpose. Pseudonymisation is the proportionate compromise. |
| Lawful basis (Art. 6) | Identified per purpose in §2.4. |
| Special-category basis (Art. 9, if applicable) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care under contract with health professional) + HIPAA BAA covering 45 CFR §164.504(e). pack-kr sensitive data: PIPA Art. 23(2) (explicit consent) at tenant onboarding. |
| Transfer basis (Arts. 44–46) | Per §2.2 cross-border: SCCs only; default residency by pack. |
| Retention | Per asset class in `microservices/observability/threat-model.md` §"Assets & Data Classification". Defaults: metrics 30d hot + 24mo cold; logs 14d hot + 12mo cold; traces 7d hot + 6mo cold; profiles 14d. HIPAA pack: ≥ 6y for audit-relevant data. PIPA: erasure-on-request honoured per DSR cascade (§6 below). |
| Rights of data subjects | Honoured per §6 mitigations: access (Art. 15), rectification (Art. 16), erasure (Art. 17), restriction (Art. 18), portability (Art. 20), objection (Art. 21), automated-decision-protections (Art. 22). |

## Step 5 — Identify and assess risks to data subjects

Risks below are scored on Likelihood (L/M/H) × Severity (L/M/H); Severity is from the perspective of the data subject, not oyatie.

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | PII leakage via trace span attributes (re-identifies end-user) | M-H | H | **H** |
| R-02 | Cross-tenant query leak (rival tenant infers business signal) | L-M | H | **H** |
| R-03 | Long retention enables surveillance pattern (longitudinal profile across years of behavior) | M | M-H | **M-H** |
| R-04 | Automated promotion gate decision affects tenant's end-users (delayed feature delivery) | M | L-M | **M** |
| R-05 | Automated rollback affects tenant's end-users (sudden feature withdrawal) | L | M | **L-M** |
| R-06 | Sub-processor (Grafana / Mimir cluster operator / cloud provider) breach exposes telemetry | L | H | **M** |
| R-07 | Tenant operator misconfiguration exposes their end-users' data via dashboard | M | M-H | **M-H** |
| R-08 | End-user-initiated DSR (right-to-erasure) is incomplete because retention has spread data across multiple stores | M | M | **M** |
| R-09 | Joint-controllership confusion: tenant doesn't disclose oyatie's processing to its end-users | M-H | M | **M-H** |
| R-10 | Re-identification via metric cardinality + timing (low-cardinality tenant) | L | M | **L-M** |
| R-11 | Cross-border transfer of EU-resident data via misrouted ingest | L | H | **M** |
| R-12 | Children's data (DPDPA 2023 §9; pack-in) processed without parental consent | L | H | **M-H** |
| R-13 | PHI processed without BAA (pack-us-healthcare; tenant doesn't sign BAA but ships clinical traces) | M | H | **H** |
| R-14 | Hashed tenant-id re-identified via small-tenant auxiliary data | L | M | **L-M** |
| R-15 | Auditor mis-pivots from tenant-A to tenant-B during engagement | L | H | **M** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE / LINDDUN threat in `microservices/observability/threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (PII leakage via traces) | OTel SDK PII redactor with `data_class` honour; sample rate 1% in prod; per-µservice CI lane validates redaction; quarterly synthetic-PII detection drill | Residual M (engineering discipline floor) | axis-observability + each workload owner |
| R-02 (cross-tenant leak) | Mimir multi-tenancy mandatory; per-tenant API keys; LEAN check `oya-check-mimir-tenancy-enforced`; annual pen-test; weekly threat hunt | L | ops-security |
| R-03 (long retention surveillance) | Retention per asset class explicit; aggressive defaults; DSR cascade honours Art. 17 erasure within 30 days; cold-tier data is aggregated (no per-event granular access without admin JIT) | L-M | council-privacy |
| R-04 (automated gate decision) | Promotion is operational decision, NOT solely-automated decision producing legal effects on data subjects per Art. 22; explanation available; tenant can manually override with 2-person rule | L | axis-observability |
| R-05 (auto-rollback) | Rollback is a safety-net; same Art. 22 carve-out (operational + tenant-supervised); rollback events emit RollbackExecuted event consumed by tenant on-call | L | axis-observability + ops-sre-reliability |
| R-06 (sub-processor breach) | Sub-processor list maintained in `microservices/observability/legal/sub-processors.md` (Slice D); each carries DPA + SCCs where applicable; quarterly sub-processor security review | M (sub-processor risk is irreducible) | council-privacy + cloud-secrets |
| R-07 (tenant misconfig) | Tenant-facing dashboards enforce Cedar-policy-checked role boundaries; misconfig requires explicit acknowledgement; defaults are private-by-design | L-M | axis-observability |
| R-08 (DSR incompleteness) | DSR cascade (per `oya-dsr-cascade-runner` skill) scans Mimir + Loki + Tempo for the identifier; 30-day SLA from request; cold-tier search supported | M (best-effort within retention windows is the accepted residual) | council-privacy |
| R-09 (joint-controllership confusion) | Tenant DPA template mandates upstream disclosure clause; tenant onboarding checklist verifies disclosure-in-tenant-privacy-notice; non-disclosure = onboarding refused | L-M | council-privacy + gtm-customer-success |
| R-10 (re-identification via timing) | Cardinality limits per tenant; small tenants (< 100 users) trigger DP-noise injection on cross-tenant aggregates | L | axis-observability |
| R-11 (cross-border misroute) | Pack-pinning enforced at OTel-ingest collector level; route by pack tag; misroute = configuration error caught by integration test | L | axis-observability |
| R-12 (children's data) | DPDPA §9 + GDPR Art. 8: tenant DPA includes child-data clause; tenant must affirm parental-consent process; observability does not collect age but inherits tenant's age-gating | L (residual depends on tenant) | council-privacy |
| R-13 (PHI without BAA) | pack-us-healthcare onboarding requires BAA sign-off before tenant ingest enabled; non-signed tenants pre-flighted to non-PHI-pack | L | council-privacy + sales-legal |
| R-14 (tenant-id re-identification) | Salted hash (salt rotated 12mo); audit-chain notes rotation; small-tenant cardinality protection | L | ops-security |
| R-15 (auditor mis-pivot) | Auditor JIT tokens tenant-scoped at Grafana folder level; pen-test of auditor-token boundary annually | L | ops-security |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-observability lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all rated L or M (no H or M-H residuals remain after mitigations). Therefore Art. 36 prior consultation with the supervisory authority is NOT triggered. The DPO advises proceeding with first-tenant onboarding subject to:
- Quarterly review of R-01 (PII leakage residual) — engineering-discipline metric over time.
- Annual review of this DPIA.
- Re-trigger DPIA on any pack-activation (each new pack engages distinct legal frameworks; pack-overlay sections must be filled prior to first-tenant in that pack).

**Outcomes documented:**
- Mitigations adopted: every measure in §6 is in-scope for the Slice A / B / C / D authoring phases (see `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md`).
- Records-of-processing register entry (per GDPR Art. 30): `microservices/observability/legal/ropa.md` (Slice D).
- Joint-controllership template: `microservices/observability/legal/dpa-template.md` (Slice D).

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 require a 개인정보영향평가 (DPIA-equivalent) for systems processing sensitive personal information at scale. This document fulfils that obligation for KR tenants.

Additional KR-specific considerations:
- **PIPA Art. 23 (sensitive personal information)**: hashed customer-id treated as sensitive when correlated with auxiliary data. Mitigation: salted-hash rotation (R-14).
- **PIPA Art. 23-2 (sensitive data cross-border transfer)**: KR-resident sensitive data stays in pack-kr Mimir cluster; no cross-pack replication.
- **PIPA Art. 28 (storage period)**: telemetry retention bounded; non-essential data removed within statutory minimum; per asset table.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6 measures to the 12 prescribed safeguards (access control + encryption + integrity + audit log ≥1yr + IDS + …).
- **PIPC Notice 2020-7 methodology**: this DPIA's structure (Steps 1–7) follows PIPC's prescribed 7-step methodology by intent + structure.
- **KR PIPA Art. 33-2 (DPO appointment)**: oyatie's council-privacy chair serves the PIPA DPO role for KR-resident tenants.

### pack-us-healthcare (HIPAA)

HIPAA does not require a "DPIA" by that name, but §164.308(a)(1)(ii)(A) requires a risk analysis substantially equivalent to a DPIA. This document fulfils that requirement.

Additional HIPAA considerations:
- **§164.502(a) (Permitted Uses and Disclosures)**: TPO (Treatment + Payment + Operations) is the only permitted scope; operational SLO computation falls under Operations.
- **§164.502(b) (Minimum Necessary Standard)**: PII / PHI redaction at SDK level enforces minimum-necessary; trace attributes carry only operational identifiers.
- **§164.504(e) (Business Associate)**: oyatie operates as Business Associate for HIPAA-scope tenants; BAA template at `microservices/observability/legal/baa-template.md` (Slice D).
- **§164.310 (Physical Safeguards)**: inherited from cloud-k8s µservice's DPIA + cloud provider's HIPAA-eligibility certification.
- **§164.312(b) (Audit Controls)**: Ed25519 audit-chain seals + Mimir audit-log retention ≥ 6y for HIPAA-tagged tenants (cost-budget.md Slice B reflects).
- **§164.404 (Notification to Individuals)**: breach notification chain documented in `incident-response.md` (Slice B); covers HIPAA's 60-day notification window + state-level overlays (CCPA, etc.).
- **45 CFR Part 164 Subpart D (Breach Notification)**: integrated into incident-response.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS)

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing.

Additional EU considerations:
- **EDPB Guidelines 4/2019 (Art. 25 — data protection by design and default)**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (personal data breach notification)**: 72-hour notification chain documented in incident-response.md (Slice B); covers Art. 33 + 34 obligations.
- **NIS2 (2022/2555)**: when oyatie crosses Annex I/II thresholds (likely on platform-wide tenant count, not on observability alone), the 24h + 72h + 1mo NIS2 reporting timelines apply; incident-response.md reflects.
- **eIDAS 910/2014**: Ed25519 audit-chain seals are advanced electronic signatures (AdES); for EU-resident transaction records, the seals satisfy Art. 26 AdES requirements.
- **Schrems II + Art. 44–46 transfers**: no cross-border transfer of EU-resident telemetry without tenant-executed SCCs; transfer register kept.
- **Children's data (Art. 8)**: inherited via tenant's age-gating; oyatie does not directly process children's data.

### pack-jp (APPI)

APPI Arts. 17–27 cover most processing rules; APPI does not mandate a DPIA-equivalent but encourages voluntary risk assessment under the Personal Information Protection Commission's voluntary scheme. This document satisfies that voluntary assessment.

- **APPI Art. 17 (purpose of use)**: telemetry purpose declared at tenant onboarding (§2.4).
- **APPI Art. 21 (cross-border transfer)**: pack-jp residency in JP-resident region; cross-pack forbidden by default.
- **APPI Art. 27 (consent for sensitive data)**: tenant-of-tenant sensitive-data inheriting requires explicit tenant-disclosure to its end-users.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/dpia-overlay.md` carry the pack-specific legal-citation depth. Each pack's overlay follows this document's 7-step structure with the local PII law's articles substituted in:

- **pack-sg (PDPA 2012)**: PDPA Part III + IV; MAS Notice 644 for financial-services tenants.
- **pack-au (Privacy Act 1988 APP)**: APP 1–13; APP 8 + APP 11 + APP 12 most relevant; APRA-CPS 234 for financial-services tenants.
- **pack-in (DPDPA 2023)**: §10 + §11 (DPIA-equivalent: "data protection impact assessment"); §9 (children's data); §8 (data fiduciary obligations).
- **pack-br (LGPD)**: Arts. 38 (RIPD — Relatório de Impacto à Proteção de Dados) + ANPD methodology; cross-border via ANPD-approved SCCs.
- **pack-ae (UAE PDPL Federal Decree-Law 45/2021)**: Art. 23 impact-assessment; Art. 9 lawful-basis.
- **pack-ksa (KSA PDPL Royal Decree M/19/2021)**: Art. 9 impact-assessment + DPO-notification.

## Re-review Triggers

This DPIA re-reviews on:
- Annually (Q2 each year).
- On every new pack activation.
- On any change to processing purpose (§2.4) or data-class taxonomy.
- On any sub-processor change (sub-processor list lives at `legal/sub-processors.md`).
- On any breach notification triggered (per Art. 33 + state laws).
- On supervisory-authority guidance change affecting any enforced framework.
- Post-incident (any Sev-1 or Sev-2 involving observability or any µservice it processes data for).

## References

- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0140: Cedar policy enforcement.
- `microservices/observability/threat-model.md` — paired security artifact.
- `microservices/observability/policy/tenant-isolation.md` (Slice A Task A3).
- `microservices/observability/policy/data-residency.md` (Slice A Task A4).
- `microservices/observability/compliance.md` (Slice B Task B4).
- `microservices/observability/incident-response.md` (Slice B Task B6).
- `microservices/observability/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md` (Slice D).
- ICO DPIA template — `ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/accountability-and-governance/data-protection-impact-assessments-dpias`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
- EDPB Guidelines 4/2019 (Art. 25 by design and default).
- EDPB Guidelines 9/2022 (personal data breach notification).
- PIPC Notice 2020-7 (KR DPIA methodology).
- GDPR Art. 35 + Art. 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- LGPD Art. 38; ANPD methodology.
- DPDPA 2023 §10–§11.
