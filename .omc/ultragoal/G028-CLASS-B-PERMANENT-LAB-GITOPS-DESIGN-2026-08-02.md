# G028 class B — permanent-lab non-CAPI GitOps bootstrap design packet — 2026-08-02

State: **PLANNING_ONLY — CLASS B SELECTED — INDEPENDENT REVIEW `REQUEST_CHANGES` — REVISION REQUIRED — NO LIVE MUTATION**  
Authority tip: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529; ARC request declared `22Gi`).  
Parent gap: `G028-GITOPS-BOOTSTRAP-GAP-2026-08-02.md`.

## Selected class and unresolved execution authority

The 2026-07-29 founder ruling makes the current three-node `admin@oya-talos` cluster a permanent CI substrate with merge authority and requires declarative operation. Therefore `KEEP_CURRENT_LAB=true → class=B` is settled.

This does **not** identify the bootstrap principal or rollback owner and does not authorize implementation or live mutation. Those identities remain required inputs to the revised packet.

Accountability state:

```text
founder-ruling-2026-07-29 | KEEP_CURRENT_LAB=true | class=B | APPROVE_CLASS_SELECTION_ONLY
bootstrap_principal=UNRESOLVED
rollback_owner=UNRESOLVED
independent_design_verdict=REQUEST_CHANGES
```

Until the two principals are named and the revised exact packet receives independent `APPROVE`, this packet remains inert.

## Measured current state (read-only)

| Fact | Evidence |
|---|---|
| current lab | `admin@oya-talos`, nodes `oya-talos-controlplane-1`, `oya-talos-worker-1`, `oya-talos-worker-2` |
| node health | all Ready; DiskPressure=False |
| allocatable ephemeral storage | 45,909,593,217 bytes on each node (42.75664 GiB) |
| live ARC desired state | AutoscalingRunnerSet + EphemeralRunnerSet request `20Gi` |
| admitted git desired state | request `22Gi`, limit `60Gi`, maxRunners 3, minRunners 0 |
| Argo CD | namespace/controllers/CRDs/Applications absent |
| CAPI | Cluster/ClusterResourceSet CRDs absent; nodes have no CAPI cluster label |
| existing deployment path | Helm release `oya-arm64` revision 12, deployed 2026-07-30; no git reconciler |

The scheduling invariant is unchanged: `2 × 22Gi = 44Gi > 42.75664Gi` allocatable, so at most one requested runner fits per current worker. No value weakening is part of class B.

## Existing tip surfaces: reuse vs reject

### Reuse unchanged

- `infra/gitops/root-app.yaml`: app-of-apps root targeting `infra/gitops` on `dev`, automated prune/self-heal.
- `infra/gitops/templates/applications.yaml`: multi-source Application template.
- `infra/gitops/values.yaml`: `oya-arm64` chart 0.14.2 + `$values/infra/arc/runner-scale-set-arm64-values.yaml`.
- `infra/arc/runner-scale-set-arm64-values.yaml`: admitted 22Gi request.
- Argo CD chart/application pins already used by the CAPI bootstrap design, after making those pins machine-readable from one source.

### Do not use as class-B apply authority

- `infra/capi/crs/clusterresourceset.yaml`: only meaningful on a CAPI management cluster selecting workload `Cluster` objects.
- `infra/capi/clusters/values.yaml`: `cells: []`; adding a row provisions a replacement cell, not adoption.
- `infra/capi/crs/render.sh`: shell + ambient Helm repositories + direct `kubectl apply` to a management cluster. It is transitional CAPI bootstrap machinery, not a protected non-CAPI lab reconciler.
- `infra/gitops/bootstrap-sync.yaml`: requires Argo already running; it cannot install Argo itself.
- current Helm release state: observed provenance only, not declaration authority.

## Independent review verdict — `REQUEST_CHANGES`

The first independent architecture review rejected this exact packet. Its findings are binding; no implementation may begin from the sections below until they are replaced by a reviewed v2.

### Critical blockers

1. **Bootstrap ordering is not executable as one bundle/apply.** A single apply cannot guarantee the Argo `Application` CRD is `Established` before submitting the root `Application`; the packet's one-bundle rule contradicts its own abort boundary.
2. **The existing root is too broad for first adoption.** Unchanged root-app/app-of-apps would fan out to roughly 18 platform Applications with prune enabled. ARC-only adoption proof cannot authorize that blast radius.
3. **The desired revision is mutable.** Existing root/template sources target `dev`; ownership transfer must use the exact admitted immutable revision.

### High blockers

4. Argo controller and CRD bytes lack a proven exact content-addressed or vendored source pin.
5. Existing ARC controller and scale-set Helm resource inventory, release identity, secret continuity, and executable rollback are not captured.
6. No actual bootstrap principal or rollback owner is named.
7. The exact Buck2 target and `oya-ci-required` fan-in are unspecified.

### Required v2 shape

The revised packet must replace the rejected single-bundle broad-root sequence with:

```text
phase 1: admitted content-addressed/vendored Argo CRDs only
→ wait for exact CRDs Established
phase 2: admitted pinned Argo controllers only
→ wait for controller readiness
phase 3: narrow immutable-revision ARC adoption Application only, prune=false
→ prove resource/release/secret identity and healthy 22Gi reconciliation
phase 4: separately reviewed expansion to broader app-of-apps; prune remains off per app until ownership proof exists
```

Each phase must be a distinct deterministic artifact/digest and an explicit abort boundary. No root Application or custom resource may exist in a phase before its CRD establishment. The first ARC adoption phase must not instantiate unrelated platform Applications.

The final anchored reviewer report, repository fan-in facts, and ARC inventory facts are pending. Unknown paths, principals, pins, and targets remain `UNRESOLVED`; they must not be invented.

## Rejected v1 proposal — provenance only, not executable

The following proposal is retained only to preserve what the independent review rejected. It is **not** the implementation specification and must not be used as apply authority.

The implementation PR should introduce the fewest new surfaces that make the declaration executable and testable:

1. **One lab bootstrap manifest bundle source** under an existing infra ownership root (owner chooses exact path during design approval), containing:
   - `argocd` Namespace;
   - Argo CD CRDs pinned to the same Argo CD app version as controllers;
   - Argo CD controller resources pinned to one reviewed chart version;
   - existing `infra/gitops/root-app.yaml` applied only after the Application CRD is established.
2. **One hermetic render target** reachable by Buck2 that produces that bundle without network fetches, ambient Helm repositories, or shell. Prefer an owned Rust renderer only if no existing hermetic chart-render target already covers this; do not add a generic framework.
3. **One declaration test fixture** that verifies:
   - exactly one Argo CD version pair (chart/app/CRDs) is selected;
   - root Application targets `dev` + `infra/gitops`;
   - rendered `oya-arm64` Application points at the admitted values path;
   - rendered ARC request is `22Gi` and limit is `60Gi`;
   - ordering cannot submit root Application before its CRD;
   - no CAPI `ClusterResourceSet`, `Cluster`, `TalosControlPlane`, or `MachineDeployment` appears in the class-B bundle.
4. **One protected admission path**: Buck2 target must fan into `oya-ci-required`; successful local render alone is not admission.
5. **One explicit one-time bootstrap runbook record** naming the admitted immutable commit and target cluster. The bootstrap act may apply the admitted Argo bundle once; all downstream resources then converge through root-app. It must not directly apply ARC values or run `helm upgrade oya-arm64`.

Do not add a second values file, duplicate root-app, committed rendered `*.generated.json`, or new multispectrum evidence file.

## Bootstrap boundary

The only class-B imperative act is the irreducible first reconciler installation after its declaration PR is independently reviewed and protected-green:

```text
admitted immutable commit
→ authorized actor applies the exact admitted Argo bootstrap bundle to admin@oya-talos
→ Argo controller establishes
→ existing root Application establishes
→ Argo renders infra/gitops
→ Application oya-arm64 reconciles chart + admitted values
→ live ARC objects roll to request 22Gi
```

The authorized actor must record:

- immutable commit SHA;
- render artifact digest;
- target cluster identity / API endpoint fingerprint;
- before/after Argo health;
- before/after ARS and ERS request;
- rollback result or reason rollback was not exercised.

A branch, worktree, local values copy, or mutable `dev` checkout is not an admissible apply input.

## Identity, secret, and continuity requirements

Before implementation approval, the owner packet must name:

1. **Argo repository credential mode.** Bootstrap currently targets public GitHub; if credentials are required later, use an existing secret distribution substrate. No credential bytes in git or rendered evidence.
2. **ARC secret continuity.** Existing `githubConfigSecret` and controller namespace ownership remain untouched by bootstrap. The first Argo sync must adopt/update the existing Helm-managed resources without deleting runner registration secrets.
3. **Ownership transition.** Existing release `oya-arm64` becomes Argo-owned. The implementation must prove resource names/selectors match the live release before enabling prune.
4. **Job continuity.** Set a bootstrap window or pause admission only if the ownership transition demonstrably disrupts listener/runner pods; do not assume zero disruption.
5. **Rollback owner.** Name who can disable the root Application and restore the last admitted release state if Argo reconciliation is unhealthy.

## Rollback contract

Rollback is declaration rollback, not an unreviewed values edit:

1. Abort if Argo controllers or Application CRDs fail health checks; do not proceed to root-app.
2. If root-app is unhealthy before ARC sync, remove/disable only the admitted root Application per the approved runbook; preserve existing Helm-managed ARC resources.
3. If ARC sync changes ownership but live request/health is wrong, revert the Git declaration through a protected PR or pin root-app to the last admitted immutable commit named in the bootstrap record.
4. Do not roll back to 20Gi as a capacity workaround. If 22Gi cannot become healthy, #1526/#1523 remain blocked and the owner reopens A/B/C choice.
5. Preserve runner secrets and audit all resource deletions; Argo prune must not be enabled until adoption identity is proven.

## Required RED/GREEN proof

### RED fixtures

- Argo controller version differs from CRD source version → render/test fails.
- root-app path or revision differs from `infra/gitops` / `dev` → fails.
- `oya-arm64` values path differs from `infra/arc/runner-scale-set-arm64-values.yaml` → fails.
- request <22Gi, limit !=60Gi, or `2 × request <= allocatable` fixture → fails.
- bundle includes CAPI objects → fails.
- root Application precedes Application CRD → fails.
- mutable/unidentified apply artifact → runbook acceptance fails.

### GREEN fixtures

- Hermetic Buck2 render produces deterministic bytes twice from a clean checkout.
- Bundle contains Namespace + pinned CRDs + pinned controllers + root-app in dependency order.
- App-of-apps render selects the single admitted ARC values file with 22Gi.
- Target is the existing lab identity, not a new CAPI cell.

No network-dependent render qualifies as Buck2-authoritative evidence.

## Live acceptance after admitted bootstrap

All probes are read-only and run against `admin@oya-talos`:

1. Argo:
   - `argocd` namespace exists;
   - controller pods Ready;
   - Application `root` Healthy/Synced;
   - Application `oya-arm64` Healthy/Synced to the admitted commit.
2. ARC desired state:
   - AutoscalingRunnerSet `.spec.template.spec.containers[0].resources.requests.ephemeral-storage` = `22Gi`;
   - EphemeralRunnerSet `.spec.ephemeralRunnerSpec.spec.containers[0].resources.requests.ephemeral-storage` = `22Gi`;
   - at least one newly-created runner pod requests `22Gi`.
3. Capacity/health:
   - no node DiskPressure;
   - no two 22Gi runner requests scheduled on one 42.75664Gi node;
   - runner listener healthy and jobs register/complete.
4. Drift:
   - a read-only Argo refresh reports no unmanaged drift for `oya-arm64`;
   - Helm v12 is no longer the sole desired-state authority.

Promoted-tip CI green without these live probes is insufficient.

## Train dependency

```text
owner signs class B (KEEP_CURRENT_LAB)
→ independent design APPROVE on exact packet
→ implementation worktree + minimal declaration/render/test diff
→ independent code APPROVE on exact head
→ push + protected PR + oya-ci-required green
→ authorized one-time bootstrap from admitted immutable artifact
→ live acceptance proves ARS/ERS/new pod = 22Gi
→ #1526 cold FULL rerun
→ #1523 restack push + admit
→ G023 deletion only after #1523 promoted green
```

## Non-actions

- No cluster mutation from this planning packet.
- No direct ARC Helm upgrade.
- No use of CRS against a non-CAPI lab.
- No CAPI replacement implied by class B.
- No implementation before a revised exact class-B packet receives independent design APPROVE and the bootstrap principal plus rollback owner are named.
- No #1526/#1528 rerun while live request remains 20Gi.
- No #1523 push.
- No #1524 mutation.
- No canonical dirty-checkout mutation.
- No transport failure treated as APPROVE.
