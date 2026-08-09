---
doc_status: published
id: ADR-0710
title: "Kubernetes admission substrate is the API server: VAP/CEL + PSA, no policy webhook"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-08
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701]
amended_by: []
depends_on: []
related: [ADR-0183, ADR-0338, ADR-0379, ADR-0702]
milestone: W0
deliverables:
  - id: ADR-0710-D1
    description: "Replace the admission-substrate clauses of ADR-0183 / ADR-0379 / ADR-0338 with in-process evaluation: ValidatingAdmissionPolicy + Pod Security Admission, and no default policy webhook."
    exit_criteria: "ADR-0338's runtime-tier enforcement is expressed as a ValidatingAdmissionPolicy with a paramKind tier map, and the removal gate INVENTORIES EVERY CANONICAL BASE OVERLAY rather than one values file: no ValidatingWebhookConfiguration and no engine-native policy resource is required by any of them. Instances known at authoring time: kubewarden-crds / kubewarden-controller / kubewarden-defaults in infra/gitops/values.yaml, and the enforcing Kyverno ClusterPolicy cloud-k8s-require-restricted-runtime (validationFailureAction Enforce) applied by k8s/iac/kustomize/base/kustomization.yaml, whose three rules — RuntimeDefault seccomp, runAsNonRoot, explicit automountServiceAccountToken — migrate to Pod Security Admission plus VAP and are deleted, not left to a webhook the platform claims to have removed."
    verified_by: "oya-ci-required"
  - id: ADR-0710-D2
    description: "Own the three components that replace the policy engine: the tier-map projection controller, the CI signing + digest-pinning verifier, and the asynchronous cluster conformance scanner."
    exit_criteria: "Each has an OWNERS file, a BUCK target, a registry catalog row, and a gate asserting manifest-to-cluster agreement for the tier map. Each ALSO names its failure mode and carries an OpenSLO objective plus a failure-injection test before it promotes past dev, because repository artifacts alone cannot show a security control fails safely: a STALE tier projection must be indistinguishable from an absent one and DENY (never admit on last-known-good); a signing-verifier outage must fail the build rather than let an unverified digest through; and the conformance scanner carries a bounded staleness objective past which its verdict reads UNKNOWN, never PASS. The gate asserts each objective exists and each injection test is executed, not merely declared."
    verified_by: "oya-ci-required"
---
# ADR-0710: Kubernetes admission substrate is the API server: VAP/CEL + PSA, no policy webhook

## Status

**Proposed.** Deliberately not Accepted: clause D-8 depends on evidence about the tenant
isolation boundary that is being measured and is not yet in. Landing this Accepted before
that evidence would assert a security posture we have not verified.

## Context

Three admission substrates exist in the corpus and the ADR that enforces tenant-facing
runtime placement cites the oldest of them.

- **ADR-0183** chose Cedar for application authorization and **Kyverno** for admission.
- **ADR-0379** superseded that with **Kubewarden**, on the grounds that WASM policy modules
  written in Rust "align admission policy with the WASM-native server-side sandbox doctrine
  (ADR-0023) and the Rust-everywhere stack," and demoted Kyverno to a non-default adapter.
- **ADR-0338** (pod runtime tiers 0–3) makes `enforce-pod-runtime-tier` BLOCKER-class and
  names **Kyverno** 48 times across 44 lines, because it predates ADR-0379.

Both named engines are **admission webhooks**. ADR-0379's own rejected-alternatives section
contains the decisive observation, and then stops one step short of it:

> "Run both engines as co-defaults — rejected: two admission webhooks on every API write is
> redundant cost and a potential conflict surface."

That reasoning, followed through, argues for **zero** webhooks. The option set ADR-0379
evaluated was Kyverno vs Kubewarden vs both; **"no webhook" was never a candidate**, because
at the time there was no in-process alternative. There is now.

**ValidatingAdmissionPolicy (VAP) is GA since Kubernetes 1.30**, and the cluster this repo pins
is well past that: `infra/capi/clusters/values.yaml` sets `kubernetesVersion: v1.36.1`, and it
is the only Kubernetes version declaration in `infra/`. An earlier draft of this ADR quoted a
range of "1.30.0 through 1.38.0" read out of `infra/`; those two figures are an Istio chart
`targetRevision` and a BusyBox image tag respectively, and neither is a Kubernetes version.
Correcting the premise strengthens the argument rather than weakening it — VAP is six minors
past GA on the version actually deployed. **Pod Security Admission is already in use in this
repo** — `pod-security.kubernetes.io/enforce` is present today. MutatingAdmissionPolicy
followed VAP, so its availability is checked against v1.36.1 rather than assumed.

This is not a criticism of the prior decisions. It is that the platform closed the gap the
webhook policy-engine category existed to fill.

## Decision

### D-1 — The API server is the default admission substrate

Validation is expressed as **ValidatingAdmissionPolicy** with CEL, evaluated in-process. The
base platform overlay ships **no policy webhook**.

### D-2 — Pod security baseline is Pod Security Admission

PSS restricted-by-default is a namespace label, not a policy. It is already deployed here.
No policy engine is required for it, and none may claim it.

### D-3 — Mutation prefers MutatingAdmissionPolicy

Where the running cluster version supports it, mutation is CEL-based and in-process. A
mutating **webhook** requires a recorded, expiring exception naming the cluster version that
forced it.

### D-4 — Resource generation is a controller concern, never admission

Creating or managing other resources is reconciliation. Kyverno's `generate` is a controller
wearing an admission hat, and that conflation is how a policy engine becomes a privileged
always-on dependency. Generation belongs in an owned Rust controller with a CRD.

### D-5 — Cross-resource data is projected, not fetched

VAP's `paramKind` covers one parameter resource. Anything broader is served by a **controller
that projects the required data into a param resource**, which VAP then reads. Admission must
never depend on a second live API call succeeding.

### D-6 — Image provenance moves to build time; admission checks a CI-issued allowlist

Signature verification requires a registry fetch and a cryptographic chain and is **not
expressible in CEL**. The verification therefore leaves the request path — but it does not
become optional, and digest pinning on its own does not replace it:

- **CI** verifies the Sigstore signature and emits an immutable `@sha256:` digest.
- **CI** projects every digest it verified into an allowlist param resource, by the same
  projection pattern as D-5.
- **VAP** refuses any pod whose image reference is not digest-pinned, **or** whose digest is
  absent from that allowlist. Either condition alone is a refusal; both must pass to admit.

The allowlist is the load-bearing half, and an earlier draft of this clause omitted it. A digest
makes bytes immutable; it does not make their provenance authorized. Pinning alone accepts any
syntactically valid `@sha256:` reference — including one CI never saw — so a manifest edited
after the CI step, or a compromised internal deploy path, would be admitted. That is a
regression against the admission-time signature control ADR-0701 carries, and this ADR may not
drop it silently while claiming to strengthen provenance.

With the allowlist the two checks are not equivalent-but-cheaper, they are **relocated**: the
cryptography runs once per artifact in CI instead of once per API write, and what remains at
admission is a set-membership test CEL can express. The reference becomes unforgeable by
construction rather than re-derived under latency pressure. This is compile-time-beats-runtime
applied to the supply chain — a relocation of the check, never its removal.

### D-7 — Kyverno, Kubewarden and Gatekeeper/OPA are ruled out as the default

All three are the same shape: a synchronous webhook in the path of every API write. They
remain available as **adapters** for an environment or pack that requires one, and any such
selection records why.

### D-8 — Tenant isolation comes from topology, not from admission policy

EKS, GKE and AKS give each cluster its own control plane; they do not use admission policy to
separate tenants. **If admission policy is the tenant boundary, the architecture is already
wrong.** A policy failure must degrade defence-in-depth, not breach isolation.

This reframes ADR-0338's runtime-tier enforcement as **placement hygiene inside a trust
domain** rather than the isolation boundary itself — which is a correct job for VAP.

**This clause is why the ADR is Proposed**, and the evidence it waits on is about the
**workload** boundary, not the control-plane boundary. A distinct tenant API server proves
nothing about where that tenant's pods land: separate control planes are compatible with two
tenants sharing a node, a hypervisor, or a physical host, and this repo's hosted-default
topology includes shared-substrate operation. Where workloads can share, a pod admitted with
the wrong RuntimeClass can breach isolation rather than merely degrade hygiene — which is
exactly why ADR-0338 places tenant-untrusted code on Kata — and the reframing above does not
hold.

D-8 is therefore gated on the sufficient half, stated as a measurement: **can two tenants be
scheduled onto the same node, the same hypervisor, or the same physical host?** Control-plane
separation is necessary and is not the answer. If the answer is yes anywhere in the hosted
default, D-8 fails and runtime-tier admission stays an isolation control with all the
obligations that carries.

### D-9 — The owned authorization PDP stays out of the admission path

The platform is building a **Cedar-compatible PDP with Zanzibar-style ReBAC** (ADR-0702
topic). It must not be placed in the admission path, for two independent reasons:

1. **Shape.** Cedar is principal-centric and Zanzibar is relationship-centric — both answer
   "may this subject act on this object." Admission is resource-shape-centric: "is this object
   well-formed." A pod spec has no subject. ADR-0183 and ADR-0379 both identified this
   mismatch and both were right; Zanzibar makes it stronger, not weaker.
2. **Coupling.** Putting our PDP in the admission path makes an application-authorization
   outage into a cluster-admission outage. Those failure domains must stay separate.

The separation ADR-0183 drew is preserved and sharpened: **the owned PDP answers principal and
relationship questions; the API server answers object-shape questions.** No overlap.

### D-10 — Accepted cost: no background scanning, and it needs an owner

VAP fires at admission and never sees resources that already exist or have drifted. Kyverno
and Gatekeeper scan continuously; we lose that.

This repo's gate culture scans **git trees, not live clusters** — a different problem. The
replacement is an **asynchronous cluster conformance scanner**, which is a controller rather
than a webhook and is therefore off the request path. It is named here as accepted cost with a
required owner, not waved away.

## Why this fits this stack better than the alternative it replaces

**Policy as data, not as a deployed program.** `specs/**` is ratified data everywhere here and
gates assert against it. A CEL policy is a record that is diffed and reviewed. A WASM module is
a program with a supply chain, a sandbox, a rollout, and its own compromise story.

**Fail-closed becomes affordable — it does not become automatic.** A webhook's `failurePolicy`
has no good setting when the control is tenant-facing: `Fail` trades the control for an outage
of every API write, because the failure being absorbed is a network hop to a separate
deployment; `Ignore` means the control **silently stops being enforced**. In-process evaluation
deletes the hop and with it the dilemma, so `Fail` stops being expensive.

It does **not** delete the settings, and an earlier draft of this ADR wrongly claimed it did.
`ValidatingAdmissionPolicy.spec.failurePolicy` still takes `Fail` or `Ignore` and governs CEL
evaluation errors and type-check failures; a parameterized `ValidatingAdmissionPolicyBinding`
still takes `paramRef.parameterNotFoundAction` with `Deny` or `Allow`. Those are precisely the
silent-admit paths D-5's projection introduces — a tier param that is missing or not yet
projected, under `Allow`, admits everything it was meant to constrain.

Therefore every policy authorized by this ADR **MUST** set `failurePolicy: Fail` and
`parameterNotFoundAction: Deny`, and D-2's gate asserts both values on every shipped policy and
binding rather than trusting a default. "A check that could not run must never read as a check
that passed" is the standing invariant, and it applies to clusters as much as to gates — but
here it is a configuration this ADR requires and tests, not a property the substrate donates.

**Rust lands where it is load-bearing.** Not in a predicate over a pod spec, but in the
tier-map projection controller and the CI signing verifier — both stateful, both worth owning,
both off the request path. This yields *more* Rust in the places that matter, not less.

**Blast radius.** A `ValidatingWebhookConfiguration` intercepts every API write, sees every
object in flight including Secrets in request bodies, and controls admission outcomes. Removing
it removes one of the highest blast-radius-per-line-of-code components in the cluster.

**Platform trajectory.** The webhook policy-engine category exists because CEL admission did
not. It does now, and SIG-Auth continues to move policy into the API server.

## Alternatives considered

- **Keep Kubewarden (ADR-0379's choice).** Rejected. Its strongest argument is authoring
  ergonomics — Rust→WASM alignment — which is not worth a synchronous webhook in the admission
  path of a product carrying an availability SLA. The "shared cross-cutting policy logic"
  benefit does not cash out: admission policies are small predicates over object shapes, and
  where sharing is real (the tier map) it is *data*, which `paramKind` provides natively.
- **Keep Kyverno (ADR-0183's choice).** Rejected, already demoted by ADR-0379, and every rule
  type it offers maps to VAP, PSA, or a controller.
- **Gatekeeper / OPA.** Rejected by the same argument as Kubewarden, with Rego instead of WASM.
  It is what GKE and Azure ship precisely because it predates VAP.
- **The owned Cedar-compatible + Zanzibar PDP for admission.** Rejected on shape and coupling —
  see D-9.
- **A bespoke Rust admission webhook.** Rejected: it keeps every failure mode of a webhook and
  adds our own availability to the critical path.

## Consequences

**Positive.** No webhook in the API write path; no `failurePolicy` with a silent-failure
setting; lower latency on every write; one fewer privileged component; policy becomes
reviewable data; PSA is already satisfied at zero cost; image provenance strengthens by moving
to build time.

**Negative / cost.** Background scanning of existing resources is lost and must be rebuilt as
an async controller (D-10). Policy reporting UX is thinner than Kyverno's PolicyReport CRs.
Cross-resource policies require the projection pattern rather than arriving free. Mutation
depends on MutatingAdmissionPolicy's status in the running version and must be verified rather
than assumed.

**Reversibility.** `door: two-way`. Kubewarden is declared in `infra/gitops/values.yaml`
(`kubewarden-crds`, `kubewarden-controller`); whether it is reconciled and running is being
measured. If it is declared-but-not-running, this change costs nothing to make and nothing to
undo.

## Relationship to existing decisions

- **Amends ADR-0701**, which carries ADR-0379's gist as a live apex.
- **Replaces the admission-substrate clauses** of ADR-0183, ADR-0379 and ADR-0338. ADR-0338's
  tier MODEL — tiers 0–3, the runtime mapping, the nodepool contract, the capacity factor — is
  unaffected and stands; only the mechanism that enforces it changes.
- **Carries ADR-0338 D-5.5's denial audit event across the mechanism change.** That clause
  requires a `pod.runtime.tier.denied` event on every deny, with `microservice`,
  `declared_tier`, `attempted_runtime_class`, `attempted_nodepool` and `reason`. It belongs to
  the standing tier contract, not to the replaced webhook, and a bare VAP denial does not emit
  it — so replacing the engine without replacing the event would delete the evidence security
  operators use to investigate placement attacks. The replacement is `validationActions:
  [Deny, Audit]` plus `auditAnnotations` carrying those five fields, and an owned consumer that
  projects the API-server audit log into the event schema. D-2's gate asserts a denial actually
  produces the event: a denial that leaves no evidence is not a control, it is a silence.
- **Preserves ADR-0183's separation** between application authorization and cluster admission,
  restated in D-9 for the owned Cedar-compatible + Zanzibar PDP.
- **ADR-0023's** WASM server-side sandbox doctrine is unaffected for its own domain; this ADR
  declines to extend it to the admission path.
