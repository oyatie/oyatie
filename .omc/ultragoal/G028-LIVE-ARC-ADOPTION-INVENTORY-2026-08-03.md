# G028 live ARC adoption inventory — 2026-08-03

State: **READ-ONLY SAFE PROJECTION — NO CREDENTIAL BYTES — NO IMPLEMENTATION — NO LIVE MUTATION**  
Observation window: `2026-08-03T00:41:35Z` through `2026-08-03T00:46:44Z`  
Kubernetes context verified before and after: `admin@oya-talos`

## Collection boundary

Only read-only metadata was collected. No apply, edit, patch, delete, create, restart, scale, upgrade, rollback, CI trigger, credential change, or Secret value read occurred. Secret `.data`, `.stringData`, decoded values, value hashes, and Helm release-storage payloads are excluded.

## Target identity

| Field | Observed value |
|---|---|
| context | `admin@oya-talos` |
| API endpoint | `https://10.5.0.1:6443` |
| API public certificate SHA-256 | `F7:6E:65:AE:02:F5:F4:B9:73:86:2C:20:B8:14:FE:D5:01:34:7B:EF:B4:ED:FB:39:5F:71:C7:00:B4:AA:20:A1` |
| certificate subject | `O=kube-master, CN=kube-apiserver` |
| certificate issuer | `O=kubernetes` |
| validity | `2026-07-29T00:34:47Z` through `2027-07-29T00:34:47Z` |

A future bootstrap must fail closed unless context, endpoint, and public certificate fingerprint equal the admitted target identity. Certificate rotation requires a separately admitted identity update; this packet grants no credentials or action.

## Helm release identity

| Release | Namespace | Revision | Chart | App | Status | Apply method |
|---|---|---:|---|---|---|---|
| `arc` | `arc-systems` | 1 | `gha-runner-scale-set-controller-0.14.2` | `0.14.2` | deployed | server-side apply |
| `oya-arm64` | `arc-runners` | 12 | `gha-runner-scale-set-0.14.2` | `0.14.2` | deployed | server-side apply |

- `arc` revision 1 installed `2026-07-29T10:20:49Z`.
- `oya-arm64` revision 12 deployed `2026-07-30T09:55:39Z`.
- Scale-set revisions 8–10 failed while `oya-arm64-gha-rs-no-permission` was terminating; revisions 11–12 succeeded.
- Retained Helm history began at revision 3; revisions 1–2 are `UNKNOWN`.
- Controller user-supplied values were null.
- Safe scale-set values at observation: `minRunners=0`, `maxRunners=3`, architecture `arm64`, request CPU `2`, memory `4Gi`, ephemeral storage `20Gi`; limits memory `8Gi`, ephemeral storage `60Gi`.
- Current rendered manifests contain no chart-rendered Secret.

These observations resolve actual live Helm release names for adoption: `arc` and `oya-arm64`. V2 must set explicit `helm.releaseName` accordingly.

## ARC CRD establishment

All four CRDs serve/store `v1alpha1`, are `Established=True`, and report managed-fields managers `helm` and `kube-apiserver`:

- `autoscalinglisteners.actions.github.com`
- `autoscalingrunnersets.actions.github.com`
- `ephemeralrunners.actions.github.com`
- `ephemeralrunnersets.actions.github.com`

This proves current CRD health only. It does not waive the staged v2 CRD establishment barrier for admitted content-addressed inputs.

## Controller identity and health

```text
(apps/v1, Deployment, arc-systems, arc-gha-rs-controller)
service_account=arc-gha-rs-controller
helm_release=arc/arc-systems
managed_fields=[helm,kube-controller-manager]
replicas desired=1 updated=1 ready=1 available=1
```

Controller pod `arc-gha-rs-controller-66987b84cb-kwtd9` was Running/Ready with zero restarts on `oya-talos-worker-2`.

## Scale-set identity and health

```text
(actions.github.com/v1alpha1, AutoscalingRunnerSet, arc-runners, oya-arm64)
helm_release=oya-arm64/arc-runners
managed_fields=[helm,manager]
phase=Running
min=0 max=3 current=0 pending=0 running=0
```

```text
(actions.github.com/v1alpha1, EphemeralRunnerSet, arc-runners, oya-arm64-lh4ch)
owner=AutoscalingRunnerSet/oya-arm64
managed_fields=[ghalistener,manager]
phase=Running
current=0 pending=0 running=0
```

Listener:

```text
(actions.github.com/v1alpha1, AutoscalingListener, arc-systems, oya-arm64-776c5b99-listener)
target=arc-runners/oya-arm64
managed_fields=[manager]
```

Listener pod was Running/Ready with zero restarts on `oya-talos-worker-2`.

Controller/listener-generated resources are not Helm-rendered ownership and must be classified separately from static release objects during adoption.

## Live-versus-rendered ownership drift

Current revision-12 rendered scale-set resources include:

- ServiceAccount `oya-arm64-gha-rs-no-permission`;
- Role and RoleBinding `oya-arm64-gha-rs-manager`;
- AutoscalingRunnerSet `oya-arm64`.

Live Helm-owned resources also include three objects absent from the current revision-12 rendered manifest:

- ServiceAccount `oya-arm64-gha-rs-kube-mode`;
- Role `oya-arm64-gha-rs-kube-mode`;
- RoleBinding `oya-arm64-gha-rs-kube-mode`.

All three retain Helm ownership for `oya-arm64/arc-runners` and manager `helm`. Their intended disposition is **UNRESOLVED**. They must be classified explicitly as `KEEP_EXTERNAL`, `ADOPT_WITH_ENUMERATED_DELTA`, or `RETIRE_WITH_PROOF` before prune can ever be enabled. No deletion or ownership change is authorized.

Listener ServiceAccount/Role/RoleBinding are controller-generated with manager `manager`; they must not be mistaken for missing Helm render objects.

## Secret metadata-only continuity record

```text
(v1, Secret, arc-runners, oyatie-arc-app)
type=Opaque
uid=e331b033-22a5-4865-926f-a8cb48fbae0d
resource_version=78247
manager=kubectl-create
owner_references=[]
labels={}
annotations={}
key_names=[github_app_id,github_app_installation_id,github_app_private_key]
```

No value bytes were inspected or recorded. The Secret is an external prerequisite, not Helm-rendered. Its durable provisioner, backup/restoration source, credential lifetime, revocation authority, and restoration actor remain **ABSENT/UNRESOLVED**. Argo must neither manage nor prune it.

## Point-in-time workload observation

At the observation time:

- EphemeralRunner objects: none;
- runner pods: none;
- GitHub Actions queued runs: none;
- GitHub Actions in-progress runs: none;
- ARC current/pending/running counts: `0/0/0`.

This is a timestamped quiescence observation, not a durable availability guarantee. A future authorized adoption must repeat it immediately before ownership transition and abort if workload is not within the admitted quiescence contract.

## Required ownership classes

A deterministic adoption inventory must classify every object as exactly one of:

- `HELM_RENDERED`
- `HELM_LIVE_NOT_RENDERED`
- `ARC_CONTROLLER_GENERATED`
- `EXTERNAL_PREREQUISITE`

The three kube-mode objects are currently `HELM_LIVE_NOT_RENDERED`; `oyatie-arc-app` is `EXTERNAL_PREREQUISITE`; listener and ephemeral resources are `ARC_CONTROLLER_GENERATED`.

## Still unresolved

- Content digests for Argo and ARC chart/CRD bytes.
- Exact admitted rendered inventory and deterministic diff against this safe projection.
- Disposition of the three live kube-mode objects.
- Secret provisioner, restoration source/actor, credential lifetime, and revocation authority.
- Bootstrap principal, scoped RBAC, audit sink, rollback owner, and escalation path.
- OOM, DiskPressure, and eviction history: not claimed.

## Non-actions

- No implementation or v2 approval.
- No Secret value collection.
- No Helm, Argo, or Kubernetes mutation.
- No prune, automated sync, rollout, rerun, or PR action.
- No inference that current health guarantees safe adoption.
