---
id: ADR-0606
title: "Operator secret-bootstrap RBAC gate (least-privilege secrets + declarative join-token provisioning)"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-28
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0535, ADR-0547, ADR-0548, ADR-0566]
amends: []
related: [ADR-0535, ADR-0547, ADR-0548, ADR-0566, ADR-0605]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0606: Operator secret-bootstrap RBAC gate (least-privilege secrets + declarative join-token provisioning)

## Status

**Proposed - 2026-06-28 (authored for founder sign-off; door: one-way).**

## Context

GH #980: the in-cluster SVID PDP operator (cloud-iam) shipped with an over-broad RBAC
posture and a non-declarative join-token bootstrap. Two defect classes:

1. **Over-privileged secret access.** The operator's Helm `Role` granted scoped verbs
   (`get`/`update`/`patch`/`delete`) on Secrets namespace-wide rather than bound by
   `resourceNames` to the single Secret it produces (`oya-cloud-iam-pdp-svid`). A
   compromised operator could read or mutate any Secret in its namespace — a lateral
   blast-radius violation of least-privilege.
2. **Non-declarative bootstrap secret.** The operator-internal join token it consumes had
   no declarative provisioning contract (ExternalSecret / SealedSecret / Secret) and no
   fail-closed chart preflight, so a missing or plaintext-in-git token could pass silently.

GH #988 extends the recurrence class to OpenBao/External Secrets wiring: a CI-scoped
ClusterSecretStore role was reused for cloud-k8s CSI secrets outside the documented `oya/ci/*`
OpenBao policy, and the OpenBao listener remained plaintext without a committed transport-isolation
artifact. That is the same failure mode at the substrate boundary: secret access relies on human
memory rather than an enforced policy tuple.

The RBAC and bootstrap fixes themselves land in the cloud-iam Helm chart (GH #980). But a
one-time fix does not prevent recurrence: a future operator chart can re-introduce the same
over-grant or a plaintext token. Per the pipeline-as-product / friction-is-a-process-failure
doctrine (ADR-0548), the recurrence class must be closed by a born-blocking gate, not a manual
review checklist.

## Decision

Ship a **self-contained cloud-ci gate**, `cloud-ci-operator-secret-bootstrap`
(`ci/facade/operator-secret-rbac`), mirroring the registration
footprint of the authz-coverage (ADR-0566) and supply-chain-audit (ADR-0605) gates: own crate,
own policy JSON, one appended matrix line in `.github/workflows/oya-ci-required.yml`, no
`libs/oya-ci-config` edit, no producer-face binding.

### D1 — Pure, policy-driven, hermetic

The gate is owned pure-Rust (`#![forbid(unsafe_code)]`, no `Command::new`, no shell-out, no
network, no clock). Dependencies are the workspace-inherited `serde_json` + `serde_yaml` only —
zero new crate enters `Cargo.lock`. The only I/O is a hermetic, read-only filesystem scan of the
policy-declared Helm charts, ExternalSecret manifests, values-backed ExternalSecret projections, and OpenBao manifest; the verdict is a pure
`evaluate_keyed(policy, observed)`. This satisfies the hermetic-gate bar (ADR-0548: every gate is
a deterministic, buck2-cacheable predicate) and rust-purity (ADR-0547).

### D2 — The four invariants (the verdict)

For each governed surface declared in the policy DATA, `evaluate_keyed` asserts:

- **(a) Least-privilege secrets.** No `Role` rule over `secrets` grants a scoped verb
  (`get`/`update`/`patch`/`delete`) unless its `resourceNames` is bound to exactly the operator's
  produced Secret. (`list`/`watch`/`create` cannot be `resourceNames`-scoped — the documented
  Kubernetes RBAC floor — and are not failed by this rule; the operator narrows `list`/`watch`
  with a field selector as a follow-up.)
- **(b) Declarative join-token bootstrap.** Any operator-internal bootstrap Secret the operator
  consumes must have a declarative provisioning path (ExternalSecret / SealedSecret / Secret) OR
  a fail-closed chart preflight that refuses to render without one.
- **(c) ESO store/key-prefix scoping.** Every committed ExternalSecret using a governed
  ClusterSecretStore must match a declared `(store kind, store name, bound OpenBao role,
  consumer namespace, remoteRef.key prefix)` tuple. A CI role/policy cannot project cloud-k8s CSI
  secrets by accident.
- **(d) OpenBao transport isolation.** If OpenBao is configured with a plaintext listener
  (`tls_disable = true`), the manifest must include a NetworkPolicy selecting the OpenBao workload
  and restricting ingress for the declared listener ports.

The gate is **fail-closed**: an empty governed-operators list, an unknown `gate_id`, a malformed
policy, a declared operator with no observed chart view, an ExternalSecret outside its declared
scope, a ClusterSecretStore role mismatch, or a plaintext OpenBao listener without NetworkPolicy
coverage all return RED — it is not an
always-pass stub. RED/GREEN fixtures in `tests/` drive the real collector.

### D3 — Policy-as-data, nothing oyatie-specific in Rust

The governed operators and their chart paths / produced-Secret names live in
`operator-secret-bootstrap-policy.json`. The Rust carries no hardcoded operator or path, so the
gate is reusable across any repo following the operator-produces-a-Secret pattern. The SVID PDP
operator is the first governed operator.

## Governed surfaces

The exact tracked paths this decision introduces and governs (born-accounting justification per
ADR-0568; one verbatim repo-relative path per line):

```
ci/facade/operator-secret-rbac/BUCK
ci/facade/operator-secret-rbac/Cargo.toml
ci/facade/operator-secret-rbac/OWNERS
ci/facade/operator-secret-rbac/operator-secret-bootstrap-policy.json
ci/facade/operator-secret-rbac/src/lib.rs
ci/facade/operator-secret-rbac/tests/operator_secret_bootstrap.rs
cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-join-token-externalsecret.yaml
cloud/cloud-iam/iac/k8s/helm/values.yaml
infra/external-secrets/clustersecretstore-openbao-oya.yaml
infra/external-secrets/externalsecret-github-ci-token.yaml
infra/external-secrets/RUNBOOK.md
infra/kms/openbao.k8s.yaml
infra/nativelink/nativelink-cas.k8s.yaml
cloud/cloud-k8s/iac/kustomize/base/openbao-secret-references.yaml
```

## Consequences

- **Born-blocking from the first commit**: the SVID RBAC + ExternalSecret fix lands together with
  the gate, so the gate is green on introduction and blocks any future regression.
- **Owned + hermetic**: no shell, no adhoc dependency, no network — clears the AWS/Google "would
  they ship this as their gate" bar (consistent with ADR-0605).
- **Accounting**: the new crate is owned by `ci/facade/operator-secret-rbac/OWNERS`
  (`cloud-ci-platform`) and reachable via its `BUCK` `rust_test` target + the `oya-ci-required`
  matrix entry; it maps to the existing `cloud-ci` capability (ADR-0562), so no new capability is
  registered.
