---
doc_class: Standard
status: Accepted
date: 2026-06-05
canonical_authority: specs/kubernetes-native-anti-patterns.json
planned_enforcement_ref: buck2 build //:kubernetes-native-anti-pattern-check
related_adrs:
  - ADR-0513
  - ADR-0514
  - ADR-0516
---

# CUE/Kubernetes desired-state authority

## Decision

CUE packages are the first-party source of truth for Oyatie cloud-cell, pod,
and Kubernetes workload desired state.

Generated Kubernetes manifests are build artifacts emitted from CUE-owned
constraints and checked through Buck2/Prow. Helm is adapter compatibility only:
it may be imported for third-party ecosystem charts or emitted as a generated
wrapper for temporary consumers, but it is not first-party policy authority.

## Why not Helm as canonical?

Helm is useful packaging glue, but first-party hyperscaler surfaces need typed
composition, constraint validation, shared schema reuse, and lane-owned shards
that reduce merge conflicts. CUE fits that role because configuration and
schema live in one language and can be exported to Kubernetes manifests.

First-party hand-authored Helm templates create the opposite pressure:

- shared `values.yaml` files become high-conflict edit surfaces;
- policy-critical defaults are easy to bypass with untyped template branches;
- reviews see rendered YAML diffs without the stronger constraint model; and
- CLI-centric install/rollback flows encourage manual mutation outside the
  controller/Buck2/Prow path.

## Required source layout

Lane-owned source:

```text
<substrate-or-service>/iac/cue/
  package.cue
  workload.cue
  policy.cue
  scale.cue
  tenant-overlays/
```

Generated output:

```text
<substrate-or-service>/iac/generated/k8s/
```

Compatibility adapters, when unavoidable:

```text
<substrate-or-service>/iac/adapters/helm/
```

Adapter directories are generated or imported compatibility surfaces. They must
point back to the CUE package that owns policy-critical defaults.

## Allowed Helm use

Helm is allowed only for:

1. consuming third-party charts from external ecosystems;
2. temporary import/export compatibility during a migration;
3. generated wrappers for customers or operators who require a chart interface;
4. historical evidence that a legacy chart existed.

Each allowed use must state the owning CUE package, migration/removal condition,
and non-authority boundary.

## Forbidden Helm use

Do not:

- create hand-authored first-party Helm templates as the source of truth;
- use Helm CLI deploys, installs, rollbacks, or chart hooks as canonical
  deployment procedure;
- treat `values.yaml` as policy authority for workload identity, runtime class,
  pod security, network policy, autoscaling, tenant labels, secrets, or
  compliance overlays;
- add shared chart helper libraries as the first edit target for multiple lanes;
- approve production-readiness claims from chart lint alone.

## Buck2/Prow checks

Every first-party desired-state change must have a Buck2/Prow verification path
that can run without live cluster mutation.

Minimum checks:

- CUE constraint validation for schema and policy defaults;
- generated Kubernetes manifest export determinism;
- Kubernetes admission-policy compatibility for pod security, identity,
  network policy, runtime class, and tenant labels;
- drift detection between CUE source and generated manifests;
- explicit adapter-boundary checks for any generated Helm wrapper.

The current umbrella guard is:

```text
buck2 build //:kubernetes-native-anti-pattern-check
```

## Source-driven basis

- CUE Kubernetes guide: https://cue.dev/docs/getting-started-with-kubernetes-cue/
- Helm chart docs: https://helm.sh/docs/topics/charts/
- CNCF Cloud Native Architecture: https://architecture.cncf.io/
- Kubernetes controllers: https://kubernetes.io/docs/concepts/architecture/controller/

## Migration rule

Existing `iac/helm` paths are compatibility scaffolding until replaced or
wrapped by a CUE-owned package. Do not expand them for new first-party policy.
Move each high-edit chart surface into a lane-owned CUE package before claiming
cloud-cell or pod configuration maturity.
