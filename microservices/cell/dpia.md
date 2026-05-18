---
doc_class: DPIA
template_id: TPL-DPIA
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cell-substrate
deciders: council-privacy, ops-security, axis-cell-substrate, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/cell/threat-model.md
  - microservices/cell/policy/cell-boundary.md
  - microservices/cell/policy/data-residency.md
  - microservices/cell/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or pack activation
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — YES (scheduler makes automated placement decisions affecting tenant data residency + isolation)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (cell binding may concern PHI under pack-us-healthcare; KR PIPA Art. 23 sensitivity for hashed tenant-id with cell adjacency)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34, A.5.31"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 33"]
  pack-us-healthcare: ["HIPAA §164.308(a)(1) risk analysis", "§164.312(b) audit", "§164.514 de-identification"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Parts III + IV"]
  pack-au: ["Privacy Act 1988 APP 1/5/6/11/12"]
  pack-in: ["DPDPA 2023 §10/11"]
  pack-br: ["LGPD Arts. 6/7/11/38"]
  pack-ae: ["UAE PDPL 45/2021 Art. 23"]
  pack-ksa: ["PDPL M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: cell µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk**. The cell substrate triggers two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): systematic + extensive evaluation including profiling | **YES** | Scheduler makes automated placement decisions affecting tenant data residency + isolation; rebalance decisions are continuous + per-tenant. |
| Art. 35(3)(b): large-scale processing of special-category data | **YES (conditional)** | Pack-us-healthcare cells host PHI; KR PIPA Art. 23 treats `(tenant_id, cell_id)` adjacency as sensitive via re-identification. |
| Art. 35(3)(c): systematic public-area monitoring | NO | n/a |

KR PIPC Notice 2020-7 also mandates DPIA for systems handling Art. 23 sensitive data at scale — engaged.

Therefore: DPIA is mandatory pre-deployment. Reviewed by EU DPAs (Art. 35) + KR PIPC (PIPA Art. 33) + HIPAA tenants' Covered Entity counsel at first-tenant onboarding per pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** The cell µservice binds each tenant to exactly one cell (or HA cohort) per pack region. The scheduler makes placement + rebalance decisions over cluster state. The lifecycle-manager creates / drains / decommissions cells. The host-pool maintains warm K8s nodes. Tenant migration moves tenant data between cells when needed.

**How:** REST + gRPC + AsyncAPI surfaces; Postgres registry per pack; Kubernetes Cluster API for cell lifecycle; SPIFFE for per-cell identity; OpenBao for credential issuance; audit-chain Ed25519 seal for every state transition.

**Where:** Per-pack regions (pack-kr → KR; pack-eu → EU; pack-us → US; etc.). Cell substrate is pack-resident; cross-pack assignment forbidden by default.

**When:** Continuous reads on the hot path; scheduler placement on tenant onboarding; rebalance on capacity-band breach; migration on operator/automation trigger; decommission on cell end-of-life.

**Who:** Per actor table in `threat-model.md`.

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume |
|---|---|---|---|
| `SENSITIVE_PIPA_ART23` | `(hashed_tenant_id, cell_id)` pair | Art. 6(1)(b) contract + Art. 6(1)(c) legal obligation (audit) | 1 per tenant + 1 per migration |
| `BEHAVIORAL_TENANT_PRODUCT` | capacity metrics joined with tenant_id; migration plans | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | varies |
| `INTERNAL_ONLY` | cell metadata; host inventory; placement scores | not personal data | varies |
| `AUDIT` | every CellAssigned / CellRebalanced / CellDecommissioned / TenantMigrated event | Art. 6(1)(c) | 1 per event |
| `SECRET` | per-cell credentials; SPIFFE SVIDs; OpenBao tokens | not personal data | rotated per policy |
| `PHI` (pack-us-healthcare only) | pod-resident in cells; cell-substrate does not process PHI directly | n/a (covered by workload µservices' BAAs) | per BAA |

**Geographical scope:**
- pack-kr → KR ap-seoul-1
- pack-eu → EU eu-frankfurt-1 + eu-amsterdam-1
- pack-us / pack-us-healthcare → US us-ashburn-1 + us-phoenix-1
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa → primary region per `multi-region.md`.

**Cross-border transfer:** Forbidden by default. Narrow exceptions per `policy/data-residency.md`: tenant-executed SCCs (GDPR Arts. 44–46); HIPAA BAA DR pair (us-ashburn ↔ us-phoenix only); controlled BCDR exercises intra-pack.

### 2.3 Context

- **Data subjects:** Tenant operators (administrative users), via `tenant_id` binding (hashed). End-users of tenant applications are NOT directly processed by cell — workload µservices process them inside cells.
- **Joint controllership:** Same model as observability — tenant is controller of its end-users; oyatie is joint controller for operational telemetry portion.
- **Industry codes:** Voluntary alignment with Kubernetes Multi-Tenancy SIG conventions.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Bind tenant to cell (residency enforcement) | Contract necessity | Art. 6(1)(b) + Art. 6(1)(c) |
| Schedule placement + rebalance | Contract necessity; operational integrity | Art. 6(1)(b) + Art. 6(1)(f) |
| Migrate tenant when needed (scale / pack rehome) | Contract necessity | Art. 6(1)(b) |
| Decommission cell (end-of-life) | Contract necessity | Art. 6(1)(b) |
| Audit-chain emission | Legal obligation (SOC 2 + ISO 27001 + KR PIPA + HIPAA + GDPR Art. 30) | Art. 6(1)(c) |
| Marketing / unrelated use | NOT a purpose | excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending §7 |
| Tenant representative (sample of 3) | Scheduled pre-GA | Feedback feeds Step 6 |
| Auditor (SOC 2 examiner) | Annual cadence | Confirms control implementation aligned with TSC |
| KR PIPC (when first KR tenant onboards) | Pre-onboarding consultation | Confirms PIPA Art. 23 + Art. 28 posture |

## Step 4 — Assess necessity + proportionality

Cell substrate is **strictly necessary** to deliver hard tenant isolation. Alternatives evaluated:

- **Shared-process multi-tenancy** (single namespace, application-layer isolation): rejected — fails GDPR Art. 25 privacy-by-design + KR PIPA Art. 23.
- **VM-per-tenant**: rejected — economically impractical; doesn't scale to 10k tenants; doesn't deliver fast migration.
- **Cell-per-tenant** (one cell per single tenant): trade-off; reserved for `tenant_scope: production` HIPAA-covered tenants; default is N-tenant-per-cell within band.

Proportionality assessment: cell binding processes minimal data (hashed tenant-id + cell metadata + audit events). No raw PII. Sensitive data class (`SENSITIVE_PIPA_ART23`) handled with salt rotation + Cedar policy + Postgres RLS — proportionate to risk.

## Step 5 — Identify and assess risks

Risks enumerated in `threat-model.md` (STRIDE + LINDDUN). DPIA-specific privacy risks:

| Risk-ID | Risk | Likelihood | Severity | Mitigation summary |
|---|---|---|---|---|
| R-01 | Cross-cell data leak (T-I-01 in threat-model) | M | H | Per-cell namespace + RLS + per-cell credentials + LEAN lane |
| R-02 | Cross-tenant identity exposure via cell adjacency (T-I-02) | M | H | Adjacency never exposed; DP-noise on aggregates; salt rotation |
| R-03 | Cross-pack residency breach (T-S-04) | L | H | Cedar policy + Postgres RLS + LEAN lane |
| R-04 | Migration race producing indeterminate state (T-D-03) | M | H | Advisory locks + idempotency keys + runbook gate |
| R-05 | Decommission destroys data without retention compliance (T-S-02) | L | H | 2-person rule + 30d soft-delete + DSR cascade |
| R-06 | Sensitive-data leak via logs / dashboards (T-I-05) | M | H | Secret-scanner + redactor + rotation |
| R-07 | Right-to-erasure cascade fails (T-L-07) | M | M | DSR cascade documented + tested; cell-decommission runbook references |
| R-08 | Insider operator mass-decommission via JIT abuse | L | H | 2-person rule + audit-chain + anomaly alert on decommission-rate |
| R-09 | Auditor pivot beyond scoped tenant via cell topology read (T-L-05) | L | M | Auditor token scoped to tenant data; infra topology separate scope |
| R-10 | Linkability of tenant across rebalance history (T-L-01) | M | M | DPA discloses operational telemetry; aggregate DP-bounded |

## Step 6 — Identify measures to reduce risk

All mitigations cross-reference `threat-model.md` §"Mitigations Catalog". DPIA-specific additions:

| Measure | Type | Implementation |
|---|---|---|
| Pseudonymisation: tenant identifiers are hashed (`tenant:sha256(...)[..16]`) | Preventive | Inherited from observability tenant-isolation model |
| Salt rotation every 12 months for cell-adjacency-hash | Preventive | OpenBao salt rotation + audit-chain record |
| DSR cascade: tenant deprovisioning → cell-assignment release → cell-decommission (if last tenant) | Preventive (compliance) | DSR runner in `compliance.md`; cell-decommission runbook references |
| Per-cell DP-noise on aggregate cell-utilisation metrics ever exposed | Preventive | `policy/data-residency.md` §"Aggregate aggregations" |
| Soft-delete (≥ 30d) on all cell-substrate deletes (cell + assignment + host) | Recovery | Runbook + Postgres tombstone columns |
| 2-person rule on cell-decommission + Postgres superuser + K8s admin | Preventive | OpenBao JIT |
| Quarterly chaos drill: induce migration race + cross-pack write attempt | Verification | Runbooks + chaos engineering schedule |
| Annual pen-test against cell-boundary | Verification | Scheduled in `compliance.md` |

## Step 7 — Sign off and record outcomes

| Role | Signature | Date |
|---|---|---|
| DPO (council-privacy chair) | pending | – |
| Information Security Lead (ops-security) | pending | – |
| Service Owner (axis-cell-substrate) | pending | – |
| Council-architecture | pending | – |

Outcome: high-risk processing approved subject to mitigations above. DPIA re-review annually + on pack activation + on architecture change.

## Step 8 — Integrate outcomes into project plan

| Action | Owner | Due |
|---|---|---|
| Land cell-boundary LEAN lane in CI (IP-006) | axis-foundry | per PHASE-01 IP-006 |
| Enable Postgres RLS on cell-registry tables (IP-002) | axis-cell-substrate | per IP-002 |
| Salt rotation OpenBao job (initial deploy) | cloud-secrets | Q1 2026 |
| Annual pen-test against cell-boundary | ops-security | Q2 2026 |
| Quarterly chaos drill (migration race + cross-pack) | ops-sre-reliability | Q1 + Q2 + Q3 + Q4 2026 |
| DSR cascade tabletop with sample tenant | council-privacy | Q2 2026 |

## Per-Pack Overlay Sections

### pack-kr

- **KR PIPA Art. 33**: this DPIA fulfils PIPA's mandatory impact-assessment requirement when processing Art. 23 sensitive data at scale.
- **KR PIPA Art. 28**: cell never crosses pack boundary; multi-region.md enforces residency.
- **KR PIPA Art. 29 (technical safeguards)**: every mitigation row above maps to one of the 12 prescribed safeguards in Art. 29.
- **KR PIPC Notice 2020-7** methodology followed; DPIA filed at first KR tenant onboarding.

### pack-us-healthcare

- **HIPAA §164.308(a)(1)(ii)(A) (risk analysis)**: this DPIA satisfies the HIPAA risk-analysis requirement.
- **HIPAA §164.502(b) (minimum necessary)**: workload µservices read only their own cell scope.
- **HIPAA §164.514 (de-identification)**: tenant identifiers are hashed; PHI never directly handled by cell substrate.
- **BAA** per Covered Entity tenant; recorded in `legal/baa-template.md` (Slice D).
- Audit retention extended to ≥ 6y per §164.316(b)(2) for pack-us-healthcare.

### pack-eu

- **GDPR Art. 35** DPIA satisfied by this document.
- **GDPR Art. 36** prior-consultation may be triggered if a DPA flags residual risks unacceptable; mechanism documented in `incident-response.md`.
- **GDPR Arts. 44–50** transfers: forbidden cross-pack; SCC exception path documented.
- **EDPB Guidelines 4/2019** on Art. 25 satisfied via privacy-by-design measures (Step 6).
- **NIS2** when oyatie crosses Annex I/II thresholds: incident-reporting per `incident-response.md`.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cell-dpia-overlay.md`.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| GDPR | Arts. 5, 6, 9, 13–14, 17, 25, 28, 30, 32, 33, 35, 36, 44 | `compliance.md` |
| KR PIPA | Arts. 3, 15, 17, 18, 22-2, 23, 24, 28, 29, 33 | `compliance.md` (pack-kr overlay) |
| HIPAA | §164.308, §164.312, §164.502, §164.514, §164.316 | `compliance.md` (pack-us-healthcare overlay) |

## References

- Bominal ADR-0009; Bominal ADR-0019.
- ADR-0028 (audit-chain); ADR-0117 (residency); ADR-0139 (SLO gate); ADR-0131 (per-µservice); ADR-0140 (Cedar).
- `microservices/cell/PRD.md`; `microservices/cell/threat-model.md`; `microservices/cell/compliance.md`.
- ICO DPIA template — `ico.org.uk`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
- KR PIPC Notice 2020-7.
- EDPB Guidelines 4/2019.
