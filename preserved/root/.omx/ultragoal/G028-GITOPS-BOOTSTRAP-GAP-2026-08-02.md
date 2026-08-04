# G028 GitOps bootstrap gap — 2026-08-02

State: **PLANNING_ONLY — LIVE OBSERVE PROVED INERT RECONCILER — NO MANUAL APPLY — CLASS A CORRECTED AFTER INDEPENDENT REVIEW**  
Authority tip: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; declared request=22Gi).

Independent review (peer `g028-gap-review`): **REQUEST_CHANGES** on prior class A wording (CAPI management/target conflation + non-adopting cells[]). LIVE_GAP_CHECK pass. This revision absorbs that finding; it is not yet a design APPROVE of a chosen repair class.

## What is already true

| Layer | Evidence |
|---|---|
| Declared values | `infra/arc/runner-scale-set-arm64-values.yaml` request `22Gi`, limit `60Gi` on tip |
| Declared wiring | `infra/gitops/values.yaml` release `oya-arm64` → chart `gha-runner-scale-set` 0.14.2, ns `arc-runners`, valueFiles that yaml |
| App-of-apps template | `infra/gitops/templates/applications.yaml` emits multi-source Application for chart+valueFiles |
| Root Application | `infra/gitops/root-app.yaml` targets path `infra/gitops` on `dev` |
| Intended bootstrap | CAPI CRS pair in `infra/capi/crs/clusterresourceset.yaml` for clusters labelled `oya.io/bootstrap: "true"` |
| PR admission | #1529 independent APPROVE + oya-ci-required SUCCESS + squash merge `0c1014b87` |

## Live observation (read-only; no mutate)

| Probe | Result |
|---|---|
| `AutoscalingRunnerSet/oya-arm64` request | **20Gi** (path `.spec.template.spec.containers[0].resources.requests.ephemeral-storage`) |
| `EphemeralRunnerSet/oya-arm64-lh4ch` request | **20Gi** (path `.spec.ephemeralRunnerSpec.spec.containers[0].resources.requests.ephemeral-storage`) |
| three live runner pods request | **20Gi** |
| Helm release `oya-arm64` | revision **12**, deployed **2026-07-30T05:55:39**, still deployed |
| `argocd` namespace | **ABSENT** |
| Argo/Flux controller pods | **ABSENT** |
| Argo CRDs / Applications | **ABSENT** |
| CAPI `clusters.cluster.x-k8s.io` | **ABSENT** |
| CAPI `ClusterResourceSet` | **ABSENT** |
| node labels `cluster.x-k8s.io/cluster-name` | **none** on controlplane-1 / worker-1 / worker-2 |

Conclusion: the declaration is correct and admitted; the **reconciler is not present on this cell**, so values cannot converge. This is not authorization for `helm upgrade`.

## Intended declarative chain (tip)

```text
CAPI management cluster
  hosts CAPI providers + ClusterResourceSet objects + rendered ConfigMaps
  → selects workload Cluster CRs labelled oya.io/bootstrap=true
  → CRS bootstrap-cilium-argocd (ApplyOnce): Cilium + Argo CD install onto workload
  → CRS bootstrap-argocd-root-app (Reconcile): root Application onto workload
  → Argo CD on workload renders infra/gitops Helm chart (values.yaml apps[])
  → Application oya-arm64 multi-source: upstream chart + $values/infra/arc/runner-scale-set-arm64-values.yaml
  → live AutoscalingRunnerSet / EphemeralRunnerSet request becomes 22Gi on workload
```

Sources:

- `infra/capi/crs/clusterresourceset.yaml` (two CRS; selector `oya.io/bootstrap: "true"`; CRS live on management, act on selected workload Clusters)
- `infra/capi/clusters/templates/clusters.yaml` (renders **new** `Cluster` + infra cluster + `TalosControlPlane` + `MachineDeployment` — not an adopt-existing path)
- `infra/capi/clusters/values.yaml` (`cells: []`)
- `infra/capi/clusters/README.md` (CAPI spoke path; Argo then pulls `infra/gitops`)
- `infra/gitops/root-app.yaml`, `Chart.yaml`, `templates/applications.yaml`, `values.yaml`

## Exact gap on the permanent CI lab cell

1. **Lab is not a CAPI workload Cluster.** Nodes `oya-talos-*` have no CAPI cluster ownership labels; CAPI CRDs are missing on the live API. There is no management-plane CRS and no selected workload `Cluster` object.
2. **`cells: []`.** Tip spoke catalog is empty; no committed cell entry describes a CAPI-provisioned replacement for this lab.
3. **Adding a `cells[]` entry cannot adopt `admin@oya-talos`.** The clusters chart provisions a replacement cell (new control plane + workers). It does not attach CRS bootstrap to an already-running non-CAPI Talos install.
4. **Argo CD is not installed on the lab.** Without CRS ApplyOnce argocd-bootstrap against a selected workload Cluster (or an admitted non-CAPI equivalent), root-app and app-of-apps cannot exist.
5. **Prior ARC install was out-of-band Helm.** Live release v12 pre-dates #1529 and still holds 20Gi; no controller re-reads git.
6. **`bootstrap-sync.yaml` is not a substitute for missing Argo.** Its own header requires Argo CD already running; it only adds local-path/VCS Applications into an existing Argo.

This matches the standing memory class “GitOps declaration wired to NOTHING” for this cell: declaration exists; reconciler path is absent.

## What is NOT a safe next action

- `helm upgrade oya-arm64 ...` from a worktree or scratchpad values copy
- `kubectl apply` of rendered Argo install without independent review + protected admission + explicit bootstrap authority
- Treating promoted tip CI green as proof that live runners are 22Gi
- Pushing #1523 restack or re-running #1526 cold FULL while live request remains 20Gi
- Inventing a second values path or weakening request arithmetic
- Claiming that a `cells[]` PR alone bootstraps the current lab without replacement/migration semantics

## Minimum repair classes (owner/design choice; plan-only)

Exactly one class must be chosen by accountable platform-infra owner before mutation.

### A. CAPI-provisioned replacement cell (canonical spoke path) — NOT in-place adopt

Class A creates or migrates to a **new** CAPI-managed workload cell. It does **not** bootstrap the already-running `admin@oya-talos` cluster in place.

Required explicit pieces:

1. **Management cluster identity** — name the cluster that will host CAPI providers, CRS objects, and rendered bootstrap ConfigMaps. This is distinct from the workload/target cell. If no management cluster exists yet, that bootstrap is a prerequisite workstream and must be scoped.
2. **Workload cell declaration** — commit a real `cells[]` entry (substrate + sizing + network) that renders `Cluster` labelled `oya.io/bootstrap=true` via `infra/capi/clusters` templates.
3. **Replacement / migration / reprovision semantics** — because templates emit new `TalosControlPlane` + `MachineDeployment`, the owner must choose and document one of:
   - **REPLACE**: new cell becomes the permanent CI substrate; old lab is drained and decommissioned;
   - **MIGRATE**: cut runner scale-set / secrets / registry / postgres dependencies to the new cell with continuity checklist;
   - **REPROVISION-IN-PLACE is out of scope for class A** unless a separate, reviewed adopt-existing design is written (current chart has no adopt path).
4. **Continuity and rollback** — what carries CI during cutover; how to revert runner registration; data/secrets inventory.
5. **Protected PR(s)** for management-plane declarations, cell catalog, and any CRS/render outputs that are committed or hermetically produced.
6. **Authorized bootstrap acts** only after admission: management-plane provider/CRS apply, then workload provision, then observe Argo on the **workload** API.

Acceptance (on the workload cell that will run `oya-arm64`, not on a management API by mistake):

- Argo `argocd` ns + controllers exist
- Application `oya-arm64` Healthy/Synced to tip declaring 22Gi
- Read-only probes all return `22Gi`:
  - AutoscalingRunnerSet: `.spec.template.spec.containers[0].resources.requests.ephemeral-storage`
  - EphemeralRunnerSet: `.spec.ephemeralRunnerSpec.spec.containers[0].resources.requests.ephemeral-storage`
  - at least one **new** runner pod after sync

### B. Permanent-lab non-CAPI bootstrap declaration (keep current `admin@oya-talos`)

Use when the laptop/permanent CI Talos lab is intentionally outside CAPI.

- Author an owned, committed bootstrap surface that installs Argo (or an owned reconciler) **on this lab** without claiming CRS can select non-CAPI nodes.
- Must not revive retired shell `bring-up.sh` as merge authority; reviewable manifests/chart + protected PR.
- Must state how root-app + `oya-arm64` Application are created and kept healthy on this API.
- Acceptance probes identical to class A (same three 22Gi paths on this lab).

### C. Explicit KEEP_INERT + alternate FULL path (only if founder accepts)

- Document that this cell will not GitOps-reconcile ARC and name a different admitted substrate for trusted FULL baselines.
- Does **not** green #1526 on hope; requires a real alternate cold-FULL substrate with DiskPressure arithmetic redone.

Default recommendation: **B if the current lab must remain the permanent CI substrate without replacement; A only if owner accepts CAPI replacement/migration of the CI cell onto a management+workload topology.** Coordinator must not self-pick live bootstrap.

## Required evidence before unblocking #1526 / #1523

1. Independent design APPROVE on the **chosen** class A/B/C packet (this gap note alone is not that APPROVE until an owner picks a class and review clears the corrected design).
2. Protected PR(s) admitted for any declaration changes.
3. Authorized bootstrap act only if the approved design requires one-time management or lab apply (still not scratchpad helm of runner values alone).
4. Read-only proof on the cell that will run FULL baselines:
   - `kubectl -n arc-runners get autoscalingrunnerset oya-arm64 -o jsonpath='{.spec.template.spec.containers[0].resources.requests.ephemeral-storage}'` → `22Gi`
   - `kubectl -n arc-runners get ephemeralrunnerset <name> -o jsonpath='{.spec.ephemeralRunnerSpec.spec.containers[0].resources.requests.ephemeral-storage}'` → `22Gi`
   - at least one new runner pod shows request `22Gi`
5. Only then: #1526 cold FULL re-run; then #1523 restack push (local cherry-pick rehearsal already clean).

## Buck2 / CI checks for any bootstrap PR

- No hand-edited `*.generated.json`.
- If charts/templates change: hermetic render test (helm template or owned Rust renderer) with fixture asserting Application `oya-arm64` valueFiles path and request=22Gi from tip values.
- Gate must fail if valueFiles path drifts from `infra/arc/runner-scale-set-arm64-values.yaml`.
- For class A PRs: render fixture must show management vs workload objects are not collapsed; `cells[]` entry renders a full new Cluster set, not an adopt fragment.
- Affected-set / oya-ci-required green on the bootstrap PR itself.
- Do not claim live convergence from PR CI alone.

## Non-actions

- No cluster mutation in this packet.
- No helm upgrade.
- No #1524 un-draft.
- No canonical dirty-checkout mutation.
- No independent APPROVE inferred from failed transport on prior research agents.
- No activation of G036/G037/G026 moves on the back of this gap.
- No owner self-disposition by the coordinator.

## Dependency order (unchanged, now precise)

```text
#1529 MERGED (done)
→ owner chooses A (replacement CAPI cell) | B (non-CAPI lab bootstrap) | C (alternate FULL substrate)
→ independent design APPROVE on chosen class
→ admitted declaration + authorized bootstrap if required
→ live request 22Gi observed on the FULL-running cell
→ #1526 cold FULL
→ #1523 restack push + admit
→ G023 deletion only after #1523 promoted green
```
