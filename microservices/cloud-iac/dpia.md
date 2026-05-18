---
doc_class: DPIA
template_id: TPL-DPIA
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cloud-iac
deciders: council-privacy, ops-security, axis-cloud-iac, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/cloud-iac/threat-model.md
  - microservices/cloud-iac/policy/iac-isolation.md
  - microservices/cloud-iac/policy/data-residency.md
  - microservices/cloud-iac/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — NO (cloud-iac processes IaC, not data-subject behaviour)"
  - "Art. 35(3)(b): large-scale processing of special-category data — CONDITIONAL (state files may carry hashed tenant ids; pack-us-healthcare PHI possible if redactor fails)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 25, 28, 30, 32, 33, 35"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII)"
  - "SOC 2 Privacy criteria (P1-P8, 2017 TSC)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 24/29/33 (영향평가)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis)", "§164.502(b) (minimum necessary)"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EDPB Guidelines 4/2019"]
  pack-jp: ["APPI Arts. 17, 18, 20"]
  pack-sg: ["PDPA Part III + IV"]
  pack-au: ["Privacy Act 1988 APP 1 + 11 + 12"]
  pack-in: ["DPDPA 2023 §10–§11"]
  pack-br: ["LGPD Arts. 38 (RIPD)"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["KSA PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: cloud-iac µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is likely to result in a high risk to data-subject rights. Cloud-iac processes IaC manifests + apply-state index data; on its face this is operational data, not data-subject data. However:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | NO | cloud-iac doesn't evaluate data subjects; it evaluates IaC manifests and cluster state. |
| Art. 35(3)(b): Large-scale processing of special-category data | **CONDITIONAL** | OpenTofu/OpenTofu state files may inadvertently carry hashed tenant identifiers; pack-us-healthcare deployments may carry PHI in resource names/labels if redactor fails. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | cloud-iac doesn't monitor any public area. |

In addition, when pack-us-healthcare or pack-kr (PIPA Art. 23 sensitive) is activated, the conditional trigger applies. Therefore: a DPIA is mandatory pre-deployment for pack-us-healthcare and pack-kr (and prudent for all packs).

This document is the canonical DPIA reviewed by EU DPAs (Art. 35) + Korean PIPC (PIPA Art. 33) + HIPAA OCR (informally) at first-tenant onboarding.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** cloud-iac renders Helm + Kustomize + Terraform/OpenTofu manifests for every oyatie µservice; validates them; applies them to per-pack workload clusters; rolls back when downstream signals (SLO gate burn-rate breach) demand; detects drift; emits apply audit events.

**How:** Per-µservice IaC sources live at `microservices/<ms>/iac/{helm,terraform,kustomize}/` in git. The iac-renderer-worker reads sources + runs Helm/Kustomize/OpenTofu CLIs; iac-validator-worker validates output + plan-previews against live cluster; iac-applier-worker applies via ArgoCD or direct Kubernetes API; iac-rollback-worker reverts on signal; iac-registry maintains per-(microservice, pack, environment) apply-state index in Postgres.

**Where:** Per-pack region-pinned components (Postgres iac-state-index + OpenTofu state buckets) per pack. Cloud-iac control-plane runs in a dedicated Kubernetes cluster.

**When:** Continuous. Renders triggered per-PR; applies triggered on EligibilityChanged (eligible) events from observability. Drift-detection runs ≤1h cycle per cluster.

**Who:** Per actor table in `microservices/cloud-iac/threat-model.md` §"Actors".

### 2.2 Scope of the processing

**Personal-data classes processed (in cloud-iac's scope):**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `INTERNAL_ONLY` | IaC manifest text | Art. 6(1)(f) legitimate interest (operational) | ~10⁴ manifests across the catalog |
| `BEHAVIORAL_TENANT_PRODUCT` | Live cluster state per tenant (drift comparison snapshots); apply-state index entries | Art. 6(1)(b) contract necessity + Art. 6(1)(f) | ~10² entries per µservice per month |
| `AUDIT` | ApplyExecuted / RenderCompleted / Rollback / Drift events | Art. 6(1)(c) legal obligation + Art. 6(1)(f) | ~10³ events per µservice per month |
| `SENSITIVE_PIPA_ART23` (KR pack) | Hashed customer ID when present in apply-state index labels | KR PIPA Art. 23 (sensitive) — pseudonymous; never materialised | varies per pack |
| `PHI` (pack-us-healthcare only; conditional) | Patient identifiers / clinical data IF redactor fails to strip them from a workload's IaC label set | HIPAA §164.502(a) Permitted Uses (Operations) per BAA | targeted to 0 |
| `SECRET` | Cluster kubeconfigs, ArgoCD tokens, OpenTofu state-encryption keys | not personal data; managed under ISO 27001 A.5.17 | varies |

**Geographical scope:** Per pack (mirrors observability data-residency contract).

**Cross-border transfer:** Forbidden by default per `data-residency.md`. Allowed only with tenant-executed SCCs (GDPR Arts. 44–46) recorded in `microservices/cloud-iac/legal/transfer-register.md` (Slice D).

### 2.3 Context of the processing

- **Data subjects:** Not the direct subject of cloud-iac processing; cloud-iac operates on IaC artifacts that describe infrastructure for tenant workloads. Tenants' end-users may be indirectly affected (e.g., a misapplied IaC could cause tenant downtime) but cloud-iac doesn't process end-user data directly.
- **Relationship to data subjects:** Indirect; via the workload µservices cloud-iac applies for.
- **Reasonable expectations:** Tenant operators expect IaC-driven deployments; end-users expect operational reliability.
- **Previous experience:** Bominal-equivalent operated under similar pattern; no DPA-triggered complaints in 24 months. Lessons inherited per `feedback_bominal_inheritance_precedence.md`.
- **Industry codes:** OpenSSF SLSA L3; CNCF best practices for GitOps + supply-chain.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Render IaC deterministically** | Operational necessity | Art. 6(1)(b) contract + Art. 6(1)(f) |
| **Validate apply scope per Cedar policy** | Operational + privacy (prevents cross-tenant mutation) | Art. 6(1)(b) + Art. 6(1)(f) |
| **Apply to workload-cluster** | Operational necessity for SLA fulfillment | Art. 6(1)(b) |
| **Detect drift** | Operational + integrity | Art. 6(1)(b) + Art. 6(1)(c) legal obligation (records retention) |
| **Auto-rollback on downstream SLO breach** | Operational + incident-response | Art. 6(1)(b) + Art. 6(1)(c) (incident notification under Art. 33) |
| **Maintain apply-state index** | Mandatory for audit / SOC 2 / ISO 27001 / HIPAA / KR PIPA | Art. 6(1)(c) legal obligation |
| **Cross-tenant aggregation** | NOT a purpose | N/A — explicitly excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (DPO) | YES — council-privacy chair | Sign-off pending; see §7 |
| Tenant representatives (sample of 3 prospective tenants for first-rollout) | Scheduled — pre-GA | Feedback to be folded into Step 6 measures |
| Information security team (ops-security) | YES — co-author of threat-model.md | Threat-model + DPIA share residual-risk catalog |
| Engineering teams (axis-cloud-iac + downstream µservice owners) | YES | Cedar policy + state-secret-scan + per-pack-pinning enforced at CI |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Prior consultation (Art. 36) — NOT triggered (no residual high risk after mitigations; see §6 + §7) | If §6 mitigations residual > Medium, Art. 36 triggers |
| External auditor (SOC 2 / ISO 27001 / SLSA-L3) | At first audit cycle | Cross-references this DPIA |

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary to achieve the purpose? | YES — IaC-driven deployment + apply audit + drift detection cannot be performed without these data flows. |
| Is there a less intrusive alternative? | Considered: per-µservice apply isolation via separate Kubernetes clusters per µservice. Rejected: prohibitive cost + operational complexity at oyatie scale. The current shared-cluster + Cedar-policy approach is the proportionate compromise. |
| Is processing proportionate? | YES — IaC artifacts are operational; tenant data appears only inadvertently in state files (mitigated by secret-scan + redactor). |
| Public interest or substantial private interest? | YES — operational reliability of tenant production systems; legitimate interest documented in tenant DPA. |
| Anonymised / pseudonymised data sufficient? | PARTIALLY — pseudonymisation (hashed customer ID) applied at tenant-identifier boundary; full anonymisation would defeat per-tenant apply attribution required for accounting + audit. |
| Lawful basis (Art. 6) | Per §2.4. |
| Special-category basis (Art. 9) | pack-us-healthcare PHI: Art. 9(2)(h) + BAA. pack-kr sensitive: PIPA Art. 23(2) explicit consent at tenant onboarding. |
| Transfer basis (Arts. 44–46) | SCC-only per `data-residency.md`. |
| Retention | Apply-state index ≥ 6y for HIPAA pack; ≥ 3y KR pack; ≥ 2y universal. |
| Rights of data subjects | Indirect — via the workload µservices cloud-iac applies for; DSR cascade is in the workload µservice's DPIA, not here. |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-µservice apply causes tenant-A's workload to mutate tenant-B's resources | L-M | H | **M-H** |
| R-02 | OpenTofu state file leaks tenant identifier or PHI/PII | M | H | **H** |
| R-03 | Misapplied IaC causes tenant downtime (affecting end-users) | M | M | **M** |
| R-04 | Drift cascade triggers alert storm; on-call fatigue masks real incident | M | M | **M** |
| R-05 | Supply-chain attack on chart cascades to multiple tenants | L | H | **M** |
| R-06 | Sub-processor (cloud provider / OpenTofu) breach exposes state files | L | H | **M** |
| R-07 | Tenant misconfiguration in own IaC causes their own exposure | M | M-H | **M-H** |
| R-08 | Joint-controllership confusion (tenant doesn't disclose cloud-iac to its end-users) | M | M | **M** |
| R-09 | Auditor mis-pivots from tenant-A to tenant-B during engagement | L | H | **M** |
| R-10 | Cross-border transfer of EU-resident state file via misrouted apply | L | H | **M** |
| R-11 | PHI in apply log (pack-us-healthcare without BAA) | M | H | **H** |

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (cross-µservice mutation) | Cedar policy `iac-isolation.md` per-µservice scope; cluster RBAC namespace-scoped; LEAN check at PR; quarterly pen-test | L | axis-cloud-iac + ops-security |
| R-02 (state file leak) | State content secret-scan + redactor; SSE-KMS at rest; access via service-account mTLS only; JIT for human; quarterly historical secret-scan | M (engineering discipline floor) | ops-security + cloud-secrets + axis-cloud-iac |
| R-03 (misapply downtime) | Plan-preview at PR; SLO gate downstream consumer (rejects bad applies); auto-rollback on breach | L-M | axis-cloud-iac + observability |
| R-04 (drift cascade) | Drift event grouping by µservice + resource-kind; backpressure on drift queue; on-call dashboard distinguishes signal from noise | L-M | axis-cloud-iac + ops-sre-reliability |
| R-05 (supply-chain) | Cosign verify + SLSA L3 + chart allowlist + Rekor transparency log | L | ops-security + axis-cloud-iac |
| R-06 (sub-processor) | Sub-processor list at `legal/sub-processors.md`; quarterly review; SSE-KMS minimizes blast radius | M (irreducible sub-processor risk) | council-privacy + cloud-secrets |
| R-07 (tenant misconfig) | Cedar policy refuses tenant operator from mutating outside own µservice; default-deny scope on tenant entitlements | L-M | axis-cloud-iac |
| R-08 (joint-controllership) | Tenant DPA includes upstream disclosure; tenant onboarding checklist verifies | L-M | council-privacy + gtm-customer-success |
| R-09 (auditor pivot) | Auditor JIT tokens iac-state-index per-tenant filtered; folder isolation tested annually | L | ops-security |
| R-10 (cross-border) | Pack-pinning enforced at apply-router level; misroute = config error caught by integration test | L | axis-cloud-iac |
| R-11 (PHI without BAA) | pack-us-healthcare onboarding requires BAA before tenant ingest enabled; non-signed tenants pre-flighted to non-PHI-pack | L | council-privacy + sales-legal |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-cloud-iac lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all rated L or M; no H or M-H residuals after mitigations. Therefore Art. 36 prior consultation NOT triggered. The DPO advises proceeding subject to:
- Quarterly review of R-02 (state file secret-leak residual) — engineering-discipline metric over time.
- Annual review of this DPIA.
- Re-trigger DPIA on every pack-activation.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 require a 개인정보영향평가 for systems handling sensitive personal information at scale. This document fulfils that obligation for KR tenants where cloud-iac touches hashed tenant identifiers.

- **PIPA Art. 24 (resident-registration-number protection)**: not processed directly.
- **PIPA Art. 29 (technical safeguards)**: mitigations §6 map to safeguards.
- **PIPA Art. 33-2 (DPO appointment)**: council-privacy chair serves PIPA DPO role for KR-resident tenants.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk analysis substantially equivalent to DPIA. This document fulfils that.

- **§164.502(a) (Permitted Uses)**: TPO — Operations scope covers cloud-iac.
- **§164.502(b) (Minimum Necessary)**: state-file secret-scan + redactor enforce.
- **§164.504(e) (Business Associate)**: cloud-iac is a sub-processor under the BAA chain.
- **§164.316(b)(2) (retention)**: ≥ 6y for apply audit-relevant data.

### pack-eu

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing in cloud-iac's scope.

- **EDPB Guidelines 4/2019 (Art. 25)**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h notification chain in `incident-response.md`.
- **NIS2 (2022/2555)**: 24h/72h/1mo timelines.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-iac-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2 each year).
- On every new pack activation.
- On any change to processing purpose (§2.4) or data-class taxonomy.
- On any sub-processor change.
- On any breach notification triggered (per Art. 33 + state laws).
- Post-incident (Sev-1 / Sev-2 in cloud-iac).

## References

- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0117: Cloud-native infrastructure.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/cloud-iac/threat-model.md`.
- `microservices/cloud-iac/policy/iac-isolation.md`.
- `microservices/cloud-iac/policy/data-residency.md`.
- `microservices/cloud-iac/compliance.md`.
- `microservices/cloud-iac/incident-response.md`.
- `microservices/observability/dpia.md` (cross-µservice reference).
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines; PIPC Notice 2020-7; HIPAA §164.308(a)(1)(ii)(A).
