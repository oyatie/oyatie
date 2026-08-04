# G028 class-B bootstrap repository facts — 2026-08-02

State: **READ-ONLY SOURCE FACTS — NO DESIGN APPROVE — NO IMPLEMENTATION — NO LIVE MUTATION**  
Immutable source baseline: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf`

## Current GitOps blast radius

`infra/gitops/root-app.yaml:5-25` declares root `Application/root`, sources moving `dev`, and immediately enables prune+selfHeal. `infra/gitops/templates/applications.yaml:1-60` ranges `.Values.apps`, propagates mutable revision, and hard-codes prune+selfHeal, CreateNamespace, and ServerSideApply.

`infra/gitops/values.yaml` renders exactly 18 child Applications:

1. `cilium` (`:14-21`)
2. `kubewarden-crds` (`:28-34`)
3. `kubewarden-controller` (`:35-41`)
4. `kubewarden-defaults` (`:42-48`)
5. `istio-base` (`:52-58`)
6. `istiod` (`:59-66`)
7. `istio-cni` (`:67-74`)
8. `ztunnel` (`:75-81`)
9. `github` (`:83-88`)
10. `openbao` (`:90-95`)
11. `external-secrets` (`:99-106`)
12. `arc` (`:111-118`)
13. `oya-arm64` (`:123-130`)
14. `registry` (`:134-139`)
15. `seaweedfs` (`:148-153`)
16. `cloud-intelligence` (`:161-165`)
17. `cell-boundaries` (`:172-176`)
18. `observability` (`:183-193`)

Standalone YAML beside the chart is not Helm fan-out. `bootstrap-sync.yaml` and `vcs-argocd-app.yaml` are separate mutable-`dev`, prune-enabled declarations; `bootstrap-sync.yaml` also references absent `infra/gitops/local-path-storage.yaml` (`:37-42`). None is safe G028 adoption authority.

## Existing protected gate mechanism

No `BUCK` exists under `infra/`. A check placed there alone would be dark.

The existing protected-fleet pattern is:

```text
ci/facade/<crate>/BUCK
  rust_library
  rust_test named ci-<crate>-unittest
  gate target named ci-<crate>-gate
```

Comparable implementations:

- YAML parsing: `ci/facade/operator-secret-rbac/BUCK:1-42`.
- Hermeticity: `ci/facade/embedded-asset-hermeticity/BUCK:1-58`.

Protected execution is already binding:

- `.github/workflows/oya-ci-required.yml:187-197`: new `ci/facade` crate + real Buck test is recursively covered;
- `:578-586`: binding `buck2 test //ci/...`;
- `:1205-1223,1241-1255`: `buck2` is a required dependency and result in final `oya-ci-required` success;
- `ci/facade/baseline-ratchet/tests/gate_registration.rs:685-817`: recursive coverage requires a real `*_test` and fan-in-reachable binding execution.

The minimum proposed G028 gate identity is therefore:

```text
crate: ci/facade/gitops-bootstrap-contract
target: root//ci/facade/gitops-bootstrap-contract:ci-gitops-bootstrap-contract-gate
unittest: root//ci/facade/gitops-bootstrap-contract:ci-gitops-bootstrap-contract-unittest
workflow execution: existing buck2 job / buck2 test //ci/...
final fan-in: existing checked needs.buck2.result
```

This is a design proposal for fresh independent review, not implementation authorization.

## Current Argo inputs are not hermetic

- `infra/capi/crs/render.sh:12-20,37-38`: chart `argo/argo-cd` version `9.5.15` from remote Helm repository.
- `:26-39`: controller render disables bundled CRDs.
- `:49-69`: dynamically resolves remote chart `appVersion` and compares it with remote CRD URL tags.
- `infra/capi/clusters/values.yaml:17-26`: three raw GitHub v3.4.2 CRD URLs.
- `infra/capi/clusters/templates/clusters.yaml:71-76`: injects those remote URLs into Talos extraManifests.
- No chart archive, `Chart.lock`, vendored CRD bytes, or content digest exists under `infra/capi` or `infra/gitops`.
- `render.sh:71-90` creates ConfigMaps and performs `kubectl apply`; it is live mutation machinery, not hermetic validation.

Version strings are pinned; bytes are not. G028 requires new content-addressed/vendored inputs before implementation can be admitted.

## Identity and rollback absence

Repository probes found:

- no bootstrap principal or bootstrap ServiceAccount/RBAC identity;
- no cryptographic target-cluster identity, only CAPI names/labels;
- no rollback-specific owner (generic path/root owner is `cloud-ci-platform`);
- no immutable Git targetRevision pattern; live declarations use moving `dev`/`main`;
- no CI invocation validating the existing CAPI render path.

`infra/capi/init.sh:4-9,17-28` consumes external kubeconfig/provider credentials but declares no principal. These identities must come from accountable authority; the coordinator must not infer them from OWNERS.

## Non-actions

- No use of `render.sh`, Helm, CRS, or root Application as apply authority.
- No claim that generic file ownership names a bootstrap principal or rollback owner.
- No implementation from proposed target names.
- No live mutation or design approval.
