---
id: ADR-0379
status: Superseded
planning_impact: false
deciders: founder, council-architecture, ops-security, axis-cloud-k8s
date: 2026-05-27
owner: council-architecture
supersedes: [ADR-0183]
superseded_by: [ADR-0519]
related: [ADR-0183, ADR-0148, ADR-0181, ADR-0023, ADR-0039, ADR-0378]
related_specs: [/specs/platform-architecture.json, /specs/cloud-production-quality-kits-target.json]
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: [/specs/platform-architecture.json]
purpose: >
  Make Kubewarden the DEFAULT Kubernetes admission/policy substrate, with Kyverno
  retained as a first-class adapter — superseding ADR-0183's choice of Kyverno as the
  default admission engine. The Cedar-vs-admission SEPARATION principle of ADR-0183
  (Cedar owns application-layer L7 authorization; a distinct engine owns Kubernetes
  resource admission) is carried forward UNCHANGED; only the admission engine changes.
  Rationale: the platform specs already designate Kubewarden as the default ("Kubewarden
  is the default Kubernetes admission/policy substrate; Kyverno remains a first-class
  adapter"), and Kubewarden's WASM-module policy model aligns with Oyatie's WASM-native
  substrate (ADR-0023 Wasmtime) and Rust-everywhere stack — policies authored in Rust
  and compiled to WASM. ADR-0183's alternatives analysis never evaluated Kubewarden.
---

# ADR-0379 — Kubewarden as the default Kubernetes admission/policy substrate (supersedes ADR-0183)

## Status
**SUPERSEDED by ADR-0519 (2026-05-31)** — layered admission: in-tree VAP+CEL+MAP becomes the
default substrate; Kubewarden is demoted to an optional Rust/WASM escalation; the
signature-verification residue moves to a bespoke-Rust cosign-verify webhook. The
Cedar-vs-admission separation below is carried forward unchanged by ADR-0519.

Originally Accepted (2026-05-27). Superseded ADR-0183 (Cedar vs Kyverno policy-engine separation).
The two-engine SEPARATION it established is retained; the admission engine was changed
from Kyverno to Kubewarden. Cedar remains the universal application-layer policy engine.

## Context
ADR-0183 mandated a clean separation — Cedar for application-layer L7 authorization
(principal × action × resource, wired at the Istio Ambient waypoint `ext_authz`) and a
distinct engine for Kubernetes resource admission (image signing, Pod Security Standards,
label/annotation discipline). That separation is correct and stands.

ADR-0183 chose **Kyverno** as the admission engine and weighed only OPA Gatekeeper,
Cedar-for-both, and Kyverno-for-both — it **never evaluated Kubewarden**. Since then the
platform specs (`platform-architecture.json`, `cloud-production-quality-kits-target.json`,
`masterplan.json`) converged on **Kubewarden as the default** admission/policy substrate,
with Kyverno demoted to a first-class adapter; image-signing admission (ADR-0181) is
spec'd via Kubewarden. The in-repo IaC (`infra/gitops`, `infra/kyverno`) still deployed
Kyverno, leaving an ADR↔spec drift this ADR resolves.

## Decision
1. **Kubewarden is the default Kubernetes admission/policy substrate.** Admission
   policies (image signature verification, PSS restricted-by-default, label/annotation
   discipline, runtime-tier enforcement) are authored as **WASM policy modules** —
   written in Rust and compiled to WASM where practical — and enforced by the Kubewarden
   policy server. This aligns admission policy with the WASM-native server-side sandbox
   doctrine (ADR-0023) and the Rust-everywhere stack.
2. **Kyverno is retained as a first-class adapter**, not the default. The existing
   Kyverno policies (`infra/kyverno/`) remain valid for environments/packs that select
   the Kyverno adapter; they are not deployed in the base platform overlay.
3. **Cedar is unchanged** — it remains the universal application-layer authorization
   engine (ADR-0007/0148/0183 separation). Zero overlap between Cedar (app authz) and
   Kubewarden (cluster admission) is preserved.
4. The app-of-apps (`infra/gitops/values.yaml`) deploys `kubewarden-crds` →
   `kubewarden-controller` → `kubewarden-defaults`; large CRDs apply via
   `ServerSideApply=true`.

## Rejected alternatives
- **Keep Kyverno as default (ADR-0183's choice)** — rejected: diverges from the specs;
  YAML/JMESPath policy DSL does not fit the WASM/Rust substrate, so cross-cutting policy
  logic cannot be shared with the rest of the stack. Kyverno stays as an adapter, not
  removed.
- **Cedar for admission** — rejected (re-affirming ADR-0183): Cedar's principal-centric
  model mismatches admission's resource-shape-centric model.
- **Run both engines as co-defaults** — rejected: two admission webhooks on every API
  write is redundant cost and a potential conflict surface; one default + on-demand
  adapter is cleaner.

## Consequences
- Positive: spec↔IaC↔ADR alignment; admission policy authored in the same Rust→WASM
  toolchain as the rest of the platform; sandboxed, distributable, signable policy modules.
- Negative/cost: policy authors learn Kubewarden's model; existing Kyverno ClusterPolicies
  (e.g. `require-signed-images.yaml`) must be re-expressed as Kubewarden policies (or run
  via the Kyverno adapter during migration). Tracked as follow-on work.
- Neutral: the ADR-0183 separation principle and Cedar's role are unchanged.

## Verification
The app-of-apps deploys the Kubewarden charts and no Kyverno in the base overlay; the
ADR index lists ADR-0379 Accepted and ADR-0183 Superseded-by ADR-0379; image-signing
admission is enforced by a Kubewarden policy (follow-on).

## References
ADR-0183 (superseded — Cedar vs Kyverno separation), ADR-0148 (Cilium + Istio Ambient,
Cedar at ext_authz), ADR-0181 (image promotion / signed-image admission), ADR-0023
(Wasmtime server-side WASM sandbox), ADR-0039 (cosign/SBOM supply chain), ADR-0378
(canonical Talos substrate). Specs: platform-architecture.json,
cloud-production-quality-kits-target.json. External: Kubewarden
(https://www.kubewarden.io/), Kyverno (https://kyverno.io/).
