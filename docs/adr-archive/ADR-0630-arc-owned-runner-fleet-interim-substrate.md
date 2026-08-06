---
id: ADR-0630
title: "Actions Runner Controller as the interim owned-runner substrate, declared in infra/arc/, behind the ADR-0515 D5 owned-CI destination"
status: Superseded
doc_status: published
planning_impact: false
deciders: founder
date: 2026-07-29
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0515, ADR-0560, ADR-0556]
amends: []
related: [ADR-0131, ADR-0523, ADR-0535, ADR-0612, ADR-0554, ADR-0639]
related_specs:
  - /infra/arc/runner-scale-set-arm64-values.yaml
  - /infra/arc/runner-scale-set-live-postgres-arm64-values.yaml
  - /infra/arc/RUNBOOK-scale-runners.md
milestone: W3
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0630: ARC as the interim owned-runner substrate

## Status

**Accepted — 2026-08-05** (originally Proposed 2026-07-29; ratified with dual-worker
general capacity amend). Ratifies a runner fleet that was brought up imperatively and
holds merge authority. Original gap: no decision record while helm revisions existed only
as cluster secrets.

## Context

Two facts forced the question at once.

**(1) GitHub-hosted runners stopped working.** Every job on `oya-ci-required` began failing to
*start* with `"The job was not started because recent account payments have failed or your
spending limit needs to be increased"` — an Actions billing wall, reached because the
repository is private (private repos bill Actions minutes; public repos do not). Merge
authority therefore had no working substrate: `oya-ci-required` is the single required context,
and it could not run at all.

**(2) The founder ruled the local Talos cluster PERMANENT and merge-authoritative.**
Verbatim: *"treat this laptop cluster as a permanent part of the cluster"* and *"should have
merge authority."* An earlier objection — that a sleeping laptop becomes merge authority — was
raised twice, considered, and explicitly overruled.

Self-hosted runners consume no Actions minutes, so owned runners resolve (1) as a side effect
of doing what (2) already required. ADR-0515 D5 names owned in-cluster runners as the
destination but scopes the cutover out; this ADR does not change that destination, it starts
the runner half of it earlier than planned because the hosted path is unavailable.

## Decision

### D1 — ARC (`gha-runner-scale-set`) is the interim runner substrate

Adopt `actions/actions-runner-controller` charts, **pinned at 0.14.2**, as the mechanism that
turns the owned cluster into GitHub Actions capacity. It is Apache-2.0, Kubernetes-native
(CRD + controller + reconciliation, no imperative agent lifecycle), and authored by the
platform we currently integrate with — which is precisely the transient-stack profile: adopted
OSS behind an owned destination, not a new permanent dependency.

**It is explicitly INTERIM.** The destination remains the bespoke Rust `cloud/cloud-ci`
product (ADR-0515 D5). ARC's replacement trigger is that product's runner surface becoming
able to execute the canonical gate set; ARC is a runner *transport*, and nothing in the gate
logic may become ARC-shaped.

### D2 — ONE GENERAL-PURPOSE SCALE SET PER ARCHITECTURE, arch-pinned by nodeSelector

Founder requirement: *"we should be agnostic. arm64 / amd64 should both be a supported path"* —
*"on this laptop arm. on another box that joins the cluster that is amd64, we can run amd64."*

So the cluster is heterogeneous and each scale set is pinned:
`nodeSelector: {kubernetes.io/arch: <arch>}`. **Without that pin a scale set schedules
anywhere**, and an "arm64" runner can land on an amd64 node and silently build for the wrong
platform — poisoning that platform's buck2 cache namespace, because action keys include `cpu:`
and `os:`. Adding capacity is joining a node; adding a PLATFORM is one more values file.
`infra/ci/install-buck2.sh` already carries digest-pinned arms for both architectures.

The one-per-architecture rule applies to the general-purpose fleet. A same-architecture sibling
scale set is allowed only when its separate label is itself an isolation boundary for a job-local
service, credential, trust, or resource profile that must not enter general-purpose Pods. Such a
set remains architecture-pinned and may not fork gate logic; workflows select only its capability
label. The dedicated live-PostgreSQL cell in D7 is the first bounded exception.

### D3 — GitHub App identity, scoped to one repository

Authentication is a GitHub App (`Administration: write`, `Metadata: read`), installed on
`jason931225/oyatie` **only** — verified `repo count: 1`. Not a personal access token, which
would have carried `repo`, `workflow`, `gist` and `admin:ssh_signing_key` across every
repository the founder can reach, in a secret readable by anyone with access to a cluster that
is **shared with another project**. Key material lives in the cluster secret and never in git.

Recorded limitation: an App installation token can push to an **existing** ghcr package but
**cannot create one** — measured
`{"code":"DENIED","message":"permission_denied: installation not allowed to Create organization package"}`.
Package creation requires Actions' `GITHUB_TOKEN`. `packages: write` on the App is therefore
necessary but not sufficient, and image publication must run as a workflow.

### D4 — Runner pods carry the CAS client labels

`infra/nativelink`'s NetworkPolicy `nativelink-cas-ingress` admits `:50051` only from pods
labelled `oya.io/nativelink-cas-writer: "true"` and `:50052` only from
`oya.io/nativelink-cas-reader: "true"`, matching the **source** pod in any namespace. The
general-purpose runner Pods therefore carry both. Without them the connection is refused at
**L3** and presents as a **timeout, not a TLS error** — a failure mode that reads as a certificate
problem and is not one. This is the mechanical precondition for the CAS ever being reachable;
the buck2-side wiring is separate and still blocked (see Consequences). The D7 `--local-only`
cell deliberately carries neither label.

### D5 — Resource bounds derived by arithmetic, on a shared cluster

The general scale set requests `cpu 2 / memory 4Gi / root ephemeral-storage 4Gi /
workspace PVC 44Gi`, limits `memory 8Gi / root ephemeral-storage 8Gi`, and carries **no CPU
limit**.

**Capacity model (amended 2026-08-05):** the general-purpose set `oya-arm64` is authorized at
**`maxRunners: 2`** with **required pod anti-affinity on `kubernetes.io/hostname`** so concurrent
general runners prefer distinct workers. Each worker that admits general workspaces exposes a
dedicated ~48Gi Talos user volume path for the general StorageClass. Local Path Provisioner does
not enforce PVC byte requests; the blast-radius boundary is **one general runner per physical
general volume (per node)**, not “free CPU implies free concurrency.” Raising above 2 requires a
new capacity measurement and an amendment.

- `cpu 2` tracks the MEASURED buck2 scaling knee (j=2 87.19s, j=4 67.06s, j=8 71.29s,
  j=18 98.09s) — over-parallelism degrades, so a wider request buys nothing and crowds
  co-tenants.
- `memory 8Gi` per runner; two concurrent general runners must still leave headroom for
  co-tenants on each worker (nativelink, control-plane components).
- The build tree is a generic ephemeral PVC mounted at `/home/runner/_work`, not a writable
  layer on Talos `EPHEMERAL`. Each claim uses StorageClass `oya-ci-workspace-general` with
  nodePathMap admitting `/var/mnt/ci-workspace-general` on the general workers.
  Local Path Provisioner does not enforce the advertised claim size, so **no soft reserve is
  treated as hard isolation**. Either runner can fill its own node’s general volume, but
  anti-affinity plus per-node volumes prevent two general runners from sharing one 48Gi volume.
- Live-postgres remains a **separate** scale set capped at `maxRunners: 1` on its own volume
  (D7). Path-optional scheduling of those jobs is governed by ADR-0639, not by raising postgres
  concurrency.
- Root `ephemeral-storage` remains bounded at 4Gi/8Gi for image, tool installer, and writable
  layer pressure. A runaway installer is evicted rather than filling the Talos system filesystem.
- **No CPU limit deliberately**: a throttled compile presents as a flaky slow test, which is
  harder to diagnose than a noisy neighbour we can observe.
- **Apply residual:** git declarations under `infra/arc/` are the apply source; capacity is not
  live until GitOps/helm apply succeeds. Human runbook: `infra/arc/RUNBOOK-scale-runners.md`.
  Agents do not mutate the cluster.

### D6 — Declared in `infra/arc/`, and the declaration is drift-checked

The fleet ships as values files under `infra/arc/` and is registered in
`infra/gitops/values.yaml` beside `external-secrets` and `registry`, the pattern those charts
already follow. The values are reviewed **desired state**. Live readback is separate evidence:
the general set, workspace provisioner, Talos user volume, and dedicated PostgreSQL set are not
live merely because their declarations render. Do not describe desired-versus-live differences
as zero drift or rollout evidence.

### D7 — Live-PostgreSQL tests use a dedicated ephemeral same-Pod cell

The two merge-blocking live-PostgreSQL jobs select `oya-live-postgres-arm64`, not the
general-purpose `oya-arm64` set. Each ARC runner Pod owns one digest-pinned PostgreSQL 16 native
sidecar as a restartable init container. Its startup probe completes before the runner starts;
both database state and credentials are size-bounded memory `emptyDir` volumes deleted with the
Pod. An ordinary init container generates separate random admin and application passwords. The
workflow reads those files from `/run/oya-ci-postgres`, masks both values before use, and preserves
the existing admin-versus-`NOBYPASSRLS` application-role assertions.

No Kubernetes Secret or shared CNPG hostname is projected into either runner scale set. The
general-purpose runner receives no database admin environment, and the dedicated Pod exposes no
Service. A namespace NetworkPolicy selects only the dedicated cell label and denies cross-Pod
ingress; the tests reach PostgreSQL only over same-Pod localhost. A Kubernetes-native egress
allowlist selects both runner-cell labels, default-denies them, and restores only DNS, NativeLink,
the in-cluster registry, and public IPv4 excluding private/link-local ranges. This excludes
`oya-data` without requiring a Cilium CRD; live CNI enforcement remains mandatory readback. The
dedicated cell has
no NativeLink reader/writer labels because every live test invocation is `--local-only`; granting
an unused CAS network capability would widen its blast radius without changing execution. This is
test-fixture isolation, not a production data-plane dependency.

The dedicated set is capped at `maxRunners: 1`, mounts a 44Gi generic-ephemeral workspace from
its own StorageClass and 48Gi filesystem on worker 2, and limits root ephemeral storage to 8Gi.
Its cap serializes the adapter and facade jobs even though the workflow keeps them as separate
required lanes. Combined with the
general set's cap on worker 1, at most one claim can consume each physically and node-separated
48Gi workspace. This
declaration is not rollout readiness: issue #1504 must verify the exact-head cold-concurrency
envelope before closure.

### D8 — Candidate admission uses a bounded exact-head GitOps bootstrap

The dedicated label creates a bootstrap dependency: Argo normally follows `dev`, while a pull
request needs the dedicated scale set before it can earn protected admission into `dev`. The
candidate therefore MUST NOT claim that committing `infra/gitops/values.yaml` alone makes its own
runner available. After #1504 capacity is verified, an authorized operator uses the Argo/Kubernetes
API or console to apply the reviewed JSON Patch template in
`infra/arc/live-postgres-admission-bootstrap.json` to the `argocd/root` Application. Both the root
chart revision and its `valuesObject.targetRevision` are set to the same SSH-signed PR head; the
latter is required so the generated child Applications read candidate values and policy from that
exact commit rather than from `dev`.

The bootstrap is limited to two hours and records PR, exact head, expiry, operator, and evidence
packet annotations. Admission evidence requires API readback that the root and the two child
Applications resolved the exact head, the network policy synced before the scale set, and the
dedicated label accepted the two exact-head jobs. The PATCH is not evidence; readback is. On
failure, supersession, or expiry, the operator applies the paired rollback patch: restore the root
to `dev`, remove the candidate values override and bootstrap annotations, then record readback that
the root is on `dev` and candidate-only children were pruned. After merge, restoring `dev` retains
the children because `dev` then owns them. This is an explicitly credential-gated bootstrap, not a
CI shell step or an assertion of live rollout. Bootstrap may begin only after the repository-side
#1504 capacity declaration is independently reviewed and pre-apply readback proves `/dev/vdb`
remains blank; issue #1504 stays open during the bounded bootstrap so that cold-concurrency,
cleanup, and rollback evidence can be recorded without a circular closure prerequisite.

### D9 — Runner workspaces use a separate fail-closed local provisioner

The existing default `local-path` StorageClass and `rancher.io/local-path` controller continue to
own CNPG, registry, NativeLink, OpenBao, and other stateful data. CI workspaces use a
second provisioner identity, `oyatie.io/ci-workspace-local-path`, and two StorageClasses,
`oya-ci-workspace-general` and `oya-ci-workspace-live-postgres`. Its node-path map names only
`oya-talos-worker-1` with `/var/mnt/ci-workspace-general` and `oya-talos-worker-2` with
`/var/mnt/ci-workspace-live-postgres`; there is no default path for unlisted nodes. Each
StorageClass selects exactly one mount path, and each runner selects the matching exact worker
hostname. `WaitForFirstConsumer` plus the provisioner's no-default node map makes an absent volume
or wrong-node placement fail closed instead of spilling builds onto stateful storage. Worker 2's
pre-existing `ci-workspace-general` user volume remains an unadmitted rollback reserve: it is not
listed in `nodePathMap`, may not receive a PVC, and is not deleted or wiped by this change.

The repository declares alerts for DiskPressure, workspace free space, root writable-layer
growth, eviction, lingering ephemeral PVC cleanup, and ARC startup/queue delay. The local cluster
currently has no Prometheus Operator CRDs or observability namespace, so the `PrometheusRule` is
deliberately not wired into Argo in this slice: a custom resource with no owning CRD would break
reconciliation. Observability-substrate admission, live scrape/readback, and alert routing remain
rollout evidence.

Rollback first sets both scale sets to `maxRunners: 0` through GitOps and waits for their Pods and
ephemeral PVCs to disappear. Only then may the workspace storage Application be reverted and,
after mount readback is empty, an admitted Talos user volume removed. Worker 2's unadmitted general
volume remains the non-destructive rollback reserve unless a later reviewed change explicitly
retires it. Rollback never repoints the existing stateful StorageClass, wipes a disk, or restores
the unsafe three-runner root-filesystem layout.
Issue #1504 remains open until exact-head cold maximum-concurrency evidence measures p50/p95/p99
and proves no DiskPressure or eviction. Both workers remain mixed compute nodes; a dedicated disk
is not a dedicated CI cell.

## Alternatives considered

- **Wait for the billing fix and stay on GitHub-hosted runners** — rejected as the *only*
  plan: it leaves merge authority dependent on a payment state, and does nothing toward the
  ADR-0515 D5 destination. Not mutually exclusive; fixing billing remains desirable.
- **A personal access token instead of an App** — rejected on blast radius (D3).
- **Reviving `oya/governance/iac/helm/lane-runner-pool/`** — rejected. It pins *legacy*
  `actions-runner-controller` 0.23.x (appVersion 0.27.0), targets `microservices/governance/…`
  paths that ADR-0131/0512 mark removal-candidate, references a `ghcr.io/oyatie/governance-runner`
  image that does not exist, sizes 8–200 replicas, and has never run. It is superseded in
  substance by this ADR and should be deleted in its own change rather than silently
  contradicted.
- **One scale set spanning both architectures** — rejected: non-deterministic platform per job,
  which breaks cache-namespace reasoning and reproducibility (D2).

## Consequences

**Positive.** Merge authority has a working substrate that consumes no Actions minutes. The
runner half of ADR-0515 D5 exists and is declared. Runners are co-resident with the
NativeLink CAS on the same architecture, which is the first time the cache and its consumer
have shared a namespace — previously the blocker that made warm-cache work unmeasurable.

**Negative / known gaps, tracked not hidden.**
- `image: ghcr.io/actions/actions-runner:latest` is a **mutable tag** in a repo where every
  workflow `uses:` is commit-pinned. It becomes a digest pin when the baked toolchain image
  lands and this reference changes anyway.
- The image carries **no toolchain**: probed, the stock runner has python3/git/curl but no
  compiler, no rustup, no zstd. Every job pays ~2 minutes installing them. This is the
  next slice.
- `minRunners: 0`, so every job pays pod scheduling plus a cold toolchain. A warm pool is
  queued *behind* the baked image, because pre-warming a pod that then installs rustc buys
  little.
- **No trust tiers.** ADR-0556 D1 distinguishes trusted-author from untrusted-author execution
  and prohibits the latter from writing the shared cache. One scale set cannot express that;
  the split arrives with the cache wiring.
- **`buck2 --num-threads` is uncapped**, so buck2 reads the node's core count from a pod
  requesting 2 — past the measured j=4 knee.
- The CAS is reachable at L3 (D4) but **not yet wired**: buck2 resolves `[buck2_re_client]`
  into `DaemonStartupConfig` from project config files only, and the resolver emits
  `--config-file`, which is measured-inert for that section.
- The shared baked runner image does not yet include `psql`; the live jobs install the distro
  client at runtime. Baking it requires publishing and verifying a new signed runner image and
  updating the image digest, which is a separate image-supply-chain change rather than an
  unreviewed expansion of this credential-containment slice.
- Single-node durability: `local-path` RWO PVCs and a one-replica CAS on a permanent substrate
  that is one laptop.

## Artifact accounting (ADR-0555)

This decision is the justification anchor for `infra/arc/OWNERS`, `infra/arc/BUCK`,
`infra/arc/README.md`, `infra/arc/controller-values.yaml`,
`infra/arc/runner-scale-set-arm64-values.yaml`,
`infra/arc/runner-scale-set-live-postgres-arm64-values.yaml`,
`infra/arc/live-postgres-runner-network-policy.yaml`,
`infra/arc/live-postgres-admission-bootstrap.json`, `infra/arc/ci-workspace-storage.yaml`,
`infra/arc/ci-workspace-alerts.yaml`, `infra/arc/tests/ci_workspace_capacity.rs`,
`infra/talos/local/patches/BUCK`,
`infra/talos/local/patches/ci-workspace-worker-1.yaml`,
`infra/talos/local/patches/ci-workspace-worker-2.yaml`, and the ARC registrations in
`infra/gitops/values.yaml`.

It is also the justification anchor for the baked runner image this fleet runs. One path per
bullet, spelled byte-exactly and unwrapped, because the born-accounting producer matches
path-like tokens and an abbreviated or line-broken path matches nothing:

- `infra/ci/runner-image/Dockerfile`
- `infra/ci/runner-image/OWNERS`

Why this decision owns them rather than a new ADR: the image exists only to serve this scale
set, and D4 already commits to a baked toolchain image for it — the recipe is that commitment's
artifact, not a separate decision. It was previously built from a copy living outside the repo,
which made the image holding merge authority unreproducible; committing it closes that hole and
is what brings it under ADR-0555 accounting in the first place.

The image additionally bakes **PowerShell** because the required lane runs its gate matrix under
`shell: pwsh` and the upstream `actions-runner` image ships none — GitHub-hosted images supplied
it invisibly, so the dependency only became apparent once this fleet took the jobs.
