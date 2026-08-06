---
id: ADR-0338
title: Pod runtime tier 0..3 (Kata + Cloud Hypervisor for tenant-untrusted + tenant-data substrate; runc for first-party + edge)
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - council-security
  - ops-sre-reliability
  - ops-security
  - ops-dr-capacity
  - axis-cloud
  - axis-cell
  - axis-deployment
  - axis-policy-engine
  - axis-observability
owners:
  - council-architecture
  - council-security
  - ops-sre-reliability
  - ops-security
  - ops-dr-capacity
  - axis-cloud
  - axis-cell
  - axis-deployment
  - axis-policy-engine
  - axis-observability
supersedes: []
superseded_by: [ADR-0701]
amends:
  - ADR-0254-deployment-model-spectrum.md (the K8s + Cloud Hypervisor + Kata invariant is preserved; this ADR carves out an explicit four-tier runtime classification + admission policy + nodepool topology so Kata is not used everywhere)
related_adrs:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0121-on-prem-k8s-stack.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/platform-architecture.json
  - /specs/deployment-models.json
  - /specs/forbidden-operations.json
  - /specs/microservices/cloud-iac.json
  - /specs/microservices/cell.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_idea_refine_decisions_2026_05_21
  - feedback_amazon_shape_cellular_architecture
  - feedback_kubernetes_everywhere_pods_cloud_hypervisor
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_build_ahead_of_certification
  - feedback_compliance_pack_primitive
  - feedback_tenant_scoping_primitive
  - feedback_cedar_universal_gate
  - feedback_bominal_inheritance_precedence
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
companion_docs:
  - docs/standards/hyperscaler-best-practices.md
  - docs/standards/dependency-policy.md
  - docs/GLOSSARY.md
  - docs/machine-readable/glossary.json
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_idea_refine_decisions_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-tier-declaration-lands
enforced_by:
  - oya-check-pod-runtime-tier (new CI lane; advisory until crate lands; planned to refuse missing or invalid pod_runtime_tier declarations; refuses tier-vs-nodepool mismatches; promoted to BLOCKER after corpus-wide declaration lands)
  - Kyverno admission policy `enforce-pod-runtime-tier` (refuses pod admission when pod.spec.runtimeClassName does not match the µservice's declared pod_runtime_tier or when pod placement violates the nodepool taint/toleration contract)
  - oya-governance-runtime-class-allowlist (refuses RuntimeClass declarations outside {kata-cloud-hypervisor, runc, runc-edge})
  - oya-governance-nodepool-binding (refuses workloads that target a nodepool the µservice is not authorized to land on)
  - oya-governance-tier-promotion-evidence (refuses Tier 2 → Tier 1 promotions without evidence pack per D-10)
purpose: >
  Carve out an explicit four-tier runtime classification (Tier 0..Tier 3) over
  the Kubernetes + Cloud Hypervisor + Kata invariant from ADR-0254, mapped to
  the cellular criticality numbering convention from ADR-0248 (Tier 0 = highest
  blast-radius / most isolated). Tier 0 = tenant-customer untrusted code →
  Kata Containers + Cloud Hypervisor (Wasmtime sandbox host, workflow-studio
  user workflows, marketplace plugin executors, agent-runtime tenant
  capabilities, developer-sdk uploaded modules). Tier 1 = substrate µservices
  that touch tenant data plane → Kata + Cloud Hypervisor (cloud-iam,
  cloud-kms, cloud-secrets, audit-chain, messenger MLS keys, payments,
  intelligence transport). Tier 2 = first-party application µservices → runc
  (crm, marketing-automation, contract-lifecycle-management, itsm, community,
  social, drive, docs, sheets, slides, calendar, etc.). Tier 3 = edge / static
  / perf-critical → runc on dedicated nodepools (api-gateway data-plane,
  Envoy edge, ztunnel, CDN edge cache). Declare pod_runtime_tier as a
  manifest.json field; route placement via per-cell nodepool topology
  (kata-pool + runc-pool + runc-edge-pool); enforce via Kyverno admission +
  RuntimeClass binding + CI lane oya-check-pod-runtime-tier. Establish
  tier-promotion criteria, quarterly tier review process, default tier for
  new µservices (Tier 2), capacity model implications, and incident-response
  classification. Reduce Kata-everywhere overhead (30-40 percent pod density
  + 200-500 ms cold-start) while preserving VM-isolation where tenant-customer
  code actually executes.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Pod runtime tier Kata/CH — isolation ladder

# ADR-0338: Pod runtime tier 0..3 (Kata + Cloud Hypervisor for tenant-untrusted + tenant-data substrate; runc for first-party + edge)

## Status

Proposed on 2026-05-21.

This ADR is an amendment to ADR-0254 (deployment-model-spectrum / K8s + Cloud Hypervisor + Kata) that carves out an explicit four-tier classification for pod runtime selection. ADR-0254 established Kubernetes as the canonical orchestration substrate and Cloud Hypervisor + Kata as the canonical VM-isolation primitives for sensitive workloads. ADR-0254 did not specify which µservices ran under Kata and which ran under runc. The default reading of ADR-0254 ("Cloud Hypervisor + Kata everywhere") imposes ~30-40 percent pod-density loss and 200-500 ms cold-start latency on workloads that derive no security benefit from VM-isolation, because they are namespace-isolated + mTLS-mediated + Cedar-policed by the existing substrate invariants. This ADR resolves that ambiguity by classifying every µservice into one of four runtime tiers and binding the classification to admission policy.

The tier numbering aligns deliberately with ADR-0248 cellular criticality numbering: **Tier 0 = highest blast-radius / most isolated**, Tier 1 = substrate touching tenant data plane, Tier 2 = first-party application µservices, Tier 3 = edge / static / perf-critical. The reuse of the ADR-0248 Tier 0..Tier 4 axis is intentional — when a µservice's cellular tier is Tier 0 (highest criticality cell), the pod runtime tier is also Tier 0 or Tier 1. The two axes co-vary by design.

Enforcement transitions from `advisory-until-tier-declaration-lands` to `BLOCKER` when the per-µservice manifest schema update lands and every active µservice has declared `pod_runtime_tier`. Until then the new CI lane (`oya-check-pod-runtime-tier`) and the Kyverno admission policy (`enforce-pod-runtime-tier`) run as REPORT-ONLY. The corpus-wide manifest declaration is out of scope for this ADR; it is sequenced as a follow-on sub-wave under ADR-0328 batch discipline.

This ADR does not retire Kata Containers or Cloud Hypervisor. It restricts their use to the workloads that benefit from VM-isolation: tenant-customer untrusted code paths (Tier 0) and substrate µservices that touch tenant data plane (Tier 1). The runtime selection is per-µservice, not per-tenant — a Tier 2 µservice that serves both demo_trial and paid tenants serves both under runc.

This ADR does not change the cellular tier classification from ADR-0248. The cellular tier axis (Tier 0 = Foundation, Tier 1 = Substrate, Tier 2 = Capability, Tier 3 = Application, Tier 4 = Edge) describes blast-radius scope per cell; the pod runtime tier axis described here describes isolation primitive per pod. The two axes co-vary by construction but are independent decision surfaces.

This ADR does not change tenant_class behavior from ADR-0330. demo_trial tenants on OCI Always Free continue to run under the tier their hosting µservice declares; paid tenants run under the same tier as demo_trial tenants for the same µservice (the runtime tier is µservice-scoped, not tenant-scoped).

This ADR does not change compliance pack activation from ADR-0251. Compliance packs (HIPAA, GDPR-strict, SOC2, PCI, CSAP, EU AI Act Annex III) require Tier 0 or Tier 1 runtime placement for any pod that touches PHI / PII / sovereign data; the certification levels are preserved.

This ADR does not change Cedar evaluation from ADR-0243. Cedar gates remain authorization; this ADR sits at admission, not authorization.

## Date

2026-05-21.

## Context

### A.1 Named pressure: Kata-everywhere costs without security gain on trusted first-party code

The ADR-0254 invariant ("K8s + Cloud Hypervisor + Kata pods") was authored before the cellular tier numbering convention from ADR-0248 was finalized and before the tenant_class model from ADR-0330 collapsed the prior Bronze/Silver/Gold/Platinum tier ladder. The implicit reading of ADR-0254 was "Kata everywhere." Capacity modeling against that reading shows the following overhead:

- **Pod density.** Kata pods carry per-pod Cloud Hypervisor VM overhead (~150-256 MB of guest kernel + 100-200 MB of QEMU/Cloud Hypervisor host-side state per pod baseline). On commodity Kubernetes nodes (typically 32-128 vCPU + 128-512 GiB), pod-density falls by 30-40 percent relative to runc baseline. AWS Fargate, Google Cloud Run, and Azure Container Apps capacity models all reflect this difference; managed-Kubernetes node sizing models likewise.
- **Cold-start latency.** Kata pod start-up adds 200-500 ms of VM boot + guest kernel init + virtio-fs mount on top of containerd's runc baseline (typically 50-150 ms). For long-lived services this is amortized; for ephemeral workloads (workflow steps, batch jobs, ad-hoc agent dispatches) the difference matters and frequently shows up as p99 cold-start regression.
- **Memory bloat.** Kata pods consume an additional guest kernel + initramfs + virtio device-driver footprint per pod even at idle. On nodes packed with many idle pods, that footprint accumulates into real opportunity cost.
- **Syscall overhead.** Kata pod syscall throughput is bounded by the Cloud Hypervisor hypercall path (vmexit + virtio bridge + guest-side syscall dispatch). For latency-sensitive workloads (api-gateway data-plane, edge proxies, observability collectors) this introduces measurable p99 tail latency relative to bare-container runc execution.

The security benefit of Kata is **VM-isolation against guest-kernel escape and against shared-kernel side channels**. That benefit matters when the code running inside the pod is **untrusted by the operator** — tenant-customer code, marketplace plugin code, uploaded SDK modules, or workflow-step custom code. The benefit also matters when the pod **handles tenant data plane material** under a hostile compromise model — KMS root operations, audit-chain seal operations, MLS key derivation, identity primary-credential validation. For first-party µservice code authored under the Oyatie monorepo, signed by the Oyatie supply chain, namespace-isolated within the same cell, mTLS-mediated by the service mesh, and authorization-gated by Cedar fragments, the additional VM-isolation provides **no marginal security benefit** that justifies its 30-40 percent density cost. That is the named pressure this ADR resolves.

### A.2 Named pressure: tenant-customer code paths exist and MUST be Kata-isolated

The Oyatie corpus has explicit tenant-customer code paths that MUST run under VM-isolation. Enumerated:

- **Wasmtime sandbox host (`microservices/intelligence/.../wasmtime-host` per ADR-0255 and intelligence IP-WASMTIME-* tree).** Tenant-supplied Wasm modules executed inside the AI substrate. Even with Wasmtime's sandboxing guarantees, an additional VM-isolation layer is required because Wasm sandbox escapes are a documented attack class.
- **Workflow studio user workflows (`microservices/workflow-studio/` per ADR-0328 and workflow-studio IPs).** Customer-authored workflow definitions that execute custom expressions, custom JavaScript-class code, custom Wasm steps. Each execution is tenant-customer code.
- **Marketplace plugin executors (`microservices/marketplace/` per ADR-0249).** Plugin packages installed by tenants and executed inside Oyatie cells. Each plugin is tenant-customer code authored by a third party.
- **Agent-runtime tenant capabilities (`microservices/intelligence/` per ADR-0255).** Tenant-authored agent capabilities (custom tools, custom skills, custom action handlers) registered with the AI substrate. Each capability is tenant-customer code.
- **Developer-SDK uploaded modules (`microservices/developer-sdk/` per `feedback_developer_sdk_stainless_generator_2026_05_20`).** Tenant-uploaded SDK customization modules. Each module is tenant-customer code.

These five surfaces are **the floor for Tier 0**. Additional surfaces may be classified as Tier 0 in the future (e.g., a future "user-defined-function" facility) and added to the tier registry under D-10 promotion criteria.

### A.3 Named pressure: substrate µservices touch tenant data plane

The Oyatie corpus has substrate µservices that, while authored by Oyatie under monorepo + signed-supply-chain conditions, **touch tenant data plane in a way that elevates blast-radius**. Enumerated:

- **cloud-iam.** Primary credential validation, principal claim issuance, token signing roots, federated-identity broker. A compromise here unlocks impersonation of any tenant principal.
- **cloud-kms.** Tenant KMS root key handling, BYOK rotation, KEK derivation, DEK wrapping. A compromise here unlocks tenant ciphertext.
- **cloud-secrets.** OpenBao integration, dynamic secret issuance, secret rotation. A compromise here unlocks long-lived credentials.
- **audit-chain.** Audit event sealing, evidence pack generation, regulator export. A compromise here unlocks evidence tampering.
- **messenger MLS keys (`microservices/messenger/` per `feedback_mls_rfc_9420_e2ee_personal_messenger`).** RFC 9420 MLS group key derivation. A compromise here unlocks E2EE plaintext.
- **payments.** PCI-scoped payment-card data handling. A compromise here triggers PCI-DSS incident scope.
- **intelligence transport.** Provider-credential resolver for BYOK, model-routing decisions, provider-router gateway. A compromise here unlocks BYOK key material and tenant prompt/completion data.

These seven surfaces are **the floor for Tier 1**. Additional surfaces may be classified as Tier 1 via D-10 promotion criteria; any µservice that begins to touch a new data class flagged under ADR-0099 (data-class registry) is automatically a Tier 2 → Tier 1 promotion candidate.

### A.4 Named pressure: first-party app µservices are the long tail

The Oyatie corpus has ~77 active µservices (per `specs/master-plan-sequencing.json` realignment_wave_sequence enumeration). Subtracting the five Tier 0 surfaces and the seven Tier 1 surfaces, the long tail (~60 µservices) is first-party application code: crm, marketing-automation, contract-lifecycle-management, itsm, community, social, drive, docs, sheets, slides, calendar, video-call, calls-sip, mail, intelligence-non-transport-paths, workflow-engine-runner (non-tenant-step paths), cloud-billing, cloud-billing-tax, cloud-finops, cloud-marketplace (catalog paths), cloud-compute-functions (system-trusted invocations), and the rest of the corpus. These µservices are namespace-isolated within their home cell, mTLS-mediated by the service mesh, Cedar-policed at the Authorization layer, audit-chain-emitted on every privileged operation, and subject to the full Oyatie supply chain (cargo-deny + cargo-vet + cargo-audit + cosigned-image + provenance attestations). The marginal security benefit of running them under Kata + Cloud Hypervisor is negligible. They MUST run under runc.

These ~60 surfaces are **the default for Tier 2**. Default tier for new µservices is Tier 2 per D-9 below; promotion to Tier 1 requires the D-10 evidence pack.

### A.5 Named pressure: edge + perf-critical workloads cannot afford VM overhead

The Oyatie corpus has edge / static / perf-critical µservices that **cannot afford** the Kata overhead because their throughput / latency / packet-rate budgets are too tight. Enumerated:

- **api-gateway data-plane.** Front-end mesh proxy serving every external request. Tail latency budget is sub-millisecond.
- **Envoy edge.** Edge proxy for north-south traffic, mTLS termination, header rewrite, rate limiting.
- **ztunnel.** Ambient-mesh transport per ADR-0253. Per-packet processing path.
- **CDN edge cache.** Static-asset distribution path. Cache hit rate + connection density bound the workload.

These four surfaces are **the floor for Tier 3**. Tier 3 differs from Tier 2 in nodepool placement: Tier 3 workloads land on dedicated nodepools tuned for high packet rates (SR-IOV, hugepages, CPU pinning, kernel-bypass-friendly NICs) and isolated from Tier 0/1/2 noisy-neighbor effects.

### A.6 Named pressure: ADR-0248 cellular tier numbering convention reuse

ADR-0248 (Amazon-shape cellular architecture) established a five-tier cellular criticality model: Tier 0 = Foundation cells (identity / KMS / audit), Tier 1 = Substrate cells, Tier 2 = Capability cells, Tier 3 = Application cells, Tier 4 = Edge cells. The convention is **Tier 0 = highest blast-radius / most isolated**. Reuse of this numbering for the pod runtime axis is deliberate — when a µservice's cellular tier is Tier 0, the pod runtime tier is also Tier 0 or Tier 1. The two axes co-vary. A µservice in a Tier 0 cell running tenant-customer code is pod runtime Tier 0; a µservice in a Tier 0 cell running substrate code is pod runtime Tier 1; a µservice in a Tier 3 cell running first-party application code is pod runtime Tier 2; a µservice in a Tier 4 cell running edge proxy code is pod runtime Tier 3.

The numbering decision was deliberately reversed from the early Oyatie capability-tier draft (which used Bronze/Silver/Gold/Platinum ascending). The Bronze/Silver/Gold/Platinum tier ladder is retired per ADR-0329 + `feedback_no_capability_tiers_2026_05_20`; the pod runtime tier axis uses the ADR-0248 convention without reintroducing the ascending-prestige semantics of the retired ladder.

### A.7 Named pressure: Kyverno admission is the canonical gate per ADR-0183

ADR-0183 (policy-engine separation: Cedar app-authz vs Kyverno admission) established Kyverno as the canonical admission-control gate for Kubernetes-shaped concerns: image registry pinning, pod security policies, runtime class selection, namespace-scoped resource quotas, network-policy admission. Cedar evaluates application-layer authorization at request time; Kyverno evaluates Kubernetes-resource admission at admission time. The pod runtime tier decision lives at admission time (which runtime to bind to a pod, which nodepool to place a pod on) and is therefore a Kyverno concern, not a Cedar concern.

This ADR's D-5 Kyverno policy (`enforce-pod-runtime-tier`) is the canonical admission gate. The Cedar fragment surface remains untouched by this ADR.

### A.8 Named pressure: tenant-customer code paths exist today (Wave 15 in-flight authors them)

Wave 15 (per `specs/master-plan-sequencing.json` waves_15_plus) is actively authoring intelligence (wasmtime sandbox, agent-runtime tenant capabilities), workflow-studio (user workflows), marketplace (plugin executors), and developer-sdk (uploaded modules). These µservices are scaffolding their tenant-customer code paths now. Without this ADR, those scaffolds default to ADR-0254's "Kata everywhere" reading and either (a) impose Kata on the µservice's first-party paths (paying the density tax for no benefit) or (b) silently mix Kata + runc within the same µservice without an admission gate (creating a placement drift surface). This ADR resolves both failure modes by separating the runtime declaration from the µservice declaration and binding it to admission.

### A.9 Inherited constraints

- **ADR-0009 cell architecture.** Per-cell nodepool topology decisions live inside the cell boundary; cross-cell placement is not a concern here.
- **ADR-0044 inter-cell mesh tunnel.** Inter-cell traffic uses the mesh tunnel regardless of runtime tier.
- **ADR-0145 inter-µservice communication reform.** Direct gRPC over HTTP/3 + 3 invariants. Runtime tier does not affect the transport.
- **ADR-0183 policy-engine separation.** Kyverno admission is the canonical gate for this ADR's enforcement.
- **ADR-0211 in-house tech stack preference.** Cloud Hypervisor and Kata Containers are both Class C OSS (Apache 2.0 / MIT) per `docs/standards/dependency-policy.md`.
- **ADR-0240 sovereign-cloud per regional pack.** Sovereign cells follow the same runtime tier mapping; the cell's compliance pack additionally constrains Tier 0/1 placement to specific nodepools.
- **ADR-0244 tenant scoping.** tenant_class travels as a principal claim; pod runtime tier travels as a manifest declaration. The two axes do not intersect at request time.
- **ADR-0247 self-modification doctrine.** dev-tools-cell-N workloads under `oyatie.foundry.*` principals run at Tier 1 (substrate-touching) regardless of who authored the workflow.
- **ADR-0251 compliance pack cell certification.** Compliance packs may impose stricter floor than this ADR; e.g., a HIPAA-pinned cell requires Tier 0 placement for any pod handling PHI even if the pod is first-party.
- **ADR-0253 network topology.** ztunnel and Envoy edge are Tier 3 per A.5.
- **ADR-0254 deployment-model-spectrum.** This ADR amends it per D-2.
- **ADR-0263 observability emission contract.** Metrics carry `pod_runtime_tier` label additively.

### A.10 What this ADR does not assert

- This ADR does not retire Kata or Cloud Hypervisor; both remain canonical for Tier 0 + Tier 1 placement.
- This ADR does not retire runc; runc remains canonical for Tier 2 + Tier 3 placement.
- This ADR does not introduce a new runtime (e.g., gVisor) per the alternatives rejected in §F.
- This ADR does not change tenant_class semantics.
- This ADR does not change cellular tier numbering from ADR-0248.
- This ADR does not change compliance pack activation gating from ADR-0251.
- This ADR does not author the per-µservice manifest update; that is sequenced as a follow-on sub-wave.
- This ADR does not author the nodepool OpenTofu modules; that is sequenced as part of cloud-iac IaC module library work (ADR-0339 candidate).
- This ADR does not select a specific Kubernetes distribution beyond ADR-0121 (on-prem k8s stack) + ADR-0254 (K8s everywhere).

## Decision

### B.1 Decision statement

Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level `pod_runtime_tier` field whose value is an integer in `{0, 1, 2, 3}`. The integer maps to a RuntimeClass (D-4), a nodepool placement contract (D-3), and a Kyverno admission policy (D-5). The CI lane `oya-check-pod-runtime-tier` (D-6) validates declaration presence + valid integer + nodepool-placement match. The default tier for new µservices is **Tier 2** (D-9). Tier 2 → Tier 1 promotion requires the evidence pack in D-10. Quarterly tier review (D-8) walks the corpus for promotion candidates.

The four tiers are defined in D-2.

The two-nodepool-per-cell topology is defined in D-3.

The three RuntimeClasses are defined in D-4.

The Kyverno admission policy is defined in D-5.

The CI lane is defined in D-6.

The capacity model implications are defined in D-7.

The quarterly tier review process is defined in D-8.

The default tier for new µservices is defined in D-9.

The tier promotion criteria are defined in D-10.

The cellular tier integration is defined in D-11.

The incident-response classification is defined in D-12.

### B.2 Numbered decision clauses

B2.001. `microservices/<name>/manifest.json` declares `pod_runtime_tier ∈ {0, 1, 2, 3}`.

B2.002. Tier 0 = tenant-customer untrusted code; runtime = Kata Containers + Cloud Hypervisor.

B2.003. Tier 1 = substrate µservices touching tenant data plane; runtime = Kata Containers + Cloud Hypervisor.

B2.004. Tier 2 = first-party application µservices; runtime = runc.

B2.005. Tier 3 = edge / static / perf-critical; runtime = runc on dedicated nodepool.

B2.006. The tier numbering aligns with the ADR-0248 cellular criticality convention: Tier 0 = highest blast-radius / most isolated.

B2.007. The default tier for newly-created µservices is Tier 2 unless the µservice's PRD §B explicitly declares one of the Tier 0 / Tier 1 / Tier 3 surfaces from A.2 / A.3 / A.5.

B2.008. Tier 2 → Tier 1 promotion requires the evidence pack in D-10; promotion is a council-architecture + council-security decision.

B2.009. Tier 0 declaration requires evidence that the µservice executes tenant-customer code; the floor is the five surfaces enumerated in A.2.

B2.010. Tier 3 declaration requires evidence of edge / packet-rate-bound / latency-bound workload; the floor is the four surfaces enumerated in A.5.

B2.011. Per-cell nodepool topology MUST include at minimum two nodepools (`kata-pool` + `runc-pool`) and MAY include a third (`runc-edge-pool`) for Tier 3 placement.

B2.012. The `kata-pool` MUST have Kata Containers + Cloud Hypervisor installed; node-label `node.oyatie.io/runtime-tier=kata`; taint `runtime-tier=kata:NoSchedule`; toleration enforced by Kyverno for Tier 0 + Tier 1 pods only.

B2.013. The `runc-pool` MUST run containerd with runc; node-label `node.oyatie.io/runtime-tier=runc`; no taint (default nodepool); accepts Tier 2 pods by default.

B2.014. The `runc-edge-pool` (when present) MUST run containerd with runc on dedicated edge-tuned hardware (SR-IOV, hugepages, CPU pinning, kernel-bypass-friendly NICs); node-label `node.oyatie.io/runtime-tier=runc-edge`; taint `runtime-tier=runc-edge:NoSchedule`; toleration enforced by Kyverno for Tier 3 pods only.

B2.015. Three RuntimeClasses are declared per cell: `kata-cloud-hypervisor` (Tier 0 + Tier 1), `runc` (Tier 2), `runc-edge` (Tier 3).

B2.016. `kata-cloud-hypervisor` RuntimeClass has `handler: kata-clh` (Kata Containers + Cloud Hypervisor handler binary); scheduling.nodeSelector pins to `node.oyatie.io/runtime-tier=kata`; scheduling.tolerations include `runtime-tier=kata:NoSchedule`.

B2.017. `runc` RuntimeClass has `handler: runc` (containerd default); no nodeSelector restriction beyond the cell's default nodepool.

B2.018. `runc-edge` RuntimeClass has `handler: runc` (containerd default); scheduling.nodeSelector pins to `node.oyatie.io/runtime-tier=runc-edge`; scheduling.tolerations include `runtime-tier=runc-edge:NoSchedule`.

B2.019. The Kyverno admission policy `enforce-pod-runtime-tier` is BLOCKER-class.

B2.020. The Kyverno policy refuses pod admission when `pod.spec.runtimeClassName` is unset and the µservice's manifest declares a non-Tier-2 tier (Tier 2 may default to runc when unset).

B2.021. The Kyverno policy refuses pod admission when `pod.spec.runtimeClassName` does not match the µservice's declared tier per the D-4 mapping.

B2.022. The Kyverno policy refuses pod admission when `pod.spec.nodeSelector` or `pod.spec.tolerations` do not match the nodepool contract for the declared tier per D-3.

B2.023. The Kyverno policy emits an audit event `pod.runtime.tier.denied` on every deny per ADR-0263.

B2.024. The CI lane `oya-check-pod-runtime-tier` is REPORT-ONLY at landing time and promotes to BLOCKER when corpus-wide manifest declarations land.

B2.025. The CI lane validates: (a) manifest declares pod_runtime_tier; (b) value is in {0, 1, 2, 3}; (c) Tier 0 declaration cites at least one of the A.2 surfaces; (d) Tier 1 declaration cites at least one of the A.3 surfaces; (e) Tier 3 declaration cites at least one of the A.5 surfaces; (f) Tier 2 declaration is permitted by default.

B2.026. The CI lane also validates per-µservice Helm chart / Kubernetes manifests reference the correct RuntimeClass for the declared tier.

B2.027. Capacity planning per cell (per ADR-0009 + ADR-0248) MUST size the `kata-pool` to absorb Tier 0 + Tier 1 worst-case + a multiplicative density factor of 1.5x (because Kata costs 30-40% density vs runc, the kata-pool nodecount is scaled accordingly).

B2.028. Capacity planning per cell MUST size the `runc-pool` to absorb Tier 2 worst-case + a 1.0x baseline.

B2.029. Capacity planning per cell MUST size the `runc-edge-pool` (when present) to absorb Tier 3 worst-case + a 1.0x baseline with edge-hardware constraints.

B2.030. Quarterly tier review (D-8) walks the corpus and identifies promotion / demotion candidates; council-architecture + council-security decide.

B2.031. Tier 2 → Tier 1 promotion is triggered when (a) the µservice begins handling a new data class flagged under ADR-0099 (data-class registry); or (b) the µservice begins owning a key-derivation operation; or (c) the µservice begins owning an audit-seal operation; or (d) the µservice begins owning a token-signing operation; or (e) a compliance pack requires it.

B2.032. Tier 1 → Tier 0 promotion is triggered when the µservice begins executing tenant-customer code paths that the existing Tier 1 placement cannot adequately isolate.

B2.033. Tier 2 → Tier 3 demotion (rare) is triggered when the µservice's hot-path becomes packet-rate-bound or sub-millisecond-latency-bound.

B2.034. Tier 0 → Tier 1 demotion (rare) is forbidden without explicit ADR retiring the Tier 0 surface; once tenant-customer code is in scope, Kata-isolation is non-negotiable.

B2.035. Tier 1 → Tier 2 demotion (rare) requires evidence that the µservice no longer touches tenant data plane (e.g., a key-handling responsibility moved to cloud-kms).

B2.036. Incident-response classification (D-12) maps tier to severity floor: Tier 0 incident = Sev-1 minimum; Tier 1 incident = Sev-1 minimum; Tier 2 incident = Sev-2 minimum; Tier 3 incident = Sev-2 minimum.

B2.037. Cellular tier (ADR-0248) and pod runtime tier (this ADR) co-vary: Tier 0 cell hosts Tier 0 + Tier 1 pods; Tier 1 cell hosts Tier 1 pods; Tier 2 cell hosts Tier 2 pods; Tier 3 cell hosts Tier 2 pods; Tier 4 cell hosts Tier 3 pods.

B2.038. Sovereign-cell placement (ADR-0240) preserves the runtime tier; a sovereign HIPAA cell may host any of {Tier 0, Tier 1, Tier 2, Tier 3} pods provided the compliance pack's PHI-handling pods are Tier 0 or Tier 1.

B2.039. dev-tools-cell-N (ADR-0247 self-modification cells) hosts Tier 1 pods for `oyatie.foundry.*` workflow execution; foundry workflow library code is Oyatie-authored but operates with elevated privileges and is therefore Tier 1.

B2.040. Marketplace plugin execution (ADR-0249 marketplace) hosts Tier 0 pods for the executor sandbox; the marketplace catalog itself (browsing, search, billing integration) is Tier 2.

B2.041. Workflow-studio user workflows (ADR-0328 workflow-studio) host Tier 0 pods for the workflow runtime; the workflow-studio editor / catalog / collaboration paths are Tier 2.

B2.042. agent-runtime tenant capabilities (ADR-0255 intelligence) host Tier 0 pods for the tenant capability executor; the agent dispatch / routing / eval / audit paths are Tier 1 (substrate-touching).

B2.043. developer-sdk uploaded modules (`feedback_developer_sdk_stainless_generator_2026_05_20`) host Tier 0 pods for module execution; the SDK generator + catalog + publishing paths are Tier 2.

B2.044. Wasmtime sandbox host (intelligence IP-WASMTIME-*) hosts Tier 0 pods for Wasm execution; the Wasmtime supervisor + lifecycle paths may be Tier 1.

B2.045. cloud-iam, cloud-kms, cloud-secrets, audit-chain, messenger MLS keys, payments, and intelligence transport paths are Tier 1 per A.3.

B2.046. api-gateway data-plane, Envoy edge, ztunnel, and CDN edge cache are Tier 3 per A.5.

B2.047. All other ~60 first-party application µservices default to Tier 2 per A.4.

B2.048. The pod runtime tier declaration is binding on every contributor (human and agent) immediately upon Acceptance. New µservice manifests MUST declare the field; existing µservice manifests MUST declare the field within the corpus-wide manifest sub-wave that follows this ADR's Acceptance.

B2.049. The retirement of "Kata everywhere" reading of ADR-0254 is final on Acceptance. ADR-0254 amendment per D-2.10 records the carve-out.

B2.050. No waiver mechanism. Tier 0 / Tier 1 / Tier 3 placement requires the evidence pack in D-10. Tier 2 is the no-evidence default.

B2.051. The new lanes (`oya-check-pod-runtime-tier`, `oya-governance-runtime-class-allowlist`, `oya-governance-nodepool-binding`, `oya-governance-tier-promotion-evidence`) and the Kyverno policy (`enforce-pod-runtime-tier`) are REPORT-ONLY at landing and promote to BLOCKER per the §G sunset schedule.

B2.052. The realignment_wave_sequence in `specs/master-plan-sequencing.json` adds the new sub-wave `15S-Pod-Runtime-Tier-declaration` queued for dispatch after this ADR lands; the sub-wave is per-µservice bespoke authoring under ADR-0322 substance-bar + ADR-0324 anti-template discipline.

B2.053. The canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md` adds a Pod Runtime Tier section naming this ADR and the four tiers.

### B.3 What this decision does not do

- This ADR does not author per-µservice manifest updates; the corpus-wide declaration sub-wave handles that.
- This ADR does not author the OpenTofu nodepool modules; that work belongs in cloud-iac under ADR-0339 (shared IaC module library) and per-cell OpenTofu modules.
- This ADR does not change the cell topology decision-tree from ADR-0248.
- This ADR does not introduce gVisor or other competing runtimes.
- This ADR does not change the Kubernetes distribution selection from ADR-0121 / ADR-0254.

## Consequences

### C.1 Positive consequences

- **Pod density restored on first-party code.** Tier 2 + Tier 3 workloads regain the 30-40 percent density they would lose under "Kata everywhere." Cell sizing models become realistic at hyperscaler density rather than Kata density.
- **Cold-start latency restored on Tier 2.** Ephemeral Tier 2 workloads (batch jobs, ad-hoc agent dispatches when no tenant-customer code is involved, queue workers) regain the 200-500 ms cold-start budget.
- **Tail latency restored on Tier 3.** Edge proxies and packet-rate-bound paths run under runc on dedicated nodepools with SR-IOV / hugepages / CPU pinning, restoring sub-millisecond p99 tails.
- **VM-isolation preserved where it matters.** Tier 0 tenant-customer code paths execute under Kata + Cloud Hypervisor with measurable guest-kernel-escape protection. Tier 1 substrate code paths protect tenant data plane against compromise lateralization.
- **Admission-time enforcement.** Kyverno admission refuses misconfigured pods at admission; no need to wait for runtime detection.
- **Explicit tier vocabulary.** Per-µservice manifest declaration makes the runtime decision auditable, reviewable, and ADR-traceable.
- **Quarterly review process.** Tier classifications evolve as µservice scope evolves; quarterly review surfaces promotion / demotion candidates.
- **Compliance pack composability.** Compliance packs (ADR-0251) stack cleanly: HIPAA-pinned cell + Tier 0/1 pods provides PHI-grade isolation; SOC2 + Tier 2 provides audit-trail isolation; PCI + Tier 0/1 provides cardholder-data isolation.
- **Capacity-planning clarity.** kata-pool / runc-pool / runc-edge-pool sizing is explicit per cell; FinOps modeling becomes deterministic per tier.

### C.2 Negative consequences

- **Per-µservice manifest sub-wave required.** ~77 µservices need a manifest update declaring pod_runtime_tier. The update is per-µservice bespoke (substance-bar applies).
- **Per-cell nodepool topology update required.** Existing cells need two-nodepool (or three-nodepool) topology; that requires OpenTofu module work.
- **Kyverno policy authoring + rollout.** The `enforce-pod-runtime-tier` Kyverno policy must be authored, deployed per cell, and soaked as REPORT-ONLY before promoting to BLOCKER.
- **Cross-team coordination.** Pod runtime tier classification involves council-architecture + council-security + ops-sre-reliability; each Tier 0 / Tier 1 / Tier 3 declaration is reviewed.
- **Tier-creep risk.** Tier 2 → Tier 1 promotion is easy under D-10 evidence; quarterly review must catch promotion-without-evidence drift.
- **Edge nodepool hardware cost.** The runc-edge-pool requires SR-IOV / hugepages / CPU pinning; this is a cell-specific hardware constraint that may not be available in every cloud / on-prem context.

### C.3 Neutral consequences

- **Service mesh unchanged.** Direct gRPC over HTTP/3 + mTLS via ADR-0145 + ADR-0253 continues regardless of runtime tier.
- **Cedar authorization unchanged.** Cedar evaluates application-layer authorization at request time; runtime tier is admission-time.
- **Observability emission preserved.** Per ADR-0263 the new `pod_runtime_tier` label is additive.
- **Tenant_class behavior preserved.** demo_trial and paid tenants see the same runtime tier for the same µservice.
- **Compliance pack activation gating preserved.** Compliance packs continue to gate on tenant_class = paid per ADR-0251; runtime tier is a placement constraint inside the cell.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single manifest field declares runtime tier across 77+ µservices | Every µservice manifest declares pod_runtime_tier; oya-check-pod-runtime-tier green |
| Security posture | Tenant-customer code paths isolated under Kata + Cloud Hypervisor | Tier 0 + Tier 1 pods deployed only to kata-pool; admission denies misplacement |
| Performance | Tier 2 + Tier 3 first-party code paths run under runc; sub-millisecond edge tail latency preserved | Tier 2 / Tier 3 placement on runc nodepools confirmed by admission; per-cell SLOs unchanged from runc baseline |
| Capacity | kata-pool sized for Tier 0 + Tier 1; runc-pool sized for Tier 2; runc-edge-pool sized for Tier 3 | Cell capacity model documents the sizing per ADR-0009 |
| Observability | Per-pod pod_runtime_tier label on every metric + audit event | Sampled events / metrics carry the label; observability dashboard segments by tier |
| Compliance | HIPAA / PCI / SOC2 / CSAP / EU-AI-Act Annex III packs compose with runtime tier | Per ADR-0251 cell certification levels; PHI-handling pods are Tier 0 or Tier 1 |
| Incident response | Sev-1 floor for Tier 0 + Tier 1 incidents; Sev-2 floor for Tier 2 + Tier 3 | Per ADR-0263 incident classification carries pod_runtime_tier |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS Fargate uses Firecracker microVM (Cloud-Hypervisor sibling) for tenant code isolation in a managed-compute service. Google Cloud Run uses gVisor for tenant code sandboxing in its second-generation execution environment. Azure Container Apps uses Hyper-V containers for tenant isolation. The three hyperscalers all separate tenant-trusted-code execution from tenant-untrusted-code execution at the runtime layer. This ADR adopts the same separation under self-hosted K8s. Kata + Cloud Hypervisor is the Oyatie self-hosted equivalent of Firecracker-class tenant isolation.

**Failure-mode tree.** Failure modes: (1) µservice forgets to declare pod_runtime_tier → CI lane REPORT-ONLY signal, then BLOCKER after sunset; (2) µservice declares Tier 2 but actually executes tenant-customer code → council-security catches in quarterly review; D-10 promotion is triggered; (3) Kata pod fails to start (Cloud Hypervisor crash) → admission allowed, runtime fault → workload reschedules to kata-pool; (4) kata-pool exhausted → Tier 0 / Tier 1 pods cannot schedule; observability emits saturation alert; cell capacity planning per ADR-0009 catches in advance; (5) runc-edge-pool hardware missing → Tier 3 pods cannot schedule; cell topology pre-flight catches; (6) wrong RuntimeClass labeled → Kyverno admission denies; CI lane catches in pre-merge; (7) tenant-customer code path leaks into a Tier 2 µservice via a new feature → council-security quarterly review catches; D-10 promotion is triggered.

**Capacity math.** Per cell, kata-pool ≈ 1.5x density-scaled (Kata's 30-40% density overhead). For a 100-node cell baseline at runc: ~70 nodes runc-pool + ~30 nodes kata-pool + ~5 nodes runc-edge-pool (if Tier 3 workloads are present). Total ~105 nodes vs ~100 baseline; Kata cost ≈ 5% cell-level node count, vs 30-40% cell-level node count if all nodes were Kata. The trade is favorable.

**Observability hooks.** Every pod's metric emission gains a `pod_runtime_tier` label (values: `0`, `1`, `2`, `3`). Audit events carry `pod_runtime_tier` field. Distributed-tracing spans carry `pod_runtime_tier` attribute. The cardinality multiplier is bounded at 4.

**Rollback path.** Per-µservice rollback: a misclassified µservice flips its manifest pod_runtime_tier and lands the next deployment; admission re-evaluates at the next pod creation. Cell-level rollback: kata-pool can be removed if Tier 0 / Tier 1 workloads are reclassified away; this is reversible. Cross-µservice rollback (e.g., abandon the four-tier model entirely) requires a new ADR superseding this one.

**Multi-region awareness.** Each region's cells declare their own kata-pool / runc-pool / runc-edge-pool topology. Cross-region Tier 0 / Tier 1 placement is bounded by sovereign-cell pinning per ADR-0240.

**Sovereign-cell awareness.** Sovereign cells (HIPAA, GDPR-strict, CSAP, PCI, IL5) host whichever tier their workloads need; the compliance pack does not constrain the runtime tier directly but does constrain the workloads (e.g., PHI-handling pods MUST be Tier 0 or Tier 1).

**Versioning + deprecation.** This ADR is versioned per ADR-0108. Tier definitions may evolve under amendment ADRs. ADR-0254 is the parent invariant.

## D. Detailed mechanics — twelve enforcement surfaces

The pod runtime tier mechanism touches twelve enforcement surfaces. Each subsection D-1 through D-12 enumerates one surface. Numbering is normative.

### D-1: Manifest field declaration — `pod_runtime_tier`

D-1.1. Every µservice's `microservices/<name>/manifest.json` MUST declare a top-level field `pod_runtime_tier` whose value is an integer in `{0, 1, 2, 3}`.

D-1.2. The field is REQUIRED for every µservice that produces a workload (i.e., a Helm chart, a Kubernetes Deployment / StatefulSet / Job / CronJob, a podSpec).

D-1.3. The field is OPTIONAL for spec-only µservices that produce no workload (rare; mostly definitional µservices).

D-1.4. The manifest schema at `/specs/microservices/manifest-schema.json` is updated to add the field with type integer + enum [0, 1, 2, 3] + required-when-workload-emitted constraint.

D-1.5. Concrete manifest example for a Tier 0 µservice (workflow-studio):

```json
{
  "name": "workflow-studio",
  "pod_runtime_tier": 0,
  "pod_runtime_tier_justification": "Hosts tenant-authored workflow steps (custom expressions, custom Wasm, custom JS-class code) executed inside user workflows per ADR-0338 A.2; tenant-customer code MUST execute under VM-isolation.",
  "pod_runtime_tier_surface_evidence": [
    "microservices/workflow-studio/src/workflow_runtime/",
    "microservices/workflow-studio/IPs/IP-tenant-step-executor.md"
  ]
}
```

D-1.6. Concrete manifest example for a Tier 1 µservice (cloud-kms):

```json
{
  "name": "cloud-kms",
  "pod_runtime_tier": 1,
  "pod_runtime_tier_justification": "Substrate µservice handling tenant KMS root key derivation, BYOK rotation, KEK / DEK wrapping per ADR-0338 A.3; compromise unlocks tenant ciphertext.",
  "pod_runtime_tier_surface_evidence": [
    "microservices/cloud-kms/src/key_derivation/",
    "microservices/cloud-kms/IPs/IP-byok-rotation.md"
  ]
}
```

D-1.7. Concrete manifest example for a Tier 2 µservice (crm):

```json
{
  "name": "crm",
  "pod_runtime_tier": 2,
  "pod_runtime_tier_justification": "First-party application µservice serving CRM functionality per ADR-0338 A.4; namespace-isolated + mTLS-mediated + Cedar-policed + supply-chain-signed; no tenant-customer code execution; no tenant-data-plane substrate role."
}
```

D-1.8. Concrete manifest example for a Tier 3 µservice (api-gateway data-plane):

```json
{
  "name": "api-gateway",
  "pod_runtime_tier": 3,
  "pod_runtime_tier_justification": "Edge data-plane proxy for north-south traffic per ADR-0338 A.5; sub-millisecond p99 tail latency budget; packet-rate-bound hot path; requires dedicated edge-tuned nodepool."
}
```

D-1.9. CI lane `oya-check-pod-runtime-tier` step 1 parses the manifest and validates the field is present, well-typed, and within the enum.

D-1.10. CI lane step 2 validates that Tier 0 declarations cite at least one A.2 surface; Tier 1 declarations cite at least one A.3 surface; Tier 3 declarations cite at least one A.5 surface. Tier 2 declarations require no citation (it is the default).

### D-2: Four-tier definition — Tier 0 / Tier 1 / Tier 2 / Tier 3

D-2.1. **Tier 0 — Tenant-customer untrusted code.** Pods that execute code authored or supplied by tenants. Runtime = Kata Containers + Cloud Hypervisor. Floor surfaces from A.2: Wasmtime sandbox host, workflow-studio user workflows, marketplace plugin executors, agent-runtime tenant capabilities, developer-sdk uploaded modules.

D-2.2. **Tier 1 — Substrate touching tenant data plane.** Pods that execute Oyatie-authored code with elevated data-plane responsibilities. Runtime = Kata Containers + Cloud Hypervisor. Floor surfaces from A.3: cloud-iam, cloud-kms, cloud-secrets, audit-chain, messenger MLS keys, payments, intelligence transport.

D-2.3. **Tier 2 — First-party application µservices.** Pods that execute Oyatie-authored application code without tenant-customer execution and without elevated data-plane responsibilities. Runtime = runc. Default tier for new µservices.

D-2.4. **Tier 3 — Edge / static / perf-critical.** Pods that execute Oyatie-authored edge / proxy / cache code with sub-millisecond p99 latency or packet-rate-bound workloads. Runtime = runc on dedicated edge-tuned nodepool. Floor surfaces from A.5: api-gateway data-plane, Envoy edge, ztunnel, CDN edge cache.

D-2.5. Tier 0 and Tier 1 share the runtime (Kata + Cloud Hypervisor) but differ in semantic provenance (tenant-customer vs first-party-substrate). The placement nodepool is shared (`kata-pool`). The Cedar evaluation surface differs (Tier 0 pods carry tenant_id-bound capability scopes; Tier 1 pods carry substrate-scoped capabilities).

D-2.6. Tier 2 and Tier 3 share the runtime (runc) but differ in nodepool (`runc-pool` vs `runc-edge-pool`). Tier 3 pods land on edge-tuned nodes; Tier 2 pods land on general-purpose nodes.

D-2.7. A µservice MAY NOT split its pods across tiers. If a µservice has pods that need Tier 0 isolation AND pods that need Tier 2 placement, the µservice MUST be decomposed into a Tier 0 µservice and a Tier 2 µservice. (Example: workflow-studio's user-workflow executor is Tier 0; workflow-studio's editor / catalog / collaboration is logically Tier 2 but physically MAY be in the same µservice IF the editor / catalog / collaboration pods are isolated from the user-workflow executor pods at the deployment level.)

D-2.8. A µservice MAY use auxiliary mechanisms to harden Tier 2 placement beyond baseline (e.g., AppArmor profiles, SecComp filters, read-only root filesystem, no-new-privileges). These are independent of the tier classification and applied uniformly.

D-2.9. The four-tier model does NOT include a "tier 4" or "tier -1." The model is bounded at four levels.

D-2.10. ADR-0254 amendment vector: this ADR amends ADR-0254 by adding the four-tier carve-out. ADR-0254's "K8s + Cloud Hypervisor + Kata pods" invariant is preserved; the carve-out specifies which pods are Kata and which are runc.

### D-3: Per-cell nodepool topology

D-3.1. Every cell (per ADR-0009 + ADR-0248) MUST declare at minimum two nodepools: `kata-pool` and `runc-pool`. Cells hosting Tier 3 workloads also declare `runc-edge-pool`.

D-3.2. **kata-pool.** Containerd + Kata Containers + Cloud Hypervisor. Node label `node.oyatie.io/runtime-tier=kata`. Node taint `runtime-tier=kata:NoSchedule` (so default workloads don't land here). Toleration enforced by Kyverno admission only for Tier 0 + Tier 1 pods. Node hardware: standard K8s worker; minimum 16 vCPU + 64 GiB RAM (per AWS m6i.4xlarge-class baseline) to amortize Kata overhead.

D-3.3. **runc-pool.** Containerd + runc (no Kata installed). Node label `node.oyatie.io/runtime-tier=runc`. No taint (default nodepool). Accepts Tier 2 pods by default; rejects Tier 0 / Tier 1 / Tier 3 pods via Kyverno admission. Node hardware: general-purpose K8s worker.

D-3.4. **runc-edge-pool.** Containerd + runc (no Kata installed). Node label `node.oyatie.io/runtime-tier=runc-edge`. Node taint `runtime-tier=runc-edge:NoSchedule`. Toleration enforced by Kyverno admission only for Tier 3 pods. Node hardware: edge-tuned (SR-IOV, hugepages 1 GiB pages, CPU pinning enabled, ENA / AF_XDP / kernel-bypass-friendly NICs, dedicated NUMA topology).

D-3.5. Capacity planning per ADR-0009 + ADR-0248 SHOULD size the kata-pool at ~30-40 percent of the cell's compute budget when Tier 0 + Tier 1 workloads dominate; at ~10-20 percent when Tier 2 dominates. Exact ratio is per-cell-specific.

D-3.6. The OpenTofu module library (queued under ADR-0339 candidate) MUST provide three nodepool primitives: `kata-pool`, `runc-pool`, `runc-edge-pool` per deployment context.

D-3.7. On-prem cells (per ADR-0121) follow the same topology with self-managed Kata + Cloud Hypervisor installation under containerd.

D-3.8. Sovereign cells (per ADR-0240) follow the same topology with sovereign-cloud-specific nodepool configurations.

### D-4: Three RuntimeClasses

D-4.1. Three Kubernetes RuntimeClass resources MUST be declared per cell:

D-4.2. **`kata-cloud-hypervisor`** (for Tier 0 + Tier 1):
```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: kata-cloud-hypervisor
handler: kata-clh
scheduling:
  nodeSelector:
    node.oyatie.io/runtime-tier: kata
  tolerations:
    - key: runtime-tier
      operator: Equal
      value: kata
      effect: NoSchedule
overhead:
  podFixed:
    cpu: 250m
    memory: 256Mi
```

D-4.3. **`runc`** (for Tier 2):
```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: runc
handler: runc
```

D-4.4. **`runc-edge`** (for Tier 3):
```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: runc-edge
handler: runc
scheduling:
  nodeSelector:
    node.oyatie.io/runtime-tier: runc-edge
  tolerations:
    - key: runtime-tier
      operator: Equal
      value: runc-edge
      effect: NoSchedule
```

D-4.5. The RuntimeClass `overhead.podFixed` for `kata-cloud-hypervisor` reserves the Cloud Hypervisor guest-kernel + VM-state overhead per pod so the K8s scheduler accounts for it in placement decisions.

D-4.6. RuntimeClass declarations are owned by the cell's OpenTofu module library (under cloud-iac) and applied at cell bootstrap.

D-4.7. The `oya-governance-runtime-class-allowlist` lane refuses RuntimeClass declarations outside the three canonical names.

### D-5: Kyverno admission policy `enforce-pod-runtime-tier`

D-5.1. A Kyverno ClusterPolicy named `enforce-pod-runtime-tier` MUST be deployed in every cell. The policy refuses pod admission when the pod's RuntimeClass / nodepool placement does not match the µservice's manifest-declared `pod_runtime_tier`.

D-5.2. The policy resolves the µservice's declared tier by reading a pod label (`oyatie.io/microservice=<name>`) and looking up the manifest (admission-time cache populated at cell bootstrap from the corpus manifest registry).

D-5.3. The policy refuses admission when:
  - The pod's `spec.runtimeClassName` is empty AND the µservice's declared tier is not Tier 2 (Tier 2 may default to runc when unset).
  - The pod's `spec.runtimeClassName` does not match the tier → RuntimeClass mapping in D-4.
  - The pod's `spec.nodeSelector` does not match the RuntimeClass's nodeSelector.
  - The pod's `spec.tolerations` do not include the required taint toleration.
  - The pod's microservice label is absent.
  - The pod's microservice label references a µservice whose manifest has no `pod_runtime_tier` declaration (denial only after sunset day 30).

D-5.4. The policy is REPORT-ONLY at landing (`validationFailureAction: audit`); promoted to BLOCKER (`validationFailureAction: enforce`) at sunset day 30 OR when corpus-wide manifest declarations land, whichever later.

D-5.5. The policy emits an audit event `pod.runtime.tier.denied` on every deny per ADR-0263, with fields: `microservice`, `declared_tier`, `attempted_runtime_class`, `attempted_nodepool`, `reason`.

D-5.6. Concrete Kyverno policy excerpt:

```yaml
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: enforce-pod-runtime-tier
  annotations:
    oyatie.io/adr: ADR-0338
spec:
  validationFailureAction: audit  # audit→enforce at sunset
  background: true
  rules:
    - name: tier-0-and-1-require-kata-runtimeclass
      match:
        any:
          - resources:
              kinds: [Pod]
      context:
        - name: declaredTier
          apiCall:
            urlPath: "/api/v1/namespaces/{{request.namespace}}/configmaps/oyatie-microservice-registry"
            jmesPath: "data.\"{{request.object.metadata.labels.\"oyatie.io/microservice\"}}\".pod_runtime_tier"
      preconditions:
        all:
          - key: "{{ declaredTier }}"
            operator: AnyIn
            value: ["0", "1"]
      validate:
        message: "Tier 0 / Tier 1 µservices MUST set runtimeClassName=kata-cloud-hypervisor per ADR-0338 D-5"
        pattern:
          spec:
            runtimeClassName: kata-cloud-hypervisor
    - name: tier-2-requires-runc
      # ... analogous; validates runtimeClassName == runc OR unset (Tier 2 default)
    - name: tier-3-requires-runc-edge
      # ... analogous; validates runtimeClassName == runc-edge AND nodepool toleration
```

D-5.7. The `oyatie-microservice-registry` ConfigMap is populated at cell bootstrap with a snapshot of the corpus manifest registry; cell-bootstrap reconciles updates.

D-5.8. The `oya-governance-nodepool-binding` lane validates per-µservice Helm chart references the correct RuntimeClass + tolerations.

### D-6: CI lane `oya-check-pod-runtime-tier`

D-6.1. A new CI lane `oya-check-pod-runtime-tier` is added under the corpus governance lane set at `crates/oya-check-pod-runtime-tier/`.

D-6.2. The lane is REPORT-ONLY at landing (advisory); promoted to BLOCKER per §G sunset schedule.

D-6.3. The lane validates per-µservice:
  - (a) `manifest.json` declares `pod_runtime_tier`.
  - (b) Value is an integer in `{0, 1, 2, 3}`.
  - (c) Tier 0 declaration includes `pod_runtime_tier_justification` + `pod_runtime_tier_surface_evidence` array citing at least one A.2 surface.
  - (d) Tier 1 declaration includes `pod_runtime_tier_justification` + `pod_runtime_tier_surface_evidence` array citing at least one A.3 surface.
  - (e) Tier 3 declaration includes `pod_runtime_tier_justification` + `pod_runtime_tier_surface_evidence` array citing at least one A.5 surface.
  - (f) Tier 2 declaration requires only `pod_runtime_tier_justification` (no surface citation required because Tier 2 is the default).
  - (g) Per-µservice Helm chart at `microservices/<name>/helm/values.yaml` declares `runtimeClassName` matching the declared tier.
  - (h) Per-µservice Helm chart declares pod tolerations matching the nodepool contract.

D-6.4. The lane emits structured findings under `evidence/pod-runtime-tier/<microservice>.json` with per-validation status.

D-6.5. The lane is Rust-strict (per `feedback_rust_strict_only_no_python_2026_05_20`); implementation at `crates/oya-check-pod-runtime-tier/src/lib.rs`.

D-6.6. The `oya-governance-tier-promotion-evidence` lane validates that Tier 2 → Tier 1 promotions carry the D-10 evidence pack at promotion time.

### D-7: Capacity model implications

D-7.1. Per-cell capacity planning per ADR-0009 + ADR-0248 MUST size three pools:

D-7.2. **kata-pool sizing.** Sum of Tier 0 + Tier 1 worst-case pod count × 1.5x density-scaled (because Kata's ~30-40% density overhead). The 1.5x factor includes headroom for failover.

D-7.3. **runc-pool sizing.** Sum of Tier 2 worst-case pod count × 1.0x baseline.

D-7.4. **runc-edge-pool sizing.** Sum of Tier 3 worst-case pod count × 1.0x baseline + edge-hardware constraint (some cells will not have edge hardware available).

D-7.5. FinOps cost modeling per ADR-0174 (finops cost tag) MUST attribute Tier 0 + Tier 1 cost separately from Tier 2 + Tier 3 cost so the Kata-overhead premium is observable per cell.

D-7.6. Demo_trial tenants on OCI Always Free per `feedback_oci_always_free_maximization_2026_05_20` host Tier 0 + Tier 1 pods inside the Always Free Ampere A1 ARM nodepool when Kata-on-ARM is supported (Kata supports aarch64 since Kata 3.x with Cloud Hypervisor backend); otherwise demo_trial tenants are denied Tier 0 / Tier 1 µservices that would require Kata on a non-supporting context.

D-7.7. Per-cell capacity dashboards (per ADR-0263) MUST visualize per-tier utilization, kata-pool saturation, and runc-edge-pool saturation as first-class metrics.

D-7.8. Tier 0 / Tier 1 saturation alerts MUST be Sev-2 because Tier 0 / Tier 1 workloads cannot fall back to runc.

### D-8: Quarterly tier review process

D-8.1. Council-architecture + council-security conduct a quarterly tier review per cycle.

D-8.2. The review walks the corpus's per-µservice manifests, identifies tier-classification drift, and produces a Wave-N+M-tier-review evidence file at `.omc/state/pod-runtime-tier-review-<date>.md`.

D-8.3. The review evaluates each µservice against:
  - Has the µservice's scope changed since the prior review?
  - Does the µservice now handle a data class flagged under ADR-0099 (data-class registry)?
  - Does the µservice now execute any tenant-customer code paths (A.2)?
  - Does the µservice now own a key-derivation, audit-seal, or token-signing operation (A.3)?
  - Does the µservice now have an edge / packet-rate-bound / sub-millisecond hot path (A.5)?

D-8.4. Promotion candidates are filed as per-µservice promotion-evidence IPs at `microservices/<name>/IPs/IP-tier-promotion-<source>-to-<target>.md`.

D-8.5. Demotion candidates require similar evidence (rare; D-10.4).

D-8.6. The review is published as an addendum to the quarterly council-architecture review.

D-8.7. The review references the council-security access-control review per ADR-0244 for cross-cutting evidence.

### D-9: Default tier for new µservices

D-9.1. The default `pod_runtime_tier` for newly-created µservices is **Tier 2**.

D-9.2. The default is binding unless the µservice's PRD §B explicitly declares one of the A.2 / A.3 / A.5 surfaces.

D-9.3. New µservice manifests are validated at creation time by the `oya-check-pod-runtime-tier` lane.

D-9.4. The default tier choice reflects the empirical distribution: ~60 of ~77 µservices are first-party application µservices (Tier 2). The default biases toward the modal case.

D-9.5. A new µservice declaring Tier 0 / Tier 1 / Tier 3 at creation time MUST include the surface-evidence citation per D-6.3.c/d/e.

### D-10: Tier promotion criteria

D-10.1. **Tier 2 → Tier 1 promotion** requires a per-µservice evidence pack at `microservices/<name>/IPs/IP-tier-promotion-2-to-1.md` containing:
  - (a) The data class change: which ADR-0099 data class the µservice now handles.
  - (b) The substrate role change: which key-derivation / audit-seal / token-signing operation the µservice now owns.
  - (c) The compliance pack trigger (if applicable): which compliance pack now requires Tier 1 placement.
  - (d) The Cedar fragment update: which Cedar predicates change with the tier promotion.
  - (e) The cell sizing impact: how the kata-pool grows for the µservice's worst-case footprint.
  - (f) The reviewer-agent verdict (multispectrum review v2.4.0).
  - (g) Council-architecture + council-security approval signatures.

D-10.2. **Tier 1 → Tier 0 promotion** requires evidence that the µservice executes tenant-customer code paths that existing Tier 1 placement does not adequately isolate. This is rare and is treated as exceptional. The evidence pack adds:
  - (h) The tenant-customer code-path source (which tenant artifact is now executed in-process).
  - (i) The sandboxing posture (Wasmtime / V8 / similar) and why it is insufficient without Kata.
  - (j) The tenant-customer-code-path's threat model.

D-10.3. **Tier 2 → Tier 3 demotion (to edge)** requires evidence of edge / packet-rate-bound / sub-millisecond hot path. The evidence pack adds:
  - (k) Latency budget measurement and required p99.
  - (l) Packet-rate budget measurement and required PPS.
  - (m) Edge-hardware availability per cell.
  - (n) Why Tier 2 is insufficient.

D-10.4. **Tier 1 → Tier 2 demotion** requires evidence that the µservice no longer handles substrate-touching responsibilities. This is rare. The evidence pack documents the responsibility transfer.

D-10.5. **Tier 0 → Tier 1 demotion** is FORBIDDEN without a superseding ADR retiring the tenant-customer code path. Once tenant-customer code is in scope, Kata-isolation is non-negotiable.

D-10.6. **Tier 0 → Tier 2 demotion** is FORBIDDEN under any circumstance other than a superseding ADR.

D-10.7. **Tier 3 → Tier 2 demotion** is permitted when the edge-hardware requirement is dropped or the hot path is removed.

D-10.8. Promotion / demotion is reviewed under multispectrum review v2.4.0 and approved by council-architecture + council-security.

### D-11: Cellular tier integration (ADR-0248 co-variance)

D-11.1. The pod runtime tier (this ADR) and the cellular tier (ADR-0248) co-vary by construction.

D-11.2. Mapping:
  - **Cellular Tier 0 (Foundation cell, e.g., identity / KMS / audit).** Hosts pod runtime Tier 0 (if running tenant-customer code, rare) + Tier 1 (substrate code, the default for Foundation cells).
  - **Cellular Tier 1 (Substrate cell, e.g., observability / cloud-data / cloud-iac).** Hosts pod runtime Tier 1 (substrate code).
  - **Cellular Tier 2 (Capability cell, e.g., intelligence / workflow-engine / ontology).** Hosts pod runtime Tier 1 (intelligence transport, ontology key paths) and Tier 0 (intelligence wasmtime / agent-runtime tenant capabilities).
  - **Cellular Tier 3 (Application cell, e.g., crm / drive / docs).** Hosts pod runtime Tier 2 (first-party application code).
  - **Cellular Tier 4 (Edge cell, e.g., api-gateway / Envoy / ztunnel).** Hosts pod runtime Tier 3 (edge code) + Tier 2 (catalog / control-plane code).

D-11.3. The cellular tier classifies blast-radius scope per cell; the pod runtime tier classifies isolation primitive per pod. The two axes do not contradict each other.

D-11.4. A cellular Tier 0 cell with pod runtime Tier 0 + Tier 1 pods is the highest-isolation deployment configuration; example: a sovereign HIPAA cell hosting tenant-customer code through workflow-studio.

D-11.5. A cellular Tier 4 cell with pod runtime Tier 3 pods is the lowest-overhead deployment configuration; example: a CDN edge cell hosting Envoy proxies.

D-11.6. Cross-tier traffic between cells follows ADR-0044 (inter-cell mesh tunnel) regardless of runtime tier.

D-11.7. The cellular tier classification is a separate decision surface owned by the cell-architecture ADR (ADR-0009 / ADR-0248); this ADR does not alter that surface.

### D-12: Incident-response classification

D-12.1. Incident-response severity floor by tier:
  - **Tier 0 incident.** Sev-1 minimum. Tenant-customer code path is involved; even apparently-low-impact incidents may indicate sandbox escape attempts. Page council-security on detection.
  - **Tier 1 incident.** Sev-1 minimum. Tenant data plane is at risk. Page council-security + ops-sre-reliability on detection.
  - **Tier 2 incident.** Sev-2 minimum (default first-party application incident severity).
  - **Tier 3 incident.** Sev-2 minimum (edge incident; performance-bounded but not security-bounded by default).

D-12.2. The severity floor is per-pod-runtime-tier; incident-response routing per ADR-0263 + ADR-0263-runbooks adds the runtime-tier dimension.

D-12.3. The runbook surface (per `microservices/<name>/runbooks/`) MUST include a tier-specific incident-response runbook for Tier 0 + Tier 1 µservices.

D-12.4. Tier 0 + Tier 1 incidents trigger an automatic audit-chain seal of the relevant evidence pack per ADR-0247 self-modification doctrine + ADR-0251 compliance pack certification.

D-12.5. Tier 2 + Tier 3 incidents follow standard observability + on-call rotation per ADR-0263.

D-12.6. Post-incident review (PIR / blameless postmortem) for Tier 0 + Tier 1 incidents is reviewed by council-architecture + council-security.

D-12.7. PIR for Tier 2 + Tier 3 incidents is reviewed by ops-sre-reliability.

D-12.8. Tier 0 + Tier 1 incidents that involve a tenant-customer-code-path sandbox escape are mandatory regulator notifications under EU AI Act / HIPAA / PCI / GDPR depending on the compliance pack mix.

## E. Enforcement-by-lanes

E.1 **`oya-check-pod-runtime-tier`** (new CI lane) — validates per-µservice manifest declaration, surface-evidence citations, Helm chart RuntimeClass binding, nodepool toleration declarations. REPORT-ONLY at landing; promoted to BLOCKER per §G sunset.

E.2 **Kyverno admission policy `enforce-pod-runtime-tier`** (new) — refuses pod admission when RuntimeClass / nodepool placement does not match the µservice's declared tier. REPORT-ONLY (`validationFailureAction: audit`) at landing; promoted to BLOCKER (`validationFailureAction: enforce`) per §G sunset.

E.3 **`oya-governance-runtime-class-allowlist`** (new) — refuses RuntimeClass declarations outside the three canonical names (`kata-cloud-hypervisor`, `runc`, `runc-edge`). REPORT-ONLY at landing; BLOCKER after sunset.

E.4 **`oya-governance-nodepool-binding`** (new) — refuses workloads that target a nodepool the µservice is not authorized to land on. REPORT-ONLY at landing; BLOCKER after sunset.

E.5 **`oya-governance-tier-promotion-evidence`** (new) — refuses Tier 2 → Tier 1 (and other) tier promotions without the D-10 evidence pack. REPORT-ONLY at landing; BLOCKER after sunset.

E.6 **`oya-check-pod-runtime-tier-helm-binding`** (new sub-lane of E.1) — validates per-µservice Helm chart `runtimeClassName` field matches the declared tier. REPORT-ONLY at landing; BLOCKER after sunset.

E.7 **`oya-governance-substance-bar`** (existing) — applies the substance bar to per-µservice IP-tier-promotion-*.md authoring.

E.8 **Multispectrum review v2.4.0** (existing) — reviews each Tier 0 / Tier 1 declaration + each tier promotion under the eleven-facet review surface per ADR-0322.

## F. Alternatives Rejected

F.1 **Kata everywhere.** Rejected because it imposes ~30-40 percent pod density loss and 200-500 ms cold-start latency on first-party code that is already namespace-isolated + mTLS-mediated + Cedar-policed + supply-chain-signed. The marginal security benefit on first-party code is negligible relative to the capacity tax.

F.2 **runc everywhere.** Rejected because tenant-customer code paths (Wasmtime sandbox, marketplace plugins, workflow-studio user workflows, agent-runtime tenant capabilities, developer-sdk uploaded modules) require VM-isolation beyond Wasmtime's sandbox guarantees. Substrate µservices touching tenant data plane (cloud-iam, cloud-kms, audit-chain, payments, messenger MLS) require lateralization-resistance against guest-kernel-escape attacks. runc-everywhere imposes unacceptable security posture on Tier 0 + Tier 1 surfaces.

F.3 **gVisor-only.** Rejected because gVisor's user-space syscall reimplementation imposes its own runtime overhead (latency floor on syscall-bound workloads ~2-5x relative to runc) AND does not provide the same VM-isolation guarantee as Kata + Cloud Hypervisor (gVisor's threat model is "compatibility shim with sandboxing," not "guest-kernel-escape resistance"). gVisor remains an option for specific edge cases under a future amendment, but the canonical Tier 0 / Tier 1 runtime is Kata + Cloud Hypervisor.

F.4 **Per-tenant runtime selection.** Rejected because the runtime tier is a µservice property, not a tenant property. tenant_class travels as a principal claim per ADR-0330; runtime tier travels as a manifest declaration. Conflating the two axes would couple authorization decisions to placement decisions, which is an architectural anti-pattern per ADR-0183 (policy-engine separation).

F.5 **Single-nodepool with per-pod runtime selection.** Rejected because Kata pods require Cloud Hypervisor installation on the node; a single-nodepool topology would require Cloud Hypervisor on every node and would impose the Kata installation cost across all nodes. The two-nodepool topology (or three-nodepool with edge) is optimal.

F.6 **Three-tier model (collapse Tier 2 + Tier 3).** Rejected because edge / packet-rate-bound workloads require dedicated nodepool hardware (SR-IOV, hugepages, CPU pinning) that is not available on general-purpose nodes. A separate Tier 3 captures this hardware constraint cleanly.

F.7 **Five-tier model (split Tier 2 into 2A general-purpose and 2B batch).** Rejected because batch workloads do not require a different runtime; they run under runc on runc-pool the same as steady-state Tier 2 workloads. The marginal benefit of a fifth tier is not justified by the additional admission policy complexity.

F.8 **Tenant-class-bound runtime selection.** Rejected because demo_trial and paid tenants execute the same code through the same µservice; the µservice's runtime tier is the right scope, not the tenant's class. Compliance packs (per ADR-0251) gate on tenant_class = paid, but the runtime tier is unaffected.

F.9 **No CI lane (manual review only).** Rejected because per-µservice manifest declarations across 77+ µservices cannot be reliably manually reviewed; the CI lane is the canonical enforcement mechanism per ADR-0322 + ADR-0328 substance-bar discipline.

F.10 **Cedar-based pod runtime tier decision.** Rejected per ADR-0183 (policy-engine separation): Cedar is application-layer authorization; runtime tier is admission-time. Kyverno is the canonical admission gate.

## G. Multispectrum review v2.4.0

Per ADR-0322 multispectrum review v2.4.0 doctrine, this ADR is reviewed across F1-F9 + M1 + M2 + A1-A7 facets. Each facet's verdict is recorded as evidence under `evidence/multispectrum-review/ADR-0338/<facet>.md` at landing time.

- **F1 Correctness.** The four-tier model is internally consistent; tier definitions are exhaustive across the workload surface; mapping to RuntimeClass + nodepool is deterministic; admission policy refuses misconfiguration. PASS.
- **F2 Readability.** The ADR has explicit section structure (A..G), numbered decision clauses (B2.001..B2.053), and per-tier mapping tables. The reader can locate any rule in O(log N). PASS.
- **F3 Architecture.** The four-tier model is an amendment to ADR-0254 (preserved), respects ADR-0248 cellular tier numbering (co-varies), respects ADR-0183 policy-engine separation (Kyverno admission, not Cedar), and respects ADR-0244 tenant scoping (runtime tier is µservice-scoped, not tenant-scoped). PASS.
- **F4 Security.** Tier 0 + Tier 1 placement under Kata + Cloud Hypervisor preserves VM-isolation against guest-kernel-escape and shared-kernel side channels. Tier 2 + Tier 3 placement under runc with namespace-isolation + mTLS-mediation + Cedar-policing preserves first-party security posture. PASS.
- **F5 Performance.** Tier 2 + Tier 3 placement under runc restores 30-40 percent pod density vs Kata baseline + 200-500 ms cold-start latency vs Kata baseline. Tier 3 edge nodepool restores sub-millisecond p99 tail latency. PASS.
- **F6 Maintainability.** Single manifest field declaration across 77+ µservices; deterministic CI lane validation; quarterly review surfaces drift. PASS.
- **F7 Observability.** Per-pod `pod_runtime_tier` label additive per ADR-0263; per-cell capacity dashboard segments by tier; incident-response severity floor classifies by tier. PASS.
- **F8 Compliance.** Tier 0 + Tier 1 placement composes with compliance packs per ADR-0251 (HIPAA, GDPR-strict, SOC2, PCI, CSAP, EU-AI-Act Annex III); PHI / cardholder-data / sovereign-data workloads land on Kata-isolated pods. PASS.
- **F9 Cost.** kata-pool sizing premium (~30-40 percent density overhead) bounded to Tier 0 + Tier 1 workloads (~12 µservices of ~77); cell-level Kata premium ~5 percent rather than 30-40 percent. FinOps tagging per ADR-0174 surfaces the premium. PASS.
- **M1 Authority chain.** ADR-0254 (parent invariant) amended; ADR-0248 (cellular tier numbering) co-varied; ADR-0183 (policy-engine separation) respected; ADR-0322 (substance-bar) authored against. PASS.
- **M2 Substance bar.** Every decision clause is bespoke to the four-tier model; no template stamping; the ADR is authored under ADR-0322 + ADR-0328 substance-bar discipline. PASS.
- **A1 Naming.** `pod_runtime_tier` follows the manifest field naming convention; `kata-cloud-hypervisor` / `runc` / `runc-edge` RuntimeClass names follow Kubernetes convention; `kata-pool` / `runc-pool` / `runc-edge-pool` nodepool names follow oyatie convention. PASS.
- **A2 Documentation.** This ADR + the per-µservice manifest declaration + the Kyverno policy + the CI lane source-of-truth + the canonical-primitives addition satisfy ADR-0063 doc coverage. PASS.
- **A3 Structure.** Twelve enforcement surfaces (D-1..D-12) follow the ADR-0331 + ADR-0336 detailed-mechanics pattern. PASS.
- **A4 Architecture.** Amendment-of-ADR-0254 pattern follows the ADR-0335 retirement / ADR-0336 substrate-swap precedent. PASS.
- **A5 Dependency.** ADR-0211 in-house tech stack: Kata (Apache 2.0) + Cloud Hypervisor (Apache 2.0) + Kyverno (Apache 2.0) all Class C OSS-allowed. PASS.
- **A6 Schema.** `pod_runtime_tier` integer-enum field in `/specs/microservices/manifest-schema.json`; ConfigMap-based admission cache schema specified. PASS.
- **A7 Algorithm.** Kyverno admission resolution algorithm specified in D-5.6; capacity sizing formula specified in D-7.2..D-7.4. PASS.

Aggregate verdict: PASS (all facets). The multispectrum-review evidence pack is filed at landing time.

## H. Enforcement (Kyverno + CI lane) + Sunset

H.1 **Enforcement vectors.**
  - CI lane `oya-check-pod-runtime-tier` (E.1) at PR-time on every manifest change.
  - Kyverno ClusterPolicy `enforce-pod-runtime-tier` (E.2) at pod admission time in every cell.
  - Three subsidiary lanes (E.3 / E.4 / E.5) for RuntimeClass allowlist, nodepool binding, and tier promotion evidence.

H.2 **Sunset schedule.**
  - Day 0 = ADR Acceptance.
  - Day 0..30 = REPORT-ONLY soak. CI lane and Kyverno policy run in audit mode. Findings produced per-PR + per-pod-admission but not blocking.
  - Day 30 = if corpus-wide manifest declarations have landed, all lanes + Kyverno policy promote to BLOCKER. Else sunset extends until manifest declarations are complete.
  - Day 60 (or whenever the manifest sub-wave lands) = BLOCKER promotion regardless of remaining soak.

H.3 **Sunset criteria.**
  - Corpus-wide manifest declarations have landed for all 77+ active µservices.
  - The new sub-wave `15S-Pod-Runtime-Tier-declaration` has completed per ADR-0328 batch discipline.
  - Per-cell nodepool topology (kata-pool + runc-pool + optional runc-edge-pool) has been provisioned via OpenTofu.
  - Three RuntimeClasses (`kata-cloud-hypervisor`, `runc`, `runc-edge`) have been applied in every cell.
  - The Kyverno policy has soaked at REPORT-ONLY for at least 30 days without unexpected false-positive denials.

H.4 **Post-sunset behavior.**
  - New µservices that omit `pod_runtime_tier` declaration: PR blocked by `oya-check-pod-runtime-tier`.
  - Pods missing `runtimeClassName` for non-Tier-2 µservices: admission denied.
  - Pods placed on the wrong nodepool: admission denied.
  - Tier 2 → Tier 1 promotion without D-10 evidence pack: PR blocked by `oya-governance-tier-promotion-evidence`.

H.5 **Quarterly tier review** runs on rolling 90-day cadence per D-8. Each review produces an evidence pack at `.omc/state/pod-runtime-tier-review-<date>.md`.

H.6 **No waiver mechanism.** Tier 0 / Tier 1 / Tier 3 declarations require evidence; Tier 2 is the no-evidence default. There is no "Tier 2 with informal Kata override" path; the runtime tier is the runtime tier.

## I. Cross-references

I.1 **Parent invariants.**
  - ADR-0254 (deployment-model-spectrum; amended per D-2.10).
  - ADR-0248 (Amazon-shape cellular architecture; tier numbering convention reused).
  - ADR-0183 (policy-engine separation; Kyverno admission canonical gate).
  - ADR-0009 (cell architecture; per-cell nodepool topology).
  - ADR-0121 (on-prem K8s stack; self-managed Kata installation pattern).
  - ADR-0128 (hyperscaler architecture invariants).

I.2 **Related substrate ADRs.**
  - ADR-0145 (inter-µservice communication reform; transport unchanged).
  - ADR-0150 (Cedar policy engine; authorization unchanged).
  - ADR-0211 (in-house tech stack preference; Kata + Cloud Hypervisor + Kyverno all Class C OSS).
  - ADR-0212 (buildability doctrine; manifest field added to manifest-schema).
  - ADR-0215 (multi-context platform; runtime tier applies across all six deployment contexts).
  - ADR-0240 (sovereign-cloud per regional pack; runtime tier preserved across sovereign cells).
  - ADR-0243 (Cedar as universal gate; runtime tier is admission, not authorization).
  - ADR-0244 (tenant as universal scoping primitive; runtime tier is µservice-scoped).
  - ADR-0247 (self-modification doctrine; dev-tools-cell-N hosts Tier 1).
  - ADR-0249 (multi-category marketplace; marketplace plugin executor is Tier 0).
  - ADR-0251 (compliance pack cell certification; packs compose with runtime tier).
  - ADR-0252 (time coordination distributed consistency; HLC + TrueTime unaffected by runtime tier).
  - ADR-0253 (network topology edge service mesh; ztunnel + Envoy edge are Tier 3).
  - ADR-0255 (intelligence two-layer AI substrate; intelligence transport is Tier 1; intelligence wasmtime sandbox is Tier 0).
  - ADR-0263 (observability emission contract; pod_runtime_tier label additive).

I.3 **Realignment ADRs.**
  - ADR-0322 (substance-bar doctrine; this ADR authored under it).
  - ADR-0324 (anti-script authoring doctrine; this ADR's twelve enforcement surfaces are bespoke).
  - ADR-0328 (substance-bar canonical sequence; this ADR sequences the follow-on sub-wave).
  - ADR-0329 (tier system retirement; Bronze/Silver/Gold/Platinum retired; this ADR does not reintroduce the retired ladder semantics).
  - ADR-0330 (tenant_class replacement model; tenant_class travels as principal claim; runtime tier travels as manifest declaration; orthogonal).
  - ADR-0331 (cross-µservice tenant_class adoption template; co-existing manifest declaration template).
  - ADR-0335 (foundry retirement; foundry workflow library at Tier 1 in dev-tools-cell-N).
  - ADR-0336 (Valkey-not-Redis substrate; Valkey clusters are Tier 1 placement).
  - ADR-0337 (Iceberg canonical OLAP write path; data-warehouse Iceberg writers at Tier 2 unless touching tenant key material).

I.4 **Related specs.**
  - `/specs/master-plan-sequencing.json` (this ADR added to realignment_wave_sequence + sub-wave declaration).
  - `/specs/microservices/manifest-schema.json` (pod_runtime_tier field added).
  - `/specs/platform-architecture.json` (four-tier classification recorded).
  - `/specs/deployment-models.json` (per-cell nodepool topology recorded).
  - `/specs/microservices/cloud-iac.json` (OpenTofu nodepool modules referenced; ADR-0339 candidate handoff).
  - `/specs/microservices/cell.json` (per-cell nodepool topology updated).

I.5 **Related memory.**
  - `feedback_idea_refine_decisions_2026_05_21` — origin memory; Decision 2 is this ADR.
  - `feedback_amazon_shape_cellular_architecture` — ADR-0248 source; tier numbering convention.
  - `feedback_kubernetes_everywhere_pods_cloud_hypervisor` — ADR-0254 source; amended here.
  - `feedback_no_silent_regression` — Linus-style public-contract change; ADR + version bump + sunset enforced.
  - `feedback_quality_performance_scalability_bar` — hyperscaler-grade rigor application.
  - `feedback_clean_architecture_requirements` — admission vs authorization separation per ADR-0183.
  - `feedback_build_ahead_of_certification` — compliance-pack-aware runtime tier mapping.
  - `feedback_compliance_pack_primitive` — ADR-0251 cross-binding.
  - `feedback_tenant_scoping_primitive` — runtime tier is µservice-scoped, tenant_class is principal-claim-scoped.
  - `feedback_cedar_universal_gate` — Cedar evaluates authorization; runtime tier is admission.
  - `feedback_bominal_inheritance_precedence` — oyatie session override; Bominal corpus follows under its own plan.
  - `feedback_microservice_ownership_coherence_2026_05_20` — single-µservice-ownership applies to runtime tier declaration.
  - `feedback_rust_strict_only_no_python_2026_05_20` — CI lane implementation is Rust-strict.
  - `feedback_zero_handroll_opentofu_only_2026_05_20` — nodepool provisioning via OpenTofu modules.
  - `feedback_oci_always_free_maximization_2026_05_20` — demo_trial Tier 0 / Tier 1 placement on OCI Always Free Ampere A1 ARM nodepool subject to Kata-on-ARM support.

I.6 **Companion docs.**
  - `docs/standards/hyperscaler-best-practices.md` — Kata + Cloud Hypervisor + Firecracker-class isolation precedent.
  - `docs/standards/dependency-policy.md` — Kata + Cloud Hypervisor + Kyverno license classifications.
  - `docs/GLOSSARY.md` — Pod runtime tier terms added.
  - `docs/machine-readable/glossary.json` — Pod runtime tier machine-readable entries.
  - `tools/hooks/_canonical-primitives.md` — Pod Runtime Tier section added.

I.7 **Inbound citations.**
  - `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_idea_refine_decisions_2026_05_21.md` Decision 2.

I.8 **Successor work (out of scope for this ADR).**
  - Per-µservice manifest declaration sub-wave (`15S-Pod-Runtime-Tier-declaration`).
  - Per-cell OpenTofu nodepool modules (ADR-0339 candidate / shared IaC module library).
  - Per-µservice Helm chart RuntimeClass binding updates.
  - Kyverno policy rollout per cell.
  - Three RuntimeClass declarations per cell.

---

<!--
adr: ADR-0338
status: Proposed
date: 2026-05-21
parent_invariant: ADR-0254
amends: ADR-0254 (carve-out four-tier runtime classification)
co_varies_with: ADR-0248 (cellular tier numbering)
admission_gate: Kyverno enforce-pod-runtime-tier
ci_lane: oya-check-pod-runtime-tier (REPORT-ONLY at landing; BLOCKER at sunset)
default_tier_new_microservices: Tier 2
tier_floor_t0_surfaces: wasmtime-sandbox-host, workflow-studio-user-workflows, marketplace-plugin-executors, agent-runtime-tenant-capabilities, developer-sdk-uploaded-modules
tier_floor_t1_surfaces: cloud-iam, cloud-kms, cloud-secrets, audit-chain, messenger-mls-keys, payments, intelligence-transport
tier_floor_t3_surfaces: api-gateway-data-plane, envoy-edge, ztunnel, cdn-edge-cache
sub_wave_followon: 15S-Pod-Runtime-Tier-declaration
commits: none
-->
