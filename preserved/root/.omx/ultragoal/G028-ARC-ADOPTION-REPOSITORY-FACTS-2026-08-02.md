# G028 ARC adoption repository facts — 2026-08-02

State: **READ-ONLY SOURCE FACTS — LIVE INVENTORY ABSENT — NO IMPLEMENTATION — NO LIVE MUTATION**  
Immutable source baseline: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf`

## Proven repository declarations

| Field | Immutable declaration |
|---|---|
| controller Application | `arc`; chart `gha-runner-scale-set-controller`; OCI chart version `0.14.2`; destination `arc-systems`; sync wave 1 (`infra/gitops/values.yaml:108-118`) |
| scale-set Application | `oya-arm64`; chart `gha-runner-scale-set`; OCI chart version `0.14.2`; destination `arc-runners`; sync wave 2 (`infra/gitops/values.yaml:120-130`) |
| Application namespace | generated Applications live in `argocd`; Application names come from `.name` (`infra/gitops/templates/applications.yaml:1-17`) |
| Helm release name | **absent**: template does not set `helm.releaseName` (`infra/gitops/templates/applications.yaml:19-31`) |
| current sync policy | all generated apps hard-code automated prune+selfHeal, CreateNamespace, and ServerSideApply (`infra/gitops/templates/applications.yaml:56-60`) |
| scale-set identity | `githubConfigUrl` is this repository; `githubConfigSecret: oyatie-arc-app`; `runnerScaleSetName: oya-arm64` (`infra/arc/runner-scale-set-arm64-values.yaml:9-14`) |
| credential model | GitHub App Secret bytes live in cluster, never git; controller has no GitHub credential and each scale set references its own Secret (`infra/arc/controller-values.yaml:3-5`; `docs/decisions/ADR-0630*`:75-81) |
| file owner | `cloud-ci-platform` (`infra/arc/OWNERS:1`) |

The two `0.14.2` version strings do not identify chart bytes. Content digests remain unresolved.

## Proven absent from repository evidence

No current executable artifact was found for:

- controller or scale-set `helm list/history/status/get values/get manifest`;
- exact live release names or revisions;
- rendered/live `(apiVersion, kind, namespace, name)` inventories;
- selectors, service accounts, Helm ownership annotations, Argo tracking identity, managed fields, or owner references;
- current controller/listener/runner/AutoscalingRunnerSet health;
- metadata/type/key-name inventory for `arc-runners/oyatie-arc-app`;
- declarative provisioner, backup, or restoration authority for that Secret;
- executable ARC rollback or post-rollback assertions.

ADR-0630's field-for-field zero-drift statement is historical narrative, not a current inventory.

## Required pre-bootstrap capture

The separately authorized read-only inventory capture must bind:

```text
timestamp
cluster context + API fingerprint
namespace inventory
helm list/history/status/get-values/get-manifest for controller + scale-set
ARC CRDs and every controller/scale-set/listener/runner resource
(apiVersion, kind, namespace, name)
selectors + service accounts
ownership labels/annotations + ownerReferences + managedFields
Argo Application specs/status if any
metadata-only arc-runners/oyatie-arc-app type + key-name set + UID/resourceVersion + ownership metadata
workload/job quiescence evidence
```

Never capture Secret values. These facts decide whether the live releases can be adopted, require an enumerated migration, or must remain untouched.

## Unknown until capture and hermetic chart render

- actual live Helm release names, revisions, managers, resources, and health;
- whether default Helm release-name inference matches live ownership;
- whether immutable selectors and names match admitted renders;
- Secret existence, schema, continuity, provisioner, backup, and restoration actor;
- whether live imperative resources are safely adoptable.

No OOM, DiskPressure, or eviction conclusion follows from this packet.

## Non-actions

- No guessed `helm.releaseName`.
- No implementation or design approval.
- No cluster query or mutation from this packet.
- No credential bytes in any capture.
- No prune, automated sync, or broad app-of-apps adoption.
