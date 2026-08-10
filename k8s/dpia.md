---
doc_class: DPIA
template_id: TPL-DPIA
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cloud
deciders: council-privacy, ops-security, axis-cloud, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0121, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - k8s/threat-model.md
  - k8s/policy/cluster-isolation.md
  - k8s/policy/data-residency.md
  - k8s/compliance.md
review_cadence: annually + on every cluster-version change, pack activation, or processing-purpose change
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation — YES (cluster hosts every tenant workload; control-plane mutations are systematic)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (conditional; PHI in pack-us-healthcare flows through pods; PV holds clinical data)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34, A.5.31"
  - "SOC 2 Privacy (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3/15/17/18/23/24/25/28/29/29-2/33", "PIPC Notice 2020-7", "KR CSAP"]
  pack-us-healthcare: ["HIPAA §164.308(a)(1)(ii)(A) risk analysis", "§164.310 physical", "§164.312 technical", "§164.502 minimum necessary", "§164.514 de-identification"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019 (Art. 25)", "NIS2", "DORA"]
  pack-jp: ["APPI Arts. 17, 18, 20, 21, 23, 27"]
  pack-sg: ["PDPA Parts III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1+11+12", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-12"]
  pack-br: ["LGPD Arts. 6/7/38 (RIPD); ANPD methodology"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["KSA PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: cloud-k8s µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. cloud-k8s triggers two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation | **YES** | cluster control-plane records every workload's pod-spec + container exec + log emit; systematic monitoring of all tenants. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional)** | pack-us-healthcare carries PHI through containers + PVs; pack-kr sensitive-data via shared cluster substrate. |
| Art. 35(3)(c): Public-area monitoring | NO | not applicable. |

KR PIPA Art. 33 + Enforcement Decree Art. 35 mandate a 개인정보영향평가 for systems processing sensitive personal information at scale — engaged.

Therefore: DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (per Art. 35), KR PIPC (per PIPA Art. 33), and HIPAA Covered Entity counsel at first-tenant onboarding.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** cloud-k8s schedules workload pods (which process tenant data), persists state in etcd (which holds pod-specs that may reference tenant-identifying labels + env vars), retains Kubernetes API audit logs (which capture every pod-lifecycle event), and emits Cilium flow logs + Istio access logs (which capture every east-west request).

**How:** kubeadm bootstraps the control-plane (kube-apiserver + etcd + scheduler + controller-manager); containerd + runc execute pods; Cilium + Istio handle network plane; CSI drivers mount PVs. The `kubernetes-api-proxy` mediates every kubectl / API call with Cedar + audit-chain.

**Where:** Per-pack region-pinned clusters (pack-kr → KR ap-seoul-1; pack-eu → EU eu-frankfurt-1; etc.) running on cloud-iac-provisioned bare-metal + OCI nodes. Per ADR-0117 + ADR-0121 + `policy/data-residency.md`.

**When:** Continuous; cluster always-on. Audit log emission per-call; flow log emission continuous.

**Who:** Per `threat-model.md` §"Actors": tenant API consumers, workload µservices, operators, Foundry agents, CI runners, sibling µservices (cell, observability), external auditors.

### 2.2 Scope of the processing

**Personal-data classes processed (cluster substrate perspective):**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Pod-spec labels (tenant-id), audit log entries | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest (operational) | ~10⁸ audit events/day per medium tenant |
| `PII_IDENTIFYING` | Env var contents that include user-ids when workload owner emits them | Art. 6(1)(b) contract | varies; redactor minimises |
| `PII_QUASI_IDENTIFIER` | Cilium flow log endpoints (IPs, ports) | Art. 6(1)(f) | ~10⁶ flows/day per medium tenant |
| `SENSITIVE_PIPA_ART23` | Tenant-id label correlated with auxiliary data | KR PIPA Art. 15 + 23 + 23-2 | 1 per scheduling event |
| `PHI` (pack-us-healthcare only) | PHI in containers + PVs if workload carries it | HIPAA §164.502(a) TPO per BAA | varies; in-cluster transit only |
| `AUDIT` | Cluster mutations (Foundry capability invocations, operator kubectl, CI runs) | Art. 6(1)(c) legal obligation | 1 record per mutation |
| `SECRET` | etcd encryption key, Istio root CA, kubeadm join token | not personal data; ISO 27001 A.5.17 | varies |

**Geographical scope:** Per pack (per `policy/data-residency.md`):
- pack-kr: ap-seoul-1.
- pack-eu: eu-frankfurt-1 + eu-amsterdam-1 (DR).
- pack-us: us-ashburn-1 + us-phoenix-1.
- pack-us-healthcare: us-ashburn-1 (HIPAA-eligible).
- pack-jp/sg/au/in/br/ae/ksa: each pinned.

**Cross-border transfer:** Forbidden by default. Allowed only with tenant-executed SCCs (GDPR) or equivalent transfer mechanism. Recorded at `microservices/cloud-k8s/legal/transfer-register.md`.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications; tenant operators; oyatie operators.
- **Relationship:** Joint controllership with tenant under GDPR Art. 26 (for tenant-end-user data); recorded in tenant DPA template.
- **Reasonable expectations:** Tenant operators expect cluster-level isolation + audit-chain. End-users expect operational data collection disclosed in tenant's privacy notice.
- **Previous experience:** Bominal cloud-k8s analogue: no DPA complaints in 24 months on equivalent processing pattern. Lessons inherited per `feedback_bominal_inheritance_precedence.md`.
- **Industry codes:** Voluntary alignment with CNCF security best-practices (CIS K8s Benchmark + NSA Hardening).

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Schedule + execute tenant workloads** | Necessary for tenant SLA | Art. 6(1)(b) contract |
| **Persist cluster state (etcd) for failure recovery** | Necessary for operational integrity | Art. 6(1)(b) + Art. 6(1)(f) |
| **Audit-chain emission per cluster mutation** | Mandatory for SOC 2 + HIPAA + KR PIPA + GDPR Art. 30 | Art. 6(1)(c) |
| **Network policy + flow logging** | Necessary for security (cross-tenant detection) | Art. 6(1)(f) |
| **Marketing / unrelated commercial use** | NOT a purpose | N/A — explicitly excluded |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| Data Protection Officer (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representative | Scheduled pre-GA | Feedback folded into §6 |
| Data subjects | Indirect via tenant onboarding | Joint-controllership clause |
| Supervisory authority | Prior consultation NOT triggered (residual ≤ Medium after mitigations) | If residual escalates, Art. 36 trigger |
| Information security team (ops-security) | YES — co-author of threat-model.md | Joint risk register |
| Engineering teams (axis-cloud) | YES | Mitigations enforced at CI |
| External auditor | At first audit cycle | Cross-references this DPIA |

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary? | YES — workload execution + cluster availability cannot happen without the substrate. |
| Less intrusive alternative? | Considered: per-tenant dedicated bare-metal (no shared substrate). Rejected: cost-prohibitive + slower; multi-tenant cluster with strong isolation is the industry norm (EKS / GKE / AKS all multi-tenant). Pack-pinning + namespace isolation + Cilium/Istio policy is the proportionate compromise. |
| Proportionate to purpose? | YES — collection limited to: cluster control-plane state; audit logs of operational events; flow logs (metadata only, no payload). PII redaction enforced at workload µservice layer per `observability` µservice's redactor; cluster substrate carries minimal PII. |
| Public / substantial private interest? | YES — operational reliability of tenant production systems. |
| Anonymised / pseudonymised alternative? | PARTIALLY — tenant-id is hashed for cross-pack routing decisions; full anonymisation prevents per-tenant cluster operations. |
| Lawful basis | Per §2.4 |
| Special-category basis | pack-us-healthcare PHI: Art. 9(2)(h) + HIPAA BAA. pack-kr sensitive: PIPA Art. 23(2) explicit consent at onboarding. |
| Transfer basis | SCCs; default residency by pack |
| Retention | Audit log: 6y (us-hc), 5y (kr), 2y default. etcd snapshots: 14d. Flow logs: 7d hot + 30d cold. Honour DSR cascade. |
| Rights of data subjects | Per §6 mitigations: access (Art. 15), rectification (Art. 16), erasure (Art. 17), restriction (Art. 18), portability (Art. 20), objection (Art. 21), automated-decision-protections (Art. 22). |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-tenant data leak via Cilium / Istio policy regression | M | H | **M-H** |
| R-02 | Container escape exposes tenant PV data | L | H | **M** |
| R-03 | etcd disk theft exposes cluster state metadata | L | H | **M** |
| R-04 | Audit log retention enables longitudinal profiling of tenant business behaviour | M | M | **M** |
| R-05 | kubectl exec captures running workload secrets including PII | M | H | **H** |
| R-06 | SA token theft + cross-namespace API access | M | H | **H** |
| R-07 | DSR (right-to-erasure) incomplete because PV snapshots persist data beyond user request | M | M | **M** |
| R-08 | Joint-controllership confusion: tenant doesn't disclose oyatie's cluster-level processing to its end-users | M-H | M | **M-H** |
| R-09 | PHI in pack-us-healthcare pod logs / PVs without proper BAA | L | H | **M-H** |
| R-10 | Sub-processor (cloud provider, registry) breach exposes cluster state | L | H | **M** |
| R-11 | Cross-border transfer of EU-resident pod-state via misrouted scheduling | L | H | **M** |
| R-12 | Operator JIT elevation token used post-engagement | L | H | **M** |
| R-13 | Foundry agent exceeds autonomy ceiling → unauthorised cluster mutation | L | H | **M** |
| R-14 | Children's data (DPDPA §9; pack-in) processed without parental consent at workload layer (cluster cannot detect) | L | H | **M-H** |
| R-15 | Auditor mis-pivot from tenant-A to cluster-wide reads | L | H | **M** |

Each risk has at least one mitigation in §6 + corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (policy regression cross-tenant) | Istio STRICT mTLS mesh-wide; Cilium NetworkPolicy kernel-layer; LEAN `oya-check-network-policy-conformance`; continuous-state validator; pen-test annually | L | axis-cloud + ops-security |
| R-02 (container escape) | seccomp/AppArmor; non-privileged; user namespaces; gVisor opt-in; kernel CVE tracking | M (residual irreducible) | ops-security |
| R-03 (etcd disk theft) | KMS-envelope encryption-at-rest; PV access via per-component IAM; physical security via cloud-iac µservice | L | ops-security + cloud-iac |
| R-04 (longitudinal profiling) | Retention bounded per pack; cold-tier aggregation; DSR cascade; data_class enforcement | L-M | council-privacy |
| R-05 (kubectl exec exposes secrets) | Cedar refuses `pods/exec` to non-operator; JIT + reason field required; refused on production-tier; audit-chain emit | L-M | ops-security |
| R-06 (SA token theft) | `automountServiceAccountToken: false` default; Bound SA tokens; api-proxy validates token-to-pod binding; Kyverno restrictions | L | ops-security |
| R-07 (DSR incompleteness on PV) | DSR cascade through CSI snapshot lifecycle; 30d SLA; documented best-effort residual | M | council-privacy |
| R-08 (joint-controllership) | Tenant DPA mandates upstream disclosure clause; onboarding checklist verifies disclosure | L-M | council-privacy |
| R-09 (PHI without BAA) | pack-us-healthcare gated by BAA at workload-µservice onboarding (cluster substrate enforces pack-routing) | L | council-privacy + sales-legal |
| R-10 (sub-processor breach) | Sub-processor list; per-vendor DPA; quarterly review; supply-chain admission (Cosign + Trivy) | M (irreducible) | council-privacy |
| R-11 (cross-border misroute) | Pack-routing enforced at cluster boundary (workload cannot schedule across packs); integration test verifies | L | axis-cloud |
| R-12 (post-engagement token use) | JIT TTL ≤ 4h; non-renewable; OpenBao revocation on engagement end | L | ops-security |
| R-13 (Foundry agent over-autonomy) | Per-capability autonomy tier; 2-person rule on T3; audit-chain emit; anomaly detection | L | axis-foundry + ops-security |
| R-14 (children's data) | pack-in DPA includes age-gating clause; cluster substrate doesn't detect ages; tenant responsibility | L (residual depends on tenant) | council-privacy |
| R-15 (auditor mis-pivot) | Auditor Cedar scope to specific tenants; pen-test annually | L | ops-security |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer | `pending` | TBA at first-tenant onboarding |
| Information Security Officer | `pending` | TBA |
| µservice owner (axis-cloud lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all L or M (no H or M-H residuals remain). Therefore Art. 36 prior consultation NOT triggered. DPO advises proceeding subject to:
- Quarterly review of R-02 (container escape residual).
- Quarterly review of R-07 (DSR cascade residual on PVs).
- Annual review of this DPIA.
- Re-trigger DPIA on any pack-activation, K8s version upgrade, or processing-purpose change.

**Outcomes documented:**
- Mitigations adopted: every measure in §6 in-scope for IP-001 .. IP-015.
- ROPA entry: `microservices/cloud-k8s/legal/ropa.md`.
- Joint-controllership template: `microservices/cloud-k8s/legal/dpa-template.md`.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + ISMS-P + CSAP)

PIPA Art. 33 + Enforcement Decree Art. 35 require a 개인정보영향평가; this document fulfils that for KR tenants.

- **PIPA Art. 23**: sensitive data via tenant-id correlation; salt rotation mitigates.
- **PIPA Art. 23-2**: cross-border forbidden; pack-pinning enforces.
- **PIPA Art. 28**: retention bounded; per asset table.
- **PIPA Art. 29**: technical safeguards cross-mapped in §6.
- **KR-ISMS-P methodology**: this DPIA's structure follows PIPC Notice 2020-7's 7-step methodology.
- **KR CSAP cloud-security**: cluster-isolation + audit retention ≥ 5y + cross-border-forbidden inherited.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk analysis substantially equivalent to a DPIA. This document fulfils that.

- **§164.502(a) TPO**: operations scope covers cluster substrate.
- **§164.502(b) minimum necessary**: redaction at workload layer; cluster carries minimal PII.
- **§164.504(e) Business Associate**: oyatie is BA for HIPAA-scope tenants; BAA template at `legal/baa-template.md`.
- **§164.310 physical**: inherited from cloud-iac + OCI HIPAA attestation.
- **§164.312(b) audit**: audit-chain seal + ≥ 6y retention.
- **§164.404 breach notification**: integrated in incident-response.

### pack-eu (GDPR + EDPB + NIS2 + DORA + eIDAS)

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing.

- **EDPB Guidelines 4/2019**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022**: 72h notification chain in incident-response.
- **NIS2**: cluster crosses Annex I thresholds when tenant count threshold met; 24h+72h+1mo timelines apply.
- **DORA 2022/2554**: for EU financial-services tenants — operational-resilience testing applies.
- **eIDAS 910/2014**: Ed25519 audit-chain seals are AdES.
- **Schrems II + Arts. 44–46**: pack-eu is EU-resident; cross-border forbidden by default.

### pack-jp (APPI)

- **APPI Art. 17 (purpose of use)**: declared at tenant onboarding.
- **APPI Art. 20 (security control measures)**: cross-mapped.
- **APPI Art. 21 (entrustee supervision)**: sub-processor list.
- **APPI Art. 27 (sensitive consent)**: tenant DPA captures.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/cloud-k8s-dpia-overlay.md`. Each follows this document's 7-step structure with local law citations.

## Re-review Triggers

- Annually (Q2).
- New pack activation.
- Kubernetes / containerd / Istio / Envoy / Cilium version change.
- Change to processing purpose (§2.4) or data classes.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.

## References

- ADR-0028 (Bominal audit chain).
- ADR-0117, ADR-0120, ADR-0121, ADR-0139, ADR-0131, ADR-0140.
- `k8s/threat-model.md`.
- `k8s/policy/{cluster-isolation, data-residency}.md`.
- `k8s/compliance.md`.
- `k8s/incident-response.md`.
- `microservices/cloud-k8s/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md`.
- ICO + CNIL DPIA templates.
- EDPB Guidelines 4/2019 + 9/2022.
- PIPC Notice 2020-7.
- GDPR Arts. 35 + 36.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
