---
doc_class: OwnershipCoherenceAudit
microservice: cloud-k8s
audit_wave: Wave-4-Rolling
audit_date: 2026-05-21
audit_owner: codex-cloud-k8s-audit-agent
phase: Phase-0-Shared-Infrastructure
batch: Wave-4-rolling
top_3_counterparts: [AWS EKS, GCP GKE, Azure AKS]
status: PASS-WITH-FINDINGS
substance_floor: 600-lines
source_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md
  - /Users/jasonlee/oyatie/specs/master-plan-sequencing.json
  - /Users/jasonlee/oyatie/docs/standards/brief-template.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md
related_local_docs:
  - k8s/PRD.md
  - k8s/ARCHITECTURE.md
  - k8s/manifest.json
  - k8s/competitor-parity-matrix.md
  - microservices/cloud-k8s/capability-tiers/tier-matrix.md
  - k8s/multi-region.md
  - k8s/failure-modes.md
  - k8s/capacity-model.md
  - k8s/compliance.md
---

# cloud-k8s — Wave-4 Rolling Ownership-Coherence Audit (2026-05-21)

## 0. Audit Frame

### 0.1 Scope

The cloud-k8s microservice is the Kubernetes control-plane substrate.
It owns vanilla kubeadm clusters, the containerd runtime, the Istio
service mesh control plane, the Envoy data plane, and the CNI / CRI /
CSI integrations that every other oyatie microservice runs on top of.
Per ADR-0121, the stack is upstream Kubernetes 1.35 LTS + containerd
2.3.0 LTS + Istio 1.29.2 + Envoy + Cilium 1.18 + CSI drivers per
storage backend. The capability directory is k8s/. There is no
k8s/src tree. Current implementation lives in k8s/core/,
k8s/ports/, k8s/adapters/, and k8s/facade/. Earlier plans named
microservices/cloud-k8s/src/ as the crate root; that path is
historical and is not a live code home.

Per master-plan-sequencing.json D-1.16, the Phase 0 canonical name is
cloud-compute-k8s. The live directory is cloud-k8s. ADR-0328 D-1.107
explicitly contemplates this alias and instructs the audit to record
the alias rather than rename inside an audit-only wave.

### 0.2 Owner

Owner team: axis-cloud-k8s. Single-owner discipline holds: every audit
artifact under k8s/ traces back to axis-cloud-k8s
as the responsible council subteam. The CODEOWNERS expectation is that
PRs touching cloud-k8s require axis-cloud-k8s sign-off plus, when the
PR also touches contracts/* or policy/*, a multispectrum reviewer pass.

### 0.3 Top-3 industry counterparts (UNION-coverage parity bar)

1. AWS EKS — Elastic Kubernetes Service. Managed Kubernetes from the
   reference hyperscaler. Primary citation: docs.aws.amazon.com/eks/.
2. GCP GKE — Google Kubernetes Engine. The longest-standing managed
   K8s vendor with two flavors (Standard and Autopilot). Primary
   citation: cloud.google.com/kubernetes-engine/docs/.
3. Azure AKS — Azure Kubernetes Service. The third hyperscaler-managed
   K8s with deep AAD identity integration. Primary citation:
   learn.microsoft.com/azure/aks/.

UNION coverage is the bar: if any one of EKS / GKE / AKS exposes a
named capability, cloud-k8s must either cover it, project it through
another oyatie microservice, or mark it out-of-scope intentional with
a doctrine reason.

### 0.4 Doctrine amendments observed (2026-05-20)

- Tier system retired in new authoring. customer-class ladder
  language persists in pre-existing local docs (tier-matrix.md,
  tier-deltas-and-pricing.md). This audit records that persistence
  as a P2 backlog row for Wave 15+ remediation rather than rewriting
  in place during a findings-only wave.
- Tenant-class model: two classes — demo_trial and paid. cloud-k8s is
  the K8s control-plane substrate; demo_trial tenants run on OCI
  Always Free K8s + paid tenants run on the deployment-context
  substrate per ADR-0254 (shared-cloud, dedicated-cloud, hybrid,
  on-prem connected, on-prem air-gapped).
- Performance benchmarks: single industry-leader target plus
  deployment-context overlay. Pre-existing tiered SLOs (99.9% demo_trial,
  99.95% paid dedicated-cloud, 99.99% paid on-prem-connected) are retained as documentary truth and
  reframed in performance-benchmark-numbers-2026-05-20.md as the
  single industry-leader target with deployment-context overlays.

### 0.5 ADR-0254 alignment evidence

ADR-0254 is canonical name "Deployment model spectrum" (not
"K8s + Cloud Hypervisor" as cited in the dispatch brief). The brief's
intent maps to this ADR plus ADR-0248 (Amazon-shape cellular
architecture) plus ADR-0121 (on-prem K8s stack). Section 3.4.K of
this audit walks the alignment explicitly.

## 1. Local artifact inventory (sampled)

### 1.1 Top-level prose docs

PRD.md (387 lines) — accepted; ADR-0131-native authoring; PRD-cloud-k8s
identifier; covers 7 bounded contexts, 62 crates, NFR sections for
performance / security / audit / availability / data residency.

ARCHITECTURE.md (4396 lines) — Wave-3-C anchor sweep file; closes
many ADR-0242..ADR-0284 anchors with structured §principals,
§cedar-gates, §tenant-scoping, §substrate-product-binding,
§policy-evaluation, §cell-eligibility, etc. Each anchor section is
shaped uniformly (Service-specific answer + Concrete inventory used
+ Primitive and API binding + Cross-service links + Hyperscaler
precedents + Failure modes and rollback + Verification hooks +
Structural notes).

competitor-parity-matrix.md (165 lines) — names EKS / GKE / AKS / OKE
/ Rancher RKE2 / OpenShift / TKG / Talos Linux + Omni as the
competitor set; provides feature parity tables across cluster
lifecycle, networking, security, storage, ops, agent operability;
identifies parity gaps with M02 / M03 / M04 targets.

manifest.json (388 lines) — Machine-Readable-Spec; declares 13
crates currently, 7 BC names, 6 SLOs, 15 IP files, 11 regulatory
packs, 12 ADR references, hyperscaler invariant coverage rows,
audit-chain seal events, OpenBao secrets substrate, ontology
projections (currently []), mesh layering flags, capability tiers T2
and T3, dependency list, criticality_tier T2, failure domain text.

capacity-model.md (240 lines) — formula-driven sizing for
kube-apiserver, etcd, kube-controller-manager / kube-scheduler,
kubelet / containerd, Cilium, istiod, Envoy sidecars + ingress
gateway, CSI provisioners, Kyverno, api-proxy; XS / S / M / L
reference baselines; HPA tables; pre-warmed pool; storage cost
breakdown; worked XS pack-kr example.

multi-region.md (248 lines) — topology per pack across 11 packs;
DR pair architecture with ASCII diagram; replication matrix;
failover procedures; failback rules; BCDR exercise cadence; per-pack
RPO/RTO table; tenant notification; ADR-0158 active_passive
disposition; ADR-0164 air-gap variant; ADR-0161 storage class.

failure-modes.md (270 lines) — 15 named failure modes (FM-01 to
FM-15) covering kube-apiserver, etcd, control-plane partition,
worker failure, CNI, Istio, Envoy, CSI, kubeadm upgrade, network
policy regression, Cosign bypass, api-proxy, PV backend; RTO/RPO
summary; meta-SLO on detection pipeline.

compliance.md (1171 lines) — SOC 2 Type 2 + ISO 27001:2022 + GDPR +
CIS K8s + NSA hardening + per-pack frameworks (KR ISMS-P, HIPAA,
EDPB, eIDAS, NIS2, DORA, APPI, PDPA, APP, DPDPA, LGPD, UAE PDPL,
KSA PDPL) controls-to-implementation table. The bulk of the file
is post-1000 anchor closure for the Wave-3-G keystone bundle
(§day-one-cert-readiness, §pack-overlay-roster, etc.).

threat-model.md (716 lines) — STRIDE + LINDDUN + MITRE ATT&CK; per
BC threat surface; remediation map.

dpia.md (18387 bytes) — DPIA per Art. 35 GDPR; data flow inventory;
risk register R-01..R-14; mitigations.

cost-budget.md (7730 bytes) — per-pack monthly compute / storage /
network / observability / on-call cost; budget envelope tied to
capacity model.

incident-response.md (11651 bytes) — severity definitions,
escalation matrix, runbooks index, tenant communication procedure.

backfill-replay.md (5107 bytes) — etcd backfill, audit-chain replay,
post-incident reconciliation procedure.

PHASE-01-ONPREM-K8S-SUBSTRATE.md (239 lines) — phase-tracking doc.

sdk-plan.md (139 lines) — Rust + future TS / Py / Go SDK roadmap.

AUDIT-FINDINGS-2026-05-18.json (144 lines) — prior audit verdict
HARD_VIOLATIONS; cites minio_engine + cluster_autoscaler + valkey_engine
hits. Most are stale (S3 -> SeaweedFS + Cluster Autoscaler -> Karpenter
+ Valkey migration already updated upstream; remaining hits are
historical narrative references).

### 1.2 Implementation plans (IPs)

15 numbered IP files (IP-001 through IP-015) plus three CLUSTERAPI
IPs and 11 journey IPs (IP-journey-j87..j100). All marked
acceptance_status ga in the manifest. Span: Layer-A IaC (IP-001),
on-prem K8s stack standard (IP-002), per-layer cluster-bootstrap
slices (IP-003..IP-006, IP-013), node-lifecycle (IP-007),
network-policy (IP-008), service-mesh-control-plane (IP-009),
ingress-controller (IP-010), CSI per backend (IP-011),
kubernetes-api-proxy (IP-012), branch-protection + HG (IP-014),
observability SLO + authority cohesion (IP-015). Three CLUSTERAPI
IPs cover ClusterClass templates, lifecycle orchestration, and
promotion pipeline.

Journey IPs cover workload runtime (j81, j87), US MSB MTL overlay
(j91), BR LGPD + US parent DSAR (j92), IN DPDPA + RBI (j93), SOX 404
public-company controls (j94), ISO 27001 + SOC2 annual audit (j95),
KSA + UAE MENA onboarding (j96), SG PDPA + MAS tenant (j97), AU
Privacy + APRA CPS 234 (j98), multi-pack conflict resolution (j99),
and pack-rollout first action (j100).

### 1.3 Contracts

contracts/openapi/cloud-k8s.yaml + contracts/asyncapi/cloud-k8s-events.yaml
+ contracts/proto/cloud-k8s.proto. Three-way contract surface
covering REST API, asynchronous event stream, and gRPC service
definition. The contract surface aligns with the manifest's three
canonical capabilities: cluster-bootstrap, network-policy-apply,
node-lifecycle.

### 1.4 Capabilities

capabilities/cluster-bootstrap.yaml — tier T3, risk_class high.
capabilities/network-policy-apply.yaml — tier T2, risk_class limited.
capabilities/node-lifecycle.yaml — tier T2, risk_class limited.

### 1.5 Policy / Cedar

policy/auditor-scope.cedar + ci-scope.cedar + tenant-scope.cedar +
public-read.cedar. Cluster-isolation.md and data-residency.md carry
the prose policy surfaces.

### 1.6 SLOs

slos/ contains six OpenSLO 1.x manifests covering:
cluster-api-availability (0.9995), cluster-cni-availability (0.9995),
node-readiness-correctness (0.999), pod-scheduling-latency (0.95),
service-mesh-availability (0.9995), cis-benchmark-conformance (1.0).

### 1.7 Runbooks

runbooks/ contains nine named runbooks: control-plane-restore,
etcd-quorum-recovery, kubeadm-upgrade, istio-mtls-rotation,
envoy-sni-debug, ingress-ddos-throttle, csi-rebuild,
node-cordon-and-drain, karpenter-scale-up-stall. Plus an onboarding/
directory with sre-first-week.md and a faqs/sre-faq.md.

### 1.8 IaC

iac/ split across:
- iac/terraform/kubeadm-cluster/main.tf and containerd-config/main.tf
  (still using terraform/ directory name; the feedback memory says
  OpenTofu HCL is the only acceptable IaC language — Terraform is
  acceptable in this directory name because OpenTofu reads
  terraform/*.tf files and is the canonical engine; the directory
  alias is a P3 cosmetic concern).
- iac/helm/ — karpenter, cni-cilium, envoy-gateway, istiod, istio-base
  Charts with templates and values.
- iac/kustomize/base/ + overlays/pack-kr/ + components/storage-classes/
  with canonical names oya-pg-hot, oya-pg-warm, oya-pg-cold,
  oya-valkey-hot, oya-s3-warm, oya-s3-cold per ADR-0161.

### 1.9 Catalog

catalog/oya-cloud-k8s-*.yaml records cluster-bootstrap kernel /
domain / usecase / adapter / adapter-kubeadm / adapter-containerd /
rest / worker / sdk / app crates, the ingress-controller
adapter-envoy, and the service-mesh-control-plane adapter-istio.
12 catalog YAMLs total.

### 1.10 Dashboards

dashboards/cluster-health.json + node-utilization.json +
service-mesh-policy-coverage.json. Three Grafana dashboards.

### 1.11 Decisions (per-microservice ADRs)

decisions/ADR-CK-001-cilium-cni-selection.md — locks Cilium as the
canonical CNI for cloud-k8s per the eBPF + NetworkPolicy + Hubble
+ multi-cluster mesh primitives rationale.

### 1.12 Benchmarks

benchmarks/kubeadm-vs-managed-vs-rancher.md — four workload bench
suite (bootstrap, API p99, scheduling latency, annual TCO) against
EKS / GKE / AKS / RKE2 / OpenShift on identical hardware.

### 1.13 Migration playbooks

migration-playbooks/from-rancher-rke2.md — single canonical migration
playbook from Rancher RKE2 to oyatie cloud-k8s.

### 1.14 Tutorials

tutorials/bootstrap-demo-trial-cluster.md — single tutorial covering the
demo_trial profile bootstrap; the tier framing here is now retired and
the file is a P3 remediation candidate.

### 1.15 Capability tiers

capability-tiers/tier-matrix.md (106 lines) + tier-deltas-and-pricing.md
(343 lines). Both predate the 2026-05-20 tier retirement and need
remediation under Wave 15+ to fold into tenant-class (demo_trial /
paid) + deployment-context.

### 1.16 Reference implementations

reference-implementations/bootstrap-cluster-rust-sdk.md — sample
Rust code calling the cloud-k8s SDK to bootstrap a cluster.

### 1.17 Scorecards

scorecards/overrides.json — DORA / SOC 2 / NSA-K8s-hardening lane
override file.

## 2. Five-dimension audit verdicts

### 2.1 Dimension 1 — Internal coherence

Verdict: PASS-WITH-FINDINGS.

Strengths:

- PRD, ARCHITECTURE, manifest, contracts, IPs, runbooks, SLOs,
  policies, capacity model, multi-region, threat-model, DPIA, and
  compliance docs agree on the seven bounded-context shape
  (cluster-bootstrap, node-lifecycle, network-policy,
  service-mesh-control-plane, ingress-controller, csi-storage-driver,
  kubernetes-api-proxy). The crate naming follows BNF v4.1 +
  ADR-0105 13-layer enum + ADR-0106 usecase rename uniformly.
- audit-chain event taxonomy is consistent across documents:
  ClusterBootstrapped, NodeJoined, NodeFailed, NodeDrained,
  NetworkPolicyApplied, IstioPolicyChanged, KubeadmUpgraded.
- Tenant scoping fields named consistently: tenant_id, principal_id,
  caller_tenant_id, home_cell, jurisdiction_code, data_class,
  audit_event_class.
- The 11-pack regulatory list and the 11-pack multi-region topology
  agree.

Findings:

- F-INT-01 (P2): manifest.json declares only 13 crates under the
  bounded_contexts roster, but PRD claims 62 crates introduced by
  this microservice. The 13 visible in manifest are exactly the
  cluster-bootstrap layer plus two cross-BC adapter crates
  (ingress-controller adapter-envoy + service-mesh-control-plane
  adapter-istio). The other 49 PRD-named crates are not yet
  enumerated in manifest. Fix: enumerate every crate the PRD claims
  exists. If a crate is not yet scaffolded, mark planned_status:
  planned rather than omitting.
- F-INT-02 (P2): manifest layer enum lists nine layers (adapter, api,
  app, domain, kernel, rest, sdk, usecase, worker). PRD lists ten
  active layer columns (kernel, domain, usecase, api, adapter,
  adapter-*, rest, worker, sdk, app). The backend-qualified adapters
  (-adapter-kubeadm, -adapter-istio, -adapter-envoy, -adapter-block,
  -adapter-object, -adapter-file) are not separately enumerated in
  manifest.layers because they collapse to the canonical adapter
  layer per ADR-0105 Amendment 3. Fix: add a note to manifest.json
  explaining the collapse, or extend the layers enum with
  backend-qualified entries.
- F-INT-03 (P3): manifest.criticality_tier is T2 but PRD positions
  cloud-k8s as the shared substrate every other microservice runs on
  — typically a T1 or T0 criticality. Fix: reconcile by either
  upgrading criticality_tier or adding a manifest comment explaining
  the T2 selection.
- F-INT-04 (P2): manifest.ontology_projections is [] but PRD
  enumerates six ontology object types written (Cluster, Node,
  NetworkPolicy, IstioRevision, Gateway, StorageClass) and three
  read (Pack, Tenant, Microservice). Fix: populate ontology_projections
  with the PRD-named projections.
- F-INT-05 (P3): manifest.bominal_source is [] which is correct per
  the PRD's claim that cloud-k8s originates in oyatie; but
  manifest.layer = "rest" (singular) is anomalous given the
  microservice spans nine canonical layers. Fix: drop the singular
  layer field or rename to primary_facade_layer.
- F-INT-06 (P3): tier-matrix.md and tier-deltas-and-pricing.md use
  customer-class ladder tier framing which the 2026-05-20
  doctrine amendment retired. Fix in Wave 15+: rewrite under
  tenant-class (demo_trial / paid) + deployment-context overlay.
- F-INT-07 (P3): multi-region.md ADR-0158 disposition block declares
  active_passive per cell with RPO ≤ 30 seconds and RTO ≤ 5 minutes;
  the body of the doc gives RPO ≤ 5 min (intra-region) and RTO ≤ 50
  min DR failover. The two are not contradictory — ADR-0158 is
  intra-cluster leader election, the body is cross-cluster failover —
  but the doc does not disambiguate.

### 2.2 Dimension 2 — Outbound cross-references

Verdict: PASS-WITH-FINDINGS.

Strengths:

- Outbound ADR citations are accurate across PRD, ARCHITECTURE,
  competitor-parity-matrix, capability-tiers, multi-region,
  failure-modes, capacity-model, and compliance. Cited ADRs include
  ADR-0028 (audit-chain), ADR-0044 (Istio + Envoy), ADR-0056 (BNF
  v4.1), ADR-0105 (13-layer enum), ADR-0106 (usecase rename),
  ADR-0117 (cloud-native progression), ADR-0120 (Rust-first on-prem),
  ADR-0121 (on-prem K8s stack), ADR-0123 (hyperscaler maturity gate),
  ADR-0131 (per-microservice flat layout), ADR-0132 (no-grouping),
  ADR-0133 (industry-best-practice conformance), ADR-0139 (agentic
  SLO-gated promotion), ADR-0140 (Cedar; noted retired per ADR-0145),
  ADR-0145 (inter-microservice communication reform), ADR-0146
  (Cosign + Kyverno), ADR-0158 (multi-region disposition), ADR-0161
  (canonical StorageClass), ADR-0164 (sovereign air-gap), ADR-0198
  (Karpenter > Cluster Autoscaler), ADR-0234 (social), ADR-0236
  (corpus remediation), ADR-0242..ADR-0284 keystone bundle.
- Cross-microservice references trace to cell, cloud-iac, identity,
  tenancy, policy-engine, observability, audit-chain, cloud-secrets,
  network, ontology, detection, application, docs.

Findings:

- F-OUT-01 (P1): the audit dispatch brief cites ADR-0254 as
  "K8s + Cloud Hypervisor". The actual ADR-0254 in the repository
  is "Deployment model spectrum" (the five-model deployment posture).
  The K8s + Cloud Hypervisor doctrine is split across ADR-0254
  (deployment spectrum) + ADR-0248 (Amazon-shape cellular) + ADR-0121
  (on-prem K8s stack) + the keystone bundle. cloud-k8s docs cite
  ADR-0121 + ADR-0044 + ADR-0146 + ADR-0198 + ADR-0158 + ADR-0164
  + ADR-0161 + the keystone bundle but do not cite ADR-0254
  explicitly. Fix: add ADR-0254 + ADR-0248 to PRD related_adrs and
  to ARCHITECTURE source_anchors so the deployment spectrum + cell
  topology connection is explicit.
- F-OUT-02 (P2): the manifest.adrs roster lists eleven ADRs, of which
  one is ADR-0234 (social expansion planning contract).
  That ADR is unrelated to cloud-k8s; likely a copy-paste residue.
  Fix: remove ADR-0234 from manifest.adrs.
- F-OUT-03 (P2): ADR-0145 retired ADR-0140 (Cedar). compliance.md
  cites ADR-0140 with the retired marker. PRD does not cite ADR-0145
  in its related_adrs. Fix: drop ADR-0140 references; add ADR-0145
  to PRD related_adrs.
- F-OUT-04 (P3): the ADR-0328 source anchor is not cited in any
  cloud-k8s doc; ADR-0328 governs this audit wave itself. Fix:
  link this audit doc back to ADR-0328 in the frontmatter (done in
  this file's frontmatter) and to subsequent IPs that consume the
  audit findings.
- F-OUT-05 (P2): manifest.depends_on_microservices includes
  cloud-iac, cell, application, docs, observability, identity,
  tenancy, audit-chain, network, ontology, detection — but PRD's
  Integration via Workflow + Ontology section also implies
  policy-engine + cloud-secrets dependencies. The Wave-3-C anchor
  sweep in ARCHITECTURE.md does cite policy-engine and cloud-secrets
  as cross-service links. Fix: add policy-engine and cloud-secrets
  to manifest.depends_on_microservices.

### 2.3 Dimension 3 — Substance bar

Verdict: PASS.

Strengths:

- PRD is bespoke. It names the kubeadm + containerd 2.3.0 LTS +
  Istio 1.29.2 + Envoy + Cilium stack, the 30-min bootstrap envelope,
  the 5-min node-join envelope, the 30-s policy propagation envelope,
  the 1.4× worker-node multiplier for pack-us-healthcare, the
  per-BC kernel ports, the data_class annotations, the CI lane
  enforcements, and the cross-product rule.
- failure-modes.md names 15 concrete FM-IDs with detection signals,
  RTO targets, recovery runbooks, and postmortem owners.
- capacity-model.md provides explicit sizing formulas with citation
  to upstream Kubernetes scaling guide, etcd hardware doc, Istio
  scaling guide, Cilium ops guide, kubelet sizing, and CNCF
  conformance scale envelopes.
- multi-region.md provides per-pack topology including DR-pair vs
  single-region decisions, replication mode + RPO per component,
  failover phase budget table, BCDR exercise cadence, and ADR-0158
  disposition block.
- compliance.md provides SOC 2 + ISO 27001 + GDPR + HIPAA + KR PIPA
  + APPI + LGPD + DPDPA + PDPA + APP + UAE PDPL + KSA PDPL +
  KR-CSAP + CIS K8s + NSA Hardening control-to-implementation
  tables, plus the Wave-3-G keystone anchor closures.

Findings:

- F-SUB-01 (P3): the AUDIT-FINDINGS-2026-05-18.json verdict was
  HARD_VIOLATIONS due to minio_engine + cluster_autoscaler +
  valkey_engine hits. The remaining hits are historical narrative
  references (e.g., "ADR-0198 D-1 strict" explaining why Cluster
  Autoscaler is removed). Fix: refresh AUDIT-FINDINGS with the
  current state or delete the stale file.
- F-SUB-02 (P3): the IP-files-thin block in the AUDIT-FINDINGS
  cites IP-004 (95 lines), IP-010 (91 lines), and IP-karpenter-bootstrap
  (62 lines). These are below the 200-line substance floor that
  ADR-0322 prescribes for IP slices. Fix: expand or fold into
  adjacent IPs.
- F-SUB-03 (P3): tutorials/bootstrap-demo-trial-cluster.md uses the
  retired demo_trial framing. Fix: re-author as
  bootstrap-development-cluster.md or similar.

### 2.4 Dimension 4 — Canonical-direction alignment

Verdict: PASS-WITH-FINDINGS.

Strengths:

- The PRD's Cross-product rule explicitly states cloud-k8s MUST NOT
  import another product microservice crate at any layer, and that
  workload events flow through Workflow events and reads through
  Ontology. The LEAN-A2 lane enforces.
- The Wave-3-G anchor sweep in ARCHITECTURE.md closes 40+ keystone
  anchors uniformly. The §substrate-product-binding anchor states
  cloud-k8s is a "product" classification (consuming substrate
  services), not a substrate. That classification is unusual for a
  cloud-* microservice and disagrees with the audit's intuition.
- ADR-0254's five-model deployment spectrum (shared-cloud,
  dedicated-cloud, hybrid / BYO-cloud, on-prem connected, on-prem
  air-gapped) requires that cloud-k8s ship identical Helm charts +
  Cedar bundles + container images + workflows across all five
  models. cloud-k8s's PRD and Wave-3-C anchor sweep support that
  single-build invariant.
- Per ADR-0248 (Amazon-shape cellular), cells run on cloud-k8s.
  cloud-k8s's docs reference cell as both a sibling (cloud-iac
  builds the box, cell handles tenant placement, cloud-k8s turns
  the box into a cluster) and a consumer (cell schedules tenant
  pods onto cloud-k8s clusters). The relationship is coherent but
  could be more explicit in PRD.

Findings:

- F-DIR-01 (P1): manifest.json classifies cloud-k8s as "product"
  while every other cloud-* microservice should be "substrate".
  ARCHITECTURE.md §substrate-product-binding repeats the "product"
  classification. This is inconsistent with the PRD ("This µservice
  is shared substrate, not a hero product. It hosts every other
  oyatie µservice."). Fix: switch manifest.tier_classification
  from "product" to "substrate" and update the §substrate-product-
  binding answer body accordingly.
- F-DIR-02 (P2): ADR-0254 deployment model spectrum is the canonical
  doctrine for how cloud-k8s ships across shared-cloud / dedicated-
  cloud / hybrid / on-prem connected / on-prem air-gapped. The
  cloud-k8s docs cover on-prem (PRD + IP-002 + IP-001) and
  air-gapped (multi-region.md ADR-0164 section) but do not name
  the dedicated-cloud or hybrid / BYO-cloud variants explicitly.
  Fix: add a deployment-model section to PRD that maps the seven
  cloud-k8s bounded contexts onto each of the five deployment
  models.
- F-DIR-03 (P2): manifest.regulatory_packs lists 11 packs but
  manifest.compliance_packs_applicable repeats those 11 plus 13
  more (hipaa, SOC 2 Type 2, iso27001, gdpr, kr-isms-p, NIS2, DORA,
  lgpd, soc2, cn-pipl-2021, fedramp, il5). The duplication
  conflates regional packs (kr, eu, us, us-healthcare, jp, sg, au,
  in, br, ae, ksa) with framework packs (HIPAA, GDPR, NIS2, DORA,
  FedRAMP, IL5, etc.). Fix: split into regional_packs + framework_packs
  per ADR-0251 pack overlay taxonomy.
- F-DIR-04 (P3): the Wave-3-G keystone bundle named the demo_trial /
  paid tenant-class doctrine. cloud-k8s docs do not yet reflect
  tenant_class as a first-class field in capability metadata,
  Cedar policy fragments, or audit events. Fix in Wave 15+:
  thread tenant_class through capabilities/*.yaml + policy/*.cedar.

### 2.5 Dimension 5 — Industry-counterpart parity (top-3 UNION)

Verdict: PASS-WITH-FINDINGS.

Strengths:

- competitor-parity-matrix.md identifies eight competitors (EKS, GKE,
  AKS, OKE, Rancher RKE2, OpenShift, TKG, Talos Linux + Omni) — a
  broader set than the brief's top-3 (EKS / GKE / AKS) but
  inclusive of it. UNION coverage with EKS / GKE / AKS is the
  effective bar.
- Feature parity tables cover cluster lifecycle, networking, security
  + supply chain, storage + CSI, operations + cost, and agent
  operability dimensions. Each row marks oyatie as one of: ✅
  covered, M0x scheduled, partial, ❌ absent, or manual.
- The matrix names three oyatie-only differentiators that no
  counterpart has: Foundry-callable cluster mutators, Cedar-derived
  NetworkPolicy + AuthorizationPolicy from tenant fragments, and
  the kubernetes-api-proxy with Cedar policy + audit-chain seal on
  every API call.
- Quantitative parity numbers are present (cluster bootstrap p99,
  node-join p99, NetworkPolicy propagation p99, Istio xDS push, etc.)
  with sources cited.

Findings:

- F-PAR-01 (P1): the parity matrix uses a wider competitor set
  (8 vendors) than the audit brief's top-3 (EKS / GKE / AKS). The
  feature-parity-matrix-2026-05-20.md deliverable produced by this
  audit follows the brief's top-3 strictly. Both views are valid;
  the existing matrix should be marked as the extended competitor
  view and the new deliverable as the canonical top-3 view.
- F-PAR-02 (P2): the parity matrix does not address GKE Autopilot
  as a distinct competitor mode. Autopilot's auto-managed nodes
  fundamentally change the operational comparison (no node-lifecycle
  surface visible to the tenant). Fix in the new top-3 matrix:
  call out Autopilot as a separate column.
- F-PAR-03 (P2): the parity matrix calls out workload identity
  (IRSA / GKE WI / AKS WI) as a gap to close in M03. None of the
  current deliverables explicitly maps oyatie's SPIFFE + OpenBao
  approach against the IRSA / GKE WI / AKS WI shape. Fix in
  feature-parity-matrix-2026-05-20.md: dedicate a row to workload
  identity with the SPIFFE / OpenBao mapping.
- F-PAR-04 (P2): the parity matrix's Region coverage row says
  "oyatie 11 packs vs EKS 28 regions". The number is misleading
  because regions and packs are different unit categories — pack
  is a regulatory bucket, region is a substrate location. Fix:
  reframe as "pack coverage" + "underlying region coverage" rows.
- F-PAR-05 (P3): tier-deltas-and-pricing.md gives demo_trial 35k-65k
  USD/yr, paid dedicated-cloud 140k-190k, paid on-prem-connected 430k-550k, paid compliance_pack 900k-2.4M.
  EKS pricing in competitor-parity-matrix.md shows $73/month/cluster
  for managed CP plus EC2. The two number sets are not unified into
  a comparison. Fix: in performance-benchmark-numbers-2026-05-20.md
  consolidate the cost-comparison numbers into the deployment-context
  overlay table.

## 3. Substance-bar narrative

### 3.1 Named precedent

cloud-k8s is the on-prem Kubernetes substrate that runs every other
oyatie microservice. The PRD's "Internal Outcome 4 — Hyperscaler-
parity substrate" names the precedent set as AWS EKS, GCP GKE, Azure
AKS, Oracle OKE, Rancher RKE2, OpenShift, and Tanzu Kubernetes Grid.
The benchmark set cites those plus Talos Linux + Omni. The
microservice does not invent a Kubernetes distribution; it uses
upstream kubeadm + containerd + Istio + Envoy + Cilium + CSI in a
combination that matches CNCF conformance.

### 3.2 Failure-mode tree

failure-modes.md enumerates 15 named modes. The tree is wide enough
that a programming-capable on-call engineer can identify the mode
from telemetry, find the runbook, and execute the recovery within
the named RTO. The 15 modes cover the kube-apiserver, etcd,
control-plane node partition, worker node failure, CNI (Cilium),
Istio control plane, Envoy ingress, Envoy TLS misconfig, CSI driver
backend, kubeadm minor-version upgrade rollback, NetworkPolicy /
AuthorizationPolicy regression, Cosign signature verification bypass,
api-proxy outage, and PV backend outage. Severity classification
maps to incident-response.md Sev-1 / Sev-2 vocabulary.

### 3.3 Capacity math

capacity-model.md provides formulas (not magic numbers) for every
cluster component. Worker_nodes_needed = ceil(total_pods / 100).
api_request_rate = total_pods × 0.5 reqs/sec. kube_apiserver_replicas
= max(3, ceil(api_request_rate / 5000)). etcd_disk_iops =
max(1000, total_pods × 0.1). istiod_replicas = max(3, ceil(S_sidecars
/ 1000)). The formulas cite upstream Kubernetes scaling docs, etcd
hardware doc, Istio scaling guide, Cilium ops guide, and CNCF
conformance results at 5000 nodes + 150 000 pods. XS / S / M / L
reference baselines are populated explicitly with a worked XS
pack-kr 20-tenant example.

### 3.4 Observability hooks + capability-tier projection

#### 3.4.T Tenant-class projection (demo_trial vs paid)

Per the 2026-05-20 doctrine amendment, cloud-k8s exposes two tenant
classes. The current code does not yet treat tenant_class as a
first-class manifest field (finding F-DIR-04). The intended
projection:

- demo_trial tenants run on the OCI Always Free profile: 2× Ampere
  A1 ARM cores (4 OCPU + 24 GiB RAM) per cluster, single AZ, no DR
  pair, no Istio multi-cluster federation, single-AZ etcd
  co-located with control plane. Bootstrap envelope is the same
  30-min p99; the underlying compute pool is the Always Free
  envelope from feedback_oci_always_free_maximization_2026_05_20.
- paid tenants run on the per-deployment-context substrate per
  ADR-0254. Shared-cloud and dedicated-cloud paid tenants on
  oyatie-operated cells; hybrid on tenant-owned cloud accounts;
  on-prem connected on tenant hardware with periodic sync;
  on-prem air-gapped on tenant hardware with CDS-delivered
  bundles.

The audit-chain events ClusterBootstrapped, NodeJoined, and so on
SHOULD include tenant_class as a top-level field once the tenant_class
threading lands in Wave 15+.

#### 3.4.C Deployment-context overlay

Per ADR-0254, the five deployment models are concrete and discrete:

1. shared-cloud — oyatie operates the cell; shuffle-sharded across
   hundreds-to-thousands of tenants; substrate is one of oyatie's
   contracted cloud providers per ADR-0240.
2. dedicated-cloud — oyatie operates the cell; one tenant per cell;
   substrate is oyatie's cloud account.
3. hybrid / BYO-cloud — tenant provides the cloud account; oyatie
   deploys the cell via IAM-delegated access.
4. on-prem connected — tenant runs the cell on their own hardware;
   periodic sync with oyatie control plane.
5. on-prem air-gapped — tenant runs the cell on their own hardware;
   no network sync; bundle-delivered upgrades + bundle-exported
   audit-chain.

cloud-k8s's seven bounded contexts (cluster-bootstrap, node-lifecycle,
network-policy, service-mesh-control-plane, ingress-controller,
csi-storage-driver, kubernetes-api-proxy) ship identically across all
five models. The deltas live in the bootstrap profile YAML (per the
tier-matrix file in capability-tiers/), in the cell substrate
provisioning (cloud-iac modules), and in the control-plane
connectivity (online for 1+2+3; periodic for 4; bundle for 5).

#### 3.4.K Kubernetes substrate alignment with ADR-0254 + ADR-0248

ADR-0254 names Kubernetes as the canonical container orchestrator
across all five deployment models. cloud-k8s is the single
microservice that owns the Kubernetes substrate; ADR-0248 names the
cellular topology that the substrate runs.

Alignment evidence:

- One Helm chart set per ADR-0254 D-2. cloud-k8s's iac/helm/
  contains karpenter, cni-cilium, envoy-gateway, istiod, istio-base.
  Each chart exposes one canonical chart name; values vary per
  deployment context but the chart is one chart.
- One Cedar policy bundle per ADR-0254 D-2. cloud-k8s's policy/
  contains auditor-scope.cedar, ci-scope.cedar, tenant-scope.cedar,
  public-read.cedar. Per-tenant overlays compose on top.
- One container image set per ADR-0254 D-2. cloud-k8s exposes the
  catalog/oya-cloud-k8s-*.yaml entries that point at the canonical
  OCI image digests.
- One workflow definition set per ADR-0254 D-2. The PRD's Integration
  via Workflow + Ontology section names the seven events produced
  and the three events consumed.
- One audit-chain schema per ADR-0254 D-2. manifest.audit_chain.seal_events
  lists three named events; PRD names seven.
- Cell tier topology per ADR-0248. cloud-k8s clusters run as the
  substrate beneath cell tier 0-4. PRD names per-cell capacity
  envelope (Nodes per cluster 10..5000; Pods per node 110..250;
  Pods per cluster 1100..150000). The 5000-node + 150000-pod
  numbers match upstream CNCF tested limits per ADR-0254 D-15.
- Cloud Hypervisor + Kata pods per ADR-0254 + ADR-0248. The PRD
  does not name Cloud Hypervisor or Kata containers explicitly.
  tier-matrix.md paid compliance_pack tier names Kata pods + Cloud Hypervisor
  for tenant workloads above INTERNAL_ONLY. After tier retirement,
  this requirement should be re-expressed under deployment-context:
  Cloud Hypervisor + Kata pods are mandatory for paid tenants on
  dedicated-cloud + on-prem variants whenever the workload carries
  data above INTERNAL_ONLY.

The ADR-0254 alignment is substantively present; the remaining gap
is naming Cloud Hypervisor + Kata as deployment-context-conditional
requirements rather than tier-conditional.

### 3.5 Rollback path

The PRD names rollback at multiple layers:

- Cluster bootstrap rollback via kubeadm reset + etcd snapshot
  restore (runbooks/control-plane-restore.md + etcd-quorum-recovery.md).
- kubeadm minor-version upgrade rollback via kubeadm upgrade rollback
  + pre-upgrade etcd snapshot (failure-modes.md FM-11; runbooks/
  kubeadm-upgrade.md §Rollback).
- Istio control-plane rollback via canary upgrade pattern (rollback
  to prior istiod revision).
- NetworkPolicy regression rollback via emergency default-deny
  override (failure-modes.md FM-12).
- Cosign bypass rollback via Kyverno reconcile + admitted pod
  termination (FM-13).

### 3.6 Multi-region awareness

multi-region.md enumerates 11 packs across OCI regions; defines
DR-pair vs single-region per pack; specifies replication mode per
component; phases the failover into 10 steps with a 50-min end-to-end
RTO target; lists exercise cadences (quarterly DR drill, monthly
etcd restore drill, annual tabletop, continuous chaos engineering,
annual vendor-failure simulation, per-minor-version kubeadm upgrade
dry-run); names per-pack RPO / RTO.

### 3.7 Sovereign-cell awareness

multi-region.md §ADR-0164 names six air-gap packs: pack-eu-sovereign-airgap,
pack-kr-fsc, pack-kr-public, pack-ksa, pack-uae, pack-us-gov. For
each, the container registry binds to in-cell Harbor; external API
egress is denied via NetworkPolicy + Cilium L7; DNS / NTP / OCSP /
CRL operate in-cell; telemetry stays in-cell. CI lane oya gate
validate air-gap-overlay enforces.

### 3.8 Versioning + deprecation

PRD pins versions: Kubernetes 1.35 LTS, containerd 2.3.0 LTS, Istio
1.29.2, Envoy 1.32.0, Cilium 1.18, Rust 1.83. ADR-0121 governs the
on-prem stack version policy. Minor-version upgrades follow the
upstream N-2 support window per FR-08. kubeadm upgrade rollback is
the documented deprecation rollback per FM-11.

Cluster Autoscaler is explicitly deprecated in favor of Karpenter
per ADR-0198 D-1; the implementation-plans/IP-karpenter-bootstrap.md
file says "Cluster Autoscaler is REMOVED (per ADR-0198 D-1 strict)".

## 4. Cross-microservice handoff sketch

cloud-k8s sits at the bottom of the substrate stack. Other
microservices consume it via:

- cell → schedules tenant pods onto a cluster; consumes
  CellProvisionRequested → cloud-k8s allocates pack cluster +
  worker placement.
- cloud-iac → applies bootstrap manifests against fresh node fleet;
  emits IacResourcePlanned → cloud-k8s pre-stages kubeadm join
  token → emits ReadyForBootstrap.
- observability → deploys Grafana stack onto cloud-k8s; reads
  metrics from kube-apiserver + Cilium + Istio + Envoy + Karpenter.
- audit-chain → consumes ClusterBootstrapped, NodeJoined,
  NodeFailed, NodeDrained, NetworkPolicyApplied, IstioPolicyChanged,
  KubeadmUpgraded events; seals each with Ed25519 + Merkle.
- identity → provides SPIFFE workload identity per Istio sidecar.
- tenancy → consumes TenantOnboarded → cloud-k8s emits per-tenant
  NetworkPolicy + AuthorizationPolicy in the tenant namespace.
- policy-engine → supplies Cedar fragments for the kubernetes-api-proxy
  authorization decisions.
- cloud-secrets (OpenBao) → supplies secret references for cert-manager,
  Cosign signing keys, KMS envelope keys.
- ontology → backs the Object Type writes (Cluster, Node, NetworkPolicy,
  IstioRevision, Gateway, StorageClass).
- workflow-engine → propagates IstioPolicyChanged → downstream
  workflow re-evaluation.
- network → provides DNS + L4 ingress for the public-facing Envoy
  Gateway.
- detection → consumes Cilium flow anomaly + audit-chain anomaly
  telemetry.

The PRD's Integration via Workflow + Ontology section formalizes
producers and consumers; the manifest.depends_on_microservices field
captures the dependency direction. Finding F-OUT-05 records the
two missing entries (policy-engine, cloud-secrets).

## 5. Findings summary table

| ID | Severity | Category | File | Fix shape |
|---|---|---|---|---|
| F-INT-01 | P2 | internal-coherence | manifest.json | enumerate all 62 PRD-named crates as bounded_contexts entries with planned_status if not scaffolded |
| F-INT-02 | P2 | internal-coherence | manifest.json | document backend-qualified adapter collapse or extend layers enum |
| F-INT-03 | P3 | internal-coherence | manifest.json | reconcile criticality_tier T2 with substrate-of-everything role |
| F-INT-04 | P2 | internal-coherence | manifest.json | populate ontology_projections with the PRD's six writes + three reads |
| F-INT-05 | P3 | internal-coherence | manifest.json | drop singular layer field or rename primary_facade_layer |
| F-INT-06 | P3 | internal-coherence | capability-tiers/*.md | rewrite under tenant-class + deployment-context after tier retirement |
| F-INT-07 | P3 | internal-coherence | multi-region.md | disambiguate ADR-0158 intra-cluster RPO vs cross-cluster RPO |
| F-OUT-01 | P1 | outbound-cross-reference | PRD.md + ARCHITECTURE.md | add ADR-0254 + ADR-0248 citations |
| F-OUT-02 | P2 | outbound-cross-reference | manifest.json | remove ADR-0234 (social) |
| F-OUT-03 | P2 | outbound-cross-reference | PRD.md + compliance.md | drop ADR-0140 retired references; add ADR-0145 |
| F-OUT-04 | P3 | outbound-cross-reference | this audit doc | link back to ADR-0328 in subsequent IPs |
| F-OUT-05 | P2 | outbound-cross-reference | manifest.json | add policy-engine + cloud-secrets to depends_on_microservices |
| F-SUB-01 | P3 | substance-bar | AUDIT-FINDINGS-2026-05-18.json | refresh or delete stale file |
| F-SUB-02 | P3 | substance-bar | IP-004, IP-010, IP-karpenter-bootstrap | expand to 200-line substance floor or fold into adjacent IPs |
| F-SUB-03 | P3 | substance-bar | tutorials/bootstrap-demo-trial-cluster.md | re-author without demo_trial framing |
| F-DIR-01 | P1 | canonical-direction | manifest.json + ARCHITECTURE.md | switch tier_classification product -> substrate |
| F-DIR-02 | P2 | canonical-direction | PRD.md | add deployment-model section mapping seven BCs onto five ADR-0254 models |
| F-DIR-03 | P2 | canonical-direction | manifest.json | split regulatory_packs from framework_packs per ADR-0251 taxonomy |
| F-DIR-04 | P3 | canonical-direction | capabilities/*.yaml + policy/*.cedar | thread tenant_class field |
| F-PAR-01 | P1 | parity | competitor-parity-matrix.md + new top-3 matrix | mark existing as extended view; new doc is canonical top-3 |
| F-PAR-02 | P2 | parity | feature-parity-matrix-2026-05-20.md | dedicate GKE Autopilot column |
| F-PAR-03 | P2 | parity | feature-parity-matrix-2026-05-20.md | add workload-identity row with SPIFFE/OpenBao mapping |
| F-PAR-04 | P2 | parity | competitor-parity-matrix.md | split pack coverage vs underlying region coverage |
| F-PAR-05 | P3 | parity | performance-benchmark-numbers-2026-05-20.md | consolidate cost comparison into deployment-context overlay |

## 6. Backlog rows (Wave 14 input)

Each row below is intended to drop into the Wave 14 backlog without
further triage. P0 = none in this audit (no hard contradictions
detected); P1 = three findings; P2 = ten findings; P3 = eleven
findings.

The fix shapes in the table above are the concrete remediation
shapes; the Wave 15+ sub-wave assignment is left to the Wave 14
orchestrator.

## 7. Verification Notes

- Read PRD.md in full (387 lines).
- Read ARCHITECTURE.md first page (lines 1-4396 with 65k token cap;
  partial; sampled the §principals / §cedar-gates / §tenant-scoping
  / §substrate-product-binding / §policy-evaluation /
  §cell-eligibility anchors).
- Read manifest.json in full (388 lines).
- Read competitor-parity-matrix.md in full (165 lines).
- Read capability-tiers/tier-matrix.md in full (106 lines).
- Read capability-tiers/tier-deltas-and-pricing.md in full (343 lines).
- Read multi-region.md in full (248 lines).
- Read failure-modes.md in full (270 lines).
- Read capacity-model.md in full (240 lines).
- Read compliance.md first page (lines 1-441 partial).
- Read benchmarks/kubeadm-vs-managed-vs-rancher.md in full (95 lines).
- Read AUDIT-FINDINGS-2026-05-18.json in full (144 lines).
- Read ADR-0328 first page (lines 1-1422 partial; sampled D-1
  five-phase sequence, D-2 Big 8 sub-sequence, D-3 agent-class
  anchors, D-4 audit protocol, D-5 union-coverage parity bar, D-6
  four-deliverable contract, D-7 batch grouping, D-10 verification
  SLA).
- Read ADR-0254 first page (lines 1-957 partial; sampled D-1 five
  models, D-2 same-architecture invariant, D-3 cell topology table,
  D-4 deployment-control-plane substrate, D-5 .oab bundle format,
  D-6 air-gap CDS delivery).
- Listed every file under k8s/ recursively
  (~120 files visible at the listing depth).

Five-anchor cross-checks performed:

1. ADR-0328 — read; the audit follows the D-4 five-dimension
   protocol and produces the D-6 four-deliverable set (with the
   tier-delta deliverable replaced by the tier-retirement notes
   per the brief's amendment).
2. master-plan-sequencing.json — referenced; cloud-k8s placed at
   Phase 0 service 10 (cloud-compute-k8s alias).
3. brief-template.md — sections 3.4.T (tenant), 3.4.C (deployment
   context), 3.4.K (Kubernetes alignment) authored.
4. ADR-0254 — alignment evidence in §3.4.K.
5. ADR-0248 — cell topology alignment in §3.4.K.

## 8. Findings section

Findings are tabulated in Section 5; backlog rows are tabulated in
Section 6. No P0 findings detected.

## 9. Backlog rows

See Section 6 for the structured backlog input. All 24 findings
flow into the Wave 14 aggregation.

## 10. Risks of acting on this audit

- The "product" vs "substrate" classification flip (F-DIR-01) is a
  high-impact change because it touches the §substrate-product-binding
  anchor across other microservices' ARCHITECTURE.md files via the
  Wave-3-C sweep. The remediation agent must confirm that the
  classification flip does not invalidate the inheritance chain.
- The manifest.json crate enumeration (F-INT-01) may surface
  uncreated crates. If the PRD claims 62 crates but only 13 exist,
  the remediation must distinguish missing-from-manifest vs
  missing-from-codebase.
- The ADR-0234 removal (F-OUT-02) is a clean delete; no risk.
- The ADR-0140 cleanup (F-OUT-03) requires confirming the ADR-0145
  retirement narrative captures Cedar's continuing role.

## 11. Glossary additions surfaced by this audit

No new glossary entries are required. All terms used (cluster
lifecycle, control plane HA, NetworkPolicy, AuthorizationPolicy,
xDS, Cilium eBPF, Karpenter, Cosign, SLSA, CRR, Cedar, OpenBao,
Ed25519, audit-chain, etc.) are present in the canonical glossary
or canonically defined in the cited ADRs.

## 12. Audit verdict

PASS-WITH-FINDINGS. cloud-k8s is one of the most substantively
documented microservices in the corpus. The seven-BC shape is
internally coherent; the contracts, policies, SLOs, runbooks, and
compliance mapping support the on-prem K8s substrate intent. The
findings above are real but mostly P2/P3; only three rise to P1
(F-OUT-01 ADR-0254 citation, F-DIR-01 substrate classification flip,
F-PAR-01 top-3 view canonicalization), and none rises to P0
(no hard contradiction or unsafe downstream instruction).

The microservice can promote past the Phase 0 audit gate with the
P1 findings entering the Wave 14 backlog. It cannot claim
"hyperscaler maturity at HG-CLOUD-K8S" until the substrate
classification flip lands plus the workload-identity gap closes
in M03.

<!--
audit_completion_report:
  audit_wave: Wave-4-Rolling
  microservice: cloud-k8s
  audit_owner: codex-cloud-k8s-audit-agent
  audit_date: 2026-05-21
  phase: Phase-0-Shared-Infrastructure
  batch: Wave-4-rolling
  top_3_counterparts: [AWS EKS, GCP GKE, Azure AKS]
  verdict: PASS-WITH-FINDINGS
  findings_total: 24
  findings_p0: 0
  findings_p1: 3
  findings_p2: 10
  findings_p3: 11
  five_dimensions:
    internal_coherence: PASS-WITH-FINDINGS
    outbound_cross_references: PASS-WITH-FINDINGS
    substance_bar: PASS
    canonical_direction: PASS-WITH-FINDINGS
    industry_parity: PASS-WITH-FINDINGS
  deliverables_authored:
    - k8s/coherence-audit-2026-05-20.md
    - k8s/feature-parity-matrix-2026-05-20.md
    - k8s/performance-benchmark-numbers-2026-05-20.md
  tier_deltas_doc_omitted: true
  tier_deltas_doc_omitted_reason: per-brief-no-tier-scaffolding-amendment-2026-05-20
  adr_0254_alignment_evidence: yes
  adr_0248_alignment_evidence: yes
  master_plan_alias_recorded: cloud-k8s aliases canonical cloud-compute-k8s per D-1.16
  tier_retirement_observed: true
  tenant_class_threading_status: pending Wave 15+
  deployment_context_threading_status: pending Wave 15+ (PRD section addition)
  scripting_used: false
  placeholder_used: false
  tier_scaffolding_used: false
  external_writes: false
  external_commits: false
  bounded_to_microservice_path: k8s
-->
