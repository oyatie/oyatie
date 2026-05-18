---
doc_class: DPIA
template_id: TPL-DPIA
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-control-plane
deciders: council-privacy, ops-security, axis-foundry-control-plane, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + EU AI Act Art. 27 (FRIA) + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/foundry-supervisor-control-plane.json]
related_artifacts:
  - microservices/foundry-supervisor/threat-model.md
  - microservices/foundry-supervisor/policy/supervisor-isolation.md
  - microservices/foundry-supervisor/policy/data-residency.md
  - microservices/foundry-supervisor/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processors, or AI Act Annex III scope
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (autonomy-policy is per-invocation evaluation with legal-effect potential when capabilities act on tenant-end-user data)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (pack-us-healthcare PHI may flow through tenant agents; pack-kr sensitive entitlement records)"
  - "EU AI Act Art. 27: Fundamental Rights Impact Assessment (FRIA) — YES when tenant capabilities fall in Annex III high-risk domains"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "EU AI Act 2024/1689 Arts. 9, 10, 12, 13, 14, 15, 27"
  - "ISO 27001:2022 A.5.34, A.5.31"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA §164.308(a)(1)(ii)(A), §164.312(b), §164.502(b), §164.514"]
  pack-eu: ["GDPR Arts. 35 + 36", "EU AI Act Arts. 9 + 27 (FRIA)", "EDPB Guidelines 4/2019, 9/2022"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12"]
  pack-in: ["DPDPA 2023 §10, §11"]
  pack-br: ["LGPD Arts. 6, 7, 11, 38"]
  pack-ae: ["UAE PDPL FDL 45/2021 Art. 23"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: foundry-supervisor µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) triggers + EU AI Act Art. 27 (FRIA) both engage:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a) — systematic + extensive evaluation incl. profiling | **YES** | The autonomy-policy precondition continuously evaluates per-invocation against per-tenant entitlement; aggregated supervision events constitute systematic monitoring. |
| Art. 35(3)(b) — special-category data at scale | **YES (conditional)** | pack-us-healthcare PHI possible inside tenant agent payloads; pack-kr PIPA Art. 23 sensitive entitlement records. |
| Art. 35(3)(c) — systematic monitoring of publicly accessible area | NO | foundry-supervisor does not monitor public-area cameras / IoT. |
| EU AI Act Art. 27 — FRIA for high-risk AI systems | **YES** (conditional, per-tenant-capability) | When a tenant deploys a capability whose domain falls in Annex III §1–8, the supervisor is the control plane for that high-risk system. |

Therefore a DPIA + FRIA (the FRIA is embedded in this DPIA's §6 + §7) is mandatory pre-deployment per jurisdiction. This document is reviewed by EU DPAs, KR PIPC, US HHS-OCR (HIPAA), and equivalent supervisory authorities.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** foundry-supervisor ingests capability definitions, fleet membership telemetry, autonomy entitlements, and supervision events; emits per-invocation autonomy decisions, deployment rollouts/rollbacks, and kill-switch state transitions; the canonical control plane for every agentic capability in oyatie's tenant base.

**How:** REST API (OIDC + Cedar) → Postgres (per-tenant RLS) + Valkey Cluster (kill-switch state + supervision-event stream) → Kubernetes Operator (CRDs) → mTLS + SPIFFE to `foundry-runtime` workers; audit-chain seals every event.

**Where:** Per-pack region-pinned cluster (pack-kr → KR, pack-eu → EU, pack-us → US, etc.) per ADR-0117. Pack-pinning enforces residency.

**When:** Continuous; per-invocation precondition checks; 60-s reconcile cadence on Operator.

**Who:** Per the actor table in `threat-model.md`.

### 2.2 Scope

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Fleet membership, capability rollout phase, kill-switch state | Art. 6(1)(b) contract necessity | ~10⁶ supervision events/day per medium tenant |
| `INTERNAL_ONLY` | Capability YAML text | Art. 6(1)(b) contract | varies |
| `SENSITIVE_PIPA_ART23` | Autonomy entitlement records (per-tenant DPO authority) | KR PIPA Art. 15 + 23 (consent) | ~10² entitlements per tenant |
| `AUDIT` | Supervision events (Ed25519-signed) | Art. 6(1)(c) legal obligation (audit) | ~10⁶/day per tenant |
| `SECRET` | OpenBao-managed credentials | Not personal data | varies |

**Geographic:** per pack region (pack-kr KR, pack-eu EU, pack-us US, pack-us-healthcare US HIPAA-eligible, pack-jp JP, pack-sg SG, pack-au AU + dual-region, pack-in IN, pack-br BR, pack-ae AE, pack-ksa KSA).

**Cross-border:** Forbidden by default per `policy/data-residency.md`. SCC-only exception for GDPR-scope tenants.

### 2.3 Context

- **Data subjects:** End-users of tenant applications (the tenant's customers); tenant operators (administrative users of the tenant); oyatie operators (internal).
- **Relationship:** Joint controllership under GDPR Art. 26 (tenant is controller of end-users' data; oyatie is joint controller for supervision/control-plane telemetry).
- **Reasonable expectations:** Tenant operators expect deployment + autonomy guarantees per SLA. End-users expect the tenant's privacy notice (with joint-controllership disclosure).
- **Industry codes:** Voluntary alignment with EU AI Act + AICPA Privacy criteria + IEEE Standards Association P7000-series (ethics).

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Capability rollout to production fleet** | Necessary for tenant's contracted agentic features | Art. 6(1)(b) contract |
| **Per-invocation autonomy precondition** | Necessary for safety + EU AI Act Art. 14 human oversight | Art. 6(1)(b) + 6(1)(f) legitimate interest |
| **Kill-switch on policy breach** | Necessary for incident response + AI Act safety | Art. 6(1)(c) legal obligation (Art. 33 breach notification) |
| **Supervision event emission for audit-chain** | Mandatory for SOC 2, ISO 27001, HIPAA audit controls, EU AI Act Art. 12 record-keeping | Art. 6(1)(c) |
| **Per-tenant fleet-state visibility** | Tenant-contracted feature | Art. 6(1)(b) |
| **Marketing / unrelated commercial** | NOT a purpose | excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending §7 |
| Sample tenants (3 prospective) | Scheduled pre-GA | Feedback folds into §6 |
| Supervisory authority | EU DPA + KR PIPC | Prior consultation under Art. 36: NOT triggered (no residual high risk after mitigations) |
| Information security (ops-security) | YES | Co-author of threat-model.md |
| Engineering (axis-foundry-control-plane) | YES | LEAN lanes enforce |
| External auditor | At first audit cycle | Will cross-reference |
| EU AI Act notified body | At first high-risk-Annex-III tenant onboarding | FRIA outcome filed |

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary for purpose? | YES — operational + safety obligations cannot be met without supervisor. |
| Less intrusive alternative? | Considered: imperative configs (no Cedar evaluation). Rejected: weaker safety + no auditability. Considered: tenant-side enforcement only. Rejected: cross-tenant safety claims would be unverifiable. |
| Proportionate? | YES — collection limited to control-plane metadata (no payload bodies); supervision events carry SLI numbers + structural fields only. Per Art. 5(1)(c) data-minimisation. |
| Public / private interest? | YES — operational + safety of tenant agentic systems. |
| Anonymisation possible? | PARTIALLY — tenant identifiers pseudonymised (hashed `tenant_id`); autonomy entitlements cannot be anonymised without breaking purpose; pseudonymisation is the proportionate compromise. |
| Lawful basis (Art. 6) | Per §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare: Art. 9(2)(h) + HIPAA BAA; pack-kr: PIPA Art. 23(2) explicit consent. |
| Transfer basis (Arts. 44–46) | Per §2.2 — SCC-only; default residency by pack. |
| Retention | Fleet-state 2y; entitlements 5y (KR-FSS) / 6y (HIPAA); audit-chain indefinite (immutable Merkle). |
| Data-subject rights | Honoured per §6 (access, rectification, erasure, restriction, portability, objection, automated-decision-protections). |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-tenant fleet-state leak (tenant-A sees tenant-B's automation shape) | M | H | **H** |
| R-02 | Autonomy entitlement leak (sensitive DPO records cross-tenant) | L | H | **M** |
| R-03 | Kill-switch failure to engage allows runaway capability to act on end-user data | M | H | **H** |
| R-04 | Kill-switch false engage halts legitimate tenant service (availability harm) | L | M | **L-M** |
| R-05 | Automated autonomy precondition refuses a legitimate end-user invocation (Art. 22) | M | L-M | **M** |
| R-06 | Rogue capability deployment processes end-user data outside declared purpose | M | H | **H** |
| R-07 | Cross-border transfer of EU tenant data via mis-routed supervision events | L | H | **M** |
| R-08 | DSR (erasure) incompleteness — supervision events retain end-user-derived identifiers in payloads | M | M | **M** |
| R-09 | Joint-controllership confusion — tenant doesn't disclose to its end-users | M-H | M | **M-H** |
| R-10 | Sub-processor breach (cloud provider / OpenBao operator) | L | H | **M** |
| R-11 | Children's data (DPDPA §9; AI Act Art. 5) processed without consent | L | H | **M-H** |
| R-12 | PHI processed without BAA (pack-us-healthcare) | M | H | **H** |
| R-13 | EU AI Act high-risk capability deployed without FRIA | L | H | **M-H** |
| R-14 | Auditor mis-pivots from tenant-A to tenant-B during engagement | L | H | **M** |

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Residual | Owner |
|---|---|---|---|
| R-01 | Postgres RLS + Cedar tenant-scope + LEAN check + annual pen-test | L | ops-security |
| R-02 | OpenBao-resident entitlements; Postgres reference is opaque token only; auditor scope per-tenant | L | ops-security + council-privacy |
| R-03 | Kill-switch p99 ≤ 1 s SLO; fail-closed on Valkey outage (assume engaged); chaos drill quarterly; AC-02 mandatory | L | axis-foundry-control-plane + ops-security |
| R-04 | 2-person rule for fleet-wide; 5-s post-engage cancel; pre-engage confirmation | L | ops-security |
| R-05 | Per-tenant override path with 2-person rule; audit-chain emission; tenant can dispute via portal | L | axis-foundry-control-plane |
| R-06 | Capability YAML LEAN schema + PR review by tenant DPO + admit-loop autonomy gate + observability rollout gate | L-M | axis-foundry-control-plane + ops-security |
| R-07 | Pack-pinning at OTel + supervision-bus level; route by pack tag; mis-route caught by integration test | L | axis-foundry-control-plane |
| R-08 | DSR cascade scans Postgres + Valkey + supervision-event-bus; soft-delete then hard-delete after 30d grace; declared best-effort limitation | M | council-privacy |
| R-09 | Tenant DPA mandates joint-controllership disclosure; onboarding checklist verifies; non-disclosure = onboarding refused | L-M | council-privacy + gtm |
| R-10 | Sub-processor list maintained; per-vendor DPA + SCCs; quarterly security review | M | council-privacy + cloud-secrets |
| R-11 | Tenant DPA includes child-data clause; AI Act Art. 5 prohibitions enforced at admit-loop (specific patterns banned) | L | council-privacy |
| R-12 | pack-us-healthcare onboarding requires BAA; non-BAA tenants routed to non-PHI-pack | L | council-privacy + sales-legal |
| R-13 | Annex III sub-domain classifier at admit-time; high-risk capabilities require FRIA artifact (this DPIA's overlay or per-tenant overlay) before deploy | L | council-privacy + axis-foundry-control-plane |
| R-14 | Auditor JIT tokens per-tenant; engagement-window-bound mTLS; annual pen-test of auditor boundary | L | ops-security |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status |
|---|---|
| DPO | `pending` |
| Information Security Officer | `pending` |
| µservice owner (axis-foundry-control-plane lead) | `pending` |
| Council-architecture chair | `pending` |
| EU AI Act notified-body (when first high-risk Annex III tenant onboards) | `not yet engaged` |

**DPO advice:** All residuals are L or M after mitigations. Art. 36 prior consultation NOT triggered. EU AI Act Art. 27 FRIA pre-condition assessed: per-tenant FRIA required at first high-risk Annex III capability admit; this DPIA + tenant-specific overlay together fulfil. Recommendation: proceed with first-tenant onboarding subject to quarterly R-03 + R-06 review.

## Per-Pack Overlay Sections

### pack-kr (PIPA + ISMS-P)

- PIPA Art. 33 + Enforcement Decree Art. 35: this DPIA fulfils 개인정보영향평가 obligation for KR tenants.
- PIPA Art. 23 (sensitive PI): autonomy entitlements treated as sensitive; salted-hash on cross-tenant aggregates.
- PIPA Art. 23-2 (cross-border): forbidden; KR fleet state in pack-kr only.
- PIPA Art. 29 (technical safeguards): cross-mapped in `compliance.md`.
- PIPC Notice 2020-7 methodology followed.

### pack-us-healthcare (HIPAA)

- §164.308(a)(1)(ii)(A) Risk Analysis: this document fulfils.
- §164.502(a) TPO: capability rollout falls under Operations; never under Marketing.
- §164.502(b) Minimum Necessary: supervision events carry control-plane metadata only; no PHI payload.
- §164.504(e) BAA: oyatie is Business Associate for HIPAA tenants; BAA at `legal/baa-template.md`.
- §164.312(b) Audit Controls: Ed25519 audit-chain + 6y retention.
- §164.404/§164.406/§164.408 Breach Notification: integrated into `incident-response.md`.

### pack-eu (GDPR + EU AI Act + EDPB + eIDAS)

- EU AI Act Art. 9 (risk management): this DPIA + threat-model + compliance.md + ongoing post-market monitoring.
- EU AI Act Art. 12 (record-keeping): supervision-event audit-chain.
- EU AI Act Art. 13 (transparency): tenant-facing dashboards + Capability registry.
- EU AI Act Art. 14 (human oversight): 2-person rule + tenant-side disengage path + DPO read access.
- EU AI Act Art. 15 (accuracy, robustness, cybersecurity): cross-mapped to threat-model.md mitigations.
- EU AI Act Art. 27 (FRIA): per-tenant overlay at first high-risk Annex III capability admit.
- GDPR Art. 22 carve-out documented in §6 R-05.
- GDPR Arts. 44–50 transfers: SCC-only.
- NIS2 (2022/2555): incident timelines in `incident-response.md`.
- eIDAS 910/2014: Ed25519 audit-chain seals satisfy AdES.

### pack-jp (APPI)

- APPI Art. 17 purpose-of-use; APPI Art. 21 cross-border; APPI Art. 27 sensitive-data consent.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-supervisor-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- Any pack activation.
- Any change to processing purpose (§2.4) or data-class taxonomy.
- Any sub-processor change.
- Any breach notification triggered.
- New high-risk Annex III sub-domain entering tenant base.
- Post-incident (Sev-1/2).

## References

- ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140.
- `microservices/foundry-supervisor/threat-model.md`.
- `microservices/foundry-supervisor/compliance.md`.
- `microservices/foundry-supervisor/policy/{supervisor-isolation, data-residency}.md`.
- `microservices/foundry-supervisor/incident-response.md`.
- `microservices/foundry-supervisor/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa, fria-template}.md` (Slice-D scope).
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019, 9/2022.
- EU AI Act 2024/1689 Arts. 9, 10, 12, 13, 14, 15, 27 — `eur-lex.europa.eu/eli/reg/2024/1689`.
- GDPR Arts. 35, 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- LGPD Art. 38; DPDPA 2023 §10–§11.
