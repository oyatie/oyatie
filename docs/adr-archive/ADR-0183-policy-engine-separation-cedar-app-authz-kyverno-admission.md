---
id: ADR-0183
status: Superseded
deciders: council-architecture, ops-security, council-privacy, axis-cloud-k8s
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0379]
related: [ADR-0121, ADR-0145, ADR-0146, ADR-0148, ADR-0182, ADR-0184, ADR-0185, ADR-0186]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0183 — Kubernetes policy engine separation: Cedar (app authz) vs Kyverno (admission)

## Status

Accepted (2026-05-18). Mandates a clean separation between **application-layer L7 authorization** (which principal may take which action on which resource) and **Kubernetes resource admission control** (which workloads, images, labels are permitted to land in the cluster). Each concern owned by exactly one engine. Zero feature overlap.

## Context

ADR-0145 invariants require per-call authorization on every state-changing inter-µservice flow. ADR-0148 wires Cedar PDP at the Istio Ambient waypoint's `ext_authz` filter. Separately, CIS Kubernetes Benchmark v1.10 and Pod Security Standards require admission-time enforcement on workloads landing in the cluster (image signing, restricted Pod Security profile, label/annotation discipline).

The shape of these two concerns is fundamentally different:

- **Application L7 authorization** is **principal × action × resource × context** evaluation per-request. Cedar v4.9.1 is purpose-built for this.
- **Kubernetes admission** is **resource-shape × cluster-policy** evaluation per-K8s-API-server-write. Kyverno 1.18.0 is purpose-built for this.

Forcing one engine to do both produces poor ergonomics in the off-target domain:

- Using Cedar for K8s admission: Cedar's entity/action/resource model is principal-centric, mismatching K8s admission's resource-shape-centric model.
- Using Kyverno (or OPA Gatekeeper) for application authz: Rego/JMESPath against arbitrary application JSON is verbose and slow vs Cedar's purpose-built policy language.

## Decision

Each engine owns exactly one concern:

### Cedar 4.9.1 LTS — application-layer L7 authorization

Cedar is the canonical engine for:

- Per-call principal × action × resource authorization in the mesh waypoint (`ext_authz` hook per ADR-0148).
- Default-deny + defence-in-depth `forbid` rules per ADR-0145.
- Regulatory pack overlays (per `docs/standards/regulatory-pack-authzpolicy-overlays.md`) compile to AuthorizationPolicy CRs but the *decision* is Cedar's.
- Per-µservice `policy/tenant-scope.cedar` is the source-of-truth.
- The Cedar PDP runs as a Deployment in the `governance` µservice namespace; the waypoint calls it over gRPC via `ext_authz`.
- Cedar verdict shapes: `ALLOW` / `DENY` / `DENY-WITH-RESPONSE-SHAPE` (regulatory packs).

Cedar never participates in Kubernetes admission control. K8s resource shape is out of Cedar's idiomatic domain.

### Kyverno 1.18.0 — Kubernetes admission control

Kyverno is the canonical engine for:

- **Pod Security Standards enforcement** at the **restricted** profile across all namespaces (per CIS K8s Benchmark v1.10).
- **Image signing verification** — every container image must carry a valid Sigstore Cosign v3 signature (per ADR-0146 baseline + supply-chain ADRs).
- **Label / annotation discipline** — every workload carries canonical oyatie labels (`oyatie/microservice`, `oyatie/bounded_context`, `oyatie/tier`, `oyatie/plane`) per ADR-0131.
- **Mutating webhooks for sidecar / waypoint injection** — Kyverno mutates manifests to inject Istio Ambient labels and ensure ServiceAccount → SPIFFE-ID binding.
- **Reject non-conformant resources** at admission time before they enter etcd.

Kyverno never evaluates application L7 authorization. Application principal × action × resource is out of Kyverno's idiomatic domain.

### Bridge: how Cedar decisions and Kyverno decisions compose

The two engines are decoupled at runtime but composed at the Cedar source-of-truth tier:

- A Kyverno policy may require that every Deployment carries an `oyatie/cedar-policy-fragment` annotation pointing to the µservice's `policy/tenant-scope.cedar` file. Kyverno blocks the Deployment if missing.
- Cedar evaluates the application traffic; Kyverno governs the K8s resources that produce that traffic.
- The governance µservice's policy compiler emits both: `CiliumNetworkPolicy` + `AuthorizationPolicy` from Cedar fragments (mesh enforcement); Kyverno `ClusterPolicy` CRs from oyatie cluster-policy templates (admission enforcement).

## Alternatives considered

### (a) OPA Gatekeeper for both admission AND application authz — REJECTED

- **Pros:** single engine; Rego is general-purpose.
- **Cons:** Rego is verbose for K8s-resource-shape rules (Kyverno's YAML-native rules are roughly 5× shorter and more readable); Rego is not purpose-built for principal × action × resource (Cedar is); OPA Gatekeeper performance at K8s admission scale lags Kyverno per public benchmarks; Cedar's analyzability guarantees (provably consistent set-difference proofs) are unmatched by Rego.
- **Rejected**: weaker on both concerns vs the chosen pair.

### (b) Cedar for both admission AND application authz — REJECTED

- **Pros:** single engine; Cedar is the better authz language.
- **Cons:** Cedar's entity model is principal-centric; K8s admission is resource-shape-centric. Force-fitting Cedar to admission produces awkward principal:Cluster, action:CreatePod, resource:Pod policies that are harder to author and audit than Kyverno's "match Pod, validate `spec.securityContext.runAsNonRoot == true`" idiom. Cedar PDP is also not in the K8s API-server admission control loop natively.
- **Rejected**: Cedar mismatches K8s admission's idiomatic shape.

### (c) Kyverno for both admission AND application authz — REJECTED

- **Pros:** single engine; Kyverno is YAML-native.
- **Cons:** Kyverno is a K8s API-server admission webhook; it does not natively integrate with Envoy `ext_authz` at the mesh waypoint; forcing application L7 evaluation through Kyverno requires routing every mesh request through K8s API, which is operationally absurd.
- **Rejected**: Kyverno's K8s admission shape is incompatible with mesh `ext_authz` performance + latency requirements.

### (d) No admission control — REJECTED

- **Pros:** zero admission complexity.
- **Cons:** CIS K8s Benchmark v1.10 requires admission-time Pod Security profile enforcement; SLSA + supply-chain ADRs require image-signature admission verification. Without admission control, the cluster cannot meet baseline compliance.
- **Rejected**: required by CIS K8s Benchmark + supply-chain doctrine.

### (e) **CHOSEN: Cedar for app authz + Kyverno for K8s admission, zero overlap**

- **Pros:**
  - Each engine purpose-built for its domain.
  - Cedar's analyzable policy language gives provably-consistent set-difference proofs at audit time.
  - Kyverno's YAML-native rules are concise + auditable for K8s admission.
  - Cedar PDP integrates natively with Envoy `ext_authz` (mesh waypoint).
  - Kyverno integrates natively with K8s ValidatingAdmissionPolicy + MutatingAdmissionWebhook.
- **Cons:** two engines to learn. Mitigation: each engine has a different audience (Cedar = app-tier µservice authors + governance axis; Kyverno = ops-sre-reliability + K8s admins). The audience separation matches the engine separation.
- **Accepted**.

## Consequences

### Positive

1. **Each engine purpose-built for its domain.** Best authoring ergonomics on both sides.
2. **Cedar's analyzability** — Cedar's formal logic backing gives provably-consistent set-difference proofs ("does this new policy strictly contain the old one?"). Indispensable for regulated-pack audits.
3. **Kyverno's YAML-native** — K8s admins author K8s admission policies in K8s YAML; no learning a new policy language.
4. **Decoupled runtime paths** — Cedar PDP outage does not block K8s admission; Kyverno outage does not block mesh L7 authorization.
5. **Decoupled rollout** — Cedar policy changes don't touch Kyverno ClusterPolicies and vice versa.

### Negative

1. **Two engines to operate.** Mitigation: each engine has a different operator audience and a different release cadence; ops-sre-reliability adds both to the runbook but the audiences are distinct.
2. **Cross-engine policy intent must be documented.** When a Kyverno ClusterPolicy backs a Cedar invariant (e.g., "every Deployment must carry `oyatie/cedar-policy-fragment` annotation"), the intent lives in the standard at `docs/standards/cedar-policy-discipline.md`.

### Operational

1. Cedar PDP deploys as a Deployment in the `governance` µservice namespace; mesh waypoints call it via Envoy `ext_authz` per ADR-0148.
2. Kyverno deploys as a cluster-level controller in the `kyverno` namespace; ClusterPolicy CRs live at `microservices/governance/iac/helm/kyverno-policies/templates/`.
3. The 8 canonical Kyverno ClusterPolicies oyatie ships:
   - `pod-security-restricted.yaml` — enforce PSS restricted profile across all namespaces.
   - `image-signature-verification.yaml` — every image must carry a valid Cosign v3 keyless signature with SLSA L3+ provenance.
   - `image-registry-allowlist.yaml` — only oyatie's internal registry + gcr.io/distroless are allowed.
   - `workload-labels-required.yaml` — every workload carries `oyatie/microservice`, `oyatie/bounded_context`, `oyatie/tier`, `oyatie/plane` per ADR-0131.
   - `cedar-policy-fragment-annotation.yaml` — every Deployment carries `oyatie/cedar-policy-fragment` pointing at the µservice's tenant-scope.cedar.
   - `istio-ambient-label-mutation.yaml` — mutating webhook to inject `istio.io/dataplane-mode=ambient` on namespace create.
   - `serviceaccount-spiffe-binding.yaml` — every ServiceAccount must have a corresponding cell-µservice SPIFFE-ID binding.
   - `runtimeclass-tier-enforcement.yaml` — workloads in sovereign-tier namespaces must declare `runtimeClassName: kata-clh-sev-snp` per ADR-0147.

## In-house roadmap

Per user directive 2026-05-18 (in-house-stack policy — "wherever possible, we should support in-house tech stack. Like how AWS, Google, Microsoft, Oracle does."), this ADR's components classify as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **Cedar 4.9.1** | KEEP (open standard; Linux Foundation; provably analyzable) | Cedar's formal-logic backing + provably-consistent set-difference proofs are unmatched. Used by Amazon Verified Permissions, Confluent, Pinterest. Industry-standard authz language. | None planned. The governance µservice's **policy compiler** (Oya-native) emits CNP + AuthorizationPolicy from Cedar fragments — this IS oyatie's in-house value layered on the standard engine. |
| **Kyverno 1.18.0** | KEEP (CNCF Incubating; widest Kubernetes-admission deployment) | Kyverno's YAML-native rules are THE K8s admission standard. Backed by Nirmata + community. | None planned. The 8 canonical oyatie ClusterPolicies (PSS-restricted, Cosign verification, workload-labels-required, etc.) ARE oyatie's in-house value layered on the standard engine. |
| **OPA Gatekeeper** (rejected) | KEEP-but-rejected | Open standard; same rationale; not chosen here per the alternatives analysis. Would also be KEEP if chosen. | n/a |
| **Sigstore Cosign v3** (referenced by Kyverno image-signature verification) | KEEP (Linux Foundation; OpenSSF) | THE standard for keyless signature verification. | None planned. |

The IS-the-standard pattern repeats: oyatie's in-house engineering effort goes into the **policy compiler + ClusterPolicy catalog** (both Oya-native), running on KEEP-classified standard engines. This is exactly how AWS, Google, Microsoft, Oracle build their stack: standard engines, in-house policy assets.

Why no in-house authz engine: Cedar's formal analyzability is a deep research investment (years of AWS + academic collaboration). Building an Oya-native authz engine would reimplement Cedar's analyzability properties with less rigor. The engineering cost would not produce a better outcome.

## Rollback

Each engine rolls back independently:

- **Cedar rollback:** revert the Cedar policy fragment; PDP picks up the prior fragment on next reconcile. Mesh waypoints continue with the previous policy.
- **Kyverno rollback:** revert the ClusterPolicy CR; admission reverts to the prior policy on next K8s reconcile.

`git revert` followed by Flux reconciliation. No persisted state is lost.

## References

- Cedar — https://www.cedarpolicy.com/ ; current v4.9.1 (Feb 2026 release).
- Cedar GitHub — https://github.com/cedar-policy/cedar
- Kyverno — https://kyverno.io/ ; current v1.18.0 (April 2026 release).
- OPA Gatekeeper (rejected) — https://open-policy-agent.github.io/gatekeeper/
- CIS Kubernetes Benchmark v1.10 — admission-time enforcement requirements.
- Sigstore Cosign v3 — keyless signing reference for Kyverno image verification.
- ADR-0121 — on-prem K8s stack.
- ADR-0145 — inter-microservice communication reform.
- ADR-0146 — distroless non-root container base image.
- ADR-0148 — service-mesh canonical (Cilium L3/L4 + Istio Ambient L7 layered).
- ADR-0182 — API gateway (north-south) vs service mesh (east-west) separation.
- ADR-0184 — storage tier layering.
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098 (LTS pin policy).
