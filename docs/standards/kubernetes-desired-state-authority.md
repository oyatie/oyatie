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
constraints and checked through Buck2/Prow. Helm is not rejected as a package
ecosystem: third-party charts may be adopted or wrapped when they pass
hyperscaler adoption fitness. Helm CLI flows and hand-authored first-party Helm
templates are not durable first-party policy authority.

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

Helm is allowed for:

1. consuming third-party charts from external ecosystems when they pass
   hyperscaler-fit, license, security, provenance, and scalability review;
2. wrapping third-party charts behind Oyatie-owned CUE/KRM/policy adapters;
3. temporary import/export compatibility during a migration;
4. generated wrappers for customers or operators who require a chart interface;
5. historical evidence that a legacy chart existed.

Each allowed use must state the owning CUE/KRM or adapter package,
hyperscaler-fit classification, migration/removal condition, and non-authority
boundary.

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

## Rust-native engine seam

Oyatie may build a Rust-native CUE-compatible desired-state engine, but it is
not first-party authority until conformance is proven against upstream CUE
semantics. The conformance lane must demonstrate a restricted-subset parser,
constraint/unification compatibility for the repository's CUE packages,
deterministic generated-KRM output parity, module/import compatibility for
approved packages, and golden tests comparing Rust output to upstream CUE
output. Until that evidence exists, CUE remains the compatibility target and
Buck2/Prow checks treat any Rust engine as an advisory accelerator.

This keeps the hyperscaler pattern intact: adopt a proven interface now,
replace the high-leverage implementation seam later, and avoid blocking
parallel product/infra lanes on a language-runtime rewrite.

## Source-driven basis

- CUE language specification: https://cuelang.org/docs/reference/spec/
- CUE Kubernetes guide: https://cue.dev/docs/getting-started-with-kubernetes-cue/
- CUE export command: https://cuelang.org/docs/concept/using-the-cue-export-command/
- CUE modules reference: https://cuelang.org/docs/reference/modules/
- Helm chart docs: https://helm.sh/docs/topics/charts/
- CNCF Cloud Native Architecture: https://architecture.cncf.io/
- Kubernetes controllers: https://kubernetes.io/docs/concepts/architecture/controller/

## Migration rule

Existing `iac/helm` paths are compatibility scaffolding until replaced or
wrapped by a CUE-owned package. Do not expand them for new first-party policy.
Move each high-edit chart surface into a lane-owned CUE package before claiming
cloud-cell or pod configuration maturity.
